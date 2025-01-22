import { executeIfSettingsChanged, initializeSettingsListeners, loadSettings, resetSettings } from "./settings.js";
import { printDebug } from "./utils.js";

const { invoke } = window.__TAURI__.core;
const { message } = window.__TAURI__.dialog;
const { debug, error } = window.__TAURI__.log;
const { listen } = window.__TAURI__.event;
const { load } = window.__TAURI__.store;

const Action = Object.freeze({
  load: "l",
  info: "i",
  next: "n",
  previous: "p",
  uncover: "u",
  clear: "c",
  reset: "r",
  settings: "s",
  settingsDone: "d",
  settingsReset: "y",
});

const state = {
  polygons: [],

  image: 0,
  svg: 0,
  svgPolygons: [],
  svgPolygonsHideIdx: 0,

  settings: 0,
};

async function getImage(u) {
  try {
    const revealObject = await invoke("get_image", { u: u, quizYear: state.inputQuizYear.checked });
    state.image.setAttribute("hidden", "hidden");
    state.image.src = `data:image/${revealObject.image_type};base64,${revealObject.image}`;

    if (revealObject.question !== undefined) {
      state.qnaAnswersDiv.innerHTML = "";
      state.qnaAnswersDiv.innerHTML = revealObject.answers
        .map(
          (a, idx) => `<button 
            data-idx=${idx}
            class='answer'>${a}
          </button>`,
        )
        .join("\n");

      for (const button of document.querySelectorAll("button.answer")) {
        button.addEventListener("pointerup", () => {
          if (Number(button.dataset.idx) === Number(revealObject.correct_answer)) {
            button.classList.add("correct");
          } else {
            button.classList.add("wrong");
          }
        });
      }
    }

    // TODO promise fail not handled ...
    return state.image.decode();
  } catch (e) {
    const errorMessage = `Failed loading image: ${e}`;
    error(errorMessage);
    message(errorMessage, { title: "Error", kind: "error" });
  }
}

function uncoverNext() {
  if (state.svgPolygons.length > 0) {
    const index = state.svgPolygonsHideIdx++ % state.svgPolygons.length;
    state.svgPolygons[index].style.opacity = "0";
  }
}

function coverFull() {
  for (const p of state.svgPolygons) {
    p.style.opacity = "1";
  }
  state.svgPolygonsHideIdx = 0;
}

function uncoverFull() {
  for (const p of state.svgPolygons) {
    p.style.opacity = "0";
  }
  state.svgPolygonsHideIdx = 0;
}

async function loadCovering() {
  const w = state.image.naturalWidth || 1;
  const h = state.image.naturalHeight || 1;
  const n = Number(state.inputObjectCount.value);
  debug(`Requesting covering for ${w}x${h} with ${n}.`);

  const polygons = await invoke("load_covering", {
    n: n,
    width: w,
    height: h,
  });

  state.polygons = polygons.map((polygon) => {
    return polygon.pnts;
  });

  // Update the svg accordingly
  state.svg.setAttribute("viewBox", `0 0 ${state.image.naturalWidth} ${state.image.naturalHeight}`);
  state.svg.setAttribute("shape-rendering", "crispEdges");
  state.svg.replaceChildren();

  state.svgPolygons = [];
  for (const points of state.polygons) {
    const polygon = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
    state.svg.appendChild(polygon);

    polygon.setAttribute("fill", getRandomColorHex());
    polygon.setAttribute("stroke-width", "1");
    polygon.setAttribute("points", points.map((p) => `${p.x},${p.y}`).join(" "));
    state.svgPolygons.push(polygon);
  }

  state.svgPolygonsHideIdx = 0;
  state.image.removeAttribute("hidden");
}

const rgbMax = 2 ** 24 - 1;
function getRandomColorHex() {
  return `#${Math.round(Math.random() * rgbMax)
    .toString(16)
    .padStart(6, "0")}`;
}

async function executeAction(actionIdentifier) {
  debug(`Executing action ${actionIdentifier}`);
  switch (actionIdentifier) {
    case Action.uncover:
      uncoverNext();
      break;
    case Action.reset:
      coverFull();
      break;
    case Action.next:
      getImage(1).then(() => loadCovering());
      break;
    case Action.previous:
      getImage(-1).then(() => loadCovering());
      break;
    case Action.clear:
      uncoverFull();
      break;
    case Action.info:
      await printDebug();
      break;
    case Action.load:
      await invoke("get_image_paths", { forceSelection: true, verbose: state.inputVerbose.checked });
      break;
    case Action.settings:
      // Toggle the state
      if (state.settingsDiv.style.display !== "inline") {
        state.settingsDiv.style.display = "inline";
      } else {
        state.settingsDiv.style.display = "none";
      }
      break;
    case Action.settingsDone:
      {
        state.settingsDiv.style.display = "none";
        executeIfSettingsChanged(() => {
          getImage(0).then(() => loadCovering());
        });
      }
      break;
    case Action.settingsReset:
      await resetSettings(state);
      break;
    default:
  }
}

function registerControlButtons() {
  for (const button of document.querySelectorAll(".control")) {
    button.addEventListener("pointerup", (e) => {
      const action = button.dataset.event.toLowerCase();
      executeAction(action);
      e.preventDefault();
      e.stopImmediatePropagation();
    });
  }
}

function registerKeyboard() {
  document.addEventListener("keyup", (event) => {
    // By default, react to the key as specified by the 'data-event' in the html.
    // But additionally react to some special keys:
    let action = event.key.toLowerCase();
    switch (event.code) {
      case "Space":
        action = Action.uncover;
        break;
      case "Delete":
        action = Action.clear;
        break;
      default:
    }
    executeAction(action);
  });
}

function registerTouch() {
  // Simple touch shall act as a single uncover.
  // Some care has to be taken to not react to pressing some of the other UI elements,
  // be it regular control buttons or settings.
  document.addEventListener("pointerup", (e) => {
    // Ignore interaction with the settings pane
    if (e.target?.closest(".settings") !== null) {
      return;
    }
    if (e.target?.classList.contains("answer")) {
      return;
    }
    executeAction(Action.uncover);
    e.preventDefault();
    e.stopImmediatePropagation();
  });

  // Handle swipe interaction:
  //   down:  fully uncover
  //   up:    reset covering (fully cover)
  //   right: next image
  //   left:  previous image
  const touchStart = { x: 0, y: 0 };
  const touchEnd = { x: 0, y: 0 };
  let touchMulti = false;

  function reactToSwipe() {
    // TODO make the minimal swipe distance configurable in settings
    const threshold = 30;
    const absX = Math.abs(touchStart.x - touchEnd.x);
    const absY = Math.abs(touchStart.y - touchEnd.y);
    if (absX > absY && absX > threshold) {
      // horizontal
      if (touchEnd.x < touchStart.x) {
        // Use "natural scrolling",
        // i.e. if the user "moves" the screen to the left,
        // go to the next image.
        executeAction(Action.next); // left
      } else {
        executeAction(Action.previous); // right
      }
    } else if (absY > threshold) {
      // vertical
      if (touchEnd.y < touchStart.y) {
        executeAction(Action.reset); // up
      } else {
        executeAction(Action.clear); // down
      }
    }
  }

  function isZoomedIn() {
    if (window.visualViewport !== undefined) {
      return window.visualViewport.scale !== 1.0;
    }
    return false;
  }

  // Take care to avoid undesired behavior when zooming,
  // i.e., when the user "pinches" into the image to look at some detail.
  // We mitigate this by not accepting a swipe if at any point during the
  // touch sequence more than one touch point was detected.
  document.addEventListener("touchstart", (e) => {
    touchStart.x = e.changedTouches[0].screenX;
    touchStart.y = e.changedTouches[0].screenY;
    touchMulti = e.touches.length > 1;
  });
  document.addEventListener("touchmove", (e) => {
    touchMulti |= e.touches.length > 1;
  });
  document.addEventListener("touchend", (e) => {
    if (!(isZoomedIn() || touchMulti)) {
      touchEnd.x = e.changedTouches[0].screenX;
      touchEnd.y = e.changedTouches[0].screenY;
      reactToSwipe();
    }
  });
}

function registerTauriEvents() {
  function tfListen(eventName, fun) {
    listen(eventName, (event) => {
      debug(`Handling event: ${JSON.stringify(event)}`);
      fun(event);
    });
  }

  tfListen("image-paths-updated", (event) => {
    if (event.payload) {
      state.locationSpan.textContent = `Images from: ${event.payload}.`;
    } else {
      state.locationSpan.textContent = "Images hand-selected.";
    }
    getImage(0).then(() => loadCovering());
  });

  tfListen("image-paths-failed", (_) => {
    state.locationSpan.textContent = "Exemplary images.";
    state.progressSpan.textContent = "";
    // Since no images have been loaded, this will return randomly picked
    // exemplary images.
    getImage(0).then(() => loadCovering());
  });

  tfListen("image-index", (event) => {
    const indexState = event.payload;
    state.progressSpan.textContent = `${indexState[0] + 1} / ${indexState[1]}`;
  });
}

window.addEventListener("DOMContentLoaded", async () => {
  debug("Loading ...");

  // GUI
  state.image = document.querySelector("#image-container");
  state.slider = document.querySelector("#sliderN");
  state.svg = document.querySelector("#overlay-svg");
  state.slider = document.querySelector("#sliderN");
  state.progressSpan = document.querySelector("#progress");
  state.locationSpan = document.querySelector("#location");
  state.qnaAnswersDiv = document.querySelector("#answers");

  // Settings
  state.settingsDiv = document.querySelector("#settings");
  state.inputShowControls = document.querySelector("#input-show-controls");
  state.inputVerbose = document.querySelector("#input-verbose");
  state.inputObjectType = document.querySelector("#input-object-type");
  state.inputObjectCount = document.querySelector("#input-object-count");
  state.inputQuizYear = document.querySelector("#input-quiz-year");

  // Before setting up everything, load the current settings,
  // which may have been persisted from a previous execution.
  state.store = await load("settings.json", { autoSave: true });
  await loadSettings(state);
  initializeSettingsListeners(state);

  // Initialize interactivity
  registerControlButtons();

  // TODO register only if available?
  registerKeyboard();
  registerTouch();

  registerTauriEvents();

  // UI ready, request image paths to be loaded.
  invoke("get_image_paths", { forceSelection: false, verbose: state.inputVerbose.checked });

  debug("Done.");
});
