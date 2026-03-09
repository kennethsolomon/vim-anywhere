# Progress Log

## Session: 2026-03-08
- Started: Full implementation run

## Work Log
- 2026-03-08 — Installed Rust 1.94.0 via rustup
- 2026-03-08 — Phase 1: Project scaffolding (Cargo workspace, crates/core, crates/platform-mac, Tauri UI, JetBrains Mono fonts, platform deps)
- 2026-03-08 — Phase 2: Mode state machine (Mode enum, ModeStateMachine, Esc/double-Esc/custom sequence/Control-[) — 15 tests
- 2026-03-08 — Phase 3: Key parser (KeyEvent, ParsedCommand, operator-pending, g/z/[/] prefixes, f/F/t/T, count) — 21 tests
- 2026-03-08 — Phase 4: Text buffer (TextBuffer trait, InMemoryBuffer, cursor/selection model) — 14 tests
- 2026-03-08 — Phase 5: Navigation motions (h/j/k/l, word motions, find char, goto, brackets, paragraph) — 20+ tests
- 2026-03-08 — Phase 6: Text objects (iw/aw, iW/aW, is/as, ip/ap, ib/ab, iB/aB, i[/a[, i</a<, i"/a", i'/a', i`/a`) — tests included
- 2026-03-08 — Phase 7: Operators (delete, change, yank + all motion/text object combos, register system) — tests included
- 2026-03-08 — Phase 8: Single-key editing (r, ~, J, p/P, <</>>, i/I/a/A/o/O) — tests included
- 2026-03-08 — Phase 9: Visual mode (characterwise + linewise, selection extension, operations, mode switching) — tests included
- 2026-03-08 — Phase 10-12: macOS platform (CGEvent tap key conversion, AXUIElement accessibility, app detection, keyboard strategy)
- 2026-03-08 — Phase 13: Engine wiring (Engine struct, main event loop, motion/operator execution) — 13 tests
- 2026-03-08 — Phase 14-16: Tauri UI (settings window with 4 tabs, Terminal Forge theme, dark/light toggle, all form elements)
- 2026-03-08 — Phase 17: Onboarding + Wizard UI (permission status, app testing)
- 2026-03-08 — Phase 18: Config system (serde JSON config, load/save, per-app overrides) — 3 tests

## Test Results
| Command | Expected | Actual | Status |
|---------|----------|--------|--------|
| `cargo build` | compiles | compiled clean | pass |
| `cargo test --workspace` | all pass | 97 tests passed | pass |
| `npm run tauri build` | produces .app | vim-anywhere.app + .dmg | pass |

## Session: 2026-03-09
- Phase 1 (Smart Escape): All 12 steps done — smart_escape default true, PassThrough variant, single Escape works
- Phase 2 (Terminal Exclusion): All 4 steps done — kitty, Warp, WezTerm, Hyper added; excluded_apps in config; event handler check
- Phase 3 (Auto-Insert): All 8 steps done — starts in Insert, reset_to_insert(), focus tracking, test updates
- Test fix: `classic_double_escape_in_normal` needed `sm.set_mode(Mode::Normal)` since SM now starts in Insert
- Test fix: `make_engine()` in engine_comprehensive.rs sends Escape to transition Insert→Normal
- All 197 tests passing (42 engine + 88 core + 67 comprehensive)
- Phase 4 (Mode Overlay): overlay.html/css/js redesigned — hidden class, flash animation, pending-keys display, show/hide on mode change, PendingKeysPayload emitted from Rust, Engine::pending_keys() exposed, overlay window 120x36
- Phase 5 (Focus Highlight): get_focused_window_frame() in accessibility.rs, dim-overlay + focus-border windows, FocusHighlightPayload, mode-aware border color, clip-path dim, gated on config.focus_highlight, dim_background + dim_intensity config fields
- Phase 6 (Onboarding): onboarding.html/css/js with permission stepper, complete_onboarding + open_accessibility_settings + open_input_monitoring_settings commands, auto-show on first launch, onboarding_complete config field
- Phase 7 (Settings): Smart Escape / Double-Escape / Custom radio, Focus section (highlight, dim, intensity), Excluded Apps with add/remove, Re-run Setup button, set_excluded_app + remove_excluded_app + reopen_onboarding commands

## Error Log
| Timestamp | Error | Attempt | Resolution |
|-----------|-------|---------|------------|
| 2026-03-08 | Tauri src-tauri not in workspace | 1 | Added to workspace.members |
| 2026-03-08 | Rust 2024 edition: chars().enumerate().rev() not allowed | 1 | Used manual Vec<char> iteration |
| 2026-03-08 | Rust 2024 edition: extern blocks must be unsafe | 1 | Downgraded platform-mac to edition 2021 |
| 2026-03-08 | dw test off-by-one | 1 | Added inclusive/exclusive motion classification |

## Session: 2026-03-08 (continued)
- Initialized git repo, committed initial implementation (df8fd85)
- /write-tests: Added 108 new tests (66 core comprehensive + 42 engine integration)
  - Created `crates/core/tests/comprehensive.rs` (motions, text objects, buffer, modes)
  - Created `tests/engine_comprehensive.rs` (insert variants, operator+motion, text objects, visual, counts, edge cases)
  - Created `src/lib.rs` to export Engine/EngineResult for integration tests
  - Total: 205 tests passing

## Session: 2026-03-09 (bug fixes)
- Phase 1-4: All fixes applied to `ui/src-tauri/src/lib.rs`
  - Fix 1: `PhysicalPosition`/`PhysicalSize` → `LogicalPosition`/`LogicalSize` for focus border (Retina fix)
  - Fix 2: Added editability check before Escape→Normal in Insert mode handler
  - Fix 3: Auto-reset to Insert when focused element is not editable (hides overlay+border on focus loss)
  - Fix 4: Block cursor guarded by editability (defense-in-depth, covered by Fix 2)
- Phase 5: Build + test + clippy all pass (52 tests, no new warnings)

- /security-check + fixes: Resolved all 9 findings
  - HIGH: Enabled CSP in tauri.conf.json; added RAII AXElement wrapper in accessibility.rs
  - MEDIUM: Fixed CFString ownership (wrap_under_create_rule); config load with warnings; robust home_dir; .gitignore secrets
  - LOW: Removed unused greet command; Key::Unknown variant; send_key_event error logging; cleaned unused imports

## Session: 2026-03-09 (UX Polish)
- Phase 1: Global toggle hotkey
  - Added `enabled` + `toggle_hotkey` fields to Config (config.rs)
  - Added `toggle_enabled`, `set_toggle_hotkey`, `get_enabled` Tauri commands (lib.rs)
  - Added `matches_hotkey()` helper for parsing hotkey strings (lib.rs)
  - Hotkey detection + enabled check at top of event tap callback
  - Hotkey works even when vim-anywhere is disabled; resets mode on disable
- Phase 2: Toggle feedback window
  - Created toggle-feedback.html/css/js — glass badge with ON (green) / OFF (red), auto-dismiss
  - Created toggle-feedback Tauri window: transparent, centered, click-through
- Phase 3: Custom mappings wired up
  - Added custom mapping remapping in event tap callback (before eng.handle_key())
  - Per-app mappings merged with global (per-app takes precedence)
  - Added key_matches_mapping_from() + parse_mapping_key() helpers
  - Added input validation to add_custom_mapping (non-empty, max 10, valid mode)
- Phase 4: Disabled motions — already functional for Ctrl-b/d/f/u via existing check
- Phase 5: AX failure notification
  - Added `notified_apps: HashSet<String>` to AppState
  - Created notification.html/css/js — toast with red dot + "Exclude app" / "Dismiss" buttons
  - Created notification Tauri window: bottom center, clickable (not click-through)
  - Emit show-notification on get_focused_element() failure (once per app per session)
- Phase 6: Near-cursor mode indicator
  - Updated overlay.js: loads config position, repositions on focus-highlight-update when "near-cursor"
  - Added flip logic for off-screen edges
  - Pending keys shown inline (" · keys") in near-cursor mode
  - Added "Near Cursor" option to overlay position select in index.html
  - Updated set_overlay_position to emit overlay-position-changed event
- Phase 7: Settings UI
  - Added "Global Toggle" section: hotkey badge, "Record..." button, status dot
  - Hotkey recording: captures keydown with modifiers, formats + saves
  - Toggle status live-updates via toggle-changed event
  - Added hotkey-badge CSS with recording pulse animation
- Phase 8: Build & verify
  - cargo build: clean compile
  - cargo test: 230 tests pass
  - cargo clippy: no new warnings (fixed starts_with, redundant closure)

Files touched:
  - crates/core/src/config.rs
  - ui/src-tauri/src/lib.rs
  - ui/src/toggle-feedback.html (new)
  - ui/src/toggle-feedback.css (new)
  - ui/src/toggle-feedback.js (new)
  - ui/src/notification.html (new)
  - ui/src/notification.css (new)
  - ui/src/notification.js (new)
  - ui/src/overlay.js
  - ui/src/index.html
  - ui/src/styles.css
  - ui/src/main.js

## Session: 2026-03-09 (/write-tests)
- Added 32 new tests to `ui/src-tauri/src/lib.rs` (#[cfg(test)] module):
  - matches_hotkey: 12 tests (ctrl-cmd-v match/reject, modifier combos, special keys, empty string, bare char)
  - key_matches_mapping_from: 7 tests (single char, case sensitivity, modifier rejection, ctrl-b, escape/return/tab)
  - parse_mapping_key: 4 tests (single char, special keys, vim specials, invalid)
  - mode_to_string: 1 test (all 4 variants)
  - overlay_xy: 4 tests (bottom-right, top-left, top-center, unknown fallback)
  - config_to_mode_entry: 4 tests (defaults, control-bracket, custom sequence, too-short sequence)
- Added 7 new tests to `crates/core/src/config.rs`:
  - enabled/toggle_hotkey defaults, deserialize enabled=false, custom hotkey, roundtrip, empty JSON defaults, custom mapping roundtrip, per-app config roundtrip
- Total: 269 tests passing (32 ui-lib + 59 core + 111 core-comprehensive + 67 engine)
