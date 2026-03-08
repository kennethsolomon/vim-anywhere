# Seamless macOS Integration

## Goal

Fix Escape key behavior, scope vim-anywhere to text fields only, auto-detect focus changes, redesign mode overlay, add window focus highlight, and create an onboarding flow for permissions.

## Plan

### Phase 1: Smart Escape (core behavior fix)

The most critical fix. Single Escape = context-aware: exits modes, or passes through in Normal.

- [x] 1.1 `crates/core/src/config.rs`: Change default `double_escape_sends_real` to `false` (line 39, 69)
- [x] 1.2 `crates/core/src/config.rs`: Add `smart_escape: bool` field to `ModeEntryConfigJson` (default `true`)
- [x] 1.3 `crates/core/src/modes.rs`: Add `smart_escape: bool` to `ModeEntryConfig` struct (line 12-18), default `true`
- [x] 1.4 `crates/core/src/modes.rs`: Add `ModeTransition::PassThrough` variant to enum (line 34-39)
- [x] 1.5 `crates/core/src/modes.rs`: Update `handle_escape()` Normal mode branch — when `smart_escape` is true, return `ModeTransition::PassThrough` instead of `ModeTransition::None` (line 90-102)
- [x] 1.6 `src/lib.rs`: In `handle_key()` `ParsedCommand::Escape` branch, when `mode_sm.handle_escape()` returns `ModeTransition::PassThrough`, return `EngineResult::PassThrough` (line 64-70)
- [x] 1.7 `ui/src-tauri/src/lib.rs`: Update `config_to_mode_entry()` to map `smart_escape` field (line 26-42)
- [x] 1.8 `ui/src-tauri/src/lib.rs`: In Normal/Visual event handler — when no focused element (line 668-677), pass through ALL keys including Escape (return `false`)
- [x] 1.9 `ui/src-tauri/src/lib.rs`: When element is not editable (line 690-698), pass through ALL keys including Escape (return `false`)
- [x] 1.10 `ui/src-tauri/src/lib.rs`: When AX value read fails (line 701-711), pass through ALL keys including Escape (return `false`)
- [x] 1.11 Update existing tests in `modes.rs` for new smart escape default + PassThrough variant
- [x] 1.12 Verify: `cargo test` passes

### Phase 2: Terminal Emulator Exclusion

- [x] 2.1 `crates/platform-mac/src/app_detection.rs`: Add missing terminal bundle IDs to `default_strategy_for_app()`: `net.kovidgoyal.kitty`, `dev.warp.Warp-Stable`, `com.github.wez.wezterm`, `co.zeit.hyper` (line 67-72)
- [x] 2.2 `crates/core/src/config.rs`: Add `excluded_apps: Vec<String>` field to `Config` with serde default containing the known terminal list
- [x] 2.3 `ui/src-tauri/src/lib.rs`: In event handler (before line 590), check `excluded_apps` from config — if frontmost app bundle_id is in the list, return `false` immediately
- [x] 2.4 Verify: `cargo build` succeeds

### Phase 3: Auto-Insert on Text Field Focus

- [x] 3.1 `crates/core/src/modes.rs`: Change `ModeStateMachine::new()` to start in `Mode::Insert` instead of `Mode::Normal` (line 50-51)
- [x] 3.2 `crates/core/src/modes.rs`: Add `reset_to_insert(&mut self)` method — sets mode to Insert, clears pending state
- [x] 3.3 `src/lib.rs`: Add `Engine::reset_to_insert()` method that calls `mode_sm.reset_to_insert()` and `parser.reset()`
- [x] 3.4 `ui/src-tauri/src/lib.rs`: Add `last_focused_element: Mutex<Option<(String, usize)>>` to `AppState` (bundle_id + element pointer hash)
- [x] 3.5 `ui/src-tauri/src/lib.rs`: In Normal/Visual handler, after getting the focused element, compare with last focused — if different, call `engine.reset_to_insert()`, emit `mode-changed` Insert, update `last_focused_element`, return `false`
- [x] 3.6 `ui/src-tauri/src/lib.rs`: Update overlay initial visibility to hidden (starts in Insert mode)
- [x] 3.7 Update tests in `modes.rs` that assume Normal as starting mode
- [x] 3.8 Verify: `cargo test` passes

### Phase 4: Mode Overlay Redesign

- [x] 4.1 `ui/src/overlay.html`: Add `<div id="pending-keys">` row below mode text
- [x] 4.2 `ui/src/overlay.css`: Restructure — wider pill to accommodate pending keys, add `.hidden` class with `opacity: 0; transform: scale(0.85)`, add mode-specific backgrounds (Normal=#00FF9F@85%, Visual=#FFAA00@85%), add `@keyframes flash` for brightness pulse, pending key row styles (10px, dimmer color)
- [x] 4.3 `ui/src/overlay.js`: On `mode-changed`, add/remove `.hidden` class (hidden for INSERT, visible for NORMAL/VISUAL), apply flash animation on transition
- [x] 4.4 `ui/src/overlay.js`: Listen for new `pending-keys-changed` event, update `#pending-keys` text content
- [x] 4.5 `ui/src-tauri/src/lib.rs`: After each key processing in Normal/Visual mode, emit `pending-keys-changed` event with payload from `parser.pending_keys()`
- [x] 4.6 `ui/src-tauri/src/lib.rs`: On mode change, show/hide overlay window via `overlay.show()` / `overlay.hide()` — show for Normal/Visual, hide for Insert
- [x] 4.7 Update overlay window size in setup to ~120x36 to accommodate pending keys
- [x] 4.8 Verify: builds and all tests pass

### Phase 5: Window Focus Highlight

- [x] 5.1 `crates/platform-mac/src/accessibility.rs`: Add `get_focused_window_frame() -> Option<(f64, f64, f64, f64)>` — traverse from focused element up to window, query AXPosition + AXSize
- [x] 5.2 `ui/src/dim-overlay.html` + `ui/src/dim-overlay.css` + `ui/src/dim-overlay.js`: Fullscreen dim layer with `rgba(0,0,0,0.15)` background, CSS clip-path polygon excluding active window rectangle, listens for `focus-highlight-update` event
- [x] 5.3 `ui/src/focus-border.html` + `ui/src/focus-border.css` + `ui/src/focus-border.js`: Transparent window with 2px border (mode-colored), 10px border-radius, subtle box-shadow glow, listens for `focus-highlight-update` event for color/visibility
- [x] 5.4 `ui/src-tauri/src/lib.rs`: Create `dim-overlay` window in setup — fullscreen, transparent, click-through, always-on-top, no decorations, starts hidden, all Spaces
- [x] 5.5 `ui/src-tauri/src/lib.rs`: Create `focus-border` window in setup — transparent, click-through, always-on-top, no decorations, starts hidden, all Spaces
- [x] 5.6 `ui/src-tauri/src/lib.rs`: On mode change to Normal/Visual — query `get_focused_window_frame()`, position `focus-border` window to match, show both dim + border windows, emit `focus-highlight-update` event with `{x, y, w, h, mode, visible: true}`
- [x] 5.7 `ui/src-tauri/src/lib.rs`: On mode change to Insert — hide both windows, emit `focus-highlight-update` with `{visible: false}`
- [x] 5.8 `ui/src-tauri/src/lib.rs`: Gate on `config.focus_highlight` — skip if disabled
- [x] 5.9 `crates/core/src/config.rs`: Add `dim_background: bool` (default `true`) and `dim_intensity: String` (default `"light"`) fields
- [x] 5.10 Verify: builds and all tests pass

### Phase 6: Onboarding Flow

- [x] 6.1 `crates/core/src/config.rs`: Add `onboarding_complete: bool` field (default `false`)
- [x] 6.2 `ui/src/onboarding.html`: Permission stepper — two cards (Accessibility + Input Monitoring) with granted/required states, "How It Works" section, "Verify & Continue" button
- [x] 6.3 `ui/src/onboarding.css`: Dark theme matching app aesthetic, step cards with status dots (green=granted, amber=required), button states, animations (dot transitions, shake on verify fail)
- [x] 6.4 `ui/src/onboarding.js`: Poll `get_permissions()` every 2s for live dot updates, "Open System Settings" buttons for each permission, "Verify & Continue" checks both permissions then calls `complete_onboarding`
- [x] 6.5 `ui/src-tauri/src/lib.rs`: Add `complete_onboarding` command — sets `onboarding_complete = true` in config, saves, closes onboarding window
- [x] 6.6 `ui/src-tauri/src/lib.rs`: Add `open_input_monitoring_settings` command + `open_accessibility_settings`
- [x] 6.7 `ui/src-tauri/src/lib.rs`: In setup, if `!config.onboarding_complete` or permissions missing, create and show onboarding window (480x560, centered, non-resizable)
- [x] 6.8 Register `complete_onboarding`, `open_accessibility_settings`, and `open_input_monitoring_settings` in `invoke_handler`
- [x] 6.9 Verify: builds and all tests pass

### Phase 7: Settings Window Updates

- [x] 7.1 `ui/src/index.html`: Replace Mode Entry section — Smart Escape radio (recommended, default), Classic Double-Escape radio, Custom Sequence Only radio, Control-[ checkbox
- [x] 7.2 `ui/src/index.html`: Add Focus section — highlight toggle, dim toggle, dim intensity dropdown
- [x] 7.3 `ui/src/index.html`: Add Terminal Exclusion section in Apps tab — checkboxes for known terminals + "Add custom" input
- [x] 7.4 `ui/src/index.html`: Add "Re-run Setup" button in About tab
- [x] 7.5 `ui/src/main.js`: Wire new settings — smart_escape radio saves mode entry, focus settings save to config, terminal exclusion management
- [x] 7.6 `ui/src-tauri/src/lib.rs`: Add commands: `set_excluded_app`, `remove_excluded_app`, `reopen_onboarding`
- [x] 7.7 Register new commands in `invoke_handler`
- [x] 7.8 Verify: builds and all tests pass

## Verification

```bash
# Build
cargo build

# Unit tests
cargo test

# Full app
cd ui && npm run tauri dev
```

### Manual Test Checklist
- [ ] Single Escape in Insert → Normal
- [ ] Single Escape in Normal → passes through to app (closes dialog/search)
- [ ] Typing in text field works immediately (auto-Insert)
- [ ] Switching text fields resets to Insert
- [ ] Terminals (Kitty, iTerm) fully excluded
- [ ] Non-text elements pass through all keys
- [ ] Overlay hidden in Insert, visible in Normal/Visual
- [ ] Overlay shows pending keys
- [ ] Window border glow in Normal/Visual
- [ ] Screen dim in Normal/Visual
- [ ] Border + dim gone in Insert
- [ ] Onboarding on first launch
- [ ] Permission dots update live
- [ ] Settings show Smart Escape, Focus, Terminal Exclusion

## Acceptance Criteria

1. Single Escape always works: exits modes or passes through in Normal
2. Typing in any text field works immediately without pressing `i`
3. Terminal emulators never intercepted
4. Non-text contexts never intercepted
5. Mode overlay hidden during typing, visible when Vim active
6. Window focus highlight shows which window Vim controls
7. First-launch onboarding guides through permissions
8. All existing tests pass (with updates for new defaults)

## Risks / Unknowns

- **AX window frame queries**: `get_focused_window_frame()` needs to traverse from focused element up to the window element to get AXPosition/AXSize. May need `AXUIElementCopyAttributeValue` with `kAXWindowAttribute` or parent traversal.
- **Focus change detection**: Comparing `(bundle_id, element_ptr)` between keystrokes is simple but only triggers on the next keystroke, not on click. Acceptable for now — true AX observer would be a future enhancement.
- **Dim overlay clip-path**: CSS polygon clip-path needs dynamic vertex updates. Multi-monitor support may require per-monitor dim windows.
- **Tauri multi-window**: 5 windows (main, overlay, dim, border, onboarding). Verify macOS handles this without visual artifacts.
- **Test updates**: Starting in Insert mode instead of Normal changes many test assumptions — systematic update needed.
