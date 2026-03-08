const { invoke } = window.__TAURI__.core;

const dotAx = document.getElementById("dot-accessibility");
const dotInput = document.getElementById("dot-input");
const btnAx = document.getElementById("btn-accessibility");
const btnInput = document.getElementById("btn-input");
const btnContinue = document.getElementById("btn-continue");

let axGranted = false;
let inputGranted = false;

async function checkPermissions() {
  try {
    const perms = await invoke("get_permissions");
    axGranted = perms.accessibility;
    inputGranted = perms.input_monitoring;

    dotAx.classList.toggle("granted", axGranted);
    dotInput.classList.toggle("granted", inputGranted);
    btnContinue.disabled = !(axGranted && inputGranted);
  } catch (e) { console.warn("Permission check failed:", e); }
}

// Poll every 2 seconds
checkPermissions();
setInterval(checkPermissions, 2000);

btnAx.addEventListener("click", () => {
  invoke("open_accessibility_settings").catch(() => {});
});

btnInput.addEventListener("click", () => {
  invoke("open_input_monitoring_settings").catch(() => {});
});

btnContinue.addEventListener("click", async () => {
  await checkPermissions();
  if (axGranted && inputGranted) {
    await invoke("complete_onboarding");
  } else {
    btnContinue.classList.add("shake");
    btnContinue.addEventListener("animationend", () => {
      btnContinue.classList.remove("shake");
    }, { once: true });
  }
});
