<p align="center">
  <img src="ui/src-tauri/icons/128x128@2x.png" alt="vim-anywhere" width="128" height="128">
</p>

<h1 align="center">vim-anywhere</h1>

<p align="center">
  <strong>Vim motions in any macOS text field.</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#supported-motions">Motions</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#development">Development</a> •
  <a href="#license">License</a>
</p>

---

## What is vim-anywhere?

vim-anywhere intercepts keyboard input globally on macOS and translates it into Vim motions — letting you navigate, edit, and select text with Vim keybindings in **any** application. Built with Rust and Tauri.

- **Normal mode** — navigate with `h/j/k/l`, `w/b/e`, `f/t`, `gg/G`, and more
- **Insert mode** — type normally, enter with `i/a/o/I/A/O`
- **Visual mode** — select text characterwise (`v`) or linewise (`V`)
- **Operators** — `d` (delete), `c` (change), `y` (yank), `>/<` (indent), `~` (toggle case)
- **Text objects** — `iw`, `i"`, `i(`, `i{`, `ip`, and all their `a` variants

## Features

- 160+ Vim motions and text objects
- Full modal editing: Normal, Insert, Visual (characterwise & linewise), Operator-Pending
- Repeat counts (`3dw`, `5j`, `2dd`)
- Dot-repeat (`.`) for last change
- Named register system for yank/paste
- Per-app configuration and strategy selection
- Auto-disables in terminal emulators (Terminal, iTerm2, Alacritty, Ghostty)
- macOS Accessibility API integration for native text manipulation
- Dark/light theme settings UI
- Launch at login support
- Custom key mappings

## Requirements

- macOS 12.0+
- **Accessibility permission** — required for reading/writing text in other apps
- **Input Monitoring permission** — required for global keyboard interception

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/kennethsolomon/vim-anywhere.git
cd vim-anywhere

# Install dependencies
cd ui && npm install && cd ..

# Build the app
cd ui && npm run tauri build
```

The built `.app` bundle will be in `ui/src-tauri/target/release/bundle/macos/`.

### Granting Permissions

On first launch, macOS will prompt for:

1. **System Settings → Privacy & Security → Accessibility** — add vim-anywhere
2. **System Settings → Privacy & Security → Input Monitoring** — add vim-anywhere

Both are required. The app will not intercept keystrokes without them.

## Usage

### Mode Switching

| Key | Action |
|-----|--------|
| `Escape` | Enter Normal mode (default) |
| `i` | Insert before cursor |
| `a` | Insert after cursor |
| `I` | Insert at line start |
| `A` | Insert at line end |
| `o` | Open line below |
| `O` | Open line above |
| `v` | Visual characterwise |
| `V` | Visual linewise |

### Double-Escape

Press `Escape` twice quickly (within 300ms) to send a real Escape to the active application. Useful for dismissing dialogs and menus.

### Mode Indicator

A floating overlay badge shows the current mode (NORMAL / INSERT / VISUAL). Configurable size or can be hidden entirely.

## Supported Motions

### Navigation

| Motion | Description |
|--------|-------------|
| `h` `j` `k` `l` | Left, down, up, right |
| `w` `W` | Word / WORD forward |
| `b` `B` | Word / WORD backward |
| `e` `E` | End of word / WORD |
| `ge` `gE` | End of previous word / WORD |
| `0` | Line start |
| `^` | First non-blank |
| `$` | Line end |
| `g_` | Last non-blank |
| `gg` | First line |
| `G` | Last line |
| `f{c}` `F{c}` | Find char forward / backward |
| `t{c}` `T{c}` | Till char forward / backward |
| `{` `}` | Paragraph backward / forward |
| `%` | Matching bracket |
| `-` | Previous line first non-blank |
| `Return` | Next line first non-blank |

### Operators

| Operator | Description |
|----------|-------------|
| `d` | Delete |
| `c` | Change (delete + enter Insert) |
| `y` | Yank (copy) |
| `>` | Indent |
| `<` | Outdent |
| `~` | Toggle case |

Operators combine with motions and text objects: `dw`, `ci"`, `ya(`, `>}`, `2dd`.

### Text Objects

| Object | Description |
|--------|-------------|
| `iw` / `aw` | Inner / a word |
| `iW` / `aW` | Inner / a WORD |
| `i"` / `a"` | Inner / a double-quoted string |
| `i'` / `a'` | Inner / a single-quoted string |
| `` i` `` / `` a` `` | Inner / a backtick string |
| `i(` / `a(` | Inner / a parentheses |
| `i[` / `a[` | Inner / a brackets |
| `i{` / `a{` | Inner / a braces |
| `i<` / `a<` | Inner / a angle brackets |
| `is` / `as` | Inner / a sentence |
| `ip` / `ap` | Inner / a paragraph |

### Line Operations

| Command | Description |
|---------|-------------|
| `dd` | Delete line |
| `cc` | Change line |
| `yy` | Yank line |
| `>>` | Indent line |
| `<<` | Outdent line |
| `J` | Join lines |
| `p` / `P` | Paste after / before |
| `.` | Repeat last change |
| `u` | Undo |

## Configuration

Configuration is stored at `~/.config/vim-anywhere/config.json`.

```json
{
  "mode_entry": {
    "method": "escape",
    "custom_sequence": null,
    "double_escape_sends_real": true
  },
  "theme": "dark",
  "overlay_size": "medium",
  "focus_highlight": true,
  "menu_bar_icon": true,
  "launch_at_login": false,
  "custom_mappings": [],
  "disabled_motions": [],
  "per_app": {}
}
```

### Mode Entry Methods

| Method | Description |
|--------|-------------|
| `escape` | Press Escape to enter Normal mode (default) |
| `control-bracket` | Press `Ctrl+[` to enter Normal mode |
| `custom` | Define a custom sequence (e.g., `jk`) |

### Per-App Configuration

Override behavior for specific applications:

```json
{
  "per_app": {
    "com.apple.TextEdit": {
      "strategy": "accessibility",
      "custom_mappings": []
    },
    "com.apple.Safari": {
      "strategy": "keyboard",
      "custom_mappings": []
    }
  }
}
```

**Strategies:**

| Strategy | Description |
|----------|-------------|
| `accessibility` | Uses macOS Accessibility API to read/write text directly (best quality) |
| `keyboard` | Simulates keystrokes (fallback for apps without Accessibility support) |
| `disabled` | vim-anywhere is inactive in this app |

### Auto-Disabled Apps

vim-anywhere automatically disables itself in terminal emulators that already have Vim support:

- Terminal.app (`com.apple.Terminal`)
- iTerm2 (`com.googlecode.iterm2`)
- Alacritty (`io.alacritty`)
- Ghostty (`com.mitchellh.ghostty`)

## Architecture

```
vim-anywhere/
├── crates/
│   ├── core/                  # Vim engine (platform-independent)
│   │   ├── modes.rs           # Modal state machine
│   │   ├── motions.rs         # 50+ navigation motions, 19 text objects
│   │   ├── parser.rs          # Keystroke → command parser
│   │   ├── buffer.rs          # Text buffer abstraction
│   │   ├── register.rs        # Yank register system
│   │   └── config.rs          # Configuration management
│   └── platform-mac/          # macOS integration layer
│       ├── accessibility.rs   # AXUIElement RAII wrappers
│       ├── keyboard.rs        # CGEvent keycode mapping
│       └── app_detection.rs   # Frontmost app detection
├── ui/                        # Tauri settings UI
│   ├── src/                   # HTML/CSS/JS frontend
│   └── src-tauri/             # Tauri Rust backend
├── src/                       # Workspace root (Engine)
│   └── lib.rs                 # Engine: key handling, motion resolution
└── tests/                     # Integration tests
```

### Design Principles

- **Core library is platform-agnostic** — all Vim logic lives in `crates/core` with no OS dependencies
- **Platform shells are thin adapters** — `crates/platform-mac` only translates between OS events and core types
- **RAII for system resources** — CoreFoundation objects wrapped in `AXElement` with automatic `CFRelease` on drop
- **No unsafe in core** — all `unsafe` code is isolated to the platform layer

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- Xcode Command Line Tools

### Build

```bash
# Build all workspace crates
cargo build

# Build the Tauri app
cd ui && npm run tauri build

# Development mode (hot reload)
cd ui && npm run tauri dev
```

### Test

```bash
# Run all tests (192 tests)
cargo test --workspace

# Run core tests only
cargo test -p vim-anywhere-core

# Run engine integration tests
cargo test --test engine_comprehensive
```

### Project Structure

| Crate | Purpose | Edition |
|-------|---------|---------|
| `vim-anywhere` | Workspace root, engine | 2024 |
| `vim-anywhere-core` | Vim motions, parser, modes, buffer | 2024 |
| `vim-anywhere-platform-mac` | macOS Accessibility & keyboard | 2021 |
| `ui` | Tauri settings application | 2021 |

The platform-mac crate uses edition 2021 for compatibility with `cocoa` and `objc` FFI crates.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes using [conventional commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, etc.)
4. Push to your branch (`git push origin feature/my-feature`)
5. Open a Pull Request

### Commit Style

```
feat(core): add support for gj/gk display-line motions
fix(platform-mac): prevent double-free on AXElement drop
test(engine): add visual mode operator coverage
docs: update supported motions table
```

## Acknowledgments

Inspired by [kindaVim](https://kindavim.app/) — the original Vim motions for macOS.

## License

MIT
