const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

const container = document.getElementById("overlay-container");
const modeEl = document.getElementById("mode");
const pendingEl = document.getElementById("pending-keys");

function updateMode(mode) {
  modeEl.textContent = mode;
  modeEl.className = "mode-indicator";
  if (mode === "NORMAL") modeEl.classList.add("normal");
  else if (mode === "INSERT") modeEl.classList.add("insert");
  else modeEl.classList.add("visual");

  // Show for Normal/Visual, hide for Insert
  if (mode === "INSERT") {
    container.classList.add("hidden");
  } else {
    container.classList.remove("hidden");
    // Flash on transition
    modeEl.classList.add("flash");
    modeEl.addEventListener("animationend", () => {
      modeEl.classList.remove("flash");
    }, { once: true });
  }
}

// Get initial mode
invoke("get_mode").then(updateMode).catch((e) => console.warn("Failed to get mode:", e));

// Listen for mode changes
listen("mode-changed", (event) => {
  updateMode(event.payload.mode);
});

// Listen for pending keys
listen("pending-keys-changed", (event) => {
  pendingEl.textContent = event.payload.keys || "";
});
