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

}


async function getImage(u) {
  try {
    const revealObject = await invoke('example'); // invoke('load_image', { u: u });
    //state.image.setAttribute("hidden", "hidden");
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
}

async function executeAction(action_identifier) {
  debug(`Executing action ${action_identifier}`)
  switch (action_identifier) {
    case Action.uncover:
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
  state.svg = document.querySelector('#overlay-svg');

  // Settings
  state.settingsDiv = document.querySelector("#settings");

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
