use vim_anywhere_core::buffer::{CursorPosition, InMemoryBuffer, TextBuffer, YankStyle};
use vim_anywhere_core::modes::{InsertVariant, Mode, ModeEntryConfig, ModeStateMachine};
use vim_anywhere_core::motions;
use vim_anywhere_core::parser::{KeyEvent, KeyParser, Motion, Operator, ParsedCommand, TextObject};
use vim_anywhere_core::register::RegisterManager;

pub struct Engine {
    mode_sm: ModeStateMachine,
    parser: KeyParser,
    registers: RegisterManager,
    last_change: Option<ParsedCommand>,
}

impl Engine {
    pub fn new(config: ModeEntryConfig) -> Self {
        Self {
            mode_sm: ModeStateMachine::new(config),
            parser: KeyParser::new(),
            registers: RegisterManager::new(),
            last_change: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode_sm.mode()
    }

    pub fn pending_keys(&self) -> &str {
        self.parser.pending_keys()
    }

    pub fn reset_to_insert(&mut self) {
        self.mode_sm.reset_to_insert();
        self.parser.reset();
    }

    pub fn handle_key(&mut self, event: &KeyEvent, buffer: &mut InMemoryBuffer) -> EngineResult {
        let mode = self.mode_sm.mode();

        // In Insert mode, check for mode exit sequences
        if mode == Mode::Insert {
            if event.key == vim_anywhere_core::parser::Key::Escape {
                let transition = self.mode_sm.handle_escape();
                self.parser.reset();
                if transition == vim_anywhere_core::modes::ModeTransition::SendEscape {
                    return EngineResult::SendRealEscape;
                }
                return EngineResult::ModeChanged(Mode::Normal);
            }
            if let vim_anywhere_core::parser::Key::Char(ch) = &event.key {
                let transition = self.mode_sm.handle_insert_char(*ch);
                if transition == vim_anywhere_core::modes::ModeTransition::To(Mode::Normal) {
                    self.parser.reset();
                    return EngineResult::ModeChanged(Mode::Normal);
                }
            }
            return EngineResult::PassThrough;
        }

        let cmd = self.parser.parse(event, mode);

        // Debug: log parsed command to file
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true).append(true)
            .open("/tmp/vim-anywhere.log")
        {
            let _ = std::io::Write::write_fmt(&mut f, format_args!("  ENGINE: key={:?} mods={:?} cmd={:?} mode={:?}\n", event.key, event.modifiers, cmd, mode));
        }

        match cmd {
            ParsedCommand::Incomplete => EngineResult::Suppressed,
            ParsedCommand::Invalid => EngineResult::Suppressed,
            ParsedCommand::Escape => {
                let transition = self.mode_sm.handle_escape();
                match transition {
                    vim_anywhere_core::modes::ModeTransition::SendEscape => EngineResult::SendRealEscape,
                    vim_anywhere_core::modes::ModeTransition::PassThrough => EngineResult::PassThrough,
                    _ => EngineResult::ModeChanged(self.mode_sm.mode()),
                }
            }
            ParsedCommand::RepeatLastChange => {
                if let Some(last) = self.last_change.clone() {
                    self.execute_command(last, buffer)
                } else {
                    EngineResult::Suppressed
                }
            }
            _ => {
                let is_change = matches!(
                    cmd,
                    ParsedCommand::OperatorMotion(_, _, _)
                        | ParsedCommand::OperatorTextObject(_, _, _)
                        | ParsedCommand::OperatorLine(_, _)
                        | ParsedCommand::ToggleCase
                        | ParsedCommand::JoinLines
                        | ParsedCommand::PasteAfter
                        | ParsedCommand::PasteBefore
                        | ParsedCommand::Replace(_)
                        | ParsedCommand::VisualOperation(_)
                );
                if is_change {
                    self.last_change = Some(cmd.clone());
                }
                self.execute_command(cmd, buffer)
            }
        }
    }

    fn execute_command(&mut self, cmd: ParsedCommand, buffer: &mut InMemoryBuffer) -> EngineResult {
        match cmd {
            // Mode entries
            ParsedCommand::EnterInsert(variant) => {
                self.handle_enter_insert(variant, buffer);
                EngineResult::ModeChanged(Mode::Insert)
            }
            ParsedCommand::EnterVisualCharacterwise => {
                self.mode_sm.enter_visual_characterwise();
                let pos = buffer.get_cursor();
                buffer.set_selection(Some(vim_anywhere_core::buffer::Selection::new(
                    pos,
                    pos,
                    vim_anywhere_core::buffer::SelectionKind::Characterwise,
                )));
                EngineResult::ModeChanged(self.mode_sm.mode())
            }
            ParsedCommand::EnterVisualLinewise => {
                self.mode_sm.enter_visual_linewise();
                let pos = buffer.get_cursor();
                buffer.set_selection(Some(vim_anywhere_core::buffer::Selection::new(
                    pos,
                    pos,
                    vim_anywhere_core::buffer::SelectionKind::Linewise,
                )));
                EngineResult::ModeChanged(self.mode_sm.mode())
            }

            // Motions
            ParsedCommand::Motion(motion, count) => {
                self.execute_motion(motion, count, buffer);
                EngineResult::BufferModified
            }

            // Operators
            ParsedCommand::OperatorMotion(op, motion, count) => {
                self.execute_operator_motion(op, motion, count, buffer);
                EngineResult::BufferModified
            }
            ParsedCommand::OperatorTextObject(op, obj, count) => {
                self.execute_operator_text_object(op, obj, count, buffer);
                EngineResult::BufferModified
            }
            ParsedCommand::OperatorLine(op, count) => {
                self.execute_operator_line(op, count, buffer);
                EngineResult::BufferModified
            }

            // Editing
            ParsedCommand::ToggleCase => {
                self.toggle_case(buffer);
                EngineResult::BufferModified
            }
            ParsedCommand::JoinLines => {
                self.join_lines(buffer);
                EngineResult::BufferModified
            }
            ParsedCommand::PasteAfter => {
                self.paste(buffer, false);
                EngineResult::BufferModified
            }
            ParsedCommand::PasteBefore => {
                self.paste(buffer, true);
                EngineResult::BufferModified
            }
            ParsedCommand::Replace(ch) => {
                self.replace_char(buffer, ch);
                EngineResult::BufferModified
            }
            ParsedCommand::OpenUrl => EngineResult::Suppressed,
            ParsedCommand::Undo => EngineResult::SimulateUndo,
            ParsedCommand::Redo => EngineResult::SimulateRedo,

            // Visual operations
            ParsedCommand::VisualOperation(op) => {
                self.execute_visual_operation(op, buffer);
                EngineResult::BufferModified
            }
            ParsedCommand::VisualSwapAnchor => {
                self.visual_swap_anchor(buffer);
                EngineResult::BufferModified
            }

            _ => EngineResult::Suppressed,
        }
    }

    fn handle_enter_insert(&mut self, variant: InsertVariant, buffer: &mut InMemoryBuffer) {
        let pos = buffer.get_cursor();
        match variant {
            InsertVariant::I => {}
            InsertVariant::A => {
                let new_col = (pos.col + 1).min(buffer.line_len(pos.line));
                buffer.set_cursor_insert(CursorPosition::new(pos.line, new_col));
            }
            InsertVariant::BigI => {
                let col = buffer
                    .line_at(pos.line)
                    .map(|l| {
                        l.chars()
                            .position(|c| !c.is_whitespace())
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                buffer.set_cursor(CursorPosition::new(pos.line, col));
            }
            InsertVariant::BigA => {
                buffer.set_cursor_insert(CursorPosition::new(pos.line, buffer.line_len(pos.line)));
            }
            InsertVariant::O => {
                let line_end = buffer.line_len(pos.line);
                let end = CursorPosition::new(pos.line, line_end);
                buffer.insert_at(end, "\n");
                buffer.set_cursor(CursorPosition::new(pos.line + 1, 0));
            }
            InsertVariant::BigO => {
                let start = CursorPosition::new(pos.line, 0);
                buffer.insert_at(start, "\n");
                buffer.set_cursor(CursorPosition::new(pos.line, 0));
            }
        }
        self.mode_sm.enter_insert(variant);
    }

    fn execute_motion(&mut self, motion: Motion, count: usize, buffer: &mut InMemoryBuffer) {
        let new_pos = self.resolve_motion(motion, count, buffer);

        if self.mode_sm.mode() == Mode::VisualCharacterwise
            || self.mode_sm.mode() == Mode::VisualLinewise
        {
            // Extend selection
            let sel = buffer.get_selection();
            let anchor = sel
                .map(|s| s.anchor)
                .unwrap_or_else(|| buffer.get_cursor());
            let kind = if self.mode_sm.mode() == Mode::VisualLinewise {
                vim_anywhere_core::buffer::SelectionKind::Linewise
            } else {
                vim_anywhere_core::buffer::SelectionKind::Characterwise
            };
            buffer.set_selection(Some(vim_anywhere_core::buffer::Selection::new(
                anchor, new_pos, kind,
            )));
        }

        buffer.set_cursor(new_pos);
        buffer.set_preferred_col(None);
    }

    fn resolve_motion(
        &mut self,
        motion: Motion,
        count: usize,
        buffer: &mut InMemoryBuffer,
    ) -> CursorPosition {
        match motion {
            Motion::Left => motions::motion_left(buffer, count),
            Motion::Right => motions::motion_right(buffer, count),
            Motion::Down => motions::motion_down(buffer, count),
            Motion::Up => motions::motion_up(buffer, count),
            Motion::LineStart => motions::motion_line_start(buffer, count),
            Motion::LineEnd => motions::motion_line_end(buffer, count),
            Motion::FirstNonBlank => motions::motion_first_non_blank(buffer, count),
            Motion::LastNonBlank => motions::motion_last_non_blank(buffer, count),
            Motion::LinePrevFirstNonBlank => motions::motion_prev_first_non_blank(buffer, count),
            Motion::LineNextFirstNonBlank | Motion::Return => {
                motions::motion_return(buffer, count)
            }
            Motion::WordForward => motions::motion_word_forward(buffer, count),
            Motion::WordForwardBig => motions::motion_word_forward_big(buffer, count),
            Motion::WordBackward => motions::motion_word_backward(buffer, count),
            Motion::WordBackwardBig => motions::motion_word_backward_big(buffer, count),
            Motion::WordEnd => motions::motion_word_end(buffer, count),
            Motion::WordEndBig => motions::motion_word_end_big(buffer, count),
            Motion::WordEndBackward => motions::motion_word_end_backward(buffer, count),
            Motion::WordEndBackwardBig => motions::motion_word_end_backward_big(buffer, count),
            Motion::FindChar(ch) => {
                self.registers.set_last_find(
                    vim_anywhere_core::register::FindRecord {
                        char: ch,
                        forward: true,
                        til: false,
                    },
                );
                motions::motion_find_char(buffer, ch, count)
            }
            Motion::FindCharBack(ch) => {
                self.registers.set_last_find(
                    vim_anywhere_core::register::FindRecord {
                        char: ch,
                        forward: false,
                        til: false,
                    },
                );
                motions::motion_find_char_back(buffer, ch, count)
            }
            Motion::TilChar(ch) => {
                self.registers.set_last_find(
                    vim_anywhere_core::register::FindRecord {
                        char: ch,
                        forward: true,
                        til: true,
                    },
                );
                motions::motion_til_char(buffer, ch, count)
            }
            Motion::TilCharBack(ch) => {
                self.registers.set_last_find(
                    vim_anywhere_core::register::FindRecord {
                        char: ch,
                        forward: false,
                        til: true,
                    },
                );
                motions::motion_til_char_back(buffer, ch, count)
            }
            Motion::RepeatFind => {
                if let Some(record) = self.registers.get_last_find().cloned() {
                    if record.forward && record.til {
                        motions::motion_til_char(buffer, record.char, count)
                    } else if record.forward {
                        motions::motion_find_char(buffer, record.char, count)
                    } else if record.til {
                        motions::motion_til_char_back(buffer, record.char, count)
                    } else {
                        motions::motion_find_char_back(buffer, record.char, count)
                    }
                } else {
                    buffer.get_cursor()
                }
            }
            Motion::RepeatFindReverse => {
                if let Some(record) = self.registers.get_last_find().cloned() {
                    if record.forward && record.til {
                        motions::motion_til_char_back(buffer, record.char, count)
                    } else if record.forward {
                        motions::motion_find_char_back(buffer, record.char, count)
                    } else if record.til {
                        motions::motion_til_char(buffer, record.char, count)
                    } else {
                        motions::motion_find_char(buffer, record.char, count)
                    }
                } else {
                    buffer.get_cursor()
                }
            }
            Motion::GoToLine => motions::motion_goto_line(buffer, count),
            Motion::GoToFirstLine => motions::motion_goto_first_line(buffer, count),
            Motion::GoToLastLine => motions::motion_goto_last_line(buffer, count),
            Motion::MatchBracket => motions::motion_match_bracket(buffer, count),
            Motion::ParagraphForward => motions::motion_paragraph_forward(buffer, count),
            Motion::ParagraphBackward => motions::motion_paragraph_backward(buffer, count),
            // Scroll motions — move cursor by page/half-page since we can't scroll AX viewport
            Motion::ScrollPageDown => motions::motion_down(buffer, count * 30),
            Motion::ScrollPageUp => motions::motion_up(buffer, count * 30),
            Motion::ScrollHalfPageDown => motions::motion_down(buffer, count * 15),
            Motion::ScrollHalfPageUp => motions::motion_up(buffer, count * 15),

            // Display motions — same as regular equivalents (no wrapped lines in AX)
            Motion::DisplayLineStart | Motion::InsertLineStart => motions::motion_line_start(buffer, count),
            Motion::DisplayLineEnd => motions::motion_line_end(buffer, count),
            Motion::DisplayFirstNonBlank => motions::motion_first_non_blank(buffer, count),
            Motion::DisplayLastNonBlank => motions::motion_last_non_blank(buffer, count),
            Motion::DisplayDown => motions::motion_down(buffer, count),
            Motion::DisplayUp => motions::motion_up(buffer, count),

            // Sentence motions
            Motion::SentenceForward => motions::motion_sentence_forward(buffer, count),
            Motion::SentenceBackward => motions::motion_sentence_backward(buffer, count),

            // Viewport-relative motions — no meaningful impl for AX, stay in place
            Motion::ScreenTop
            | Motion::ScreenMiddle
            | Motion::ScreenBottom
            | Motion::ScrollCursorTop
            | Motion::ScrollCursorCenter
            | Motion::ScrollCursorBottom
            | Motion::ScrollCursorTopFirstNonBlank
            | Motion::ScrollCursorCenterFirstNonBlank
            | Motion::ScrollCursorBottomFirstNonBlank
            | Motion::DisplayMiddle => buffer.get_cursor(),

            Motion::UnmatchedParenForward
            | Motion::UnmatchedParenBackward
            | Motion::UnmatchedBraceForward
            | Motion::UnmatchedBraceBackward => buffer.get_cursor(),
            Motion::SearchForward
            | Motion::SearchBackward
            | Motion::NextSearch
            | Motion::PrevSearch => buffer.get_cursor(),
            Motion::OpenUrl | Motion::WholeLine => buffer.get_cursor(),
        }
    }

    fn resolve_text_object(
        &self,
        obj: TextObject,
        buffer: &dyn TextBuffer,
    ) -> Option<(CursorPosition, CursorPosition)> {
        match obj {
            TextObject::InnerWord => motions::text_object_inner_word(buffer),
            TextObject::AWord => motions::text_object_a_word(buffer),
            TextObject::InnerWordBig => motions::text_object_inner_word_big(buffer),
            TextObject::AWordBig => motions::text_object_a_word_big(buffer),
            TextObject::InnerSentence => motions::text_object_inner_sentence(buffer),
            TextObject::ASentence => motions::text_object_a_sentence(buffer),
            TextObject::InnerParagraph => motions::text_object_inner_paragraph(buffer),
            TextObject::AParagraph => motions::text_object_a_paragraph(buffer),
            TextObject::InnerParen => motions::text_object_inner_paren(buffer),
            TextObject::AParen => motions::text_object_a_paren(buffer),
            TextObject::InnerBrace => motions::text_object_inner_brace(buffer),
            TextObject::ABrace => motions::text_object_a_brace(buffer),
            TextObject::InnerBracket => motions::text_object_inner_bracket(buffer),
            TextObject::ABracket => motions::text_object_a_bracket(buffer),
            TextObject::InnerAngle => motions::text_object_inner_angle(buffer),
            TextObject::AAngle => motions::text_object_a_angle(buffer),
            TextObject::InnerDoubleQuote => motions::text_object_inner_double_quote(buffer),
            TextObject::ADoubleQuote => motions::text_object_a_double_quote(buffer),
            TextObject::InnerSingleQuote => motions::text_object_inner_single_quote(buffer),
            TextObject::ASingleQuote => motions::text_object_a_single_quote(buffer),
            TextObject::InnerBacktick => motions::text_object_inner_backtick(buffer),
            TextObject::ABacktick => motions::text_object_a_backtick(buffer),
        }
    }

    fn execute_operator_motion(
        &mut self,
        op: Operator,
        motion: Motion,
        count: usize,
        buffer: &mut InMemoryBuffer,
    ) {
        let start = buffer.get_cursor();
        let end = self.resolve_motion(motion, count, buffer);
        buffer.set_cursor(start); // restore cursor before operation

        let inclusive = matches!(
            motion,
            Motion::WordEnd
                | Motion::WordEndBig
                | Motion::FindChar(_)
                | Motion::FindCharBack(_)
                | Motion::TilChar(_)
                | Motion::TilCharBack(_)
                | Motion::LineEnd
                | Motion::LastNonBlank
                | Motion::GoToLine
                | Motion::GoToFirstLine
                | Motion::GoToLastLine
                | Motion::MatchBracket
                | Motion::WordEndBackward
                | Motion::WordEndBackwardBig
        );

        let (range_start, mut range_end) = if start.line < end.line
            || (start.line == end.line && start.col <= end.col)
        {
            let end_col = if inclusive { end.col + 1 } else { end.col };
            (start, CursorPosition::new(end.line, end_col))
        } else {
            let start_col = if inclusive { start.col + 1 } else { start.col };
            (end, CursorPosition::new(start.line, start_col))
        };

        // When motion didn't move (hit boundary) but cursor is on a character,
        // include the character under cursor (e.g., x/dl at end of line)
        if range_start == range_end
            && matches!(motion, Motion::Right)
            && buffer.char_at(range_start).is_some()
        {
            range_end = CursorPosition::new(range_start.line, range_start.col + 1);
        }

        self.apply_operator(op, range_start, range_end, YankStyle::Characterwise, buffer);
    }

    fn execute_operator_text_object(
        &mut self,
        op: Operator,
        obj: TextObject,
        _count: usize,
        buffer: &mut InMemoryBuffer,
    ) {
        if let Some((start, end)) = self.resolve_text_object(obj, buffer) {
            self.apply_operator(op, start, end, YankStyle::Characterwise, buffer);
        }
    }

    fn execute_operator_line(
        &mut self,
        op: Operator,
        count: usize,
        buffer: &mut InMemoryBuffer,
    ) {
        let pos = buffer.get_cursor();
        let end_line = (pos.line + count.max(1) - 1).min(buffer.line_count().saturating_sub(1));
        let start = CursorPosition::new(pos.line, 0);

        let end = if end_line + 1 < buffer.line_count() {
            CursorPosition::new(end_line + 1, 0)
        } else {
            CursorPosition::new(end_line, buffer.line_len(end_line))
        };

        self.apply_operator(op, start, end, YankStyle::Linewise, buffer);
    }

    fn apply_operator(
        &mut self,
        op: Operator,
        start: CursorPosition,
        end: CursorPosition,
        style: YankStyle,
        buffer: &mut InMemoryBuffer,
    ) {
        let text = buffer.text_range(start, end);

        match op {
            Operator::Delete => {
                self.registers.yank(text, style);
                buffer.replace_range(start, end, "");
                buffer.set_cursor(start);
            }
            Operator::Change => {
                self.registers.yank(text, style);
                buffer.replace_range(start, end, "");
                buffer.set_cursor(start);
                self.mode_sm
                    .enter_insert(InsertVariant::I);
            }
            Operator::Yank => {
                self.registers.yank(text, style);
            }
            Operator::Indent => {
                for line in start.line..=end.line.min(buffer.line_count().saturating_sub(1)) {
                    let line_start = CursorPosition::new(line, 0);
                    buffer.insert_at(line_start, "    ");
                }
            }
            Operator::Outdent => {
                for line in start.line..=end.line.min(buffer.line_count().saturating_sub(1)) {
                    if let Some(l) = buffer.line_at(line) {
                        let spaces = l.chars().take(4).take_while(|c| *c == ' ').count();
                        if spaces > 0 {
                            buffer.replace_range(
                                CursorPosition::new(line, 0),
                                CursorPosition::new(line, spaces),
                                "",
                            );
                        }
                    }
                }
            }
            Operator::ToggleCase => {
                let toggled: String = text
                    .chars()
                    .map(|c| {
                        if c.is_uppercase() {
                            c.to_lowercase().next().unwrap_or(c)
                        } else {
                            c.to_uppercase().next().unwrap_or(c)
                        }
                    })
                    .collect();
                buffer.replace_range(start, end, &toggled);
                buffer.set_cursor(start);
            }
            Operator::Lowercase => {
                let lowered: String = text.chars().flat_map(|c| c.to_lowercase()).collect();
                buffer.replace_range(start, end, &lowered);
                buffer.set_cursor(start);
            }
            Operator::Uppercase => {
                let uppered: String = text.chars().flat_map(|c| c.to_uppercase()).collect();
                buffer.replace_range(start, end, &uppered);
                buffer.set_cursor(start);
            }
        }
    }

    fn toggle_case(&mut self, buffer: &mut InMemoryBuffer) {
        let pos = buffer.get_cursor();
        if let Some(ch) = buffer.char_at(pos) {
            let toggled = if ch.is_uppercase() {
                ch.to_lowercase().next().unwrap_or(ch)
            } else {
                ch.to_uppercase().next().unwrap_or(ch)
            }
            .to_string();
            let end = CursorPosition::new(pos.line, pos.col + 1);
            buffer.replace_range(pos, end, &toggled);
            buffer.set_cursor(CursorPosition::new(
                pos.line,
                (pos.col + 1).min(buffer.line_len(pos.line).saturating_sub(1)),
            ));
        }
    }

    fn join_lines(&mut self, buffer: &mut InMemoryBuffer) {
        let pos = buffer.get_cursor();
        if pos.line + 1 >= buffer.line_count() {
            return;
        }
        let current_len = buffer.line_len(pos.line);
        let next_line = buffer.line_at(pos.line + 1).unwrap_or("").to_string();
        let trimmed = next_line.trim_start();
        let join_pos = CursorPosition::new(pos.line, current_len);
        let next_end = CursorPosition::new(pos.line + 1, buffer.line_len(pos.line + 1));

        buffer.replace_range(join_pos, next_end, &format!(" {}", trimmed));
        buffer.set_cursor(CursorPosition::new(pos.line, current_len));
    }

    fn paste(&mut self, buffer: &mut InMemoryBuffer, before: bool) {
        let entry = match self.registers.get_unnamed() {
            Some(e) => e.clone(),
            None => return,
        };

        let pos = buffer.get_cursor();

        match entry.style {
            YankStyle::Characterwise => {
                if before {
                    buffer.insert_at(pos, &entry.content);
                } else {
                    let insert_pos =
                        CursorPosition::new(pos.line, (pos.col + 1).min(buffer.line_len(pos.line)));
                    buffer.insert_at(insert_pos, &entry.content);
                }
            }
            YankStyle::Linewise => {
                if before {
                    let line_start = CursorPosition::new(pos.line, 0);
                    let content = if entry.content.ends_with('\n') {
                        entry.content.clone()
                    } else {
                        format!("{}\n", entry.content)
                    };
                    buffer.insert_at(line_start, &content);
                    buffer.set_cursor(CursorPosition::new(pos.line, 0));
                } else {
                    let next_line_start = if pos.line + 1 < buffer.line_count() {
                        CursorPosition::new(pos.line + 1, 0)
                    } else {
                        let len = buffer.line_len(pos.line);
                        CursorPosition::new(pos.line, len)
                    };
                    let content = if pos.line + 1 >= buffer.line_count() {
                        if entry.content.ends_with('\n') && entry.content.len() > 1 {
                            format!("\n{}", &entry.content[..entry.content.len() - 1])
                        } else if entry.content.ends_with('\n') {
                            "\n".to_string()
                        } else {
                            format!("\n{}", entry.content)
                        }
                    } else if entry.content.ends_with('\n') {
                        entry.content.clone()
                    } else {
                        format!("{}\n", entry.content)
                    };
                    buffer.insert_at(next_line_start, &content);
                    buffer.set_cursor(CursorPosition::new(pos.line + 1, 0));
                }
            }
        }
    }

    fn replace_char(&mut self, buffer: &mut InMemoryBuffer, ch: char) {
        let pos = buffer.get_cursor();
        if buffer.char_at(pos).is_some() {
            let end = CursorPosition::new(pos.line, pos.col + 1);
            buffer.replace_range(pos, end, &ch.to_string());
            buffer.set_cursor(pos);
        }
    }

    fn execute_visual_operation(&mut self, op: Operator, buffer: &mut InMemoryBuffer) {
        if let Some(sel) = buffer.get_selection() {
            let start = sel.start();
            let mut end = sel.end();
            let style = match sel.kind {
                vim_anywhere_core::buffer::SelectionKind::Characterwise => {
                    end.col += 1;
                    YankStyle::Characterwise
                }
                vim_anywhere_core::buffer::SelectionKind::Linewise => {
                    let end_line_len = buffer.line_len(end.line);
                    end = if end.line + 1 < buffer.line_count() {
                        CursorPosition::new(end.line + 1, 0)
                    } else {
                        CursorPosition::new(end.line, end_line_len)
                    };
                    let start = CursorPosition::new(start.line, 0);
                    self.apply_operator(op, start, end, YankStyle::Linewise, buffer);
                    buffer.set_selection(None);
                    self.mode_sm.set_mode(Mode::Normal);
                    return;
                }
            };
            self.apply_operator(op, start, end, style, buffer);
        }
        buffer.set_selection(None);
        self.mode_sm.set_mode(Mode::Normal);
    }

    fn visual_swap_anchor(&mut self, buffer: &mut InMemoryBuffer) {
        if let Some(mut sel) = buffer.get_selection() {
            std::mem::swap(&mut sel.anchor, &mut sel.head);
            buffer.set_cursor(sel.head);
            buffer.set_selection(Some(sel));
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineResult {
    PassThrough,
    Suppressed,
    ModeChanged(Mode),
    BufferModified,
    /// Double-escape detected — pass a real Escape key through to the app.
    SendRealEscape,
    /// Simulate native undo (Cmd+Z).
    SimulateUndo,
    /// Simulate native redo (Cmd+Shift+Z).
    SimulateRedo,
}
