# Findings — 2026-03-09 — UX Polish: Wire Up Dead Features + Friction Reduction

## Problem Statement

vim-anywhere has several UX friction points:

1. **Dead settings** — Custom mappings, disabled motions (beyond Ctrl-b/d/f/u), and per-app strategy config are stored in config and exposed in the settings UI, but never executed at runtime. Users configure these and nothing happens.
2. **No global toggle** — No way to quickly disable/enable vim-anywhere without opening settings or adding apps to the exclusion list. When vim-anywhere interferes, the only escape is to close the app or navigate to settings.
3. **Silent failure on Electron apps** — Teams, Discord, Perplexity (Electron) have no AX API support. vim-anywhere silently does nothing — no indication to the user that it's not working.
4. **Mode indicator in corner** — The overlay badge is fixed in a screen corner (default: bottom-right). Users must look away from their text to check mode. A near-cursor indicator would feel more native.

## Target Apps

| App | Type | AX Works? | Current Status |
|-----|------|-----------|---------------|
| Apple Notes | Native | Yes | Works well |
| Comet Browser | WebKit | Likely | Should work |
| Raycast | Native | Yes | Works (transient fields) |
| Microsoft Teams | Electron | No | Silent no-op |
| Discord | Electron | No | Silent no-op |
| Perplexity | Electron | No | Silent no-op |

## Chosen Approach: A — "Polish What Works"

Focus on UX quality for apps where AX API already works. Wire up dead features. Add friction-reduction controls. Follow up with Approach B (keyboard fallback for Electron) in a separate cycle.

### Feature 1: Global Toggle Hotkey
- Register a global hotkey (e.g., Cmd+Ctrl+V) to toggle vim-anywhere on/off
- When off: all keys pass through, overlay hidden, no vim processing
- When toggled: show brief notification ("vim-anywhere ON" / "vim-anywhere OFF")
- Persist toggle state (or always start enabled — TBD)
- Hotkey should be configurable in settings

### Feature 2: Wire Up Custom Mappings in Parser
- Parser currently ignores `config.custom_mappings`
- Need to intercept parsed commands and remap according to custom mappings
- Format: `{mode: "normal", from: "H", to: "^"}` — remap H to go to first non-blank
- The UI add/remove already works (`ui/src/main.js:209-294`), config stores them, Tauri commands exist
- Just need the engine/parser to read and apply them

### Feature 3: Wire Up Disabled Motions in Parser
- Currently only Ctrl-b/d/f/u are checked in `lib.rs:867`
- `config.disabled_motions` stores arbitrary motion names but parser never checks
- Need to check disabled_motions list before executing any motion
- UI already has checkboxes for Ctrl motions; could expand to other motions later

### Feature 4: Apply Per-App Config at Runtime
- `config.per_app` stores `{bundle_id: {strategy, custom_mappings}}`
- Strategy is read at runtime (line 840-860 in lib.rs) — this DOES work
- Per-app custom_mappings are stored but not applied
- Need to merge per-app custom_mappings with global custom_mappings when processing events for that app

### Feature 5: AX Failure Notification
- When AX API fails to get focused element or read text, currently passes through silently
- Add a brief overlay notification: "vim-anywhere: not supported in this app"
- Show once per app session (don't spam on every keystroke)
- Helps users understand why vim isn't working in Electron apps

### Feature 6: Mode Indicator Near Focus Border
- Instead of (or in addition to) the corner overlay, show mode near the focused text field
- Position relative to the focus border (e.g., top-right corner of the border)
- Moves with focus as user clicks different text fields
- Corner overlay remains as fallback/option in settings

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Approach A first, B later | A is lower complexity, immediately improves daily UX for working apps |
| Global toggle > per-app toggle | Users need instant escape regardless of which app |
| Notification on AX failure | Silent failure is the worst UX — at least tell the user |
| Near-cursor indicator | Looking at the corner breaks flow; near-cursor is natural |
| Wire up existing scaffolding | Config + UI already exist; just need engine integration |

## Scope Exclusions (deferred to Approach B cycle)

- Keyboard fallback strategy for Electron apps
- Search UI (/ and ? mini-buffer)
- In-app cheat sheet / help overlay
- Visual Block mode (Ctrl+V)
- Replace mode (R)
- Marks, ex commands

## Open Questions

- What should the default global toggle hotkey be? (Cmd+Ctrl+V? Hyper+V? Configurable?)
- Should the near-cursor indicator replace the corner overlay or be an additional option?
- Should the "not supported" notification include a "Configure..." button to jump to settings?

## Files Likely Modified

| File | Changes |
|------|---------|
| `ui/src-tauri/src/lib.rs` | Global toggle state, AX failure notification, per-app mapping merge |
| `crates/core/src/parser.rs` | Custom mapping application, disabled motion checks |
| `crates/core/src/config.rs` | Global toggle hotkey config field |
| `crates/platform-mac/src/event_tap.rs` | Global hotkey registration |
| `ui/src/main.js` | Toggle hotkey config UI, near-cursor indicator settings |
| `ui/src/overlay.js` | Near-cursor positioning logic |
| `ui/src/overlay.html` | Notification toast markup |

## Constraints from Lessons/Security

- No active lessons
- Security audit clean (0 findings) — maintain CSP, RAII AXElement, textContent for DOM, no debug logging to /tmp
- Bundle ID validation exists (non-empty, max 255, ASCII alphanumeric + .-_) — apply same validation to custom mapping keys
