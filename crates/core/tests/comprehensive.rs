use vim_anywhere_core::buffer::{
    CursorPosition, InMemoryBuffer, Selection, SelectionKind, TextBuffer,
};
use vim_anywhere_core::modes::{
    InsertVariant, Mode, ModeEntryConfig, ModeStateMachine, ModeTransition,
};
use vim_anywhere_core::motions::*;

fn buf(text: &str) -> InMemoryBuffer {
    InMemoryBuffer::new(text)
}

fn buf_at(text: &str, line: usize, col: usize) -> InMemoryBuffer {
    let mut b = InMemoryBuffer::new(text);
    b.set_cursor(CursorPosition::new(line, col));
    b
}

// =============================================================================
// Motions gaps
// =============================================================================
mod motion_tests {
    use super::*;

    #[test]
    fn word_forward_big() {
        let b = buf_at("hello.world foo", 0, 0);
        // W skips entire non-whitespace WORD
        let pos = motion_word_forward_big(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 12));
    }

    #[test]
    fn word_forward_big_multiple() {
        let b = buf_at("aaa bbb ccc", 0, 0);
        let pos = motion_word_forward_big(&b, 2);
        assert_eq!(pos, CursorPosition::new(0, 8));
    }

    #[test]
    fn word_backward_big() {
        let b = buf_at("hello.world foo", 0, 12);
        let pos = motion_word_backward_big(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 0));
    }

    #[test]
    fn word_backward_big_at_start() {
        let b = buf_at("aaa bbb", 0, 0);
        let pos = motion_word_backward_big(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 0));
    }

    #[test]
    fn word_end_big() {
        let b = buf_at("hello.world foo", 0, 0);
        // E lands on last char of WORD
        let pos = motion_word_end_big(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 10));
    }

    #[test]
    fn word_end_big_crosses_to_next_word() {
        let b = buf_at("aa bb", 0, 1);
        // Already at end of first WORD, E goes to end of next
        let pos = motion_word_end_big(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 4));
    }

    #[test]
    fn word_end_backward_ge() {
        // From col 12 ('t' in "test"), ge goes back past space to "world", landing at start
        let b = buf_at("hello world test", 0, 12);
        let pos = motion_word_end_backward(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 6));
    }

    #[test]
    fn word_end_backward_at_col_zero() {
        let b = buf_at("hello\nworld", 1, 0);
        let pos = motion_word_end_backward(&b, 1);
        // Goes to end of previous line
        assert_eq!(pos.line, 0);
    }

    #[test]
    fn last_non_blank_g_underscore() {
        let b = buf_at("  hello  ", 0, 0);
        let pos = motion_last_non_blank(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 6));
    }

    #[test]
    fn last_non_blank_no_trailing_spaces() {
        let b = buf_at("hello", 0, 0);
        let pos = motion_last_non_blank(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 4));
    }

    #[test]
    fn goto_last_line() {
        let b = buf("abc\ndef\nghi");
        let pos = motion_goto_last_line(&b, 1);
        assert_eq!(pos, CursorPosition::new(2, 0));
    }

    #[test]
    fn goto_last_line_with_indent() {
        let b = buf("abc\ndef\n  ghi");
        let pos = motion_goto_last_line(&b, 1);
        assert_eq!(pos, CursorPosition::new(2, 2));
    }

    #[test]
    fn til_char_back() {
        let b = buf_at("abcdef", 0, 5);
        let pos = motion_til_char_back(&b, 'b', 1);
        // T lands one char after found char
        assert_eq!(pos, CursorPosition::new(0, 2));
    }

    #[test]
    fn til_char_back_not_found() {
        let b = buf_at("abcdef", 0, 5);
        let pos = motion_til_char_back(&b, 'z', 1);
        assert_eq!(pos, CursorPosition::new(0, 5)); // stays
    }

    #[test]
    fn paragraph_backward() {
        let b = buf_at("aaa\nbbb\n\nccc\nddd", 3, 0);
        let pos = motion_paragraph_backward(&b, 1);
        assert!(pos.line <= 2);
    }

    #[test]
    fn paragraph_backward_at_top() {
        let b = buf_at("aaa\nbbb", 0, 0);
        let pos = motion_paragraph_backward(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 0));
    }

    #[test]
    fn prev_first_non_blank_minus() {
        let b = buf_at("  abc\n  def\n  ghi", 2, 0);
        let pos = motion_prev_first_non_blank(&b, 1);
        assert_eq!(pos, CursorPosition::new(1, 2));
    }

    #[test]
    fn prev_first_non_blank_at_top() {
        let b = buf_at("  abc", 0, 4);
        let pos = motion_prev_first_non_blank(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 2));
    }

    #[test]
    fn line_end_empty_line() {
        let b = buf_at("abc\n\ndef", 1, 0);
        let pos = motion_line_end(&b, 1);
        assert_eq!(pos, CursorPosition::new(1, 0));
    }

    #[test]
    fn first_non_blank_all_whitespace() {
        let b = buf_at("     ", 0, 3);
        let pos = motion_first_non_blank(&b, 1);
        // No non-blank found, returns 0
        assert_eq!(pos, CursorPosition::new(0, 0));
    }

    #[test]
    fn word_forward_crosses_lines() {
        let b = buf_at("hello\nworld", 0, 0);
        let pos = motion_word_forward(&b, 1);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn word_forward_crosses_multiple_lines() {
        let b = buf_at("aa\nbb\ncc", 0, 0);
        let pos = motion_word_forward(&b, 2);
        assert_eq!(pos, CursorPosition::new(2, 0));
    }

    #[test]
    fn word_backward_crosses_lines() {
        let b = buf_at("hello\nworld", 1, 0);
        let pos = motion_word_backward(&b, 1);
        assert_eq!(pos.line, 0);
    }

    #[test]
    fn match_bracket_curly() {
        let b = buf_at("{ foo }", 0, 0);
        let pos = motion_match_bracket(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 6));

        let b2 = buf_at("{ foo }", 0, 6);
        let pos2 = motion_match_bracket(&b2, 1);
        assert_eq!(pos2, CursorPosition::new(0, 0));
    }

    #[test]
    fn match_bracket_square() {
        let b = buf_at("[a, b]", 0, 0);
        let pos = motion_match_bracket(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 5));

        let b2 = buf_at("[a, b]", 0, 5);
        let pos2 = motion_match_bracket(&b2, 1);
        assert_eq!(pos2, CursorPosition::new(0, 0));
    }

    #[test]
    fn match_bracket_not_a_bracket_stays() {
        let b = buf_at("hello", 0, 2);
        let pos = motion_match_bracket(&b, 1);
        assert_eq!(pos, CursorPosition::new(0, 2));
    }

    #[test]
    fn find_char_not_found_stays() {
        let b = buf_at("abcdef", 0, 0);
        let pos = motion_find_char(&b, 'z', 1);
        assert_eq!(pos, CursorPosition::new(0, 0));
    }

    #[test]
    fn find_char_back_not_found_stays() {
        let b = buf_at("abcdef", 0, 5);
        let pos = motion_find_char_back(&b, 'z', 1);
        assert_eq!(pos, CursorPosition::new(0, 5));
    }
}

// =============================================================================
// Text object gaps
// =============================================================================
mod text_object_tests {
    use super::*;

    #[test]
    fn inner_word_big() {
        // iW treats punctuation as part of WORD
        let b = buf_at("foo.bar baz", 0, 1);
        let (start, end) = text_object_inner_word_big(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        assert_eq!(end, CursorPosition::new(0, 7));
    }

    #[test]
    fn a_word_big() {
        let b = buf_at("foo.bar baz", 0, 1);
        let (start, end) = text_object_a_word_big(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        // aW includes trailing whitespace
        assert_eq!(end, CursorPosition::new(0, 8));
    }

    #[test]
    fn inner_bracket() {
        let b = buf_at("x[abc]y", 0, 3);
        let (start, end) = text_object_inner_bracket(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 2));
        assert_eq!(end, CursorPosition::new(0, 5));
    }

    #[test]
    fn a_bracket() {
        let b = buf_at("x[abc]y", 0, 3);
        let (start, end) = text_object_a_bracket(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 1));
        assert_eq!(end, CursorPosition::new(0, 6));
    }

    #[test]
    fn inner_angle() {
        let b = buf_at("x<abc>y", 0, 3);
        let (start, end) = text_object_inner_angle(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 2));
        assert_eq!(end, CursorPosition::new(0, 5));
    }

    #[test]
    fn a_angle() {
        let b = buf_at("x<abc>y", 0, 3);
        let (start, end) = text_object_a_angle(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 1));
        assert_eq!(end, CursorPosition::new(0, 6));
    }

    #[test]
    fn inner_single_quote() {
        let b = buf_at("say 'hello' ok", 0, 6);
        let (start, end) = text_object_inner_single_quote(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 5));
        assert_eq!(end, CursorPosition::new(0, 10));
    }

    #[test]
    fn a_single_quote() {
        let b = buf_at("say 'hello' ok", 0, 6);
        let (start, end) = text_object_a_single_quote(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 4));
        assert_eq!(end, CursorPosition::new(0, 11));
    }

    #[test]
    fn inner_backtick() {
        let b = buf_at("say `hello` ok", 0, 6);
        let (start, end) = text_object_inner_backtick(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 5));
        assert_eq!(end, CursorPosition::new(0, 10));
    }

    #[test]
    fn a_backtick() {
        let b = buf_at("say `hello` ok", 0, 6);
        let (start, end) = text_object_a_backtick(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 4));
        assert_eq!(end, CursorPosition::new(0, 11));
    }

    #[test]
    fn inner_sentence() {
        let b = buf_at("Hello world. Goodbye.", 0, 2);
        let (start, end) = text_object_inner_sentence(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        // Should end at the period
        assert_eq!(end, CursorPosition::new(0, 12));
    }

    #[test]
    fn a_sentence_includes_trailing_space() {
        let b = buf_at("Hello. Goodbye.", 0, 2);
        let (start, end) = text_object_a_sentence(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        // 'a' sentence includes trailing whitespace after period
        assert!(end.col > 6);
    }

    #[test]
    fn a_paragraph() {
        let b = buf_at("aaa\nbbb\n\nccc", 0, 0);
        let (start, end) = text_object_a_paragraph(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        // a_paragraph includes trailing blank lines
        assert!(end.line >= 2);
    }

    #[test]
    fn text_object_at_word_start() {
        let b = buf_at("hello world", 0, 0);
        let (start, end) = text_object_inner_word(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        assert_eq!(end, CursorPosition::new(0, 5));
    }

    #[test]
    fn text_object_at_word_end() {
        let b = buf_at("hello world", 0, 4);
        let (start, end) = text_object_inner_word(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        assert_eq!(end, CursorPosition::new(0, 5));
    }

    #[test]
    fn nested_bracket_text_objects() {
        // Cursor inside inner parens
        let b = buf_at("((inner))", 0, 4);
        let (start, end) = text_object_inner_paren(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 2));
        assert_eq!(end, CursorPosition::new(0, 7));
    }

    #[test]
    fn nested_bracket_outer() {
        // Cursor on outer paren
        let b = buf_at("((inner))", 0, 0);
        let (start, end) = text_object_inner_paren(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 1));
        assert_eq!(end, CursorPosition::new(0, 8));
    }

    #[test]
    fn quote_cursor_on_quote_char() {
        let b = buf_at("say \"hello\" ok", 0, 4);
        let result = text_object_inner_double_quote(&b);
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        assert_eq!(start, CursorPosition::new(0, 5));
        assert_eq!(end, CursorPosition::new(0, 10));
    }

    #[test]
    fn bracket_no_match_returns_none() {
        let b = buf_at("no brackets here", 0, 3);
        assert!(text_object_inner_paren(&b).is_none());
    }
}

// =============================================================================
// Buffer gaps
// =============================================================================
mod buffer_tests {
    use super::*;

    #[test]
    fn text_range_reversed_start_end() {
        let b = buf("hello world");
        // Pass end before start -- implementation should handle it
        let range = b.text_range(CursorPosition::new(0, 5), CursorPosition::new(0, 0));
        assert_eq!(range, "hello");
    }

    #[test]
    fn insert_at_multiline_text() {
        let mut b = buf("ab");
        b.insert_at(CursorPosition::new(0, 1), "X\nY");
        assert_eq!(b.get_text(), "aX\nYb");
        assert_eq!(b.line_count(), 2);
    }

    #[test]
    fn insert_at_end_of_buffer() {
        let mut b = buf("abc");
        b.insert_at(CursorPosition::new(0, 3), "def");
        assert_eq!(b.get_text(), "abcdef");
    }

    #[test]
    fn set_selection_and_get_selection() {
        let mut b = buf("hello world");
        assert!(b.get_selection().is_none());

        let sel = Selection::new(
            CursorPosition::new(0, 0),
            CursorPosition::new(0, 4),
            SelectionKind::Characterwise,
        );
        b.set_selection(Some(sel.clone()));
        let got = b.get_selection().unwrap();
        assert_eq!(got.anchor, CursorPosition::new(0, 0));
        assert_eq!(got.head, CursorPosition::new(0, 4));
        assert_eq!(got.kind, SelectionKind::Characterwise);

        b.set_selection(None);
        assert!(b.get_selection().is_none());
    }

    #[test]
    fn set_selection_linewise() {
        let mut b = buf("abc\ndef");
        let sel = Selection::new(
            CursorPosition::new(0, 0),
            CursorPosition::new(1, 2),
            SelectionKind::Linewise,
        );
        b.set_selection(Some(sel));
        let got = b.get_selection().unwrap();
        assert_eq!(got.kind, SelectionKind::Linewise);
    }

    #[test]
    fn preferred_col_getter_setter() {
        let mut b = buf("abc");
        assert_eq!(b.preferred_col(), None);

        b.set_preferred_col(Some(5));
        assert_eq!(b.preferred_col(), Some(5));

        b.set_preferred_col(None);
        assert_eq!(b.preferred_col(), None);
    }

    #[test]
    fn multiple_operations_sequence() {
        let mut b = buf("hello world");
        b.set_cursor(CursorPosition::new(0, 5));
        assert_eq!(b.get_cursor(), CursorPosition::new(0, 5));

        // Insert text
        b.insert_at(CursorPosition::new(0, 5), " beautiful");
        assert_eq!(b.get_text(), "hello beautiful world");

        // Replace range
        b.replace_range(
            CursorPosition::new(0, 6),
            CursorPosition::new(0, 15),
            "great",
        );
        assert_eq!(b.get_text(), "hello great world");

        // Set text entirely
        b.set_text("new content");
        assert_eq!(b.get_text(), "new content");
        assert_eq!(b.line_count(), 1);
    }

    #[test]
    fn text_range_cross_line() {
        let b = buf("abc\ndef\nghi");
        let range = b.text_range(CursorPosition::new(0, 0), CursorPosition::new(2, 3));
        assert_eq!(range, "abc\ndef\nghi");
    }

    #[test]
    fn text_range_same_position() {
        let b = buf("hello");
        let range = b.text_range(CursorPosition::new(0, 2), CursorPosition::new(0, 2));
        assert_eq!(range, "");
    }
}

// =============================================================================
// Modes gaps
// =============================================================================
mod mode_tests {
    use super::*;

    fn make_sm() -> ModeStateMachine {
        ModeStateMachine::new(ModeEntryConfig::default())
    }

    #[test]
    fn control_bracket_disabled() {
        let mut sm = ModeStateMachine::new(ModeEntryConfig {
            control_bracket: false,
            ..Default::default()
        });
        sm.enter_insert(InsertVariant::I);
        let t = sm.handle_control_bracket();
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Insert); // stays in insert
    }

    #[test]
    fn handle_insert_char_not_in_insert() {
        let mut sm = make_sm();
        // In Normal mode, insert char should do nothing
        let t = sm.handle_insert_char('j');
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn handle_insert_char_no_custom_sequence() {
        let mut sm = make_sm(); // default has no custom_sequence
        sm.enter_insert(InsertVariant::I);
        let t = sm.handle_insert_char('j');
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Insert);
    }

    #[test]
    fn pending_sequence_char_getter() {
        let mut sm = ModeStateMachine::new(ModeEntryConfig {
            custom_sequence: Some(['j', 'k']),
            ..Default::default()
        });
        sm.enter_insert(InsertVariant::I);
        assert_eq!(sm.pending_sequence_char(), None);

        sm.handle_insert_char('j');
        assert_eq!(sm.pending_sequence_char(), Some('j'));

        // After completing sequence, pending is cleared
        sm.handle_insert_char('k');
        assert_eq!(sm.pending_sequence_char(), None);
    }

    #[test]
    fn set_mode_direct() {
        let mut sm = make_sm();
        assert_eq!(sm.mode(), Mode::Normal);

        sm.set_mode(Mode::Insert);
        assert_eq!(sm.mode(), Mode::Insert);

        sm.set_mode(Mode::VisualLinewise);
        assert_eq!(sm.mode(), Mode::VisualLinewise);
    }

    #[test]
    fn visual_linewise_escape_to_normal() {
        let mut sm = make_sm();
        sm.enter_visual_linewise();
        assert_eq!(sm.mode(), Mode::VisualLinewise);

        let t = sm.handle_escape();
        assert_eq!(t, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn enter_visual_from_insert_ignored() {
        let mut sm = make_sm();
        sm.enter_insert(InsertVariant::I);
        assert_eq!(sm.mode(), Mode::Insert);

        let t = sm.enter_visual_characterwise();
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Insert);
    }

    #[test]
    fn enter_visual_linewise_from_insert_ignored() {
        let mut sm = make_sm();
        sm.enter_insert(InsertVariant::I);
        let t = sm.enter_visual_linewise();
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Insert);
    }

    #[test]
    fn escape_in_normal_returns_none() {
        let mut sm = ModeStateMachine::new(ModeEntryConfig {
            double_escape_sends_real: false,
            ..Default::default()
        });
        let t = sm.handle_escape();
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn control_bracket_in_visual_linewise() {
        let mut sm = make_sm();
        sm.enter_visual_linewise();
        let t = sm.handle_control_bracket();
        assert_eq!(t, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }
}
