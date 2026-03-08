const { listen } = window.__TAURI__.event;

const dimLayer = document.getElementById("dim-layer");

// Clip-path polygon that covers the full screen EXCEPT the active window rectangle
function updateClipPath(x, y, w, h) {
  // Polygon: outer rectangle (screen) with inner cutout (window)
  // Screen corners: 0%,0% -> 100%,0% -> 100%,100% -> 0%,100%
  // Window cutout traced counterclockwise
  const sw = window.screen.width;
  const sh = window.screen.height;
  const left = (x / sw * 100).toFixed(2);
  const top = (y / sh * 100).toFixed(2);
  const right = ((x + w) / sw * 100).toFixed(2);
  const bottom = ((y + h) / sh * 100).toFixed(2);

  dimLayer.style.clipPath = `polygon(
    0% 0%, 100% 0%, 100% 100%, 0% 100%, 0% 0%,
    ${left}% ${top}%, ${left}% ${bottom}%, ${right}% ${bottom}%, ${right}% ${top}%, ${left}% ${top}%
  )`;
}

listen("focus-highlight-update", (event) => {
  const { visible, x, y, w, h } = event.payload;
  if (visible) {
    dimLayer.classList.remove("hidden");
    updateClipPath(x, y, w, h);
  } else {
    dimLayer.classList.add("hidden");
    dimLayer.style.clipPath = "";
  }
});
