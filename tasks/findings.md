# Findings — 2026-03-08 — vim-anywhere

## Requirements
- Clone all features of kindaVim (macOS Vim motions app)
- Cross-platform goal (macOS first, then Linux, Windows)
- All ~160+ Vim motions from day one
- Both strategies: Accessibility Strategy + Keyboard Strategy fallback
- Personal/open-source project
- Near-zero keystroke latency is critical

## What We're Building
A system-level app that intercepts keyboard input globally and translates it into Vim motions across any macOS application — text fields, text areas, dropdowns, lists, menus, and non-text UI elements.

## Repo / Stack Notes
- **Language:** Rust
- **Architecture:** Monorepo with multiple crates (core library + platform shells)
- **UI Framework:** Tauri (settings window, overlay/characters window)
- **macOS APIs:** CGEvent tap (keyboard interception), AXUIElement (Accessibility), objc2/core-graphics crates
- **Future platforms:** Linux (X11/Wayland + AT-SPI), Windows (Win32 hooks + UIA)

## Architecture — Approach B: Rust Core Library + Platform Shells

```
vim-anywhere/
├── crates/
│   ├── core/            # Vim engine, motions, state machine, text objects (pure Rust, no OS deps)
│   ├── platform-mac/    # CGEvent tap, AXUIElement, macOS Accessibility
│   ├── platform-linux/  # (future) X11/Wayland input, AT-SPI accessibility
│   └── platform-win/    # (future) Win32 low-level hooks, UIA
├── src/                 # Binary entry point, wires platform + core
└── ui/                  # Tauri app — settings, overlay characters window, focus highlight
```

### Core Engine (crates/core/)
Pure Rust, zero platform dependencies. Contains:
- **Mode state machine:** Normal, Insert, Visual (Characterwise), Visual (Linewise)
- **Motion parser:** Handles counts, operators, motions, text objects (e.g., `3ciw`, `d2f"`)
- **~160+ motions** organized by category (see full list below)
- **Text manipulation engine:** Operates on a text buffer abstraction
- **Clipboard/register management:** Yank/paste with characterwise vs linewise tracking

### Platform Layer (crates/platform-mac/)
Thin OS-specific code behind a shared trait:
- **KeyboardInterceptor trait** — CGEvent tap on macOS
- **AccessibilityProvider trait** — AXUIElement on macOS (read cursor, selection, text; apply edits)
- **KeyboardStrategy fallback** — Simulate macOS shortcuts (Cmd+A, arrow keys, etc.) when Accessibility unavailable
- **App detection** — Identify active app, determine which strategy to use

### UI Layer (ui/)
Tauri-based:
- **Characters window** — Floating overlay showing current mode/pending command
- **Focus highlighting** — Visual border on active window
- **Settings/Preferences** — Mode entry config, per-app settings, custom mappings
- **The Wizard** — App compatibility checker

## Decisions
| Decision | Rationale |
|----------|-----------|
| Rust over Swift | Cross-platform from day one; no rewrite tax; zero-GC latency |
| Approach B (core lib + shells) | Vim engine is 70%+ of work, pure logic, write-once; platform layer is thin |
| Tauri over Electron | ~10MB vs ~150MB; Rust backend integration is native |
| All motions from start | User is experienced Vim user, partial support would be frustrating |
| Both strategies | Accessibility for precision, Keyboard for universal fallback |

## Modes
1. **Normal Mode** — Full Vim navigation and editing
2. **Insert Mode** — Pass-through typing to active app
3. **Visual Mode (Characterwise)** — Character-level selection
4. **Visual Mode (Linewise)** — Line-level selection

## Normal Mode Entry Options
- `Esc` key (default)
- Double `Esc` (sends real escape to app)
- Custom two-letter sequences (e.g., `jk`)
- `Control-[`
- Configurable modifier-based entry

## Complete Motion List (from kindaVim)

### Normal Mode (~160+ moves)

**Basic Navigation:** `h`, `j`, `k`, `l`, `0`, `$`, `^`, `_`, `-`, `w`, `W`, `b`, `B`, `e`, `E`, `ge`, `gE`, `f{char}`, `F{char}`, `t{char}`, `T{char}`, `;`, `,`, `return`, `/{pattern}`, `?{pattern}`, `n`, `N`, `G`, `gg`, `H`, `L`, `M`

**Display/Soft Navigation:** `g0`, `g$`, `g^`, `g_`, `gj`, `gk`, `gm`, `gI`

**Scrolling:** `Ctrl-b`, `Ctrl-d`, `Ctrl-f`, `Ctrl-u`, `zt`, `zz`, `zb`, `z.`, `z-`, `z<Return>`

**Entering Insert Mode:** `a`, `A`, `i`, `I`, `o`, `O`

**Single-char Editing:** `r{char}`, `~`, `J`, `p`, `P`, `gx`

**Indentation:** `<<`, `>>`

**Change (c) + motion:** `cc`, `c$`, `c0`, `cb`, `cB`, `ce`, `cE`, `cf`, `cF`, `cG`, `ch`, `cj`, `ck`, `cl`, `ct`, `cT`, `cw`, `cW`, `cg$`, `cg0`, `cgg`

**Change (c) + text objects:** `ciw`, `ciW`, `cib`, `ciB`, `cip`, `cis`, `ci"`, `ci'`, `ci[`, `ci<`, `` ci` ``, `caw`, `caW`, `cab`, `caB`, `cap`, `cas`, `ca"`, `ca'`, `ca[`, `ca<`, `` ca` ``

**Delete (d) + motion:** `dd`, `d$`, `d0`, `db`, `dB`, `de`, `dE`, `df`, `dF`, `dG`, `dh`, `dj`, `dk`, `dl`, `dt`, `dT`, `dw`, `dW`, `dg$`, `dg0`, `dgg`

**Delete (d) + text objects:** `diw`, `diW`, `dib`, `diB`, `dip`, `dis`, `di"`, `di'`, `di[`, `di<`, `` di` ``, `daw`, `daW`, `dab`, `daB`, `dap`, `das`, `da"`, `da'`, `da[`, `da<`, `` da` ``

**Yank (y) + motion:** `yy`, `y$`, `y0`, `yf`, `yF`, `yh`, `yl`, `yt`, `yT`, `yg$`, `yg0`

**Yank (y) + text objects:** `yiw`, `yiW`, `yib`, `yiB`, `yip`, `yis`, `yi"`, `yi'`, `yi[`, `yi<`, `` yi` ``, `yaw`, `yaW`, `yab`, `yaB`, `yap`, `yas`, `ya"`, `ya'`, `ya[`, `ya<`, `` ya` ``

**Block/Bracket Navigation:** `%`, `(`, `)`, `{`, `}`, `[(`, `[{`, `])`, `]}`

### Visual Mode — Characterwise

**Navigation:** `h`, `j`, `k`, `l`, `0`, `$`, `^`, `_`, `-`, `w`, `W`, `b`, `B`, `e`, `E`, `f`, `F`, `t`, `T`, `;`, `,`, `(`, `)`, `{`, `}`, `return`, `g_`, `G`, `g$`, `ge`, `gE`, `gg`, `gI`, `gj`, `gk`

**Text Objects:** `iw`, `iW`, `ip`, `ap`, `ib`, `iB`, `i"`, `i'`, `i[`, `i<`, `` i` ``, `ab`, `aB`, `a"`, `a'`, `a[`, `a<`, `` a` ``

**Operations:** `c`, `d`, `y`, `o`, `v`, `V`, `escape`, `<`, `>`, `~`, `C`, `D`, `gx`, `R`, `S`, `u`, `U`, `Y`

### Visual Mode — Linewise
Same navigation and text objects as Characterwise, plus linewise-specific behavior for operations.

## UI Features
- **Characters Window:** Floating overlay showing pending keys and current mode
- **Configurable size** or fully hidden
- **Focus highlighting:** Border/highlight on active window
- **The Wizard:** Per-app compatibility checker and strategy selector
- **Count display:** Shows numeric prefix as it's typed
- **Per-app JSON config:** Enable/disable strategies, custom mappings per app
- **Custom key mappings:** User-defined remaps
- **Preference syncing:** Across machines

## Open Questions
- Tauri overlay window: need to verify Tauri can create always-on-top, click-through, borderless floating windows on macOS
- CGEvent tap requires Accessibility permission — need clean onboarding flow
- Per-app strategy detection: how to determine if an app supports AXUIElement well enough (The Wizard equivalent)

## Resources
- [kindaVim main site](https://kindavim.app/)
- [kindaVim docs](https://docs.kindavim.app/)
- [kindaVim GitHub](https://github.com/godbout/kindaVim.blahblah)
- [AccessibilityStrategyTestApp](https://github.com/godbout/AccessibilityStrategyTestApp) — full list of implemented motions
- [Alternatives: Karabiner-Elements, VimMode.spoon, SketchyVim, ShadowVim]
