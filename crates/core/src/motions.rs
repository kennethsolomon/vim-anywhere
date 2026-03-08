use crate::buffer::{CursorPosition, InMemoryBuffer, TextBuffer};

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn first_non_blank_col(line: &str) -> usize {
    line.chars()
        .position(|c| !c.is_whitespace())
        .unwrap_or(0)
}

fn last_non_blank_col(line: &str) -> usize {
    if line.is_empty() {
        return 0;
    }
    let chars: Vec<char> = line.chars().collect();
    for i in (0..chars.len()).rev() {
        if !chars[i].is_whitespace() {
            return i;
        }
    }
    0
}

// --- Navigation motions ---

pub fn motion_left(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    CursorPosition::new(pos.line, pos.col.saturating_sub(count))
}

pub fn motion_right(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let max = buf.line_len(pos.line).saturating_sub(1);
    CursorPosition::new(pos.line, (pos.col + count).min(max))
}

pub fn motion_down(buf: &mut InMemoryBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let pref = buf.preferred_col().unwrap_or(pos.col);
    let new_line = (pos.line + count).min(buf.line_count().saturating_sub(1));
    let max_col = buf.line_len(new_line).saturating_sub(1);
    let new_col = if buf.line_len(new_line) == 0 {
        0
    } else {
        pref.min(max_col)
    };
    buf.set_preferred_col(Some(pref));
    CursorPosition::new(new_line, new_col)
}

pub fn motion_up(buf: &mut InMemoryBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let pref = buf.preferred_col().unwrap_or(pos.col);
    let new_line = pos.line.saturating_sub(count);
    let max_col = buf.line_len(new_line).saturating_sub(1);
    let new_col = if buf.line_len(new_line) == 0 {
        0
    } else {
        pref.min(max_col)
    };
    buf.set_preferred_col(Some(pref));
    CursorPosition::new(new_line, new_col)
}

pub fn motion_line_start(_buf: &dyn TextBuffer, _count: usize) -> CursorPosition {
    let pos = _buf.get_cursor();
    CursorPosition::new(pos.line, 0)
}

pub fn motion_line_end(buf: &dyn TextBuffer, _count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let len = buf.line_len(pos.line);
    CursorPosition::new(pos.line, if len == 0 { 0 } else { len - 1 })
}

pub fn motion_first_non_blank(buf: &dyn TextBuffer, _count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let col = buf
        .line_at(pos.line)
        .map(first_non_blank_col)
        .unwrap_or(0);
    CursorPosition::new(pos.line, col)
}

pub fn motion_last_non_blank(buf: &dyn TextBuffer, _count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let col = buf
        .line_at(pos.line)
        .map(last_non_blank_col)
        .unwrap_or(0);
    CursorPosition::new(pos.line, col)
}

pub fn motion_word_forward(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let mut pos = buf.get_cursor();
    for _ in 0..count {
        let line = match buf.line_at(pos.line) {
            Some(l) => l,
            None => break,
        };
        let chars: Vec<char> = line.chars().collect();
        if pos.col >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                pos.col = 0;
                let next_line = buf.line_at(pos.line).unwrap_or("");
                pos.col = first_non_blank_col(next_line);
            }
            continue;
        }

        let start_ch = chars[pos.col];
        let mut i = pos.col;

        // Skip current word class
        if is_word_char(start_ch) {
            while i < chars.len() && is_word_char(chars[i]) {
                i += 1;
            }
        } else if !start_ch.is_whitespace() {
            while i < chars.len() && !is_word_char(chars[i]) && !chars[i].is_whitespace() {
                i += 1;
            }
        }

        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                pos.col = 0;
                if let Some(next_line) = buf.line_at(pos.line) {
                    pos.col = first_non_blank_col(next_line);
                }
            } else {
                pos.col = chars.len().saturating_sub(1);
            }
        } else {
            pos.col = i;
        }
    }
    pos
}

pub fn motion_word_forward_big(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let mut pos = buf.get_cursor();
    for _ in 0..count {
        let line = match buf.line_at(pos.line) {
            Some(l) => l,
            None => break,
        };
        let chars: Vec<char> = line.chars().collect();
        if pos.col >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                pos.col = 0;
            }
            continue;
        }

        let mut i = pos.col;
        // Skip non-whitespace
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                pos.col = 0;
            } else {
                pos.col = chars.len().saturating_sub(1);
            }
        } else {
            pos.col = i;
        }
    }
    pos
}

pub fn motion_word_backward(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let mut pos = buf.get_cursor();
    for _ in 0..count {
        if pos.col == 0 {
            if pos.line > 0 {
                pos.line -= 1;
                pos.col = buf.line_len(pos.line).saturating_sub(1);
            }
            continue;
        }

        let line = match buf.line_at(pos.line) {
            Some(l) => l,
            None => break,
        };
        let chars: Vec<char> = line.chars().collect();
        let mut i = pos.col.saturating_sub(1);

        // Skip whitespace backward
        while i > 0 && chars[i].is_whitespace() {
            i -= 1;
        }

        if is_word_char(chars[i]) {
            while i > 0 && is_word_char(chars[i - 1]) {
                i -= 1;
            }
        } else if !chars[i].is_whitespace() {
            while i > 0 && !is_word_char(chars[i - 1]) && !chars[i - 1].is_whitespace() {
                i -= 1;
            }
        }

        pos.col = i;
    }
    pos
}

pub fn motion_word_backward_big(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let mut pos = buf.get_cursor();
    for _ in 0..count {
        if pos.col == 0 {
            if pos.line > 0 {
                pos.line -= 1;
                pos.col = buf.line_len(pos.line).saturating_sub(1);
            }
            continue;
        }

        let line = match buf.line_at(pos.line) {
            Some(l) => l,
            None => break,
        };
        let chars: Vec<char> = line.chars().collect();
        let mut i = pos.col.saturating_sub(1);

        while i > 0 && chars[i].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }

        pos.col = i;
    }
    pos
}

pub fn motion_word_end(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let mut pos = buf.get_cursor();
    for _ in 0..count {
        let line = match buf.line_at(pos.line) {
            Some(l) => l,
            None => break,
        };
        let chars: Vec<char> = line.chars().collect();
        let mut i = pos.col + 1;

        if i >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                pos.col = 0;
                i = 0;
                let next_line = buf.line_at(pos.line).unwrap_or("");
                let next_chars: Vec<char> = next_line.chars().collect();
                // skip whitespace
                while i < next_chars.len() && next_chars[i].is_whitespace() {
                    i += 1;
                }
                if i < next_chars.len() {
                    if is_word_char(next_chars[i]) {
                        while i + 1 < next_chars.len() && is_word_char(next_chars[i + 1]) {
                            i += 1;
                        }
                    } else {
                        while i + 1 < next_chars.len()
                            && !is_word_char(next_chars[i + 1])
                            && !next_chars[i + 1].is_whitespace()
                        {
                            i += 1;
                        }
                    }
                }
                pos.col = i;
                continue;
            } else {
                pos.col = chars.len().saturating_sub(1);
                continue;
            }
        }

        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                pos.col = 0;
            } else {
                pos.col = chars.len().saturating_sub(1);
            }
            continue;
        }

        if is_word_char(chars[i]) {
            while i + 1 < chars.len() && is_word_char(chars[i + 1]) {
                i += 1;
            }
        } else {
            while i + 1 < chars.len()
                && !is_word_char(chars[i + 1])
                && !chars[i + 1].is_whitespace()
            {
                i += 1;
            }
        }

        pos.col = i;
    }
    pos
}

pub fn motion_word_end_big(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let mut pos = buf.get_cursor();
    for _ in 0..count {
        let line = match buf.line_at(pos.line) {
            Some(l) => l,
            None => break,
        };
        let chars: Vec<char> = line.chars().collect();
        let mut i = pos.col + 1;

        if i >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                let next_line = buf.line_at(pos.line).unwrap_or("");
                let next_chars: Vec<char> = next_line.chars().collect();
                i = 0;
                while i < next_chars.len() && next_chars[i].is_whitespace() {
                    i += 1;
                }
                while i + 1 < next_chars.len() && !next_chars[i + 1].is_whitespace() {
                    i += 1;
                }
                pos.col = i;
            } else {
                pos.col = chars.len().saturating_sub(1);
            }
            continue;
        }

        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            if pos.line + 1 < buf.line_count() {
                pos.line += 1;
                pos.col = 0;
            }
            continue;
        }
        while i + 1 < chars.len() && !chars[i + 1].is_whitespace() {
            i += 1;
        }
        pos.col = i;
    }
    pos
}

pub fn motion_word_end_backward(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let mut pos = buf.get_cursor();
    for _ in 0..count {
        if pos.col == 0 {
            if pos.line > 0 {
                pos.line -= 1;
                pos.col = buf.line_len(pos.line).saturating_sub(1);
            }
            continue;
        }

        let line = match buf.line_at(pos.line) {
            Some(l) => l,
            None => break,
        };
        let chars: Vec<char> = line.chars().collect();
        let mut i = pos.col.saturating_sub(1);

        while i > 0 && chars[i].is_whitespace() {
            i -= 1;
        }

        if i == 0 {
            pos.col = 0;
            continue;
        }

        if is_word_char(chars[i]) {
            while i > 0 && is_word_char(chars[i]) {
                i -= 1;
            }
            if !is_word_char(chars[i]) {
                i += 1;
            }
        } else if !chars[i].is_whitespace() {
            while i > 0 && !is_word_char(chars[i]) && !chars[i].is_whitespace() {
                i -= 1;
            }
            if is_word_char(chars[i]) || chars[i].is_whitespace() {
                i += 1;
            }
        }

        pos.col = i;
    }
    pos
}

pub fn motion_find_char(buf: &dyn TextBuffer, ch: char, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let line = match buf.line_at(pos.line) {
        Some(l) => l,
        None => return pos,
    };
    let chars: Vec<char> = line.chars().collect();
    let mut found = 0;
    for i in (pos.col + 1)..chars.len() {
        if chars[i] == ch {
            found += 1;
            if found == count {
                return CursorPosition::new(pos.line, i);
            }
        }
    }
    pos
}

pub fn motion_find_char_back(buf: &dyn TextBuffer, ch: char, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let line = match buf.line_at(pos.line) {
        Some(l) => l,
        None => return pos,
    };
    let chars: Vec<char> = line.chars().collect();
    let mut found = 0;
    for i in (0..pos.col).rev() {
        if chars[i] == ch {
            found += 1;
            if found == count {
                return CursorPosition::new(pos.line, i);
            }
        }
    }
    pos
}

pub fn motion_til_char(buf: &dyn TextBuffer, ch: char, count: usize) -> CursorPosition {
    let target = motion_find_char(buf, ch, count);
    let pos = buf.get_cursor();
    if target.col > pos.col {
        CursorPosition::new(target.line, target.col - 1)
    } else {
        pos
    }
}

pub fn motion_til_char_back(buf: &dyn TextBuffer, ch: char, count: usize) -> CursorPosition {
    let target = motion_find_char_back(buf, ch, count);
    let pos = buf.get_cursor();
    if target.col < pos.col {
        CursorPosition::new(target.line, target.col + 1)
    } else {
        pos
    }
}

pub fn motion_goto_line(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let target_line = count.saturating_sub(1).min(buf.line_count().saturating_sub(1));
    let col = buf
        .line_at(target_line)
        .map(first_non_blank_col)
        .unwrap_or(0);
    CursorPosition::new(target_line, col)
}

pub fn motion_goto_first_line(buf: &dyn TextBuffer, _count: usize) -> CursorPosition {
    let col = buf.line_at(0).map(first_non_blank_col).unwrap_or(0);
    CursorPosition::new(0, col)
}

pub fn motion_goto_last_line(buf: &dyn TextBuffer, _count: usize) -> CursorPosition {
    let last = buf.line_count().saturating_sub(1);
    let col = buf.line_at(last).map(first_non_blank_col).unwrap_or(0);
    CursorPosition::new(last, col)
}

pub fn motion_paragraph_forward(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let mut line = pos.line;
    let mut found = 0;

    // Skip current non-empty lines
    while line < buf.line_count() && buf.line_at(line).map_or(true, |l| !l.trim().is_empty()) {
        line += 1;
    }

    for i in line..buf.line_count() {
        let is_empty = buf.line_at(i).map_or(true, |l| l.trim().is_empty());
        let prev_empty = if i > 0 {
            buf.line_at(i - 1).map_or(true, |l| l.trim().is_empty())
        } else {
            true
        };
        if !is_empty && prev_empty {
            found += 1;
            if found == count {
                return CursorPosition::new(i.saturating_sub(1), 0);
            }
        }
    }

    CursorPosition::new(buf.line_count().saturating_sub(1), 0)
}

pub fn motion_paragraph_backward(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let mut line = pos.line;
    let mut found = 0;

    // Skip current empty lines going backward
    while line > 0 && buf.line_at(line).map_or(true, |l| l.trim().is_empty()) {
        line -= 1;
    }
    // Skip current paragraph
    while line > 0 && buf.line_at(line).map_or(false, |l| !l.trim().is_empty()) {
        line -= 1;
    }

    if line == 0 {
        found += 1;
    }

    if found >= count {
        return CursorPosition::new(0, 0);
    }

    while line > 0 {
        let is_empty = buf.line_at(line).map_or(true, |l| l.trim().is_empty());
        let prev_non_empty = if line > 0 {
            buf.line_at(line - 1)
                .map_or(false, |l| !l.trim().is_empty())
        } else {
            false
        };
        if is_empty && prev_non_empty {
            found += 1;
            if found >= count {
                return CursorPosition::new(line, 0);
            }
        }
        line -= 1;
    }

    CursorPosition::new(0, 0)
}

pub fn motion_match_bracket(buf: &dyn TextBuffer, _count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let ch = match buf.char_at(pos) {
        Some(c) => c,
        None => return pos,
    };

    let (target, forward) = match ch {
        '(' => (')', true),
        ')' => ('(', false),
        '[' => (']', true),
        ']' => ('[', false),
        '{' => ('}', true),
        '}' => ('{', false),
        '<' => ('>', true),
        '>' => ('<', false),
        _ => return pos,
    };

    let text = buf.get_text();
    let chars: Vec<char> = text.chars().collect();

    // Compute flat offset
    let mut offset = 0;
    for i in 0..pos.line {
        offset += buf.line_len(i) + 1;
    }
    offset += pos.col;

    let mut depth = 1i32;
    if forward {
        for i in (offset + 1)..chars.len() {
            if chars[i] == ch {
                depth += 1;
            } else if chars[i] == target {
                depth -= 1;
            }
            if depth == 0 {
                return offset_to_pos(buf, i);
            }
        }
    } else {
        for i in (0..offset).rev() {
            if chars[i] == ch {
                depth += 1;
            } else if chars[i] == target {
                depth -= 1;
            }
            if depth == 0 {
                return offset_to_pos(buf, i);
            }
        }
    }

    pos
}

fn offset_to_pos(buf: &dyn TextBuffer, offset: usize) -> CursorPosition {
    let mut remaining = offset;
    for line in 0..buf.line_count() {
        let len = buf.line_len(line);
        if remaining <= len {
            return CursorPosition::new(line, remaining);
        }
        remaining -= len + 1;
    }
    let last = buf.line_count().saturating_sub(1);
    CursorPosition::new(last, buf.line_len(last))
}

pub fn motion_return(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let new_line = (pos.line + count).min(buf.line_count().saturating_sub(1));
    let col = buf
        .line_at(new_line)
        .map(first_non_blank_col)
        .unwrap_or(0);
    CursorPosition::new(new_line, col)
}

pub fn motion_prev_first_non_blank(buf: &dyn TextBuffer, count: usize) -> CursorPosition {
    let pos = buf.get_cursor();
    let new_line = pos.line.saturating_sub(count);
    let col = buf
        .line_at(new_line)
        .map(first_non_blank_col)
        .unwrap_or(0);
    CursorPosition::new(new_line, col)
}

// --- Text Objects ---

pub fn text_object_inner_word(buf: &dyn TextBuffer) -> Option<(CursorPosition, CursorPosition)> {
    let pos = buf.get_cursor();
    let line = buf.line_at(pos.line)?;
    let chars: Vec<char> = line.chars().collect();
    if pos.col >= chars.len() {
        return None;
    }

    let ch = chars[pos.col];
    let mut start = pos.col;
    let mut end = pos.col;

    if is_word_char(ch) {
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }
        while end + 1 < chars.len() && is_word_char(chars[end + 1]) {
            end += 1;
        }
    } else if ch.is_whitespace() {
        while start > 0 && chars[start - 1].is_whitespace() {
            start -= 1;
        }
        while end + 1 < chars.len() && chars[end + 1].is_whitespace() {
            end += 1;
        }
    } else {
        while start > 0 && !is_word_char(chars[start - 1]) && !chars[start - 1].is_whitespace() {
            start -= 1;
        }
        while end + 1 < chars.len()
            && !is_word_char(chars[end + 1])
            && !chars[end + 1].is_whitespace()
        {
            end += 1;
        }
    }

    Some((
        CursorPosition::new(pos.line, start),
        CursorPosition::new(pos.line, end + 1),
    ))
}

pub fn text_object_a_word(buf: &dyn TextBuffer) -> Option<(CursorPosition, CursorPosition)> {
    let (start, mut end) = text_object_inner_word(buf)?;
    let line = buf.line_at(start.line)?;
    let chars: Vec<char> = line.chars().collect();

    // Include trailing whitespace
    while end.col < chars.len() && chars[end.col].is_whitespace() {
        end.col += 1;
    }

    Some((start, end))
}

pub fn text_object_inner_word_big(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    let pos = buf.get_cursor();
    let line = buf.line_at(pos.line)?;
    let chars: Vec<char> = line.chars().collect();
    if pos.col >= chars.len() {
        return None;
    }

    let ch = chars[pos.col];
    let mut start = pos.col;
    let mut end = pos.col;

    if ch.is_whitespace() {
        while start > 0 && chars[start - 1].is_whitespace() {
            start -= 1;
        }
        while end + 1 < chars.len() && chars[end + 1].is_whitespace() {
            end += 1;
        }
    } else {
        while start > 0 && !chars[start - 1].is_whitespace() {
            start -= 1;
        }
        while end + 1 < chars.len() && !chars[end + 1].is_whitespace() {
            end += 1;
        }
    }

    Some((
        CursorPosition::new(pos.line, start),
        CursorPosition::new(pos.line, end + 1),
    ))
}

pub fn text_object_a_word_big(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    let (start, mut end) = text_object_inner_word_big(buf)?;
    let line = buf.line_at(start.line)?;
    let chars: Vec<char> = line.chars().collect();

    while end.col < chars.len() && chars[end.col].is_whitespace() {
        end.col += 1;
    }

    Some((start, end))
}

fn find_matching_pair(
    buf: &dyn TextBuffer,
    open: char,
    close: char,
    inner: bool,
) -> Option<(CursorPosition, CursorPosition)> {
    let text = buf.get_text();
    let chars: Vec<char> = text.chars().collect();
    let pos = buf.get_cursor();

    // Compute flat offset
    let mut cursor_offset = 0;
    for i in 0..pos.line {
        cursor_offset += buf.line_len(i) + 1;
    }
    cursor_offset += pos.col;

    // Find opening bracket (search backward from cursor or at cursor)
    let mut open_offset = None;
    let mut depth = 0i32;

    // First check if cursor is on the open bracket
    if cursor_offset < chars.len() && chars[cursor_offset] == open {
        open_offset = Some(cursor_offset);
    } else {
        // Search backward
        for i in (0..=cursor_offset.min(chars.len() - 1)).rev() {
            if chars[i] == close {
                depth += 1;
            } else if chars[i] == open {
                if depth == 0 {
                    open_offset = Some(i);
                    break;
                }
                depth -= 1;
            }
        }
    }

    let open_off = open_offset?;

    // Find matching close bracket
    depth = 1;
    for i in (open_off + 1)..chars.len() {
        if chars[i] == open {
            depth += 1;
        } else if chars[i] == close {
            depth -= 1;
        }
        if depth == 0 {
            let start = if inner { open_off + 1 } else { open_off };
            let end = if inner { i } else { i + 1 };
            return Some((offset_to_pos(buf, start), offset_to_pos(buf, end)));
        }
    }

    None
}

pub fn text_object_inner_paren(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '(', ')', true)
}

pub fn text_object_a_paren(buf: &dyn TextBuffer) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '(', ')', false)
}

pub fn text_object_inner_brace(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '{', '}', true)
}

pub fn text_object_a_brace(buf: &dyn TextBuffer) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '{', '}', false)
}

pub fn text_object_inner_bracket(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '[', ']', true)
}

pub fn text_object_a_bracket(buf: &dyn TextBuffer) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '[', ']', false)
}

pub fn text_object_inner_angle(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '<', '>', true)
}

pub fn text_object_a_angle(buf: &dyn TextBuffer) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_pair(buf, '<', '>', false)
}

fn find_matching_quote(
    buf: &dyn TextBuffer,
    quote: char,
    inner: bool,
) -> Option<(CursorPosition, CursorPosition)> {
    let pos = buf.get_cursor();
    let line = buf.line_at(pos.line)?;
    let chars: Vec<char> = line.chars().collect();

    // Find first quote at or after cursor, or search backward
    let mut first = None;
    let mut second = None;

    // Find surrounding quotes on the same line
    // Look for quote before or at cursor
    for i in (0..=pos.col.min(chars.len().saturating_sub(1))).rev() {
        if chars[i] == quote {
            first = Some(i);
            break;
        }
    }

    if let Some(f) = first {
        // Look for closing quote after
        for i in (f + 1)..chars.len() {
            if chars[i] == quote {
                second = Some(i);
                break;
            }
        }

        if let Some(s) = second {
            if pos.col >= f && pos.col <= s {
                let start = if inner { f + 1 } else { f };
                let end = if inner { s } else { s + 1 };
                return Some((
                    CursorPosition::new(pos.line, start),
                    CursorPosition::new(pos.line, end),
                ));
            }
        }
    }

    // Try finding quote pair ahead of cursor
    first = None;
    for i in pos.col..chars.len() {
        if chars[i] == quote {
            if first.is_none() {
                first = Some(i);
            } else {
                second = Some(i);
                break;
            }
        }
    }

    if let (Some(f), Some(s)) = (first, second) {
        let start = if inner { f + 1 } else { f };
        let end = if inner { s } else { s + 1 };
        return Some((
            CursorPosition::new(pos.line, start),
            CursorPosition::new(pos.line, end),
        ));
    }

    None
}

pub fn text_object_inner_double_quote(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_quote(buf, '"', true)
}

pub fn text_object_a_double_quote(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_quote(buf, '"', false)
}

pub fn text_object_inner_single_quote(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_quote(buf, '\'', true)
}

pub fn text_object_a_single_quote(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_quote(buf, '\'', false)
}

pub fn text_object_inner_backtick(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_quote(buf, '`', true)
}

pub fn text_object_a_backtick(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    find_matching_quote(buf, '`', false)
}

pub fn text_object_inner_paragraph(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    let pos = buf.get_cursor();
    let mut start = pos.line;
    let mut end = pos.line;

    // If on an empty line, select contiguous empty lines
    if buf.line_at(pos.line).map_or(true, |l| l.trim().is_empty()) {
        while start > 0 && buf.line_at(start - 1).map_or(false, |l| l.trim().is_empty()) {
            start -= 1;
        }
        while end + 1 < buf.line_count()
            && buf.line_at(end + 1).map_or(false, |l| l.trim().is_empty())
        {
            end += 1;
        }
    } else {
        while start > 0 && buf.line_at(start - 1).map_or(false, |l| !l.trim().is_empty()) {
            start -= 1;
        }
        while end + 1 < buf.line_count()
            && buf.line_at(end + 1).map_or(false, |l| !l.trim().is_empty())
        {
            end += 1;
        }
    }

    Some((
        CursorPosition::new(start, 0),
        CursorPosition::new(end, buf.line_len(end)),
    ))
}

pub fn text_object_a_paragraph(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    let (start, mut end) = text_object_inner_paragraph(buf)?;

    // Include trailing blank lines
    while end.line + 1 < buf.line_count()
        && buf
            .line_at(end.line + 1)
            .map_or(false, |l| l.trim().is_empty())
    {
        end.line += 1;
        end.col = buf.line_len(end.line);
    }

    Some((start, end))
}

pub fn text_object_inner_sentence(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    let pos = buf.get_cursor();
    let line = buf.line_at(pos.line)?;
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // Find sentence boundaries (. ! ?)
    let mut start = pos.col;
    let mut end = pos.col;

    // Search backward for sentence start
    while start > 0 {
        if start >= 2
            && (chars[start - 2] == '.' || chars[start - 2] == '!' || chars[start - 2] == '?')
            && chars[start - 1].is_whitespace()
        {
            break;
        }
        start -= 1;
    }

    // Search forward for sentence end
    while end < chars.len() {
        if chars[end] == '.' || chars[end] == '!' || chars[end] == '?' {
            end += 1;
            break;
        }
        end += 1;
    }

    Some((
        CursorPosition::new(pos.line, start),
        CursorPosition::new(pos.line, end),
    ))
}

pub fn text_object_a_sentence(
    buf: &dyn TextBuffer,
) -> Option<(CursorPosition, CursorPosition)> {
    let (start, mut end) = text_object_inner_sentence(buf)?;
    let line = buf.line_at(start.line)?;
    let chars: Vec<char> = line.chars().collect();

    // Include trailing whitespace
    while end.col < chars.len() && chars[end.col].is_whitespace() {
        end.col += 1;
    }

    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(text: &str) -> InMemoryBuffer {
        InMemoryBuffer::new(text)
    }

    fn buf_at(text: &str, line: usize, col: usize) -> InMemoryBuffer {
        let mut b = InMemoryBuffer::new(text);
        b.set_cursor(CursorPosition::new(line, col));
        b
    }

    // --- Basic Navigation ---

    #[test]
    fn test_motion_left() {
        let b = buf_at("hello", 0, 3);
        assert_eq!(motion_left(&b, 1), CursorPosition::new(0, 2));
        assert_eq!(motion_left(&b, 5), CursorPosition::new(0, 0));
    }

    #[test]
    fn test_motion_right() {
        let b = buf_at("hello", 0, 1);
        assert_eq!(motion_right(&b, 1), CursorPosition::new(0, 2));
        assert_eq!(motion_right(&b, 100), CursorPosition::new(0, 4));
    }

    #[test]
    fn test_motion_down() {
        let mut b = buf_at("abc\ndef\nghi", 0, 1);
        assert_eq!(motion_down(&mut b, 1), CursorPosition::new(1, 1));
        assert_eq!(motion_down(&mut b, 10), CursorPosition::new(2, 1));
    }

    #[test]
    fn test_motion_up() {
        let mut b = buf_at("abc\ndef\nghi", 2, 1);
        assert_eq!(motion_up(&mut b, 1), CursorPosition::new(1, 1));
        assert_eq!(motion_up(&mut b, 10), CursorPosition::new(0, 1));
    }

    #[test]
    fn test_motion_down_preferred_col() {
        let mut b = buf_at("abcdef\nab\nabcdef", 0, 5);
        let pos = motion_down(&mut b, 1);
        assert_eq!(pos, CursorPosition::new(1, 1)); // clamped to short line
        b.set_cursor(pos);
        let pos2 = motion_down(&mut b, 1);
        assert_eq!(pos2, CursorPosition::new(2, 5)); // restores preferred col
    }

    #[test]
    fn test_line_start_end() {
        let b = buf_at("  hello  ", 0, 4);
        assert_eq!(motion_line_start(&b, 1), CursorPosition::new(0, 0));
        assert_eq!(motion_line_end(&b, 1), CursorPosition::new(0, 8));
    }

    #[test]
    fn test_first_non_blank() {
        let b = buf_at("  hello", 0, 0);
        assert_eq!(motion_first_non_blank(&b, 1), CursorPosition::new(0, 2));
    }

    // --- Word Motions ---

    #[test]
    fn test_word_forward() {
        let b = buf_at("hello world foo", 0, 0);
        assert_eq!(motion_word_forward(&b, 1), CursorPosition::new(0, 6));
        let b2 = buf_at("hello world foo", 0, 6);
        assert_eq!(motion_word_forward(&b2, 1), CursorPosition::new(0, 12));
    }

    #[test]
    fn test_word_backward() {
        let b = buf_at("hello world", 0, 6);
        assert_eq!(motion_word_backward(&b, 1), CursorPosition::new(0, 0));
    }

    #[test]
    fn test_word_end() {
        let b = buf_at("hello world", 0, 0);
        assert_eq!(motion_word_end(&b, 1), CursorPosition::new(0, 4));
    }

    // --- Find Char ---

    #[test]
    fn test_find_char() {
        let b = buf_at("hello world", 0, 0);
        assert_eq!(motion_find_char(&b, 'o', 1), CursorPosition::new(0, 4));
        assert_eq!(motion_find_char(&b, 'o', 2), CursorPosition::new(0, 7));
    }

    #[test]
    fn test_find_char_back() {
        let b = buf_at("hello world", 0, 10);
        assert_eq!(
            motion_find_char_back(&b, 'l', 1),
            CursorPosition::new(0, 9)
        );
    }

    #[test]
    fn test_til_char() {
        let b = buf_at("hello world", 0, 0);
        assert_eq!(motion_til_char(&b, 'o', 1), CursorPosition::new(0, 3));
    }

    // --- Goto ---

    #[test]
    fn test_goto_line() {
        let b = buf("abc\ndef\nghi");
        assert_eq!(motion_goto_line(&b, 2), CursorPosition::new(1, 0));
        assert_eq!(motion_goto_line(&b, 100), CursorPosition::new(2, 0));
    }

    #[test]
    fn test_goto_first_line() {
        let b = buf_at("abc\ndef", 1, 2);
        assert_eq!(motion_goto_first_line(&b, 1), CursorPosition::new(0, 0));
    }

    // --- Brackets ---

    #[test]
    fn test_match_bracket() {
        let b = buf_at("(hello)", 0, 0);
        assert_eq!(motion_match_bracket(&b, 1), CursorPosition::new(0, 6));
        let b2 = buf_at("(hello)", 0, 6);
        assert_eq!(motion_match_bracket(&b2, 1), CursorPosition::new(0, 0));
    }

    #[test]
    fn test_match_bracket_nested() {
        let b = buf_at("((a)b)", 0, 0);
        assert_eq!(motion_match_bracket(&b, 1), CursorPosition::new(0, 5));
    }

    // --- Paragraph ---

    #[test]
    fn test_paragraph_forward() {
        let b = buf_at("aaa\nbbb\n\nccc\nddd", 0, 0);
        let pos = motion_paragraph_forward(&b, 1);
        // Should land at the empty line or after it
        assert!(pos.line >= 2);
    }

    // --- Text Objects ---

    #[test]
    fn test_inner_word() {
        let b = buf_at("hello world", 0, 1);
        let (start, end) = text_object_inner_word(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        assert_eq!(end, CursorPosition::new(0, 5));
    }

    #[test]
    fn test_a_word() {
        let b = buf_at("hello world", 0, 1);
        let (start, end) = text_object_a_word(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        assert_eq!(end, CursorPosition::new(0, 6)); // includes trailing space
    }

    #[test]
    fn test_inner_paren() {
        let b = buf_at("foo(bar)baz", 0, 5);
        let (start, end) = text_object_inner_paren(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 4));
        assert_eq!(end, CursorPosition::new(0, 7));
    }

    #[test]
    fn test_a_paren() {
        let b = buf_at("foo(bar)baz", 0, 5);
        let (start, end) = text_object_a_paren(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 3));
        assert_eq!(end, CursorPosition::new(0, 8));
    }

    #[test]
    fn test_inner_double_quote() {
        let b = buf_at("say \"hello\" ok", 0, 6);
        let (start, end) = text_object_inner_double_quote(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 5));
        assert_eq!(end, CursorPosition::new(0, 10));
    }

    #[test]
    fn test_a_double_quote() {
        let b = buf_at("say \"hello\" ok", 0, 6);
        let (start, end) = text_object_a_double_quote(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 4));
        assert_eq!(end, CursorPosition::new(0, 11));
    }

    #[test]
    fn test_inner_paragraph() {
        let b = buf_at("aaa\nbbb\n\nccc", 0, 0);
        let (start, end) = text_object_inner_paragraph(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 0));
        assert_eq!(end, CursorPosition::new(1, 3));
    }

    #[test]
    fn test_inner_brace() {
        let b = buf_at("fn() { body }", 0, 8);
        let (start, end) = text_object_inner_brace(&b).unwrap();
        assert_eq!(start, CursorPosition::new(0, 6));
        assert_eq!(end, CursorPosition::new(0, 12));
    }

    #[test]
    fn test_motion_return() {
        let b = buf_at("  abc\n  def", 0, 0);
        assert_eq!(motion_return(&b, 1), CursorPosition::new(1, 2));
    }
}
