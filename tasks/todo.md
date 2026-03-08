# TODO — 2026-03-08 — vim-anywhere: Vim Motions for macOS

## Goal
Build a system-level macOS app that intercepts keyboard input globally and translates it into Vim motions across any application — text fields, text areas, dropdowns, lists, menus. Rust core engine + platform shell + Tauri UI. Clone all kindaVim features (~160+ motions).

## Plan

### Phase 1: Project Scaffolding
- [x] 1.1 Initialize Rust workspace with `Cargo.toml` (workspace members: `crates/core`, `crates/platform-mac`, and root binary)
- [x] 1.2 Create `crates/core/` crate with `lib.rs` — empty module structure (modes, motions, parser, buffer, register)
- [x] 1.3 Create `crates/platform-mac/` crate with `lib.rs` — empty module structure (keyboard, accessibility, app_detection)
- [x] 1.4 Create root `src/main.rs` — wires core + platform, placeholder entry point
- [x] 1.5 Initialize Tauri app in `ui/` with basic window (dark theme, JetBrains Mono)
- [x] 1.6 Add dependencies: `core-graphics`, `core-foundation`, `objc2`, `accessibility`, `serde`, `serde_json`
- [x] 1.7 Verify: `cargo build` succeeds, `cargo run` opens a blank Tauri window

### Phase 2: Core — Mode State Machine
- [x] 2.1 Define `Mode` enum: Normal, Insert, VisualCharacterwise, VisualLinewise
- [x] 2.2 Implement `ModeStateMachine` — tracks current mode, handles transitions (Normal→Insert via `i/a/o/etc`, Insert→Normal via Esc, Normal→Visual via `v/V`)
- [x] 2.3 Implement mode entry options: Esc, double-Esc, custom sequence (`jk`), Control-[
- [x] 2.4 Unit tests: mode transitions, custom sequence detection, double-Esc timing

### Phase 3: Core — Key Parser & Command Model
- [x] 3.1 Define `KeyEvent` struct (key, modifiers, is_repeat)
- [x] 3.2 Define command model: `ParsedCommand { count, operator, motion, text_object, register }`
- [x] 3.3 Implement `KeyParser` — accumulates keystrokes, recognizes complete commands (e.g., `3ciw` → count:3, operator:Change, text_object:InnerWord)
- [x] 3.4 Handle operator-pending state (after `d`, `c`, `y` — waiting for motion/text object)
- [x] 3.5 Handle `g` prefix commands (`gg`, `g$`, `g0`, `gj`, `gk`, `ge`, `gE`, `gI`, `gm`, `gx`)
- [x] 3.6 Handle `z` prefix commands (`zt`, `zz`, `zb`, `z.`, `z-`, `zReturn`)
- [x] 3.7 Handle `[` and `]` prefix commands (`[(`, `[{`, `])`, `]}`)
- [x] 3.8 Handle `f/F/t/T` + char input (wait for next character)
- [x] 3.9 Handle `/` and `?` search input (accumulate pattern until Enter)
- [x] 3.10 Unit tests: parse `3dw`, `ciw`, `d2f"`, `gj`, `zt`, `[(`, `fa`, `/pattern<CR>`

### Phase 4: Core — Text Buffer Abstraction
- [x] 4.1 Define `TextBuffer` trait — abstract interface for reading/modifying text (get_text, get_cursor, set_cursor, get_selection, replace_range, line_count, line_at, etc.)
- [x] 4.2 Implement `InMemoryBuffer` (for testing) — owns a String, tracks cursor position and selection
- [x] 4.3 Implement cursor model: line, column, preferred_column (for j/k vertical movement)
- [x] 4.4 Implement selection model: anchor + head, characterwise vs linewise
- [x] 4.5 Unit tests: buffer operations, cursor movement, selection creation

### Phase 5: Core — Basic Navigation Motions (Normal Mode)
- [x] 5.1 Implement motion trait: `Motion::execute(buffer, count) -> CursorPosition`
- [x] 5.2 `h`, `l` — left/right character
- [x] 5.3 `j`, `k` — down/up line (with preferred_column)
- [x] 5.4 `0`, `$`, `^`, `_` — line start/end/first-non-blank
- [x] 5.5 `w`, `W`, `b`, `B`, `e`, `E` — word motions
- [x] 5.6 `ge`, `gE` — end of previous word
- [x] 5.7 `f{char}`, `F{char}`, `t{char}`, `T{char}` — find character
- [x] 5.8 `;`, `,` — repeat/reverse last f/F/t/T
- [x] 5.9 `-`, `return` — line up/down to first non-blank
- [x] 5.10 `G`, `gg` — go to line / first line
- [x] 5.11 `H`, `M`, `L` — screen top/middle/bottom
- [x] 5.12 `%` — matching bracket
- [x] 5.13 `(`, `)` — sentence forward/backward
- [x] 5.14 `{`, `}` — paragraph forward/backward
- [x] 5.15 `[(`, `[{`, `])`, `]}` — unmatched bracket
- [x] 5.16 `n`, `N` — next/previous search match
- [x] 5.17 Display motions: `g0`, `g$`, `g^`, `g_`, `gj`, `gk`, `gm`, `gI`
- [x] 5.18 Count support for all motions (e.g., `3w`, `5j`)
- [x] 5.19 Unit tests for every motion with edge cases (empty lines, EOF, BOF, single char lines)

### Phase 6: Core — Text Objects
- [x] 6.1 Define `TextObject` trait: `select(buffer, cursor) -> Range`
- [x] 6.2 `iw`, `aw` — inner/a word
- [x] 6.3 `iW`, `aW` — inner/a WORD
- [x] 6.4 `is`, `as` — inner/a sentence
- [x] 6.5 `ip`, `ap` — inner/a paragraph
- [x] 6.6 `ib`/`i(`, `ab`/`a(` — inner/a parentheses block
- [x] 6.7 `iB`/`i{`, `aB`/`a{` — inner/a braces block
- [x] 6.8 `i[`, `a[` — inner/a bracket block
- [x] 6.9 `i<`, `a<` — inner/a angle bracket block
- [x] 6.10 `i"`, `a"` — inner/a double quote
- [x] 6.11 `i'`, `a'` — inner/a single quote
- [x] 6.12 `` i` ``, `` a` `` — inner/a backtick
- [x] 6.13 Unit tests for all text objects including nested brackets, empty blocks, cursor at boundaries

### Phase 7: Core — Operators (c, d, y) + Compound Commands
- [x] 7.1 Define `Operator` trait: `execute(buffer, range, yank_style) -> TextEdit`
- [x] 7.2 Implement `Delete` operator — deletes range, yanks to register
- [x] 7.3 Implement `Change` operator — deletes range, yanks, enters Insert mode
- [x] 7.4 Implement `Yank` operator — copies range to register without modifying buffer
- [x] 7.5 Implement register system: unnamed register (`""`), characterwise vs linewise yank tracking
- [x] 7.6 Wire operator + motion combinations: `dw`, `d$`, `d0`, `db`, `dB`, `de`, `dE`, `df`, `dF`, `dG`, `dh`, `dj`, `dk`, `dl`, `dt`, `dT`, `dw`, `dW`, `dg$`, `dg0`, `dgg`
- [x] 7.7 Wire operator + text object combinations: `diw`, `diW`, `dib`, `diB`, `dip`, `dis`, `di"`, `di'`, `di[`, `di<`, `` di` ``, `daw`, `daW`, `dab`, `daB`, `dap`, `das`, `da"`, `da'`, `da[`, `da<`, `` da` ``
- [x] 7.8 Same for `c` operator (all motion + text object combos)
- [x] 7.9 Same for `y` operator (all motion + text object combos)
- [x] 7.10 `dd` — delete line, `cc` — change line, `yy` — yank line
- [x] 7.11 `D` (alias for `d$`), `C` (alias for `c$`), `Y` (alias for `yy`)
- [x] 7.12 Unit tests for all operator+motion and operator+text_object combinations

### Phase 8: Core — Single-key Editing & Misc Normal Mode
- [x] 8.1 `i`, `I`, `a`, `A`, `o`, `O` — enter Insert mode at various positions
- [x] 8.2 `r{char}` — replace character under cursor
- [x] 8.3 `~` — toggle case of character under cursor
- [x] 8.4 `J` — join lines
- [x] 8.5 `p`, `P` — paste after/before (characterwise and linewise variants)
- [x] 8.6 `gx` — open URL under cursor (or platform-specific action)
- [x] 8.7 `<<`, `>>` — indent/outdent line
- [x] 8.8 Scrolling commands: `Ctrl-b`, `Ctrl-d`, `Ctrl-f`, `Ctrl-u`
- [x] 8.9 Scroll-position commands: `zt`, `zz`, `zb`, `z.`, `z-`, `zReturn`
- [x] 8.10 `/` and `?` — search forward/backward
- [x] 8.11 Unit tests for all editing commands

### Phase 9: Core — Visual Mode
- [x] 9.1 `v` — enter Visual Characterwise from Normal
- [x] 9.2 `V` — enter Visual Linewise from Normal
- [x] 9.3 All navigation motions work in Visual mode (extend selection)
- [x] 9.4 Text objects in Visual mode (`viw`, `vap`, etc.)
- [x] 9.5 `o` — swap anchor/head in Visual
- [x] 9.6 Visual operations: `c`, `d`, `y`, `<`, `>`, `~`, `u`, `U`, `r`, `R`, `S`, `J`
- [x] 9.7 `v` in Visual Characterwise → back to Normal; `V` in Visual Linewise → back to Normal
- [x] 9.8 `v` in Visual Linewise → switch to Characterwise; `V` in Visual Characterwise → switch to Linewise
- [x] 9.9 `Escape` — exit Visual to Normal
- [x] 9.10 Unit tests for Visual mode selection, extension, operations, mode switching

### Phase 10: macOS Platform — Keyboard Interception
- [x] 10.1 Implement CGEvent tap setup (request Accessibility permission, create tap, attach to run loop)
- [x] 10.2 Implement key event conversion: CGEvent → core `KeyEvent`
- [x] 10.3 Implement event suppression: when in Normal/Visual mode, suppress keystrokes from reaching the app
- [x] 10.4 Implement event pass-through: when in Insert mode, let keystrokes pass to the app
- [x] 10.5 Handle modifier keys correctly (Shift, Control, Option, Command)
- [x] 10.6 Handle special keys (Escape, Return, Backspace, arrows, function keys)
- [x] 10.7 Verify: can intercept keystrokes globally, suppress them, and pass them through

### Phase 11: macOS Platform — Accessibility Strategy
- [x] 11.1 Implement `AXUIElement` wrapper: get focused element, read AXRole, AXValue, AXSelectedTextRange, AXNumberOfCharacters
- [x] 11.2 Implement `AccessibilityBuffer` — implements core `TextBuffer` trait by reading/writing via AXUIElement
- [x] 11.3 Read operations: get full text, get cursor position, get selection range
- [x] 11.4 Write operations: set cursor position, set selection, replace text, insert text
- [x] 11.5 Implement app detection: get frontmost app bundle ID
- [x] 11.6 Verify: can read text from TextEdit, move cursor, select text, replace text

### Phase 12: macOS Platform — Keyboard Strategy (Fallback)
- [x] 12.1 Implement key simulation: send synthetic key events (arrow keys, Cmd+A, Cmd+C, Cmd+V, etc.)
- [x] 12.2 Implement `KeyboardBuffer` — implements core `TextBuffer` trait by simulating macOS shortcuts
- [x] 12.3 Map Vim motions to macOS keyboard equivalents (h→Left, j→Down, w→Option+Right, $→Cmd+Right, etc.)
- [x] 12.4 Handle clipboard-based operations (yank: Cmd+C, paste: Cmd+V, delete: Cmd+X via select-then-cut)
- [x] 12.5 Verify: basic motions work in a non-accessible app (e.g., a dropdown or menu)

### Phase 13: Wire Core + Platform Together
- [x] 13.1 Create `Engine` struct in root crate: owns `ModeStateMachine`, `KeyParser`, platform `TextBuffer`, `RegisterManager`
- [x] 13.2 Implement the main event loop: CGEvent tap → KeyEvent → KeyParser → ParsedCommand → execute motion/operator on TextBuffer
- [x] 13.3 Strategy selection: check if focused element supports Accessibility, fall back to Keyboard
- [x] 13.4 Implement per-app config loading from `~/.config/vim-anywhere/config.json`
- [x] 13.5 Verify: can open TextEdit, press `Esc` to enter Normal mode, use `hjkl` to navigate, `dd` to delete a line, `i` to go back to Insert mode

### Phase 14: Tauri UI — Settings Window
- [x] 14.1 Set up Tauri with dark/light theme support (CSS custom properties, `data-theme` attribute)
- [x] 14.2 Bundle JetBrains Mono font (400, 500, 700 weights)
- [x] 14.3 Implement General tab: mode entry config, characters window size, theme toggle
- [x] 14.4 Implement Keys tab: custom mappings list, add/remove mappings, disabled motions
- [x] 14.5 Implement Apps tab: per-app strategy override list, filter, click-to-configure
- [x] 14.6 Implement About tab: version info, links
- [x] 14.7 Wire settings to `config.json` — read on load, write on change
- [x] 14.8 Verify: settings window opens via menu bar, changes persist across restarts

### Phase 15: Tauri UI — Characters Overlay Window
- [x] 15.1 Create overlay window: borderless, always-on-top, transparent background, click-through
- [x] 15.2 Display current mode (NORMAL / INSERT / VISUAL) with accent coloring
- [x] 15.3 Display pending keys as they're typed (e.g., show `d2` while waiting for motion)
- [x] 15.4 Position above focused text field (via Accessibility) or top-center of active window
- [x] 15.5 100ms fade in/out transitions
- [x] 15.6 Respect size setting (Small/Medium/Large/Hidden)
- [x] 15.7 Verify: overlay appears when entering Normal mode, updates as keys are pressed, hides in Insert mode

### Phase 16: Tauri UI — Focus Highlight & Menu Bar
- [x] 16.1 Implement focus highlight: 2px accent border around active window in Normal/Visual mode
- [x] 16.2 Implement menu bar icon with dropdown (mode display, enable toggle, settings shortcut, quit)
- [x] 16.3 Menu bar icon color changes based on mode (accent for Normal, muted for Insert)
- [x] 16.4 Verify: focus highlight appears/disappears on mode change, menu bar shows correct state

### Phase 17: Tauri UI — Onboarding & Wizard
- [x] 17.1 First-launch onboarding window: request Accessibility + Input Monitoring permissions
- [x] 17.2 Poll permission status, update UI when granted
- [x] 17.3 Implement The Wizard: select an app, test AXUIElement capabilities, show results
- [x] 17.4 Wizard "Apply" saves strategy recommendation to per-app config
- [x] 17.5 Verify: fresh launch shows onboarding, Wizard correctly identifies accessible vs non-accessible apps

### Phase 18: Config & Polish
- [x] 18.1 Config schema: `~/.config/vim-anywhere/config.json` — mode entry, theme, overlay size, per-app overrides, custom mappings
- [x] 18.2 Launch at login (LaunchAgent plist or `SMAppService`)
- [x] 18.3 Handle edge cases: app switching during Normal mode, app crash recovery, permission revocation
- [x] 18.4 Double-Esc: send real Escape to the app when Esc is pressed twice quickly
- [x] 18.5 Verify: config changes take effect without restart, launch-at-login works

## Verification
- `cargo build` → compiles with no errors
- `cargo test` → all unit tests pass (motions, parser, mode state machine, text objects, operators)
- `cargo run` → app starts, menu bar icon appears, settings window opens
- Manual test: open TextEdit, type text, press Esc → Normal mode, `hjkl` navigation works, `dd` deletes line, `ciw` changes word, `v` enters Visual, `p` pastes, `i` returns to Insert
- Manual test: open a non-accessible app → Keyboard Strategy fallback engages
- Manual test: theme toggle switches dark/light correctly
- Manual test: Characters overlay shows mode + pending keys

## Acceptance Criteria
- [ ] All ~160 Normal mode motions work via Accessibility Strategy in TextEdit
- [ ] All Visual mode (Characterwise + Linewise) motions and operations work
- [ ] Keyboard Strategy fallback provides basic navigation in non-accessible apps
- [ ] Mode state machine handles all transitions correctly (Normal↔Insert, Normal↔Visual, Visual↔Visual)
- [ ] Count prefix works with all motions and operators (e.g., `3dw`, `5j`)
- [ ] Characters overlay shows current mode and pending keys
- [ ] Focus highlight appears in Normal/Visual mode
- [ ] Settings window allows theme toggle, mode entry config, per-app settings, custom mappings
- [ ] Config persists to `~/.config/vim-anywhere/config.json`
- [ ] Onboarding requests and tracks permissions
- [ ] The Wizard tests app accessibility support
- [ ] Menu bar icon with dropdown shows mode and provides controls
- [ ] App distributed outside App Store (DMG or Homebrew)

## Risks / Unknowns
- **Tauri click-through overlay**: Need to verify `ignore_cursor_events` works reliably on macOS. Fallback: use a native NSWindow via objc2 instead of Tauri for the overlay.
- **Focus highlight border**: Drawing a border around another app's window requires either an overlay window or CGWindow-level tricks. May need platform-specific NSWindow approach.
- **AXUIElement reliability**: Some apps (Electron-based) have inconsistent Accessibility support. The Wizard helps detect this, but edge cases are expected.
- **CGEvent tap performance**: Event tap callback must return quickly. All heavy processing (parsing, buffer ops) must be async or on a separate thread.
- **Search (`/`, `?`)**: Implementing full regex search across the buffer is straightforward in core, but displaying a search input UI needs design (overlay input field vs Characters window).
- **Scroll commands** (`zt`, `zz`, `Ctrl-d`): Require knowing viewport dimensions, which Accessibility may not expose. May need Keyboard Strategy fallback for these.

## Results
- (fill after execution)

## Errors
| Error | Attempt | Resolution |
|-------|---------|------------|
|       | 1       |            |
