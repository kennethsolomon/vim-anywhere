use std::sync::{Arc, Mutex};
use std::io::Write;
use tauri::{Emitter, Manager};
use tauri::tray::TrayIconBuilder;
use tauri::menu::{MenuBuilder, MenuItemBuilder};

use vim_anywhere::Engine;
use vim_anywhere_core::buffer::{CursorPosition, InMemoryBuffer, SelectionKind, TextBuffer};
use vim_anywhere_core::config::Config;
use vim_anywhere_core::modes::{Mode, ModeEntryConfig};
use vim_anywhere_core::parser::Key;
use vim_anywhere_platform_mac::accessibility;
use vim_anywhere_platform_mac::app_detection;
use vim_anywhere_platform_mac::event_tap;

use vim_anywhere::EngineResult;

// ── Shared application state ────────────────────────────────────────────────

struct AppState {
    engine: Mutex<Engine>,
    config: Mutex<Config>,
}

fn config_to_mode_entry(cfg: &Config) -> ModeEntryConfig {
    let custom_seq = cfg.mode_entry.custom_sequence.as_ref().and_then(|s| {
        let mut chars = s.chars();
        let a = chars.next()?;
        let b = chars.next()?;
        Some([a, b])
    });

    ModeEntryConfig {
        escape_key: cfg.mode_entry.method == "escape",
        control_bracket: cfg.mode_entry.method == "control-bracket",
        custom_sequence: if cfg.mode_entry.method == "custom" { custom_seq } else { None },
        double_escape_sends_real: cfg.mode_entry.double_escape_sends_real,
        double_escape_timeout_ms: 300,
        sequence_timeout_ms: 200,
    }
}

// ── Payload types ───────────────────────────────────────────────────────────

#[derive(Clone, serde::Serialize)]
struct ModeChangedPayload {
    mode: String,
}

fn mode_to_string(mode: Mode) -> String {
    match mode {
        Mode::Normal => "NORMAL".to_string(),
        Mode::Insert => "INSERT".to_string(),
        Mode::VisualCharacterwise => "VISUAL".to_string(),
        Mode::VisualLinewise => "V-LINE".to_string(),
    }
}

// ── Tauri commands ──────────────────────────────────────────────────────────

#[tauri::command]
fn get_mode(state: tauri::State<'_, Arc<AppState>>) -> String {
    let engine = state.engine.lock().unwrap();
    mode_to_string(engine.mode())
}

#[tauri::command]
fn get_permissions() -> serde_json::Value {
    let ax_trusted = accessibility::is_accessibility_trusted();
    serde_json::json!({
        "accessibility": ax_trusted,
        "input_monitoring": ax_trusted
    })
}

#[tauri::command]
fn load_config(state: tauri::State<'_, Arc<AppState>>) -> serde_json::Value {
    let cfg = state.config.lock().unwrap();
    serde_json::to_value(&*cfg).unwrap_or(serde_json::json!({}))
}

#[tauri::command]
fn save_config_full(state: tauri::State<'_, Arc<AppState>>, config: Config) -> bool {
    let mut cfg = state.config.lock().unwrap();
    *cfg = config;
    let mode_entry = config_to_mode_entry(&cfg);
    drop(cfg);
    let mut engine = state.engine.lock().unwrap();
    *engine = Engine::new(mode_entry);
    let cfg = state.config.lock().unwrap();
    cfg.save().is_ok()
}

#[tauri::command]
fn set_mode_entry(state: tauri::State<'_, Arc<AppState>>, method: String, custom_sequence: Option<String>, double_escape: bool) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.mode_entry.method = method;
    cfg.mode_entry.custom_sequence = custom_sequence;
    cfg.mode_entry.double_escape_sends_real = double_escape;
    let mode_entry = config_to_mode_entry(&cfg);
    let save_result = cfg.save().is_ok();
    drop(cfg);
    let mut engine = state.engine.lock().unwrap();
    *engine = Engine::new(mode_entry);
    save_result
}

#[tauri::command]
fn set_theme(state: tauri::State<'_, Arc<AppState>>, theme: String) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.theme = theme;
    cfg.save().is_ok()
}

#[tauri::command]
fn set_overlay_size(state: tauri::State<'_, Arc<AppState>>, size: String) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.overlay_size = size;
    cfg.save().is_ok()
}

#[tauri::command]
fn set_focus_highlight(state: tauri::State<'_, Arc<AppState>>, enabled: bool) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.focus_highlight = enabled;
    cfg.save().is_ok()
}

#[tauri::command]
fn set_menu_bar_icon(state: tauri::State<'_, Arc<AppState>>, enabled: bool) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.menu_bar_icon = enabled;
    cfg.save().is_ok()
}

#[tauri::command]
fn set_launch_at_login(state: tauri::State<'_, Arc<AppState>>, enabled: bool) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.launch_at_login = enabled;
    cfg.save().is_ok()
}

#[tauri::command]
fn add_custom_mapping(state: tauri::State<'_, Arc<AppState>>, mode: String, from: String, to: String) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.custom_mappings.push(vim_anywhere_core::config::CustomMapping { mode, from, to });
    cfg.save().is_ok()
}

#[tauri::command]
fn remove_custom_mapping(state: tauri::State<'_, Arc<AppState>>, index: usize) -> bool {
    let mut cfg = state.config.lock().unwrap();
    if index < cfg.custom_mappings.len() {
        cfg.custom_mappings.remove(index);
        cfg.save().is_ok()
    } else {
        false
    }
}

#[tauri::command]
fn set_disabled_motion(state: tauri::State<'_, Arc<AppState>>, motion: String, disabled: bool) -> bool {
    let mut cfg = state.config.lock().unwrap();
    if disabled {
        if !cfg.disabled_motions.contains(&motion) {
            cfg.disabled_motions.push(motion);
        }
    } else {
        cfg.disabled_motions.retain(|m| m != &motion);
    }
    cfg.save().is_ok()
}

#[tauri::command]
fn set_app_strategy(state: tauri::State<'_, Arc<AppState>>, bundle_id: String, strategy: String) -> bool {
    let mut cfg = state.config.lock().unwrap();
    let app_config = cfg.per_app.entry(bundle_id).or_insert_with(|| {
        vim_anywhere_core::config::AppConfig {
            strategy: "accessibility".to_string(),
            custom_mappings: vec![],
        }
    });
    app_config.strategy = strategy;
    cfg.save().is_ok()
}

#[tauri::command]
fn open_privacy_settings() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

// ── Wizard ──────────────────────────────────────────────────────────────────

#[derive(Clone, serde::Serialize)]
struct AppResult {
    name: String,
    bundle_id: String,
    strategy: String,
    status: String,
    status_class: String,
}

#[tauri::command]
fn run_wizard(state: tauri::State<'_, Arc<AppState>>) -> Vec<AppResult> {
    let cfg = state.config.lock().unwrap();
    let apps = get_running_apps();
    let mut results = Vec::new();

    for (name, bundle_id) in &apps {
        let (strategy_str, status, status_class) = if let Some(app_cfg) = cfg.per_app.get(bundle_id) {
            match app_cfg.strategy.as_str() {
                "disabled" => ("Disabled".into(), "excluded".into(), "inactive".into()),
                "keyboard" => ("Keyboard".into(), "partial".into(), "partial".into()),
                _ => ("Accessibility".into(), "supported".into(), "active".into()),
            }
        } else {
            let strategy = app_detection::default_strategy_for_app(bundle_id);
            match strategy {
                app_detection::Strategy::Disabled => ("Disabled".into(), "excluded".into(), "inactive".into()),
                app_detection::Strategy::Keyboard => ("Keyboard".into(), "partial".into(), "partial".into()),
                app_detection::Strategy::Accessibility => ("Accessibility".into(), "supported".into(), "active".into()),
            }
        };

        results.push(AppResult {
            name: name.clone(),
            bundle_id: bundle_id.clone(),
            strategy: strategy_str,
            status,
            status_class,
        });
    }

    results.sort_by(|a, b| {
        let order = |s: &str| match s { "active" => 0, "partial" => 1, "inactive" => 2, _ => 3 };
        order(&a.status_class).cmp(&order(&b.status_class))
    });
    results
}

#[allow(deprecated, unexpected_cfgs)]
fn get_running_apps() -> Vec<(String, String)> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    let mut apps = Vec::new();
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let running: id = msg_send![workspace, runningApplications];
        let count: usize = msg_send![running, count];

        for i in 0..count {
            let app: id = msg_send![running, objectAtIndex: i];
            let activation_policy: i64 = msg_send![app, activationPolicy];
            if activation_policy != 0 { continue; }

            let bundle_id_ns: id = msg_send![app, bundleIdentifier];
            let name_ns: id = msg_send![app, localizedName];

            let bundle_id = if bundle_id_ns != nil {
                let bytes: *const std::ffi::c_char = msg_send![bundle_id_ns, UTF8String];
                if bytes.is_null() { continue; }
                std::ffi::CStr::from_ptr(bytes).to_string_lossy().into_owned()
            } else { continue; };

            let name = if name_ns != nil {
                let bytes: *const std::ffi::c_char = msg_send![name_ns, UTF8String];
                if bytes.is_null() { bundle_id.clone() }
                else { std::ffi::CStr::from_ptr(bytes).to_string_lossy().into_owned() }
            } else { bundle_id.clone() };

            if bundle_id == "com.kennethsolomon.vim-anywhere" { continue; }
            apps.push((name, bundle_id));
        }
    }
    apps.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    apps.dedup_by(|a, b| a.1 == b.1);
    apps
}

// ── Helper: apply buffer state back to AX element ───────────────────────────

fn apply_buffer_to_ax(
    element: &accessibility::AXElement,
    buffer: &InMemoryBuffer,
    original_text: &str,
    mode: Mode,
) {
    let new_text = buffer.get_text();

    // Write modified text back
    let text_changed = new_text != original_text;
    if text_changed {
        let _ = accessibility::set_ax_value(element, &new_text);
        // AX API needs time to process text changes before cursor can be set
        // (same hack as SketchyVim's usleep(15000))
        std::thread::sleep(std::time::Duration::from_micros(15000));
    }

    // Handle cursor / selection
    if mode == Mode::VisualCharacterwise || mode == Mode::VisualLinewise {
        // Visual mode: set AX selection to match the visual selection so user sees highlight
        if let Some(sel) = buffer.get_selection() {
            let start = sel.start();
            let end = sel.end();

            let (sel_start_offset, sel_end_offset) = match sel.kind {
                SelectionKind::Linewise => {
                    let start_offset = accessibility::cursor_to_offset(&new_text, start.line, 0);
                    let end_line_len = new_text.split('\n').nth(end.line).map(|l| l.len()).unwrap_or(0);
                    let end_offset = accessibility::cursor_to_offset(&new_text, end.line, end_line_len);
                    (start_offset, end_offset)
                }
                SelectionKind::Characterwise => {
                    let start_offset = accessibility::cursor_to_offset(&new_text, start.line, start.col);
                    // +1 because visual selection is inclusive of the end character
                    let end_offset = accessibility::cursor_to_offset(&new_text, end.line, end.col + 1);
                    (start_offset, end_offset)
                }
            };

            let length = sel_end_offset.saturating_sub(sel_start_offset);
            let _ = accessibility::set_ax_selected_range(element, sel_start_offset, length);
        }
    } else {
        let cursor = buffer.get_cursor();
        let offset = accessibility::cursor_to_offset(&new_text, cursor.line, cursor.col);
        // Normal mode: block cursor (select 1 character to highlight it)
        // Insert mode: thin cursor (selection length 0)
        let length = if mode == Mode::Normal && buffer.char_at(cursor).is_some() {
            1
        } else {
            0
        };
        let _ = accessibility::set_ax_selected_range(element, offset, length);
    }
}

// ── App entry point ─────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Config::load();
    let mode_entry = config_to_mode_entry(&config);
    let engine = Engine::new(mode_entry);

    let state = Arc::new(AppState {
        engine: Mutex::new(engine),
        config: Mutex::new(config),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            get_mode,
            get_permissions,
            load_config,
            save_config_full,
            set_mode_entry,
            set_theme,
            set_overlay_size,
            set_focus_highlight,
            set_menu_bar_icon,
            set_launch_at_login,
            add_custom_mapping,
            remove_custom_mapping,
            set_disabled_motion,
            set_app_strategy,
            run_wizard,
            open_privacy_settings,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // ── System tray ─────────────────────────────────────────────
            let show_item = MenuItemBuilder::with_id("show", "Show Window")
                .build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit vim-anywhere")
                .build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .tooltip("vim-anywhere")
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&tray_menu)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // ── Make overlay click-through + visible on all Spaces ───
            if let Some(overlay_win) = app.get_webview_window("overlay") {
                #[cfg(target_os = "macos")]
                #[allow(deprecated, unexpected_cfgs)]
                {
                    use objc::{msg_send, sel, sel_impl};
                    let ns_window = overlay_win.ns_window().unwrap() as cocoa::base::id;
                    unsafe {
                        let _: () = msg_send![ns_window, setIgnoresMouseEvents: true];
                        let _: () = msg_send![ns_window, setLevel: 25i64]; // NSStatusWindowLevel
                        // Show on all Spaces/desktops (canJoinAllSpaces = 1 << 0)
                        let behavior: u64 = msg_send![ns_window, collectionBehavior];
                        let _: () = msg_send![ns_window, setCollectionBehavior: behavior | (1u64 << 0)];
                    }
                }
            }

            // ── Hide window on close instead of quitting ────────────────
            let main_window = app.get_webview_window("main").unwrap();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    // Hide the window; the tray keeps the app alive
                    if let Some(win) = app_handle.get_webview_window("main") {
                        let _ = win.hide();
                    }
                }
            });

            // ── Event tap ───────────────────────────────────────────────
            let app_handle2 = app.handle().clone();
            let state_for_tap = state.clone();

            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));

                if !accessibility::is_accessibility_trusted() {
                    eprintln!("[vim-anywhere] Accessibility not granted. Features limited.");
                }

                // Log event tap start
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true).append(true)
                    .open("/tmp/vim-anywhere.log")
                {
                    let _ = writeln!(f, "=== Event tap thread starting ===");
                    let _ = writeln!(f, "Accessibility trusted: {}", accessibility::is_accessibility_trusted());
                }

                let callback: event_tap::KeyEventCallback = Box::new(move |key_event: vim_anywhere_core::parser::KeyEvent| -> bool {
                    // Always pass through Cmd shortcuts
                    if key_event.modifiers.contains(&vim_anywhere_core::parser::Modifier::Command) {
                        return false;
                    }

                    // Pass through Ctrl/Option + non-character keys (system shortcuts)
                    if (key_event.modifiers.contains(&vim_anywhere_core::parser::Modifier::Control)
                        || key_event.modifiers.contains(&vim_anywhere_core::parser::Modifier::Option))
                        && !matches!(key_event.key, Key::Char(_))
                    {
                        return false;
                    }

                    // Check frontmost app
                    if let Some(app_info) = app_detection::get_frontmost_app() {
                        let cfg = state_for_tap.config.lock().unwrap();
                        let strategy = if let Some(app_cfg) = cfg.per_app.get(&app_info.bundle_id) {
                            match app_cfg.strategy.as_str() {
                                "disabled" => app_detection::Strategy::Disabled,
                                "keyboard" => app_detection::Strategy::Keyboard,
                                _ => app_detection::Strategy::Accessibility,
                            }
                        } else {
                            app_detection::default_strategy_for_app(&app_info.bundle_id)
                        };
                        drop(cfg);

                        if strategy == app_detection::Strategy::Disabled { return false; }
                        if app_info.bundle_id == "com.kennethsolomon.vim-anywhere" { return false; }
                    }

                    // Only intercept known vim Ctrl motions (b, d, f, u), pass through rest
                    if key_event.modifiers.contains(&vim_anywhere_core::parser::Modifier::Control) {
                        if let Key::Char(ch) = &key_event.key {
                            let cfg = state_for_tap.config.lock().unwrap();
                            let motion_key = format!("ctrl-{}", ch);
                            if cfg.disabled_motions.contains(&motion_key) {
                                return false;
                            }
                            drop(cfg);
                            if !matches!(ch, 'b' | 'd' | 'f' | 'u') {
                                return false;
                            }
                        }
                    }

                    let mut eng = match state_for_tap.engine.lock() {
                        Ok(e) => e,
                        Err(_) => return false,
                    };

                    let current_mode = eng.mode();

                    // ── Insert mode: pass ALL keys through (like SketchyVim) ──
                    if current_mode == Mode::Insert {
                        let is_escape = key_event.key == Key::Escape;
                        let is_possible_sequence = matches!(key_event.key, Key::Char(_));

                        if !is_escape && !is_possible_sequence {
                            return false; // pass through backspace, arrows, etc.
                        }

                        let mut dummy_buffer = InMemoryBuffer::new("");
                        let result = eng.handle_key(&key_event, &mut dummy_buffer);

                        return match result {
                            EngineResult::ModeChanged(new_mode) => {
                                // Mode exit (Escape or custom sequence) — suppress and sync
                                let _ = app_handle2.emit("mode-changed", ModeChangedPayload {
                                    mode: mode_to_string(new_mode),
                                });
                                // Set block cursor when entering normal mode
                                if new_mode == Mode::Normal {
                                    if let Some(el) = accessibility::get_focused_element() {
                                        if let Some((loc, _)) = accessibility::get_ax_selected_range(&el) {
                                            let _ = accessibility::set_ax_selected_range(&el, loc, 1);
                                        }
                                    }
                                }
                                true
                            }
                            EngineResult::SendRealEscape => {
                                // Double-escape: pass the real Escape key through to the app
                                false
                            }
                            _ => false, // still in insert — pass through
                        };
                    }

                    // ── Normal / Visual mode ─────────────────────────────────
                    // Get focused AX element — if we can't, pass through (like SketchyVim)
                    let element = match accessibility::get_focused_element() {
                        Some(el) => el,
                        None => {
                            // No focused element — still handle Escape to enter normal mode
                            if key_event.key == Key::Escape {
                                let mut dummy = InMemoryBuffer::new("");
                                let _ = eng.handle_key(&key_event, &mut dummy);
                                let _ = app_handle2.emit("mode-changed", ModeChangedPayload {
                                    mode: mode_to_string(eng.mode()),
                                });
                                return true;
                            }
                            return false;
                        }
                    };

                    // Read text and cursor from AX — if either fails, pass through
                    // (this implicitly checks the element is a text field we can work with)
                    let text = match accessibility::get_ax_value(&element) {
                        Some(t) => t,
                        None => {
                            // Not a text element — only handle Escape for mode management
                            if key_event.key == Key::Escape {
                                let mut dummy = InMemoryBuffer::new("");
                                let _ = eng.handle_key(&key_event, &mut dummy);
                                let _ = app_handle2.emit("mode-changed", ModeChangedPayload {
                                    mode: mode_to_string(eng.mode()),
                                });
                                return true;
                            }
                            return false;
                        }
                    };
                    let cursor_offset = accessibility::get_ax_selected_range(&element)
                        .map(|(loc, _)| loc)
                        .unwrap_or(0);

                    let (cursor_line, cursor_col) = accessibility::offset_to_cursor(&text, cursor_offset);

                    let mut buffer = InMemoryBuffer::new(&text);
                    buffer.set_cursor(CursorPosition::new(cursor_line, cursor_col));

                    // In visual mode, restore the selection from AX
                    if current_mode == Mode::VisualCharacterwise || current_mode == Mode::VisualLinewise {
                        if let Some((loc, len)) = accessibility::get_ax_selected_range(&element) {
                            if len > 0 {
                                let (anchor_line, anchor_col) = accessibility::offset_to_cursor(&text, loc);
                                let (head_line, head_col) = accessibility::offset_to_cursor(&text, loc + len);
                                let kind = if current_mode == Mode::VisualLinewise {
                                    SelectionKind::Linewise
                                } else {
                                    SelectionKind::Characterwise
                                };
                                buffer.set_selection(Some(vim_anywhere_core::buffer::Selection::new(
                                    CursorPosition::new(anchor_line, anchor_col),
                                    CursorPosition::new(head_line, head_col.saturating_sub(1)),
                                    kind,
                                )));
                                buffer.set_cursor(CursorPosition::new(head_line, head_col.saturating_sub(1)));
                            }
                        }
                    }

                    let result = eng.handle_key(&key_event, &mut buffer);

                    match result {
                        EngineResult::PassThrough => false,
                        EngineResult::Suppressed => true,
                        EngineResult::SendRealEscape => false,
                        EngineResult::ModeChanged(new_mode) => {
                            let _ = app_handle2.emit("mode-changed", ModeChangedPayload {
                                mode: mode_to_string(new_mode),
                            });
                            apply_buffer_to_ax(&element, &buffer, &text, new_mode);
                            true
                        }
                        EngineResult::BufferModified => {
                            let new_mode = eng.mode();
                            apply_buffer_to_ax(&element, &buffer, &text, new_mode);
                            if new_mode != current_mode {
                                let _ = app_handle2.emit("mode-changed", ModeChangedPayload {
                                    mode: mode_to_string(new_mode),
                                });
                            }
                            true
                        }
                    }
                });

                if let Err(e) = event_tap::start_event_tap(callback) {
                    eprintln!("[vim-anywhere] {}", e);
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .create(true).append(true)
                        .open("/tmp/vim-anywhere.log")
                    {
                        let _ = writeln!(f, "EVENT TAP FAILED: {}", e);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
