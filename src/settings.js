import { isMobile } from "./utils.js";

const { debug } = window.__TAURI__.log;

let settingsDirty = false;

function showControlButtons(show) {
  for (const element of document.querySelectorAll(".controls-optional")) {
    if (show) {
      element.style.display = "block";
    } else {
      element.style.display = "none";
    }
  }
}

async function loadSettings(state) {
  await state.store.get("show_controls").then((v) => {
    if (v !== undefined) {
      state.inputShowControls.checked = JSON.parse(v);
    } else {
      state.inputShowControls.checked = !isMobile();
    }
    showControlButtons(state.inputShowControls.checked);
  });

  await state.store.get("object_type").then((v) => {
    if (v !== undefined) {
      state.inputObjectType.value = v;
    } else {
      state.inputObjectType.value = "Triangles";
    }
  });

  await state.store.get("object_count").then((v) => {
    if (v !== undefined) {
      state.inputObjectCount.value = v;
    } else {
      state.inputObjectCount.value = 10;
    }
  });

  await state.store.get("verbose").then((v) => {
    if (v !== undefined) {
      state.inputVerbose.checked = JSON.parse(v);
    } else {
      state.inputVerbose.checked = true;
    }
  });

  await state.store.get("quiz_guess_year").then((v) => {
    if (v !== undefined) {
      state.inputQuizYear.checked = JSON.parse(v);
    } else {
      state.inputQuizYear.checked = false;
    }
  });

  debug(`Loaded initial settings: ${JSON.stringify(await state.store.entries(), null, "  ")}.`);
}

function initializeSettingsListeners(state) {
  // Register listeners to update settings based on user-interaction.
  state.inputShowControls.addEventListener("input", (e) => {
    showControlButtons(e.target.checked);
    state.store.set("show_controls", e.target.checked);
  });

  state.inputVerbose.addEventListener("input", (e) => {
    state.store.set("verbose", e.target.checked);
  });

  state.inputObjectType.addEventListener("input", (e) => {
    state.store.set("object_type", e.target.value);
    settingsDirty = true;
  });

  // Note: use 'change' event to not trigger while dragging
  state.inputObjectCount.addEventListener("change", (e) => {
    state.store.set("object_count", e.target.value);
    settingsDirty = true;
  });

  state.inputQuizYear.addEventListener("input", (e) => {
    state.store.set("quiz_guess_year", e.target.checked);
    settingsDirty = true;
  });
}

// Execute 'fun' if settings have been changed that require content updates.
function executeIfSettingsChanged(fun) {
  if (settingsDirty) {
    fun();
    settingsDirty = false;
  }
}

async function resetSettings(state) {
  await state.store.clear();
  await loadSettings(state);
}

export { executeIfSettingsChanged, initializeSettingsListeners, loadSettings, resetSettings };
