const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", async () => {
  // ── Tab switching ───────────────────────────────────────────────────────
  const tabs = document.querySelectorAll(".tab");
  const contents = document.querySelectorAll(".tab-content");
  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      tabs.forEach((t) => t.classList.remove("active"));
      contents.forEach((c) => c.classList.remove("active"));
      tab.classList.add("active");
      const target = document.getElementById("tab-" + tab.dataset.tab);
      if (target) target.classList.add("active");
    });
  });

  // ── Load config from backend ────────────────────────────────────────────
  let config = {};
  try {
    config = await invoke("load_config");
  } catch (e) {
    console.warn("Could not load config:", e);
  }

  // ── Mode display ────────────────────────────────────────────────────────
  const modeDisplay = document.getElementById("mode-display");

  function updateModeDisplay(mode) {
    if (!modeDisplay) return;
    modeDisplay.textContent = mode;
    modeDisplay.className = "mode-badge";
    if (mode === "NORMAL") modeDisplay.classList.add("normal");
    else if (mode === "INSERT") modeDisplay.classList.add("insert");
    else if (mode === "VISUAL" || mode === "V-LINE")
      modeDisplay.classList.add("visual");
  }

  try {
    const mode = await invoke("get_mode");
    updateModeDisplay(mode);
  } catch (e) {}

  await listen("mode-changed", (event) => {
    updateModeDisplay(event.payload.mode);
  });

  // ── Mode Entry ──────────────────────────────────────────────────────────
  const modeRadios = document.querySelectorAll('input[name="mode-entry"]');
  const customSeqInput = document.getElementById("custom-sequence");
  const doubleEscCheck = document.getElementById("double-esc");

  // Populate from config
  if (config.mode_entry) {
    modeRadios.forEach((r) => {
      r.checked = r.value === config.mode_entry.method;
    });
    if (customSeqInput && config.mode_entry.custom_sequence) {
      customSeqInput.value = config.mode_entry.custom_sequence;
    }
    if (doubleEscCheck) {
      doubleEscCheck.checked = config.mode_entry.double_escape_sends_real;
    }
  }

  async function saveModeEntry() {
    let method = "escape";
    modeRadios.forEach((r) => {
      if (r.checked) method = r.value;
    });
    const customSeq = customSeqInput ? customSeqInput.value || null : null;
    const doubleEsc = doubleEscCheck ? doubleEscCheck.checked : true;
    await invoke("set_mode_entry", {
      method,
      customSequence: customSeq,
      doubleEscape: doubleEsc,
    });
  }

  modeRadios.forEach((r) => r.addEventListener("change", saveModeEntry));
  if (customSeqInput)
    customSeqInput.addEventListener("change", saveModeEntry);
  if (doubleEscCheck)
    doubleEscCheck.addEventListener("change", saveModeEntry);

  // ── Overlay Size ────────────────────────────────────────────────────────
  const overlaySize = document.getElementById("overlay-size");
  if (overlaySize && config.overlay_size) {
    overlaySize.value = config.overlay_size;
  }
  if (overlaySize) {
    overlaySize.addEventListener("change", async (e) => {
      await invoke("set_overlay_size", { size: e.target.value });
    });
  }

  // ── Focus Highlight ─────────────────────────────────────────────────────
  const focusHighlight = document.getElementById("focus-highlight");
  if (focusHighlight && config.focus_highlight !== undefined) {
    focusHighlight.checked = config.focus_highlight;
  }
  if (focusHighlight) {
    focusHighlight.addEventListener("change", async (e) => {
      await invoke("set_focus_highlight", { enabled: e.target.checked });
    });
  }

  // ── Theme ───────────────────────────────────────────────────────────────
  const themeSelect = document.getElementById("theme-select");
  if (themeSelect && config.theme) {
    themeSelect.value = config.theme;
    applyTheme(config.theme);
  }
  if (themeSelect) {
    themeSelect.addEventListener("change", async (e) => {
      const theme = e.target.value;
      applyTheme(theme);
      await invoke("set_theme", { theme });
    });
  }

  function applyTheme(theme) {
    if (theme === "system") {
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)"
      ).matches;
      document.documentElement.setAttribute(
        "data-theme",
        prefersDark ? "dark" : "light"
      );
    } else {
      document.documentElement.setAttribute("data-theme", theme);
    }
  }

  // ── Launch at Login ─────────────────────────────────────────────────────
  const launchLogin = document.getElementById("launch-login");
  if (launchLogin && config.launch_at_login !== undefined) {
    launchLogin.checked = config.launch_at_login;
  }
  if (launchLogin) {
    launchLogin.addEventListener("change", async (e) => {
      await invoke("set_launch_at_login", { enabled: e.target.checked });
    });
  }

  // ── Menu Bar Icon ───────────────────────────────────────────────────────
  const menuBarIcon = document.getElementById("menu-bar-icon");
  if (menuBarIcon && config.menu_bar_icon !== undefined) {
    menuBarIcon.checked = config.menu_bar_icon;
  }
  if (menuBarIcon) {
    menuBarIcon.addEventListener("change", async (e) => {
      await invoke("set_menu_bar_icon", { enabled: e.target.checked });
    });
  }

  // ── Custom Mappings ─────────────────────────────────────────────────────
  const mappingsList = document.getElementById("mappings-list");
  const addMappingBtn = document.getElementById("add-mapping");

  function renderMappings(mappings) {
    if (!mappingsList) return;
    mappingsList.innerHTML = "";

    if (!mappings || mappings.length === 0) {
      mappingsList.innerHTML =
        '<span class="value-display">No custom mappings</span>';
      return;
    }

    for (let i = 0; i < mappings.length; i++) {
      const m = mappings[i];
      const row = document.createElement("div");
      row.className = "mapping-row";
      row.innerHTML = `
        <span>${m.mode}</span>
        <span class="mono">${m.from}</span>
        <span class="mono">${m.to}</span>
        <button class="btn-delete" data-index="${i}" title="Remove">x</button>
      `;
      mappingsList.appendChild(row);
    }

    // Attach delete handlers
    mappingsList.querySelectorAll(".btn-delete").forEach((btn) => {
      btn.addEventListener("click", async (e) => {
        const idx = parseInt(e.target.dataset.index);
        await invoke("remove_custom_mapping", { index: idx });
        const updated = await invoke("load_config");
        renderMappings(updated.custom_mappings);
      });
    });
  }

  renderMappings(config.custom_mappings);

  if (addMappingBtn) {
    addMappingBtn.addEventListener("click", () => {
      // Show inline add form
      if (document.getElementById("mapping-add-form")) return;

      const form = document.createElement("div");
      form.id = "mapping-add-form";
      form.className = "mapping-row";
      form.innerHTML = `
        <select class="select-field" id="new-map-mode">
          <option value="normal">Normal</option>
          <option value="insert">Insert</option>
          <option value="visual">Visual</option>
        </select>
        <input type="text" class="input-field input-small" id="new-map-from" placeholder="from" maxlength="10" />
        <input type="text" class="input-field input-small" id="new-map-to" placeholder="to" maxlength="10" />
        <button class="btn-primary" id="new-map-save" style="padding:4px 8px;font-size:11px">Add</button>
      `;
      addMappingBtn.parentNode.insertBefore(form, addMappingBtn);

      document
        .getElementById("new-map-save")
        .addEventListener("click", async () => {
          const mode = document.getElementById("new-map-mode").value;
          const from = document.getElementById("new-map-from").value.trim();
          const to = document.getElementById("new-map-to").value.trim();
          if (from && to) {
            await invoke("add_custom_mapping", { mode, from, to });
            const updated = await invoke("load_config");
            renderMappings(updated.custom_mappings);
          }
          form.remove();
        });
    });
  }

  // ── Disabled Motions ────────────────────────────────────────────────────
  const motionCheckboxes = document.querySelectorAll(
    "#tab-keys .setting-row .checkbox-label input[type='checkbox']"
  );
  const motionKeys = ["ctrl-b", "ctrl-f", "ctrl-d", "ctrl-u"];

  motionCheckboxes.forEach((cb, i) => {
    if (i >= motionKeys.length) return;
    const motionKey = motionKeys[i];

    // If motion is in disabled list, uncheck it
    if (config.disabled_motions && config.disabled_motions.includes(motionKey)) {
      cb.checked = false;
    }

    cb.addEventListener("change", async (e) => {
      // checked = enabled (not disabled), unchecked = disabled
      await invoke("set_disabled_motion", {
        motion: motionKey,
        disabled: !e.target.checked,
      });
    });
  });

  // ── App Filter ──────────────────────────────────────────────────────────
  const appFilter = document.getElementById("app-filter");
  if (appFilter) {
    appFilter.addEventListener("input", (e) => {
      const query = e.target.value.toLowerCase();
      const rows = document.querySelectorAll(".app-table .app-row");
      rows.forEach((row) => {
        const name = row.querySelector("span").textContent.toLowerCase();
        row.style.display = name.includes(query) ? "" : "none";
      });
    });
  }

  // ── Wizard ──────────────────────────────────────────────────────────────
  const wizardBtn = document.getElementById("run-wizard");
  const appTable = document.querySelector(".app-table");

  function renderAppRows(apps) {
    if (!appTable) return;
    const header = appTable.querySelector(".app-header");
    appTable.innerHTML = "";
    appTable.appendChild(header);

    for (const app of apps) {
      const row = document.createElement("div");
      row.className = "app-row";
      row.innerHTML = `
        <span title="${app.bundle_id}">${app.name}</span>
        <select class="select-field app-strategy-select" data-bundle="${app.bundle_id}">
          <option value="accessibility" ${app.strategy === "Accessibility" ? "selected" : ""}>Accessibility</option>
          <option value="keyboard" ${app.strategy === "Keyboard" ? "selected" : ""}>Keyboard</option>
          <option value="disabled" ${app.strategy === "Disabled" ? "selected" : ""}>Disabled</option>
        </select>
        <span><span class="status-dot ${app.status_class}"></span> ${app.status}</span>
      `;
      appTable.appendChild(row);
    }

    // Strategy change handlers
    appTable.querySelectorAll(".app-strategy-select").forEach((sel) => {
      sel.addEventListener("change", async (e) => {
        const bundleId = e.target.dataset.bundle;
        const strategy = e.target.value;
        await invoke("set_app_strategy", { bundleId, strategy });
        // Update status dot
        const statusSpan = e.target.parentElement.querySelector(
          "span:last-child .status-dot"
        );
        const textNode = e.target.parentElement.querySelector("span:last-child");
        if (statusSpan) {
          statusSpan.className = "status-dot";
          if (strategy === "accessibility") {
            statusSpan.classList.add("active");
            textNode.innerHTML = `<span class="status-dot active"></span> supported`;
          } else if (strategy === "keyboard") {
            statusSpan.classList.add("partial");
            textNode.innerHTML = `<span class="status-dot partial"></span> partial`;
          } else {
            statusSpan.classList.add("inactive");
            textNode.innerHTML = `<span class="status-dot inactive"></span> excluded`;
          }
        }
      });
    });
  }

  if (wizardBtn) {
    wizardBtn.addEventListener("click", async () => {
      wizardBtn.disabled = true;
      wizardBtn.textContent = "Scanning...";
      try {
        const apps = await invoke("run_wizard");
        renderAppRows(apps);
      } catch (e) {
        console.error("Wizard failed:", e);
      }
      wizardBtn.disabled = false;
      wizardBtn.textContent = "Run The Wizard";
    });
  }

  // ── Permissions ─────────────────────────────────────────────────────────
  try {
    const perms = await invoke("get_permissions");
    const axDot = document.getElementById("ax-status");
    const imDot = document.getElementById("im-status");

    if (axDot) {
      axDot.querySelector(".status-dot").className =
        "status-dot " + (perms.accessibility ? "active" : "inactive");
      axDot.querySelector(".perm-text").textContent = perms.accessibility
        ? "granted"
        : "not granted";
    }
    if (imDot) {
      imDot.querySelector(".status-dot").className =
        "status-dot " + (perms.input_monitoring ? "active" : "inactive");
      imDot.querySelector(".perm-text").textContent = perms.input_monitoring
        ? "granted"
        : "not granted";
    }
  } catch (e) {}

  // ── Open Privacy Settings button ──────────────────────────────────────
  const openPrivacyBtn = document.getElementById("open-privacy-settings");
  if (openPrivacyBtn) {
    openPrivacyBtn.addEventListener("click", async () => {
      await invoke("open_privacy_settings");
    });
  }
});
