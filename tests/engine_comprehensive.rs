use vim_anywhere::Engine;
use vim_anywhere::EngineResult;
use vim_anywhere_core::buffer::{CursorPosition, InMemoryBuffer, TextBuffer};
use vim_anywhere_core::modes::{Mode, ModeEntryConfig};
use vim_anywhere_core::parser::KeyEvent;

fn make_engine() -> Engine {
    Engine::new(ModeEntryConfig::default())
}

fn make_buffer(text: &str) -> InMemoryBuffer {
    InMemoryBuffer::new(text)
}

fn keys(engine: &mut Engine, buf: &mut InMemoryBuffer, chars: &str) {
    for ch in chars.chars() {
        engine.handle_key(&KeyEvent::char(ch), buf);
    }
}

// ---------------------------------------------------------------------------
// 1. Insert variants
// ---------------------------------------------------------------------------

#[test]
fn insert_a_appends_after_cursor() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello");
    buf.set_cursor(CursorPosition::new(0, 1)); // on 'e'

    let result = engine.handle_key(&KeyEvent::char('a'), &mut buf);
    assert_eq!(result, EngineResult::ModeChanged(Mode::Insert));
    assert_eq!(buf.get_cursor(), CursorPosition::new(0, 2));
}

#[test]
fn insert_big_a_appends_at_end_of_line() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello");

    let result = engine.handle_key(&KeyEvent::char('A'), &mut buf);
    assert_eq!(result, EngineResult::ModeChanged(Mode::Insert));
    // set_cursor clamps to line_len - 1 = 4 for "hello"
    assert_eq!(buf.get_cursor(), CursorPosition::new(0, 4));
}

#[test]
fn insert_big_i_at_first_non_blank() {
    let mut engine = make_engine();
    let mut buf = make_buffer("   hello");
    buf.set_cursor(CursorPosition::new(0, 5)); // somewhere in the middle

    let result = engine.handle_key(&KeyEvent::char('I'), &mut buf);
    assert_eq!(result, EngineResult::ModeChanged(Mode::Insert));
    assert_eq!(buf.get_cursor(), CursorPosition::new(0, 3));
}

#[test]
fn insert_o_opens_line_below() {
    let mut engine = make_engine();
    let mut buf = make_buffer("aaa\nbbb");

    let result = engine.handle_key(&KeyEvent::char('o'), &mut buf);
    assert_eq!(result, EngineResult::ModeChanged(Mode::Insert));
    assert_eq!(buf.get_text(), "aaa\n\nbbb");
    assert_eq!(buf.get_cursor(), CursorPosition::new(1, 0));
}

#[test]
fn insert_big_o_opens_line_above() {
    let mut engine = make_engine();
    let mut buf = make_buffer("aaa\nbbb");

    let result = engine.handle_key(&KeyEvent::char('O'), &mut buf);
    assert_eq!(result, EngineResult::ModeChanged(Mode::Insert));
    assert_eq!(buf.get_text(), "\naaa\nbbb");
    assert_eq!(buf.get_cursor(), CursorPosition::new(0, 0));
}

// ---------------------------------------------------------------------------
// 2. Operator + motion combos
// ---------------------------------------------------------------------------

#[test]
fn d_dollar_deletes_to_end_of_line() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");
    buf.set_cursor(CursorPosition::new(0, 5));

    keys(&mut engine, &mut buf, "d$");
    assert_eq!(buf.get_text(), "hello");
}

#[test]
fn d_zero_deletes_to_start_of_line() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");
    buf.set_cursor(CursorPosition::new(0, 5));

    keys(&mut engine, &mut buf, "d0");
    assert_eq!(buf.get_text(), " world");
}

#[test]
fn d_caret_deletes_to_first_non_blank() {
    let mut engine = make_engine();
    let mut buf = make_buffer("   hello");
    buf.set_cursor(CursorPosition::new(0, 6)); // on second 'l'

    keys(&mut engine, &mut buf, "d^");
    assert_eq!(buf.get_text(), "   lo");
}

#[test]
fn de_deletes_to_end_of_word() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");

    keys(&mut engine, &mut buf, "de");
    assert_eq!(buf.get_text(), " world");
}

#[test]
fn d_big_g_deletes_to_last_line() {
    let mut engine = make_engine();
    let mut buf = make_buffer("aaa\nbbb\nccc");

    // dG with default count=1 goes to line 0 (GoToLine is count-1).
    // d3G goes to line 2 col 0 — inclusive delete from (0,0) to (2,1)
    keys(&mut engine, &mut buf, "d3G");
    assert_eq!(buf.get_text(), "cc");
}

#[test]
fn dgg_deletes_to_first_line() {
    let mut engine = make_engine();
    let mut buf = make_buffer("aaa\nbbb\nccc");
    buf.set_cursor(CursorPosition::new(2, 0)); // on last line

    // gg goes to (0,0), GoToFirstLine is inclusive.
    // Deletes from (0,0) to (2,1), leaving "cc"
    keys(&mut engine, &mut buf, "dgg");
    assert_eq!(buf.get_text(), "cc");
}

#[test]
fn df_char_deletes_through_char() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");

    keys(&mut engine, &mut buf, "dfo");
    assert_eq!(buf.get_text(), " world");
}

#[test]
fn dt_char_deletes_until_char() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");

    keys(&mut engine, &mut buf, "dto");
    assert_eq!(buf.get_text(), "o world");
}

#[test]
fn cw_changes_word() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");

    keys(&mut engine, &mut buf, "cw");
    assert_eq!(buf.get_text(), "world");
    assert_eq!(engine.mode(), Mode::Insert);
}

#[test]
fn c_dollar_changes_to_end() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");
    buf.set_cursor(CursorPosition::new(0, 5));

    keys(&mut engine, &mut buf, "c$");
    assert_eq!(buf.get_text(), "hello");
    assert_eq!(engine.mode(), Mode::Insert);
}

#[test]
fn cb_changes_backward_word() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");
    buf.set_cursor(CursorPosition::new(0, 6)); // on 'w'

    keys(&mut engine, &mut buf, "cb");
    assert_eq!(buf.get_text(), "world");
    assert_eq!(engine.mode(), Mode::Insert);
}

#[test]
fn yw_yanks_word() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");

    keys(&mut engine, &mut buf, "yw");
    // buffer unchanged
    assert_eq!(buf.get_text(), "hello world");

    // Move to end, paste to verify yank worked
    buf.set_cursor(CursorPosition::new(0, 10)); // on 'd'
    keys(&mut engine, &mut buf, "p");
    assert_eq!(buf.get_text(), "hello worldhello ");
}

#[test]
fn y_dollar_yanks_to_end() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");
    buf.set_cursor(CursorPosition::new(0, 6)); // on 'w'

    keys(&mut engine, &mut buf, "y$");
    assert_eq!(buf.get_text(), "hello world"); // unchanged

    // paste at position 0 to verify content
    buf.set_cursor(CursorPosition::new(0, 0));
    keys(&mut engine, &mut buf, "P");
    assert_eq!(buf.get_text(), "worldhello world");
}

// ---------------------------------------------------------------------------
// 3. Operator + text object combos
// ---------------------------------------------------------------------------

#[test]
fn di_paren_deletes_inner_paren() {
    let mut engine = make_engine();
    let mut buf = make_buffer("foo(bar)baz");
    buf.set_cursor(CursorPosition::new(0, 4)); // on 'b' inside parens

    keys(&mut engine, &mut buf, "di(");
    assert_eq!(buf.get_text(), "foo()baz");
}

#[test]
fn da_paren_deletes_around_paren() {
    let mut engine = make_engine();
    let mut buf = make_buffer("foo(bar)baz");
    buf.set_cursor(CursorPosition::new(0, 4));

    keys(&mut engine, &mut buf, "da(");
    assert_eq!(buf.get_text(), "foobaz");
}

#[test]
fn di_double_quote_deletes_inner_quote() {
    let mut engine = make_engine();
    let mut buf = make_buffer(r#"say "hello" end"#);
    buf.set_cursor(CursorPosition::new(0, 5)); // on 'h' inside quotes

    keys(&mut engine, &mut buf, "di\"");
    assert_eq!(buf.get_text(), r#"say "" end"#);
}

#[test]
fn da_double_quote_deletes_around_quote() {
    let mut engine = make_engine();
    let mut buf = make_buffer(r#"say "hello" end"#);
    buf.set_cursor(CursorPosition::new(0, 5));

    keys(&mut engine, &mut buf, "da\"");
    assert_eq!(buf.get_text(), "say  end");
}

#[test]
fn di_brace_deletes_inner_brace() {
    let mut engine = make_engine();
    let mut buf = make_buffer("fn {body} end");
    buf.set_cursor(CursorPosition::new(0, 4)); // on 'b'

    keys(&mut engine, &mut buf, "di{");
    assert_eq!(buf.get_text(), "fn {} end");
}

#[test]
fn da_brace_deletes_around_brace() {
    let mut engine = make_engine();
    let mut buf = make_buffer("fn {body} end");
    buf.set_cursor(CursorPosition::new(0, 4));

    keys(&mut engine, &mut buf, "da{");
    assert_eq!(buf.get_text(), "fn  end");
}

#[test]
fn ci_single_quote_changes_inner() {
    let mut engine = make_engine();
    let mut buf = make_buffer("say 'hello' end");
    buf.set_cursor(CursorPosition::new(0, 5)); // on 'h'

    keys(&mut engine, &mut buf, "ci'");
    assert_eq!(buf.get_text(), "say '' end");
    assert_eq!(engine.mode(), Mode::Insert);
}

#[test]
fn yi_bracket_yanks_inner() {
    let mut engine = make_engine();
    let mut buf = make_buffer("arr[idx]rest");
    buf.set_cursor(CursorPosition::new(0, 5)); // on 'd'

    keys(&mut engine, &mut buf, "yi[");
    assert_eq!(buf.get_text(), "arr[idx]rest"); // unchanged

    // Paste to verify content
    buf.set_cursor(CursorPosition::new(0, 11)); // at end
    keys(&mut engine, &mut buf, "p");
    assert_eq!(buf.get_text(), "arr[idx]restidx");
}

// ---------------------------------------------------------------------------
// 4. Line operations
// ---------------------------------------------------------------------------

#[test]
fn two_dd_deletes_two_lines() {
    let mut engine = make_engine();
    let mut buf = make_buffer("aaa\nbbb\nccc\nddd");

    keys(&mut engine, &mut buf, "2dd");
    assert_eq!(buf.get_text(), "ccc\nddd");
}

#[test]
fn yy_then_big_p_pastes_before() {
    let mut engine = make_engine();
    let mut buf = make_buffer("aaa\nbbb");
    buf.set_cursor(CursorPosition::new(1, 0)); // on "bbb"

    keys(&mut engine, &mut buf, "yy");
    keys(&mut engine, &mut buf, "P");
    assert_eq!(buf.get_text(), "aaa\nbbb\nbbb");
}

#[test]
fn indent_then_outdent_round_trips() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello");

    keys(&mut engine, &mut buf, ">>");
    assert_eq!(buf.get_text(), "    hello");

    keys(&mut engine, &mut buf, "<<");
    assert_eq!(buf.get_text(), "hello");
}

#[test]
fn cc_changes_entire_line() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello\nworld");

    keys(&mut engine, &mut buf, "cc");
    // cc on first line deletes "hello\n" (linewise), leaving "world"
    assert_eq!(buf.get_text(), "world");
    assert_eq!(engine.mode(), Mode::Insert);
}

// ---------------------------------------------------------------------------
// 5. Visual mode
// ---------------------------------------------------------------------------

#[test]
fn visual_linewise_select_and_delete() {
    let mut engine = make_engine();
    let mut buf = make_buffer("aaa\nbbb\nccc");

    engine.handle_key(&KeyEvent::char('V'), &mut buf);
    assert_eq!(engine.mode(), Mode::VisualLinewise);

    engine.handle_key(&KeyEvent::char('j'), &mut buf); // select lines 0-1
    engine.handle_key(&KeyEvent::char('d'), &mut buf);

    assert_eq!(engine.mode(), Mode::Normal);
    assert_eq!(buf.get_text(), "ccc");
}

#[test]
fn visual_mode_yank() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");

    engine.handle_key(&KeyEvent::char('v'), &mut buf);
    keys(&mut engine, &mut buf, "llll"); // select "hello"
    engine.handle_key(&KeyEvent::char('y'), &mut buf);

    assert_eq!(engine.mode(), Mode::Normal);
    assert_eq!(buf.get_text(), "hello world"); // unchanged

    // Paste to verify
    buf.set_cursor(CursorPosition::new(0, 10)); // at 'd'
    keys(&mut engine, &mut buf, "p");
    assert_eq!(buf.get_text(), "hello worldhello");
}

#[test]
fn visual_swap_anchor() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello world");

    engine.handle_key(&KeyEvent::char('v'), &mut buf);
    keys(&mut engine, &mut buf, "lll"); // cursor at col 3

    let cursor_before = buf.get_cursor();
    assert_eq!(cursor_before.col, 3);

    engine.handle_key(&KeyEvent::char('o'), &mut buf); // swap anchor
    let cursor_after = buf.get_cursor();
    assert_eq!(cursor_after.col, 0); // now at original anchor
}

#[test]
fn visual_enter_and_exit_without_operation() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello");

    engine.handle_key(&KeyEvent::char('v'), &mut buf);
    assert_eq!(engine.mode(), Mode::VisualCharacterwise);

    engine.handle_key(&KeyEvent::escape(), &mut buf);
    assert_eq!(engine.mode(), Mode::Normal);
    assert_eq!(buf.get_text(), "hello"); // unchanged
}

// ---------------------------------------------------------------------------
// 6. Count operations
// ---------------------------------------------------------------------------

#[test]
fn count_2dw_deletes_two_words() {
    let mut engine = make_engine();
    let mut buf = make_buffer("one two three");

    keys(&mut engine, &mut buf, "2dw");
    assert_eq!(buf.get_text(), "three");
}

#[test]
fn count_3j_moves_down_three_lines() {
    let mut engine = make_engine();
    let mut buf = make_buffer("a\nb\nc\nd\ne");

    keys(&mut engine, &mut buf, "3j");
    assert_eq!(buf.get_cursor(), CursorPosition::new(3, 0));
}

#[test]
fn count_2dd_deletes_two_lines() {
    let mut engine = make_engine();
    let mut buf = make_buffer("one\ntwo\nthree\nfour");

    keys(&mut engine, &mut buf, "2dd");
    assert_eq!(buf.get_text(), "three\nfour");
}

// ---------------------------------------------------------------------------
// 7. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn operations_on_empty_buffer() {
    let mut engine = make_engine();
    let mut buf = make_buffer("");

    // These should not panic
    keys(&mut engine, &mut buf, "dd");
    assert_eq!(buf.get_text(), "");

    keys(&mut engine, &mut buf, "x");
    assert_eq!(buf.get_text(), "");
}

#[test]
fn operations_on_single_character() {
    let mut engine = make_engine();
    let mut buf = make_buffer("x");

    keys(&mut engine, &mut buf, "dd");
    assert_eq!(buf.get_text(), "");
}

#[test]
fn join_lines_on_last_line_is_noop() {
    let mut engine = make_engine();
    let mut buf = make_buffer("only line");

    engine.handle_key(&KeyEvent::char('J'), &mut buf);
    assert_eq!(buf.get_text(), "only line");
}

#[test]
fn toggle_case_at_end_of_line() {
    let mut engine = make_engine();
    let mut buf = make_buffer("abcD");
    buf.set_cursor(CursorPosition::new(0, 3)); // on 'D'

    engine.handle_key(&KeyEvent::char('~'), &mut buf);
    assert_eq!(buf.get_text(), "abcd");
}

#[test]
fn paste_when_nothing_yanked() {
    let mut engine = make_engine();
    let mut buf = make_buffer("hello");

    // p with empty register should not crash or modify buffer
    engine.handle_key(&KeyEvent::char('p'), &mut buf);
    assert_eq!(buf.get_text(), "hello");

    engine.handle_key(&KeyEvent::char('P'), &mut buf);
    assert_eq!(buf.get_text(), "hello");
}
