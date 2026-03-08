# Findings — 2026-03-09 — Seamless macOS Integration

## Problem Statement

vim-anywhere has three major seamlessness issues plus UX gaps:

1. **Escape requires double-press** — `double_escape_sends_real` is on by default. First Escape goes to Normal, second sends real Escape. Users expect: one Escape = Normal if in other mode, real Escape if already in Normal.
2. **Active everywhere, even where it shouldn't be** — intercepts keys in non-text elements, terminal emulators (Kitty, iTerm2, etc.), and contexts where Vim bindings make no sense.
3. **No auto-detection of text field focus** — starts in Normal mode globally, so clicking into a text field and typing doesn't work until you press `i`.
4. **Mode indicator is unclear** — hard to tell what mode you're in, no feedback on mode change.
5. **No visual focus indication** — no way to tell which app/window vim-anywhere is actively controlling.

## Chosen Approach: Full Seamless Package (Approach C + Focus Highlight)

### 1. Smart Escape (context-aware single Escape)

- **Insert/Visual mode + Escape** → transition to Normal mode (suppress Escape)
- **Normal mode + Escape** → pass through as real Escape to the app
- Remove `double_escape_sends_real` as the default behavior (can remain as opt-in config)
- Affected files: `crates/core/src/modes.rs` (ModeStateMachine.handle_escape), `ui/src-tauri/src/lib.rs` (event handler)

### 2. Terminal Emulator Exclusion

- Maintain a list of known terminal bundle IDs to fully exclude:
  - `net.kovidgoyal.kitty`, `com.apple.Terminal`, `com.googlecode.iterm2`, `dev.warp.Warp-Stable`, `io.alacritty`, `com.github.wez.wezterm`, `co.zeit.hyper`
- Full passthrough when frontmost app is a terminal — vim-anywhere is invisible
- Make the exclusion list configurable via user config
- Affected files: `crates/platform-mac/src/app_detection.rs`, `crates/core/src/config.rs`

### 3. Text-Field Scoping

- When focused element is NOT a text field → full passthrough of everything (including Escape)
- vim-anywhere is completely invisible outside text fields
- Current behavior already checks `is_editable_text()` but still suppresses Escape in non-text contexts — fix this
- Affected files: `ui/src-tauri/src/lib.rs` (Normal/Visual mode handler, lines ~665-698)

### 4. Auto-Insert on Text Field Focus

- When user clicks/tabs into a text field → start in Insert mode (typing just works)
- When leaving a text field → reset to Insert mode so next field entry is seamless
- Detection via AX notifications (kAXFocusedUIElementChangedNotification) or polling focused element
- Affected files: `ui/src-tauri/src/lib.rs`, `crates/platform-mac/src/accessibility.rs`

### 5. Improved Mode Overlay

- **Auto-hide in Insert mode** — overlay only visible in Normal/Visual mode (when vim is intercepting)
- **Flash on mode change** — brief highlight/animation when switching modes for clear feedback
- **Color-coded modes** — distinct colors per mode (e.g., green=Normal, orange/yellow=Visual, hidden=Insert)
- **Pending key display** — show partial commands (`d` waiting for motion, count `3` being typed)
- Affected files: `ui/src/` (HTML/CSS/JS overlay), `ui/src-tauri/src/lib.rs` (mode notification)

### 6. Window Focus Highlight

- **Border glow** on the active window when vim-anywhere is intercepting (Normal/Visual mode only)
- **Slight dim** on other windows simultaneously
- Subtle, not overdone — thin border (2-3px), low-opacity dim
- Window-level granularity (not text-field level) for reliability
- Hidden in Insert mode (same as overlay) — only shows when vim is "active"
- Implementation: transparent borderless overlay window positioned over the active window via AX position/size queries, or Core Graphics window compositing
- Affected files: `ui/src-tauri/src/` (new overlay window management), `crates/platform-mac/src/accessibility.rs` (window position queries)

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Single Escape = context-aware | Matches user expectation: Escape exits modes, or acts as real Escape in Normal |
| `double_escape_sends_real` off by default | Source of confusion; opt-in for power users who want it |
| Auto-Insert on focus | "Click and type" is the universal expectation; Vim mode is opt-in via Escape |
| Terminal full exclusion | Terminals have their own Vim; intercepting breaks everything |
| Overlay hidden in Insert mode | Insert = transparent typing; no distraction needed |
| Focus highlight in Normal/Visual only | Signals "vim is controlling this" without distracting during regular typing |
| Window-level highlight | Reliable across all apps; text-field-level is fragile |

## Open Questions

- AX notifications vs polling for focus change detection — notifications are cleaner but may need observer setup per-app
- Window border overlay: Tauri window overlay vs Core Graphics layer — need to evaluate which is less intrusive and more performant
- Should the dim effect cover the entire screen or just other windows? Screen-level is simpler, window-level looks better
- Config UI for exclusion list — defer to future iteration or include now?
