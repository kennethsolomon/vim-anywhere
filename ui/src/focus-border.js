const { listen } = window.__TAURI__.event;

const border = document.getElementById("focus-border");

listen("focus-highlight-update", (event) => {
  const { visible, mode } = event.payload;
  if (!visible) return;
  border.className = "focus-border";
  if (mode === "VISUAL" || mode === "V-LINE") {
    border.classList.add("visual");
  }
});
