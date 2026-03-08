const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

const modeEl = document.getElementById("mode");

function updateMode(mode) {
  modeEl.textContent = mode;
  modeEl.className = "mode-indicator";
  if (mode === "NORMAL") modeEl.classList.add("normal");
  else if (mode === "INSERT") modeEl.classList.add("insert");
  else modeEl.classList.add("visual");
}

// Get initial mode
invoke("get_mode").then(updateMode).catch(() => {});

// Listen for mode changes
listen("mode-changed", (event) => {
  updateMode(event.payload.mode);
});
