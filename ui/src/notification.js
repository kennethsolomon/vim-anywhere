const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

const toast = document.getElementById("notification-toast");
const message = document.getElementById("notification-message");
const btnExclude = document.getElementById("btn-exclude");
const btnDismiss = document.getElementById("btn-dismiss");

let dismissTimer = null;
let currentBundleId = null;

function dismiss() {
  if (dismissTimer) {
    clearTimeout(dismissTimer);
    dismissTimer = null;
  }
  toast.classList.remove("show-anim");
  toast.classList.add("hide-anim");
  toast.addEventListener("animationend", () => {
    toast.classList.remove("hide-anim");
    toast.classList.add("hidden");
  }, { once: true });
}

listen("show-notification", (event) => {
  const { app_name, bundle_id } = event.payload;

  currentBundleId = bundle_id;
  message.textContent = "Not supported in " + (app_name || "this app");

  // Clear pending dismiss
  if (dismissTimer) {
    clearTimeout(dismissTimer);
  }

  // Show toast
  toast.className = "notification-toast show-anim";

  // Auto-dismiss after 4 seconds
  dismissTimer = setTimeout(dismiss, 4000);
});

// Pause auto-dismiss on hover
toast.addEventListener("mouseenter", () => {
  if (dismissTimer) {
    clearTimeout(dismissTimer);
    dismissTimer = null;
  }
});

toast.addEventListener("mouseleave", () => {
  dismissTimer = setTimeout(dismiss, 2000);
});

btnExclude.addEventListener("click", () => {
  if (currentBundleId) {
    invoke("set_excluded_app", { bundleId: currentBundleId }).catch((e) => {
      console.warn("Failed to exclude app:", e);
    });
  }
  dismiss();
});

btnDismiss.addEventListener("click", dismiss);
