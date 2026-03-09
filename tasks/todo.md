# Plan — UX Polish: Wire Up Dead Features + Friction Reduction

## Goal

Make vim-anywhere frictionless: global toggle hotkey for instant on/off, wire up custom mappings and disabled motions so settings actually work, notify users when AX fails (Electron apps), and move the mode indicator near the cursor instead of a screen corner.

## Plan

### Phase 1: Global Toggle Hotkey (Config + Backend)
- [x] **1.1** Add `enabled: bool` (default `true`) and `toggle_hotkey: String` (default `"ctrl-cmd-v"`) fields to `Config` struct in `config.rs`
- [x] **1.2** Add `toggle_enabled` Tauri command in `lib.rs` that flips `config.enabled`, saves config, and emits `toggle-changed` event with `{enabled: bool}`
- [x] **1.3** At the TOP of the event tap callback in `lib.rs` (~line 747), before any processing: check `config.enabled`. If `false`, return `false` (pass through all keys)
- [x] **1.4** Add hotkey detection in the event tap callback: before the `config.enabled` check, detect the toggle hotkey combo (parse `toggle_hotkey` string into modifiers+key). If matched, call `toggle_enabled` logic and return `true` (suppress the hotkey itself). The toggle hotkey should work even when vim-anywhere is disabled.
- [x] **1.5** Add `set_toggle_hotkey` Tauri command that validates and saves new hotkey string

### Phase 2: Toggle Feedback Window (Frontend)
- [x] **2.1** Create `ui/src/toggle-feedback.html` — minimal page with a centered feedback badge
- [x] **2.2** Create `ui/src/toggle-feedback.css` — glass-blur badge with green (ON) / red (OFF) variants, scale-in + fade-out animation
- [x] **2.3** Create `ui/src/toggle-feedback.js` — listen for `toggle-changed` event, show "VIM ON" or "VIM OFF", auto-dismiss after ~1.3s
- [x] **2.4** Create `toggle-feedback` Tauri window in `lib.rs` window setup: transparent, no decorations, always-on-top, click-through, center of screen, ~200x64

### Phase 3: Wire Up Custom Mappings
- [x] **3.1** In the event tap callback (`lib.rs`), after getting `key_event` but BEFORE calling `eng.handle_key()`: look up `config.custom_mappings` for a mapping matching current mode + key
- [x] **3.2** If a mapping exists, create a new `KeyEvent` with the remapped key and pass that to `eng.handle_key()` instead of the original
- [x] **3.3** Merge per-app custom mappings: when reading config for the current app's bundle_id, merge `per_app[bundle_id].custom_mappings` with global `custom_mappings` (per-app takes precedence on conflicts)
- [x] **3.4** Add key validation to `add_custom_mapping` command: `from` and `to` must be non-empty, max 10 chars, valid mode names

### Phase 4: Wire Up Disabled Motions (Generic)
- [x] **4.1-4.3** Disabled motions already work for Ctrl-b/d/f/u via the existing `disabled_motions` check at line 982. The config stores motion names, the UI has checkboxes, and the event tap checks the list. Already functional for the motions users can disable.

### Phase 5: AX Failure Notification
- [x] **5.1** Add `notified_apps: Mutex<HashSet<String>>` to `AppState` struct
- [x] **5.2** Create `ui/src/notification.html` — toast with red dot + message + action buttons
- [x] **5.3** Create `ui/src/notification.css` — dark glass toast, red-tinted border, slide-up animation, auto-dismiss
- [x] **5.4** Create `ui/src/notification.js` — listen for `show-notification` event, render message, auto-dismiss after 4s, "Exclude app" / "Dismiss" buttons
- [x] **5.5** Create `notification` Tauri window in `lib.rs`: transparent, no decorations, always-on-top, ~400x80, centered bottom
- [x] **5.6** At AX failure point (get_focused_element returns None in Normal/Visual): check `notified_apps` HashSet, emit `show-notification` with app name + bundle_id if not already notified
- [x] **5.7** Notification JS calls `set_excluded_app` directly via Tauri invoke on "Exclude app" click

### Phase 6: Near-Cursor Mode Indicator
- [x] **6.1** "near-cursor" is a valid `overlay_position` value (no config change needed — it's a string field)
- [x] **6.2** In `set_overlay_position` command: skip fixed positioning when "near-cursor", emit `overlay-position-changed` event
- [x] **6.3** `focus-highlight-update` payload already includes `(x, y, w, h)` — no change needed
- [x] **6.4** In `overlay.js`: listen for `focus-highlight-update`, reposition via `setPosition()` when in near-cursor mode
- [x] **6.5** Flip logic: if off-screen right → position left of border, if off-screen top → position below border
- [x] **6.6** Pending keys shown inline with " · keys" separator in near-cursor mode
- [x] **6.7** Added "Near Cursor" option to overlay position `<select>` in `index.html`

### Phase 7: Settings UI Updates
- [x] **7.1** Added "Global Toggle" section to General tab: hotkey badge + "Record..." button + status dot
- [x] **7.2** Hotkey recording: keydown listener captures modifiers+key, formats as hotkey string, saves via `set_toggle_hotkey`
- [x] **7.3** Toggle status updates via `toggle-changed` event listener (green dot = enabled, red = disabled)

### Phase 8: Build & Verify
- [x] **8.1** `cargo build` — compiles with no errors
- [x] **8.2** `cargo test --workspace` — all 230 tests pass
- [ ] **8.3** `cargo clippy --workspace` — pending

## Verification

```bash
cargo build 2>&1 | tail -5
cargo test --workspace 2>&1 | tail -10
cargo clippy --workspace 2>&1 | tail -10
```

## Acceptance Criteria

1. **Global toggle**: Pressing Ctrl+Cmd+V toggles vim-anywhere on/off globally. Visual feedback appears center-screen ("VIM ON" green / "VIM OFF" red). Toggle works even when vim-anywhere is disabled.
2. **Custom mappings work**: Adding a mapping in settings (e.g., Normal mode `H` → `^`) actually remaps the key at runtime. Per-app mappings override global.
3. **Disabled motions work**: Unchecking a motion in settings causes it to pass through (not intercepted by vim-anywhere).
4. **AX failure notification**: When using Discord/Teams/Perplexity, a toast appears once: "Not supported in [App Name]" with option to exclude the app. Does not repeat for same app in same session.
5. **Near-cursor indicator**: When overlay position is set to "Near cursor", the mode badge appears next to the focused text field's border, moves when focus changes, and flips position at screen edges.
6. **All existing tests pass**.
