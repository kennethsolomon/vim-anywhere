use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, EventField};
use core_graphics::event_source::CGEventSource;
use core_graphics::event_source::CGEventSourceStateID;
use vim_anywhere_core::parser::{Key, KeyEvent, Modifier};

pub fn cgevent_flags_to_modifiers(flags: CGEventFlags) -> Vec<Modifier> {
    let mut mods = vec![];
    if flags.contains(CGEventFlags::CGEventFlagShift) {
        mods.push(Modifier::Shift);
    }
    if flags.contains(CGEventFlags::CGEventFlagControl) {
        mods.push(Modifier::Control);
    }
    if flags.contains(CGEventFlags::CGEventFlagAlternate) {
        mods.push(Modifier::Option);
    }
    if flags.contains(CGEventFlags::CGEventFlagCommand) {
        mods.push(Modifier::Command);
    }
    mods
}

pub fn keycode_to_key(keycode: u16, _flags: CGEventFlags) -> Key {
    match keycode {
        0x00 => Key::Char('a'),
        0x01 => Key::Char('s'),
        0x02 => Key::Char('d'),
        0x03 => Key::Char('f'),
        0x04 => Key::Char('h'),
        0x05 => Key::Char('g'),
        0x06 => Key::Char('z'),
        0x07 => Key::Char('x'),
        0x08 => Key::Char('c'),
        0x09 => Key::Char('v'),
        0x0B => Key::Char('b'),
        0x0C => Key::Char('q'),
        0x0D => Key::Char('w'),
        0x0E => Key::Char('e'),
        0x0F => Key::Char('r'),
        0x10 => Key::Char('y'),
        0x11 => Key::Char('t'),
        0x12 => Key::Char('1'),
        0x13 => Key::Char('2'),
        0x14 => Key::Char('3'),
        0x15 => Key::Char('4'),
        0x16 => Key::Char('6'),
        0x17 => Key::Char('5'),
        0x18 => Key::Char('='),
        0x19 => Key::Char('9'),
        0x1A => Key::Char('7'),
        0x1B => Key::Char('-'),
        0x1C => Key::Char('8'),
        0x1D => Key::Char('0'),
        0x1E => Key::Char(']'),
        0x1F => Key::Char('o'),
        0x20 => Key::Char('u'),
        0x21 => Key::Char('['),
        0x22 => Key::Char('i'),
        0x23 => Key::Char('p'),
        0x25 => Key::Char('l'),
        0x26 => Key::Char('j'),
        0x27 => Key::Char('\''),
        0x28 => Key::Char('k'),
        0x29 => Key::Char(';'),
        0x2A => Key::Char('\\'),
        0x2B => Key::Char(','),
        0x2C => Key::Char('/'),
        0x2D => Key::Char('n'),
        0x2E => Key::Char('m'),
        0x2F => Key::Char('.'),
        0x32 => Key::Char('`'),
        0x24 => Key::Return,
        0x30 => Key::Tab,
        0x31 => Key::Char(' '),
        0x33 => Key::Backspace,
        0x35 => Key::Escape,
        0x7B => Key::Left,
        0x7C => Key::Right,
        0x7D => Key::Down,
        0x7E => Key::Up,
        0x73 => Key::Home,
        0x77 => Key::End,
        0x74 => Key::PageUp,
        0x79 => Key::PageDown,
        0x75 => Key::Delete,
        0x7A => Key::F(1),
        0x78 => Key::F(2),
        0x63 => Key::F(3),
        0x76 => Key::F(4),
        0x60 => Key::F(5),
        0x61 => Key::F(6),
        0x62 => Key::F(7),
        0x64 => Key::F(8),
        0x65 => Key::F(9),
        0x6D => Key::F(10),
        0x67 => Key::F(11),
        0x6F => Key::F(12),
        other => Key::Unknown(other),
    }
}

pub fn apply_shift_to_key(key: Key, has_shift: bool) -> Key {
    if !has_shift {
        return key;
    }
    match key {
        Key::Char(ch) => {
            let shifted = match ch {
                'a'..='z' => ch.to_ascii_uppercase(),
                '1' => '!',
                '2' => '@',
                '3' => '#',
                '4' => '$',
                '5' => '%',
                '6' => '^',
                '7' => '&',
                '8' => '*',
                '9' => '(',
                '0' => ')',
                '-' => '_',
                '=' => '+',
                '[' => '{',
                ']' => '}',
                '\\' => '|',
                ';' => ':',
                '\'' => '"',
                ',' => '<',
                '.' => '>',
                '/' => '?',
                '`' => '~',
                other => other,
            };
            Key::Char(shifted)
        }
        other => other,
    }
}

pub fn cgevent_to_key_event(event: &CGEvent) -> KeyEvent {
    let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
    let flags = event.get_flags();
    let is_repeat =
        event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0;
    let modifiers = cgevent_flags_to_modifiers(flags);
    let has_shift = modifiers.contains(&Modifier::Shift);
    let raw_key = keycode_to_key(keycode, flags);
    let key = apply_shift_to_key(raw_key, has_shift);

    KeyEvent {
        key,
        modifiers,
        is_repeat,
    }
}

pub fn send_key_event(keycode: u16, key_down: bool, flags: CGEventFlags) {
    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[vim-anywhere] warning: failed to create CGEventSource for keycode {}", keycode);
            return;
        }
    };
    match CGEvent::new_keyboard_event(source, keycode, key_down) {
        Ok(event) => {
            event.set_flags(flags);
            event.post(CGEventTapLocation::HID);
        }
        Err(_) => {
            eprintln!("[vim-anywhere] warning: failed to create keyboard event for keycode {}", keycode);
        }
    }
}

pub fn send_key_sequence(keycode: u16, flags: CGEventFlags) {
    send_key_event(keycode, true, flags);
    send_key_event(keycode, false, flags);
}

/// Simulate Cmd+Z (native undo).
pub fn send_undo() {
    send_key_sequence(0x06, CGEventFlags::CGEventFlagCommand);
}

/// Simulate Cmd+Shift+Z (native redo).
pub fn send_redo() {
    send_key_sequence(
        0x06,
        CGEventFlags::CGEventFlagCommand | CGEventFlags::CGEventFlagShift,
    );
}
