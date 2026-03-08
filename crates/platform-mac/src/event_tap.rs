use std::ffi::c_void;
use std::ptr;

use vim_anywhere_core::parser::{Key, KeyEvent, Modifier};

// Raw FFI — event tap creation
type CGEventTapCallBackRaw = unsafe extern "C" fn(
    proxy: *mut c_void,
    event_type: u32,
    event: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBackRaw,
        user_info: *mut c_void,
    ) -> *mut c_void;

    fn CGEventTapEnable(tap: *mut c_void, enable: bool);

    fn CGEventGetIntegerValueField(event: *mut c_void, field: u32) -> i64;
    fn CGEventGetFlags(event: *mut c_void) -> u64;
    fn CGEventKeyboardGetUnicodeString(
        event: *mut c_void,
        max_length: u64,
        actual_length: *mut u64,
        unicode_string: *mut u16,
    );
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: *mut c_void);
    fn CFRelease(cf: *const c_void);
}

extern "C" {
    static kCFRunLoopCommonModes: *const c_void;
}

// CGEventTapLocation — annotated session (like SketchyVim)
const K_CG_ANNOTATED_SESSION_EVENT_TAP: u32 = 2;
// CGEventTapPlacement
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
// CGEventTapOptions — active filter (can suppress events)
const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;

// Event masks — only KeyDown (like SketchyVim)
const K_CG_EVENT_KEY_DOWN: u64 = 1 << 10;

// CGEventType values
const CG_EVENT_KEY_DOWN: u32 = 10;
const CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
const CG_EVENT_TAP_DISABLED_BY_USER: u32 = 0xFFFFFFFF;

// EventField constants
const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const K_CG_KEYBOARD_EVENT_AUTOREPEAT: u32 = 11;

// CGEventFlags bit masks
const K_CG_EVENT_FLAG_SHIFT: u64 = 0x00020000;
const K_CG_EVENT_FLAG_CONTROL: u64 = 0x00040000;
const K_CG_EVENT_FLAG_ALTERNATE: u64 = 0x00080000;
const K_CG_EVENT_FLAG_COMMAND: u64 = 0x00100000;

/// Callback: receives a KeyEvent. Return `true` to suppress the event, `false` to pass through.
pub type KeyEventCallback = Box<dyn Fn(KeyEvent) -> bool + Send + 'static>;

struct TapContext {
    callback: KeyEventCallback,
    run_loop: *mut c_void,
    tap_port: *mut c_void,
}

unsafe impl Send for TapContext {}

static mut TAP_CONTEXT: *mut TapContext = ptr::null_mut();

fn flags_to_modifiers(flags: u64) -> Vec<Modifier> {
    let mut mods = vec![];
    if flags & K_CG_EVENT_FLAG_SHIFT != 0 {
        mods.push(Modifier::Shift);
    }
    if flags & K_CG_EVENT_FLAG_CONTROL != 0 {
        mods.push(Modifier::Control);
    }
    if flags & K_CG_EVENT_FLAG_ALTERNATE != 0 {
        mods.push(Modifier::Option);
    }
    if flags & K_CG_EVENT_FLAG_COMMAND != 0 {
        mods.push(Modifier::Command);
    }
    mods
}

/// Convert a keycode to a non-character Key (arrows, function keys, etc.)
fn keycode_to_special_key(keycode: u16) -> Option<Key> {
    match keycode {
        0x24 => Some(Key::Return),
        0x30 => Some(Key::Tab),
        0x33 => Some(Key::Backspace),
        0x35 => Some(Key::Escape),
        0x75 => Some(Key::Delete),
        0x7B => Some(Key::Left),
        0x7C => Some(Key::Right),
        0x7D => Some(Key::Down),
        0x7E => Some(Key::Up),
        0x73 => Some(Key::Home),
        0x77 => Some(Key::End),
        0x74 => Some(Key::PageUp),
        0x79 => Some(Key::PageDown),
        0x7A => Some(Key::F(1)),
        0x78 => Some(Key::F(2)),
        0x63 => Some(Key::F(3)),
        0x76 => Some(Key::F(4)),
        0x60 => Some(Key::F(5)),
        0x61 => Some(Key::F(6)),
        0x62 => Some(Key::F(7)),
        0x64 => Some(Key::F(8)),
        0x65 => Some(Key::F(9)),
        0x6D => Some(Key::F(10)),
        0x67 => Some(Key::F(11)),
        0x6F => Some(Key::F(12)),
        _ => None,
    }
}

unsafe extern "C" fn event_tap_callback(
    _proxy: *mut c_void,
    event_type: u32,
    event: *mut c_void,
    _user_info: *mut c_void,
) -> *mut c_void {
    // Re-enable tap if disabled by system (like SketchyVim)
    if event_type == CG_EVENT_TAP_DISABLED_BY_TIMEOUT
        || event_type == CG_EVENT_TAP_DISABLED_BY_USER
    {
        if !TAP_CONTEXT.is_null() {
            let ctx = &*TAP_CONTEXT;
            CGEventTapEnable(ctx.tap_port, true);
        }
        return event;
    }

    // Only process KeyDown
    if event_type != CG_EVENT_KEY_DOWN {
        return event;
    }

    if TAP_CONTEXT.is_null() {
        return event;
    }
    let ctx = &*TAP_CONTEXT;

    let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u16;
    let flags = CGEventGetFlags(event);
    let is_repeat = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_AUTOREPEAT) != 0;
    let modifiers = flags_to_modifiers(flags);

    // Use CGEventKeyboardGetUnicodeString to get the actual typed character
    // This is layout-aware — works with any keyboard layout (like SketchyVim)
    let key = if let Some(special) = keycode_to_special_key(keycode) {
        special
    } else {
        let mut actual_length: u64 = 0;
        let mut unicode_char: u16 = 0;
        CGEventKeyboardGetUnicodeString(event, 1, &mut actual_length, &mut unicode_char);

        if actual_length > 0 {
            if let Some(ch) = char::from_u32(unicode_char as u32) {
                // For Ctrl+letter, the unicode string returns control characters (0x01-0x1A).
                // Map them back to the letter (Ctrl+A = 0x01 → 'a', etc.)
                if modifiers.contains(&Modifier::Control) && (1..=26).contains(&unicode_char) {
                    Key::Char((b'a' + unicode_char as u8 - 1) as char)
                } else {
                    Key::Char(ch)
                }
            } else {
                Key::Unknown(keycode)
            }
        } else {
            Key::Unknown(keycode)
        }
    };

    let key_event = KeyEvent {
        key,
        modifiers,
        is_repeat,
    };

    let suppress = (ctx.callback)(key_event);

    if suppress {
        ptr::null_mut()
    } else {
        event
    }
}

/// Start the global event tap on the current thread. **Blocks** until `stop_event_tap()` is called.
pub fn start_event_tap(callback: KeyEventCallback) -> Result<(), String> {
    // Only listen for KeyDown (like SketchyVim)
    let events_of_interest = K_CG_EVENT_KEY_DOWN;

    unsafe {
        let tap = CGEventTapCreate(
            K_CG_ANNOTATED_SESSION_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_DEFAULT,
            events_of_interest,
            event_tap_callback,
            ptr::null_mut(),
        );

        if tap.is_null() {
            return Err(
                "Failed to create event tap. Grant Input Monitoring permission in System Settings."
                    .to_string(),
            );
        }

        let run_loop_source = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
        if run_loop_source.is_null() {
            CFRelease(tap);
            return Err("Failed to create run loop source.".to_string());
        }

        let run_loop = CFRunLoopGetCurrent();

        let ctx = Box::new(TapContext {
            callback,
            run_loop,
            tap_port: tap,
        });
        TAP_CONTEXT = Box::into_raw(ctx);

        CFRunLoopAddSource(run_loop, run_loop_source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);

        CFRunLoopRun(); // blocks until stopped

        // Cleanup
        CGEventTapEnable(tap, false);
        let _ = Box::from_raw(TAP_CONTEXT);
        TAP_CONTEXT = ptr::null_mut();
        CFRelease(run_loop_source);
        CFRelease(tap);
    }

    Ok(())
}

/// Stop the event tap (safe to call from any thread).
pub fn stop_event_tap() {
    unsafe {
        if !TAP_CONTEXT.is_null() {
            let ctx = &*TAP_CONTEXT;
            CFRunLoopStop(ctx.run_loop);
        }
    }
}
