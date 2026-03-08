use vim_anywhere::{Engine, EngineResult};
use vim_anywhere_core::buffer::{CursorPosition, InMemoryBuffer, TextBuffer};
use vim_anywhere_core::modes::{Mode, ModeEntryConfig};
use vim_anywhere_core::parser::KeyEvent;

fn main() {
    println!("vim-anywhere v0.1.0");
    println!("Run the Tauri UI from ui/ directory: cd ui && npm run tauri dev");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> Engine {
        Engine::new(ModeEntryConfig::default())
    }

    fn make_buffer(text: &str) -> InMemoryBuffer {
        InMemoryBuffer::new(text)
    }

    #[test]
    fn engine_starts_normal() {
        let engine = make_engine();
        assert_eq!(engine.mode(), Mode::Normal);
    }

    #[test]
    fn hjkl_navigation() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello\nworld");

        engine.handle_key(&KeyEvent::char('l'), &mut buf);
        assert_eq!(buf.get_cursor(), CursorPosition::new(0, 1));

        engine.handle_key(&KeyEvent::char('j'), &mut buf);
        assert_eq!(buf.get_cursor(), CursorPosition::new(1, 1));

        engine.handle_key(&KeyEvent::char('h'), &mut buf);
        assert_eq!(buf.get_cursor(), CursorPosition::new(1, 0));

        engine.handle_key(&KeyEvent::char('k'), &mut buf);
        assert_eq!(buf.get_cursor(), CursorPosition::new(0, 0));
    }

    #[test]
    fn dd_deletes_line() {
        let mut engine = make_engine();
        let mut buf = make_buffer("aaa\nbbb\nccc");

        engine.handle_key(&KeyEvent::char('d'), &mut buf);
        engine.handle_key(&KeyEvent::char('d'), &mut buf);

        assert_eq!(buf.get_text(), "bbb\nccc");
    }

    #[test]
    fn ciw_changes_word() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello world");
        buf.set_cursor(CursorPosition::new(0, 1)); // on 'e' in "hello"

        engine.handle_key(&KeyEvent::char('c'), &mut buf);
        engine.handle_key(&KeyEvent::char('i'), &mut buf);
        engine.handle_key(&KeyEvent::char('w'), &mut buf);

        assert_eq!(buf.get_text(), " world");
        assert_eq!(engine.mode(), Mode::Insert);
    }

    #[test]
    fn yy_p_yank_paste() {
        let mut engine = make_engine();
        let mut buf = make_buffer("aaa\nbbb");

        // yy
        engine.handle_key(&KeyEvent::char('y'), &mut buf);
        engine.handle_key(&KeyEvent::char('y'), &mut buf);

        // p (paste after)
        engine.handle_key(&KeyEvent::char('p'), &mut buf);

        assert_eq!(buf.get_text(), "aaa\naaa\nbbb");
    }

    #[test]
    fn enter_insert_and_escape() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello");

        let result = engine.handle_key(&KeyEvent::char('i'), &mut buf);
        assert_eq!(result, EngineResult::ModeChanged(Mode::Insert));
        assert_eq!(engine.mode(), Mode::Insert);

        // Characters pass through in insert mode
        let result = engine.handle_key(&KeyEvent::char('x'), &mut buf);
        assert_eq!(result, EngineResult::PassThrough);

        // Escape returns to normal
        let result = engine.handle_key(&KeyEvent::escape(), &mut buf);
        assert_eq!(result, EngineResult::ModeChanged(Mode::Normal));
        assert_eq!(engine.mode(), Mode::Normal);
    }

    #[test]
    fn visual_mode_select_and_delete() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello world");

        engine.handle_key(&KeyEvent::char('v'), &mut buf); // enter visual
        assert_eq!(engine.mode(), Mode::VisualCharacterwise);

        engine.handle_key(&KeyEvent::char('l'), &mut buf);
        engine.handle_key(&KeyEvent::char('l'), &mut buf);
        engine.handle_key(&KeyEvent::char('l'), &mut buf);
        engine.handle_key(&KeyEvent::char('l'), &mut buf); // select "hello"

        engine.handle_key(&KeyEvent::char('d'), &mut buf); // delete
        assert_eq!(engine.mode(), Mode::Normal);
        assert_eq!(buf.get_text(), " world");
    }

    #[test]
    fn replace_char() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello");

        engine.handle_key(&KeyEvent::char('r'), &mut buf);
        engine.handle_key(&KeyEvent::char('X'), &mut buf);

        assert_eq!(buf.get_text(), "Xello");
    }

    #[test]
    fn toggle_case() {
        let mut engine = make_engine();
        let mut buf = make_buffer("Hello");

        engine.handle_key(&KeyEvent::char('~'), &mut buf);
        assert_eq!(buf.get_text(), "hello");
    }

    #[test]
    fn join_lines() {
        let mut engine = make_engine();
        let mut buf = make_buffer("aaa\n  bbb");

        engine.handle_key(&KeyEvent::char('J'), &mut buf);
        assert_eq!(buf.get_text(), "aaa bbb");
    }

    #[test]
    fn indent_line() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello");

        engine.handle_key(&KeyEvent::char('>'), &mut buf);
        engine.handle_key(&KeyEvent::char('>'), &mut buf);
        assert_eq!(buf.get_text(), "    hello");
    }

    #[test]
    fn dw_delete_word() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello world");

        engine.handle_key(&KeyEvent::char('d'), &mut buf);
        engine.handle_key(&KeyEvent::char('w'), &mut buf);

        // dw deletes from cursor to start of next word
        assert_eq!(buf.get_text(), "world");
    }

    #[test]
    fn count_motion() {
        let mut engine = make_engine();
        let mut buf = make_buffer("hello");

        engine.handle_key(&KeyEvent::char('3'), &mut buf);
        engine.handle_key(&KeyEvent::char('l'), &mut buf);
        assert_eq!(buf.get_cursor(), CursorPosition::new(0, 3));
    }
}
