const { invoke } = window.__TAURI__.core;
const { message } = window.__TAURI__.dialog;
const { debug, error } = window.__TAURI__.log;
const { listen } = window.__TAURI__.event;
const { load } = window.__TAURI__.store;

import { isMobile, printDebug } from './utils.js';

const Action = Object.freeze({
  load: 'l',
  info: 'i',
  next: 'n',
  previous: 'p',
  uncover: 'u',
  clear: 'c',
  reset: 'r',
  settings: 's',
  settings_done: 'd',
  settings_reset: 'y',
});

let state = {

  polygons: [],

  image: 0,
  svg: 0,
  svgPolygons: [],
  svgPolygonsHideIdx: 0,

  settings: 0,

  verbose: true,
}

async function loadSettings() {
  await state.store.get("show_controls").then((v) => {
    if (v !== undefined) {
      state.inputShowControls.checked = JSON.parse(v);
    } else {
      state.inputShowControls.checked = !isMobile();
    }
  });

  // TODO defaults
  await state.store.get("object_type").then((v) => {
    if (v !== undefined) {
      state.inputObjectType.value = v;
    }
  });
  await state.store.get("object_count").then((v) => {
    if (v !== undefined) {
      state.inputObjectCount.value = v;
    }
  });

  await state.store.get("verbose").then((v) => {
    state.verbose = v || true;
  });
}

function persistSettings() {
  state.store.set("show_controls", state.inputShowControls.checked);
  state.store.set("object_type", state.inputObjectType.value);
  state.store.set("object_count", state.inputObjectCount.value);
}

async function getSettings() {
  state.store = await load('settings.json', { autoSave: true });

  await loadSettings();

  debug(`Loaded initial settings: ${JSON.stringify(state.settings, null, "  ")}.`);

  persistSettings();

  // TODO ability to clear the default path ...
}


function initializeSettingsListeners(state) {

  // TODO create inner function to be used that stores settings after execution below

  // Slider
  state.inputObjectCount.oninput = function (e) {
    persistSettings();
    e.preventDefault();
    e.stopImmediatePropagation();

    // TODO maybe only reload on done button press?
    loadCovering();
  }

  // Set default controls state
  let updateFun = event => {
    const showControls = state.inputShowControls.checked;
    if (event) {
      persistSettings();
      event.stopImmediatePropagation();
    }
    document.querySelectorAll(".controls-optional").forEach(element => {
      if (showControls) {
        element.style.display = 'block';
      } else {
        element.style.display = 'none';
      }
    });
    //TODO ... event? based on .settings?!
  };
  // TODO need this to initialize the visibility state of the controls
  updateFun();

  // 'click' fires after input checkbox state has changed
  // TODO rather use an onchange?
  state.inputShowControls.addEventListener("click", updateFun);
}

async function getImage(u) {
  try {
    const revealObject = await invoke('get_image', { u: u });
    state.image.setAttribute("hidden", "hidden");
    state.image.src = `data:image/${revealObject.image_type};base64,${revealObject.image}`;

    // TODO promise fail not handled ...
    return state.image.decode()
  } catch (e) {
    const error_message = `Failed loading image: ${e}`;
    error(error_message);
    message(error_message, { title: 'Error', kind: 'error' });
  }
}

function uncoverNext() {
  if (state.svgPolygons.length > 0) {
    let index = state.svgPolygonsHideIdx++ % state.svgPolygons.length;
    state.svgPolygons[index].style.opacity = '0';
  }
}

function coverFull() {
  state.svgPolygons.forEach(p => {
    p.style.opacity = '1';
  });
  state.svgPolygonsHideIdx = 0;
}

function uncoverFull() {
  state.svgPolygons.forEach(p => {
    p.style.opacity = '0';
  });
  state.svgPolygonsHideIdx = 0;
}

async function loadCovering() {
  const w = state.image.naturalWidth || 1;
  const h = state.image.naturalHeight || 1;
  const n = Number(state.inputObjectCount.value);
  debug(`Requesting covering for ${w}x${h} with ${n}.`);

  const polygons = await invoke('load_covering', {
    n: n,
    width: w,
    height: h,
  });

  state.polygons = polygons.map(polygon => {
    return polygon.pnts;
  });

  // Update the svg accordingly
  state.svg.setAttribute("viewBox", `0 0 ${state.image.naturalWidth} ${state.image.naturalHeight}`);
  state.svg.setAttribute("shape-rendering", "crispEdges");
  state.svg.replaceChildren();

  state.svgPolygons = [];
  state.polygons.forEach(points => {
    const polygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
    state.svg.appendChild(polygon);

    polygon.setAttribute('fill', getRandomColorHex());
    polygon.setAttribute('stroke-width', '1');
    polygon.setAttribute('points', points.map(p => `${p.x},${p.y}`).join(' '));
    state.svgPolygons.push(polygon);
  });

  state.svgPolygonsHideIdx = 0;
  state.image.removeAttribute("hidden");
}

const rgbMax = Math.pow(2, 24) - 1;
function getRandomColorHex() {
  return `#${Math.round(Math.random() * rgbMax).toString(16).padStart(6, '0')}`;
}

async function executeAction(action_identifier) {
  debug(`Executing action ${action_identifier}`)
  switch (action_identifier) {
    case Action.uncover:
      uncoverNext();
      break;
    case Action.reset:
      coverFull();
      break;
    case Action.next:
      getImage(1)
        .then(() => loadCovering());
      break;
    case Action.previous:
      getImage(-1)
        .then(() => loadCovering());
      break;
    case Action.clear:
      uncoverFull();
      break;
    case Action.info:
      await printDebug();
      break;
    case Action.load:
      await invoke('get_image_paths', { forceSelection: true, verbose: state.verbose });
      break;
    case Action.settings:
      // Toggle the state
      if (state.settingsDiv.style.display !== 'inline') {
        state.settingsDiv.style.display = 'inline';
      } else {
        state.settingsDiv.style.display = 'none';
      }
      break;
    case Action.settings_done:
      state.settingsDiv.style.display = 'none';
      // TODO react to changes accordingly
      break;
    case Action.settings_reset:
      // TODO
      break;
    default:
  }
}

function registerControlButtons() {
  document.querySelectorAll(".control").forEach((button) => {
    button.addEventListener('pointerup', (e) => {
      let action = button.dataset.event.toLowerCase();
      executeAction(action);
      e.preventDefault();
      e.stopImmediatePropagation();
    });
  });
}

function registerKeyboard() {
  document.addEventListener('keyup', (event) => {
    // By default, react to the key as specified by the 'data-event' in the html.
    // But additionally react to some special keys:
    let action = event.key.toLowerCase();
    switch (event.code) {
      case 'Space':
        action = Action.uncover;
        break;
      case 'Delete':
        action = Action.clear;
    }
    executeAction(action);
  });
}

function registerTouch() {
  // Simple touch shall act as a single uncover.
  // Some care has to be taken to not react to pressing some of the other UI elements,
  // be it regular control buttons or settings.
  document.addEventListener('pointerup', (e) => {
    // TODO ignore every action on the settings pane
    if (e.target === state.inputObjectCount || e.target == state.inputShowControls) {
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
  let touchStart = { x: 0, y: 0, };
  let touchEnd = { x: 0, y: 0, };

  function reactToSwipe() {
    // TODO make the minimal swipe distance configurable in settings
    const threshold = 30;
    let absX = Math.abs(touchStart.x - touchEnd.x);
    let absY = Math.abs(touchStart.y - touchEnd.y);
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

  // TODO there's some undesired behavior when zooming,
  // i.e., the user "pinches" into the image to look at some detail.
  // Maybe we can avoid this by checking for the number of touch points?
  //   (e.touches.length)
  // Note that the user may start touching with one finger only and 
  // adding the other one only later.
  document.addEventListener('touchstart', e => {
    touchStart.x = e.changedTouches[0].screenX;
    touchStart.y = e.changedTouches[0].screenY;
  });
  document.addEventListener('touchend', e => {
    touchEnd.x = e.changedTouches[0].screenX;
    touchEnd.y = e.changedTouches[0].screenY;
    reactToSwipe()
  });
}

function registerTauriEvents() {

  function tf_listen(event_name, fun) {
    listen(event_name, (event) => {
      debug("Handling event: " + JSON.stringify(event));
      fun(event);
    });
  }

  tf_listen("image-paths-updated", (event) => {
    if (event.payload) {
      state.locationSpan.textContent = `Images from: ${event.payload}.`;
    } else {
      state.locationSpan.textContent = `Images hand-selected.`;
    }
    getImage(0)
      .then(() => loadCovering());
  });

  tf_listen("image-index", (event) => {
    const indexState = event.payload;
    state.progressSpan.textContent = `${indexState[0] + 1} / ${indexState[1]}`;
  });
}

window.addEventListener("DOMContentLoaded", async () => {

  debug("Loading ...");

  // GUI
  state.image = document.querySelector("#image-container");
  state.slider = document.querySelector("#sliderN");
  state.svg = document.querySelector('#overlay-svg');
  state.slider = document.querySelector('#sliderN');
  state.progressSpan = document.querySelector("#progress");
  state.locationSpan = document.querySelector("#location");

  // Settings
  state.settingsDiv = document.querySelector("#settings");
  state.inputShowControls = document.querySelector("#input-show-controls");
  state.inputObjectType = document.querySelector("#input-object-type");
  state.inputObjectCount = document.querySelector("#input-object-count");

  // Before setting up everything, load the current settings,
  // which may have been persisted from a previous execution.
  await getSettings();
  initializeSettingsListeners(state);

  // Initialize interactivity
  registerControlButtons();

  // TODO register only if available?
  registerKeyboard();
  registerTouch();

  registerTauriEvents();

  // UI ready, request image paths to be loaded.
  invoke('get_image_paths', { forceSelection: false, verbose: state.verbose });

  debug("Done.");
});
