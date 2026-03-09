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
  } catch (e) { console.warn("Failed to get mode:", e); }

  await listen("mode-changed", (event) => {
    updateModeDisplay(event.payload.mode);
  });

  // ── Mode Entry ──────────────────────────────────────────────────────────
  const modeRadios = document.querySelectorAll('input[name="mode-entry"]');
  const customSeqInput = document.getElementById("custom-sequence");
  const controlBracketCheck = document.getElementById("control-bracket");

  // Populate from config
  if (config.mode_entry) {
    // Map config method to radio values
    let radioValue = "escape"; // default: smart escape
    if (config.mode_entry.double_escape_sends_real && !config.mode_entry.smart_escape) {
      radioValue = "double-escape";
    } else if (config.mode_entry.method === "custom") {
      radioValue = "custom";
    }
    modeRadios.forEach((r) => {
      r.checked = r.value === radioValue;
    });
    if (customSeqInput && config.mode_entry.custom_sequence) {
      customSeqInput.value = config.mode_entry.custom_sequence;
    }
  }

  async function saveModeEntry() {
    let selected = "escape";
    modeRadios.forEach((r) => {
      if (r.checked) selected = r.value;
    });
    // Map radio to config fields
    let method = selected === "custom" ? "custom" : "escape";
    let doubleEsc = selected === "double-escape";
    let smartEsc = selected === "escape";
    const customSeq = customSeqInput ? customSeqInput.value || null : null;
    await invoke("set_mode_entry", {
      method,
      customSequence: customSeq,
      doubleEscape: doubleEsc,
      smartEscape: smartEsc,
    });
  }

  modeRadios.forEach((r) => r.addEventListener("change", saveModeEntry));
  if (customSeqInput)
    customSeqInput.addEventListener("change", saveModeEntry);

  // ── Focus Highlight ────────────────────────────────────────────────────
  const focusHighlightCheck = document.getElementById("focus-highlight");
  const dimBackgroundCheck = document.getElementById("dim-background");
  const dimIntensitySelect = document.getElementById("dim-intensity");

  if (focusHighlightCheck && config.focus_highlight !== undefined) {
    focusHighlightCheck.checked = config.focus_highlight;
  }
  if (dimBackgroundCheck && config.dim_background !== undefined) {
    dimBackgroundCheck.checked = config.dim_background;
  }
  if (dimIntensitySelect && config.dim_intensity) {
    dimIntensitySelect.value = config.dim_intensity;
  }

  if (focusHighlightCheck) {
    focusHighlightCheck.addEventListener("change", async (e) => {
      await invoke("set_focus_highlight", { enabled: e.target.checked });
    });
  }
  if (dimBackgroundCheck) {
    dimBackgroundCheck.addEventListener("change", async (e) => {
      config.dim_background = e.target.checked;
      config.focus_highlight = focusHighlightCheck ? focusHighlightCheck.checked : true;
      await invoke("save_config_full", { config });
    });
  }
  if (dimIntensitySelect) {
    dimIntensitySelect.addEventListener("change", async (e) => {
      config.dim_intensity = e.target.value;
      await invoke("save_config_full", { config });
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

  // ── Show Overlay ───────────────────────────────────────────────────────
  const showOverlay = document.getElementById("show-overlay");
  if (showOverlay && config.show_overlay !== undefined) {
    showOverlay.checked = config.show_overlay;
  }
  if (showOverlay) {
    showOverlay.addEventListener("change", async (e) => {
      await invoke("set_show_overlay", { enabled: e.target.checked });
    });
  }

  // ── Overlay Size ─────────────────────────────────────────────────────
  const overlaySizeSelect = document.getElementById("overlay-size-select");
  if (overlaySizeSelect && config.overlay_size) {
    overlaySizeSelect.value = config.overlay_size;
  }
  if (overlaySizeSelect) {
    overlaySizeSelect.addEventListener("change", async (e) => {
      await invoke("set_overlay_size", { size: e.target.value });
    });
  }

  // ── Overlay Position ───────────────────────────────────────────────
  const overlayPosSelect = document.getElementById("overlay-position-select");
  if (overlayPosSelect && config.overlay_position) {
    overlayPosSelect.value = config.overlay_position;
  }
  if (overlayPosSelect) {
    overlayPosSelect.addEventListener("change", async (e) => {
      await invoke("set_overlay_position", { position: e.target.value });
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
      const modeSpan = document.createElement("span");
      modeSpan.textContent = m.mode;
      const fromSpan = document.createElement("span");
      fromSpan.className = "mono";
      fromSpan.textContent = m.from;
      const toSpan = document.createElement("span");
      toSpan.className = "mono";
      toSpan.textContent = m.to;
      const delBtn = document.createElement("button");
      delBtn.className = "btn-delete";
      delBtn.dataset.index = i;
      delBtn.title = "Remove";
      delBtn.textContent = "x";
      row.appendChild(modeSpan);
      row.appendChild(fromSpan);
      row.appendChild(toSpan);
      row.appendChild(delBtn);
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
      const nameSpan = document.createElement("span");
      nameSpan.title = app.bundle_id;
      nameSpan.textContent = app.name;
      const select = document.createElement("select");
      select.className = "select-field app-strategy-select";
      select.dataset.bundle = app.bundle_id;
      for (const [val, label] of [["accessibility", "Accessibility"], ["keyboard", "Keyboard"], ["disabled", "Disabled"]]) {
        const opt = document.createElement("option");
        opt.value = val;
        opt.textContent = label;
        if (app.strategy === label) opt.selected = true;
        select.appendChild(opt);
      }
      const statusSpan = document.createElement("span");
      const dot = document.createElement("span");
      dot.className = "status-dot " + app.status_class;
      statusSpan.appendChild(dot);
      statusSpan.appendChild(document.createTextNode(" " + app.status));
      row.appendChild(nameSpan);
      row.appendChild(select);
      row.appendChild(statusSpan);
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
        if (textNode) {
          textNode.textContent = "";
          const newDot = document.createElement("span");
          newDot.className = "status-dot";
          if (strategy === "accessibility") {
            newDot.classList.add("active");
            textNode.appendChild(newDot);
            textNode.appendChild(document.createTextNode(" supported"));
          } else if (strategy === "keyboard") {
            newDot.classList.add("partial");
            textNode.appendChild(newDot);
            textNode.appendChild(document.createTextNode(" partial"));
          } else {
            newDot.classList.add("inactive");
            textNode.appendChild(newDot);
            textNode.appendChild(document.createTextNode(" excluded"));
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
  } catch (e) { console.warn("Failed to get permissions:", e); }

  // ── Open Privacy Settings button ──────────────────────────────────────
  const openPrivacyBtn = document.getElementById("open-privacy-settings");
  if (openPrivacyBtn) {
    openPrivacyBtn.addEventListener("click", async () => {
      await invoke("open_privacy_settings");
    });
  }

  // ── Re-run Setup button ───────────────────────────────────────────────
  const reopenBtn = document.getElementById("reopen-onboarding");
  if (reopenBtn) {
    reopenBtn.addEventListener("click", async () => {
      await invoke("reopen_onboarding");
    });
  }

  // ── Excluded Apps ─────────────────────────────────────────────────────
  const excludedList = document.getElementById("excluded-apps-list");
  const addExcludedInput = document.getElementById("add-excluded-app");
  const addExcludedBtn = document.getElementById("btn-add-excluded");

  function renderExcludedApps(apps) {
    if (!excludedList) return;
    excludedList.innerHTML = "";
    if (!apps || apps.length === 0) {
      excludedList.innerHTML = '<span class="value-display">No excluded apps</span>';
      return;
    }
    for (const app of apps) {
      const row = document.createElement("div");
      row.className = "setting-row";
      const span = document.createElement("span");
      span.className = "mono";
      span.style.fontSize = "12px";
      span.textContent = app;
      const btn = document.createElement("button");
      btn.className = "btn-delete";
      btn.dataset.bundle = app;
      btn.title = "Remove";
      btn.textContent = "x";
      row.appendChild(span);
      row.appendChild(btn);
      excludedList.appendChild(row);
    }
    excludedList.querySelectorAll(".btn-delete").forEach((btn) => {
      btn.addEventListener("click", async (e) => {
        const bundleId = e.target.dataset.bundle;
        await invoke("remove_excluded_app", { bundleId });
        const updated = await invoke("load_config");
        renderExcludedApps(updated.excluded_apps);
      });
    });
  }

  renderExcludedApps(config.excluded_apps);

  if (addExcludedBtn && addExcludedInput) {
    addExcludedBtn.addEventListener("click", async () => {
      const bundleId = addExcludedInput.value.trim();
      if (bundleId) {
        await invoke("set_excluded_app", { bundleId });
        addExcludedInput.value = "";
        const updated = await invoke("load_config");
        renderExcludedApps(updated.excluded_apps);
      }
    });
  }

  // ── Global Toggle ──────────────────────────────────────────────────────
  const hotkeyDisplay = document.getElementById("hotkey-display");
  const recordBtn = document.getElementById("btn-record-hotkey");
  const toggleStatusDot = document.getElementById("toggle-status-dot");
  const toggleStatusText = document.getElementById("toggle-status-text");

  // Format hotkey string for display (ctrl-cmd-v → Ctrl+Cmd+V)
  function formatHotkey(hotkey) {
    if (!hotkey) return "Not set";
    return hotkey.split("-").map((part) => {
      if (part === "ctrl") return "Ctrl";
      if (part === "cmd") return "Cmd";
      if (part === "opt" || part === "option") return "Opt";
      if (part === "shift") return "Shift";
      return part.toUpperCase();
    }).join("+");
  }

  // Initialize hotkey display
  if (hotkeyDisplay && config.toggle_hotkey) {
    hotkeyDisplay.textContent = formatHotkey(config.toggle_hotkey);
  }

  // Initialize toggle status
  async function updateToggleStatus() {
    try {
      const enabled = await invoke("get_enabled");
      if (toggleStatusDot) {
        toggleStatusDot.className = "status-dot " + (enabled ? "active" : "inactive");
      }
      if (toggleStatusText) {
        toggleStatusText.textContent = enabled ? "Enabled" : "Disabled";
      }
    } catch (e) { console.warn("Failed to get enabled:", e); }
  }
  await updateToggleStatus();

  // Listen for toggle changes
  await listen("toggle-changed", (event) => {
    const enabled = event.payload.enabled;
    if (toggleStatusDot) {
      toggleStatusDot.className = "status-dot " + (enabled ? "active" : "inactive");
    }
    if (toggleStatusText) {
      toggleStatusText.textContent = enabled ? "Enabled" : "Disabled";
    }
  });

  // Hotkey recording
  if (recordBtn) {
    let recording = false;

    recordBtn.addEventListener("click", () => {
      if (recording) return;
      recording = true;
      recordBtn.textContent = "Press keys...";
      if (hotkeyDisplay) hotkeyDisplay.classList.add("recording");

      function onKeyDown(e) {
        e.preventDefault();
        e.stopPropagation();

        // Ignore modifier-only presses
        if (["Control", "Meta", "Alt", "Shift"].includes(e.key)) return;

        const parts = [];
        if (e.ctrlKey) parts.push("ctrl");
        if (e.metaKey) parts.push("cmd");
        if (e.altKey) parts.push("opt");
        if (e.shiftKey) parts.push("shift");

        // Get the key character
        let keyName = e.key.length === 1 ? e.key.toLowerCase() : e.key.toLowerCase();
        if (keyName === "escape") keyName = "escape";
        else if (keyName === "enter") keyName = "return";
        else if (keyName === "tab") keyName = "tab";
        else if (keyName === "backspace") keyName = "backspace";
        parts.push(keyName);

        const hotkey = parts.join("-");

        // Save and update display
        invoke("set_toggle_hotkey", { hotkey }).then(() => {
          if (hotkeyDisplay) {
            hotkeyDisplay.textContent = formatHotkey(hotkey);
            hotkeyDisplay.classList.remove("recording");
          }
        });

        recording = false;
        recordBtn.textContent = "Record...";
        document.removeEventListener("keydown", onKeyDown, true);
      }

      document.addEventListener("keydown", onKeyDown, true);
    });
  }
});
