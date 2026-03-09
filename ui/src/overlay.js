const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

const container = document.getElementById("overlay-container");
const modeEl = document.getElementById("mode");
const pendingEl = document.getElementById("pending-keys");

let positionMode = "bottom-right"; // default, updated from config

// Load position mode from config
invoke("load_config").then((cfg) => {
  if (cfg && cfg.overlay_position) {
    positionMode = cfg.overlay_position;
  }
}).catch(() => {});

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

// Listen for pending keys — show inline in near-cursor mode
listen("pending-keys-changed", (event) => {
  const keys = event.payload.keys || "";
  if (positionMode === "near-cursor" && keys) {
    pendingEl.textContent = " \u00b7 " + keys; // " · keys"
  } else {
    pendingEl.textContent = keys;
  }
});

// Listen for focus highlight updates to reposition in near-cursor mode
listen("focus-highlight-update", (event) => {
  if (positionMode !== "near-cursor") return;

  const { visible, x, y, w, h } = event.payload;
  if (!visible) return;

  // Position overlay at top-right of focus border, offset 8px outside
  const badgeW = 120;
  const badgeH = 36;
  const gap = 8;

  let newX = x + w + gap;
  let newY = y - badgeH - gap;

  // Flip if off-screen right (use screen width from window)
  const screenW = window.screen.width;
  const screenH = window.screen.height;

  if (newX + badgeW > screenW) {
    newX = x - badgeW - gap;
  }
  // Flip if off-screen top
  if (newY < 0) {
    newY = y + h + gap;
  }
  // Clamp to screen
  if (newX < 0) newX = gap;
  if (newY + badgeH > screenH) newY = screenH - badgeH - gap;

  const win = window.__TAURI__.window.getCurrentWindow();
  win.setPosition(new window.__TAURI__.window.LogicalPosition(newX, newY));
});

// Listen for overlay position changes from settings
listen("overlay-position-changed", (event) => {
  if (event.payload && event.payload.position) {
    positionMode = event.payload.position;
  }
});
