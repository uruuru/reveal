const { invoke } = window.__TAURI__.core;
const { debug } = window.__TAURI__.log;

import { printDebug } from './utils.js';

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });

  let settings = await invoke("get_settings");
  console.log(settings);

  let revealObject = await invoke("example");
  document.querySelector("#image-container").src = `data:${revealObject.image_media_type};base64,${revealObject.image}`;
}

window.addEventListener("DOMContentLoaded", async () => {

  debug("Loading finished.");
  await printDebug();

});
