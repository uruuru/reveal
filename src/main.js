const { invoke } = window.__TAURI__.core;
const { message } = window.__TAURI__.dialog;
const { debug, error } = window.__TAURI__.log;

import { printDebug } from './utils.js';

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

}

async function getSettings() {
  state.settings = await invoke('get_settings');
  updateSettingsUi();
}

function updateSettingsUi() {
  // To be called when settings changed in the background
  try {
    state.inputPath.value = state.settings.image_source;
    state.inputShowControls.checked = JSON.parse(state.settings.show_control_buttons);
    state.inputObjectType.value = state.settings.covering_type;
    state.inputObjectCount.value = state.settings.covering_object_count;
  } catch (e) {
    error(`Failed updating settings UI (${e}).`);
  }
}

async function setSettings() {
  // TODO send back to the backend
}

function initializeSettingsListeners(state) {
  // Slider
  state.inputObjectCount.oninput = function (e) {
    state.settings.covering_object_count = Number(state.inputObjectCount.value || 10);
    e.preventDefault();
    e.stopImmediatePropagation();

    // TODO maybe only reload on done button press?
    loadCovering();
  }

  state.inputPath.addEventListener('input', e => {
    state.inputPath.value = e.target.value;

    // TODO reload on done button press?
    // state.settingsPathChanged = true;
  });

  // Set default controls state
  let updateFun = event => {
    debug(`State ${state.inputShowControls.checked} ${event}`)
    const showControls = state.inputShowControls.checked;
    if (event) {
      state.settings.show_control_buttons = showControls;
      event.stopImmediatePropagation();
    }
    document.querySelectorAll(".controls-optional").forEach(element => {
      if (showControls) {
        element.style.display = 'block';
      } else {
        element.style.display = 'none';
      }
    });
  };
  // TODO still needed?
  updateFun();

  // 'click' fires after input checkbox state has changed
  // TODO rather use an onchange?
  state.inputShowControls.addEventListener("click", updateFun);
}

async function getImage(u) {
  try {
    const revealObject = await invoke('example'); // invoke('load_image', { u: u });
    state.image.setAttribute("hidden", "hidden");
    state.image.src = `data:image/${revealObject.image_type};base64,${revealObject.image}`;
    //state.imageIndex = (state.imageIndex + state.imageCount + u) % state.imageCount;
    //updateProgress();
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
  const n = state.settings.covering_object_count;
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
      break;
    case Action.settings:
      state.settingsDiv.style.display = 'inline';
      break;
    case Action.settings_done:
      state.settingsDiv.style.display = 'none';
      break;
    case Action.settings_reset:
      break;
    default:
  }
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
  state.inputPath = document.querySelector("#input-path");
  state.inputShowControls = document.querySelector("#input-show-controls");
  state.inputObjectType = document.querySelector("#input-object-type");
  state.inputObjectCount = document.querySelector("#input-object-count");

  // Before setting up everything, load the current settings,
  // which may have been persisted from a previous execution.
  await getSettings();
  initializeSettingsListeners(state);
  debug(`Loaded initial settings: ${JSON.stringify(state.settings, null, "  ")}.`);

  document.querySelectorAll(".control").forEach((button) => {
    button.addEventListener('pointerup', (e) => {
      let action = button.dataset.event.toLowerCase();
      executeAction(action);
      e.preventDefault();
      e.stopImmediatePropagation();
    });
  });

  debug("Done.");
});
