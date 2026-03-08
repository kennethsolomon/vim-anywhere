#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    pub line: usize,
    pub col: usize,
}

impl CursorPosition {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn zero() -> Self {
        Self { line: 0, col: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionKind {
    Characterwise,
    Linewise,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub anchor: CursorPosition,
    pub head: CursorPosition,
    pub kind: SelectionKind,
}

impl Selection {
    pub fn new(anchor: CursorPosition, head: CursorPosition, kind: SelectionKind) -> Self {
        Self { anchor, head, kind }
    }

    pub fn start(&self) -> CursorPosition {
        if self.anchor.line < self.head.line
            || (self.anchor.line == self.head.line && self.anchor.col <= self.head.col)
        {
            self.anchor
        } else {
            self.head
        }
    }

    pub fn end(&self) -> CursorPosition {
        if self.anchor.line < self.head.line
            || (self.anchor.line == self.head.line && self.anchor.col <= self.head.col)
        {
            self.head
        } else {
            self.anchor
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YankStyle {
    Characterwise,
    Linewise,
}

pub trait TextBuffer {
    fn get_text(&self) -> String;
    fn set_text(&mut self, text: &str);
    fn get_cursor(&self) -> CursorPosition;
    fn set_cursor(&mut self, pos: CursorPosition);
    fn get_selection(&self) -> Option<Selection>;
    fn set_selection(&mut self, sel: Option<Selection>);
    fn line_count(&self) -> usize;
    fn line_at(&self, line: usize) -> Option<&str>;
    fn line_len(&self, line: usize) -> usize;
    fn replace_range(&mut self, start: CursorPosition, end: CursorPosition, replacement: &str);
    fn insert_at(&mut self, pos: CursorPosition, text: &str);
    fn char_at(&self, pos: CursorPosition) -> Option<char>;
    fn text_range(&self, start: CursorPosition, end: CursorPosition) -> String;
    fn total_chars(&self) -> usize;
}

pub struct InMemoryBuffer {
    lines: Vec<String>,
    cursor: CursorPosition,
    selection: Option<Selection>,
    preferred_col: Option<usize>,
}

impl InMemoryBuffer {
    pub fn new(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n').map(String::from).collect()
        };
        Self {
            lines,
            cursor: CursorPosition::zero(),
            selection: None,
            preferred_col: None,
        }
    }

    pub fn preferred_col(&self) -> Option<usize> {
        self.preferred_col
    }

    pub fn set_preferred_col(&mut self, col: Option<usize>) {
        self.preferred_col = col;
    }

    fn clamp_cursor(&mut self) {
        if self.cursor.line >= self.lines.len() {
            self.cursor.line = self.lines.len().saturating_sub(1);
        }
        let max_col = self.line_len(self.cursor.line).saturating_sub(1);
        if self.cursor.col > max_col && !self.lines[self.cursor.line].is_empty() {
            self.cursor.col = max_col;
        }
        if self.lines[self.cursor.line].is_empty() {
            self.cursor.col = 0;
        }
    }

    fn rebuild_from_text(text: &str) -> Vec<String> {
        if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n').map(String::from).collect()
        }
    }

    fn offset_of(&self, pos: CursorPosition) -> usize {
        let mut offset = 0;
        for i in 0..pos.line.min(self.lines.len()) {
            offset += self.lines[i].len() + 1; // +1 for \n
        }
        if pos.line < self.lines.len() {
            offset += pos.col.min(self.lines[pos.line].len());
        }
        offset
    }

    fn pos_from_offset(&self, offset: usize) -> CursorPosition {
        let mut remaining = offset;
        for (i, line) in self.lines.iter().enumerate() {
            if remaining <= line.len() {
                return CursorPosition::new(i, remaining);
            }
            remaining -= line.len() + 1; // +1 for \n
        }
        let last = self.lines.len().saturating_sub(1);
        CursorPosition::new(last, self.lines[last].len())
    }
}

impl TextBuffer for InMemoryBuffer {
    fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    fn set_text(&mut self, text: &str) {
        self.lines = Self::rebuild_from_text(text);
        self.clamp_cursor();
    }

    fn get_cursor(&self) -> CursorPosition {
        self.cursor
    }

    fn set_cursor(&mut self, pos: CursorPosition) {
        self.cursor = pos;
        self.clamp_cursor();
    }

    fn get_selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    fn set_selection(&mut self, sel: Option<Selection>) {
        self.selection = sel;
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn line_at(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    fn line_len(&self, line: usize) -> usize {
        self.lines.get(line).map_or(0, |s| s.len())
    }

    fn replace_range(&mut self, start: CursorPosition, end: CursorPosition, replacement: &str) {
        let mut text = self.get_text();
        let start_off = self.offset_of(start);
        let end_off = self.offset_of(end);
        let start_off = start_off.min(text.len());
        let end_off = end_off.min(text.len());
        text.replace_range(start_off..end_off, replacement);
        self.lines = Self::rebuild_from_text(&text);
        self.cursor = self.pos_from_offset(start_off + replacement.len());
        self.clamp_cursor();
    }

    fn insert_at(&mut self, pos: CursorPosition, text: &str) {
        let full = self.get_text();
        let off = self.offset_of(pos);
        let off = off.min(full.len());
        let mut new_text = String::with_capacity(full.len() + text.len());
        new_text.push_str(&full[..off]);
        new_text.push_str(text);
        new_text.push_str(&full[off..]);
        self.lines = Self::rebuild_from_text(&new_text);
        self.cursor = self.pos_from_offset(off + text.len());
        self.clamp_cursor();
    }

    fn char_at(&self, pos: CursorPosition) -> Option<char> {
        self.lines
            .get(pos.line)
            .and_then(|line| line.chars().nth(pos.col))
    }

    fn text_range(&self, start: CursorPosition, end: CursorPosition) -> String {
        let text = self.get_text();
        let start_off = self.offset_of(start).min(text.len());
        let end_off = self.offset_of(end).min(text.len());
        if start_off <= end_off {
            text[start_off..end_off].to_string()
        } else {
            text[end_off..start_off].to_string()
        }
    }

    fn total_chars(&self) -> usize {
        let text = self.get_text();
        text.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_empty() {
        let buf = InMemoryBuffer::new("");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.get_cursor(), CursorPosition::zero());
    }

    #[test]
    fn new_buffer_with_text() {
        let buf = InMemoryBuffer::new("hello\nworld");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_at(0), Some("hello"));
        assert_eq!(buf.line_at(1), Some("world"));
    }

    #[test]
    fn set_cursor_clamps() {
        let mut buf = InMemoryBuffer::new("abc");
        buf.set_cursor(CursorPosition::new(0, 100));
        assert_eq!(buf.get_cursor(), CursorPosition::new(0, 2));
    }

    #[test]
    fn set_cursor_clamps_line() {
        let mut buf = InMemoryBuffer::new("abc\ndef");
        buf.set_cursor(CursorPosition::new(10, 0));
        assert_eq!(buf.get_cursor(), CursorPosition::new(1, 0));
    }

    #[test]
    fn char_at() {
        let buf = InMemoryBuffer::new("abc\ndef");
        assert_eq!(buf.char_at(CursorPosition::new(0, 0)), Some('a'));
        assert_eq!(buf.char_at(CursorPosition::new(0, 2)), Some('c'));
        assert_eq!(buf.char_at(CursorPosition::new(1, 0)), Some('d'));
        assert_eq!(buf.char_at(CursorPosition::new(1, 5)), None);
    }

    #[test]
    fn text_range() {
        let buf = InMemoryBuffer::new("hello world");
        let range = buf.text_range(CursorPosition::new(0, 0), CursorPosition::new(0, 5));
        assert_eq!(range, "hello");
    }

    #[test]
    fn text_range_multiline() {
        let buf = InMemoryBuffer::new("hello\nworld");
        let range = buf.text_range(CursorPosition::new(0, 3), CursorPosition::new(1, 2));
        assert_eq!(range, "lo\nwo");
    }

    #[test]
    fn replace_range() {
        let mut buf = InMemoryBuffer::new("hello world");
        buf.replace_range(
            CursorPosition::new(0, 5),
            CursorPosition::new(0, 11),
            " rust",
        );
        assert_eq!(buf.get_text(), "hello rust");
    }

    #[test]
    fn insert_at() {
        let mut buf = InMemoryBuffer::new("helo");
        buf.insert_at(CursorPosition::new(0, 2), "l");
        assert_eq!(buf.get_text(), "hello");
    }

    #[test]
    fn replace_range_multiline() {
        let mut buf = InMemoryBuffer::new("aaa\nbbb\nccc");
        buf.replace_range(CursorPosition::new(0, 1), CursorPosition::new(2, 1), "X");
        assert_eq!(buf.get_text(), "aXcc");
    }

    #[test]
    fn selection_start_end() {
        let sel = Selection::new(
            CursorPosition::new(1, 5),
            CursorPosition::new(0, 3),
            SelectionKind::Characterwise,
        );
        assert_eq!(sel.start(), CursorPosition::new(0, 3));
        assert_eq!(sel.end(), CursorPosition::new(1, 5));
    }

    #[test]
    fn empty_line_cursor() {
        let mut buf = InMemoryBuffer::new("abc\n\ndef");
        buf.set_cursor(CursorPosition::new(1, 0));
        assert_eq!(buf.get_cursor(), CursorPosition::new(1, 0));
        buf.set_cursor(CursorPosition::new(1, 5));
        assert_eq!(buf.get_cursor(), CursorPosition::new(1, 0));
    }

    #[test]
    fn total_chars() {
        let buf = InMemoryBuffer::new("abc\ndef");
        assert_eq!(buf.total_chars(), 7); // "abc\ndef" = 7 bytes
    }

    #[test]
    fn set_text_resets() {
        let mut buf = InMemoryBuffer::new("abc\ndef\nghi");
        buf.set_cursor(CursorPosition::new(2, 2));
        buf.set_text("xy");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.get_cursor().line, 0);
    }
}
