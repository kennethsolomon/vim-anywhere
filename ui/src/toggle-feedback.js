const { listen } = window.__TAURI__.event;

const badge = document.getElementById("feedback-badge");
const text = document.getElementById("feedback-text");

let dismissTimer = null;

listen("toggle-changed", (event) => {
  const enabled = event.payload.enabled;

  // Clear any pending dismiss
  if (dismissTimer) {
    clearTimeout(dismissTimer);
    dismissTimer = null;
  }

  // Set content and style
  text.textContent = enabled ? "VIM ON" : "VIM OFF";
  badge.className = "feedback-badge";
  badge.classList.add(enabled ? "on" : "off");
  badge.classList.add("show-anim");

  // Auto-dismiss after hold
  dismissTimer = setTimeout(() => {
    badge.classList.remove("show-anim");
    badge.classList.add("hide-anim");

    badge.addEventListener("animationend", () => {
      badge.classList.remove("hide-anim");
      badge.classList.add("hidden");
    }, { once: true });
  }, 800);
});
