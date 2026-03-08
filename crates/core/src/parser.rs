use crate::modes::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Shift,
    Control,
    Option,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Escape,
    Return,
    Backspace,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Unknown(u16),
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Vec<Modifier>,
    pub is_repeat: bool,
}

impl KeyEvent {
    pub fn char(ch: char) -> Self {
        Self {
            key: Key::Char(ch),
            modifiers: vec![],
            is_repeat: false,
        }
    }

    pub fn ctrl(ch: char) -> Self {
        Self {
            key: Key::Char(ch),
            modifiers: vec![Modifier::Control],
            is_repeat: false,
        }
    }

    pub fn escape() -> Self {
        Self {
            key: Key::Escape,
            modifiers: vec![],
            is_repeat: false,
        }
    }

    pub fn enter() -> Self {
        Self {
            key: Key::Return,
            modifiers: vec![],
            is_repeat: false,
        }
    }

    pub fn has_modifier(&self, m: Modifier) -> bool {
        self.modifiers.contains(&m)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Change,
    Yank,
    Indent,
    Outdent,
    ToggleCase,
    Lowercase,
    Uppercase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Down,
    Up,
    LineStart,
    LineEnd,
    FirstNonBlank,
    LastNonBlank,
    LinePrevFirstNonBlank,
    LineNextFirstNonBlank,
    WordForward,
    WordForwardBig,
    WordBackward,
    WordBackwardBig,
    WordEnd,
    WordEndBig,
    WordEndBackward,
    WordEndBackwardBig,
    FindChar(char),
    FindCharBack(char),
    TilChar(char),
    TilCharBack(char),
    RepeatFind,
    RepeatFindReverse,
    Return,
    SearchForward,
    SearchBackward,
    NextSearch,
    PrevSearch,
    GoToLine,
    GoToFirstLine,
    GoToLastLine,
    ScreenTop,
    ScreenMiddle,
    ScreenBottom,
    MatchBracket,
    SentenceForward,
    SentenceBackward,
    ParagraphForward,
    ParagraphBackward,
    UnmatchedParenForward,
    UnmatchedParenBackward,
    UnmatchedBraceForward,
    UnmatchedBraceBackward,
    DisplayLineStart,
    DisplayLineEnd,
    DisplayFirstNonBlank,
    DisplayLastNonBlank,
    DisplayDown,
    DisplayUp,
    DisplayMiddle,
    InsertLineStart,
    ScrollPageUp,
    ScrollPageDown,
    ScrollHalfPageUp,
    ScrollHalfPageDown,
    ScrollCursorTop,
    ScrollCursorCenter,
    ScrollCursorBottom,
    ScrollCursorTopFirstNonBlank,
    ScrollCursorCenterFirstNonBlank,
    ScrollCursorBottomFirstNonBlank,
    OpenUrl,
    WholeLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextObject {
    InnerWord,
    AWord,
    InnerWordBig,
    AWordBig,
    InnerSentence,
    ASentence,
    InnerParagraph,
    AParagraph,
    InnerParen,
    AParen,
    InnerBrace,
    ABrace,
    InnerBracket,
    ABracket,
    InnerAngle,
    AAngle,
    InnerDoubleQuote,
    ADoubleQuote,
    InnerSingleQuote,
    ASingleQuote,
    InnerBacktick,
    ABacktick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCommand {
    Motion(Motion, usize),
    OperatorMotion(Operator, Motion, usize),
    OperatorTextObject(Operator, TextObject, usize),
    OperatorLine(Operator, usize),
    EnterInsert(crate::modes::InsertVariant),
    Replace(char),
    ToggleCase,
    JoinLines,
    PasteAfter,
    PasteBefore,
    OpenUrl,
    EnterVisualCharacterwise,
    EnterVisualLinewise,
    VisualOperation(Operator),
    VisualSwapAnchor,
    RepeatLastChange,
    Undo,
    Redo,
    Escape,
    Incomplete,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    Ready,
    WaitingOperatorMotion(Operator),
    WaitingGPrefix,
    WaitingZPrefix,
    WaitingOpenBracket,
    WaitingCloseBracket,
    WaitingFindChar(FindCharKind),
    WaitingReplaceChar,
    WaitingOperatorGPrefix(Operator),
    WaitingOperatorFindChar(Operator, FindCharKind),
    WaitingOperatorTextObjectKind(Operator, bool), // bool = is_inner
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FindCharKind {
    Find,
    FindBack,
    Til,
    TilBack,
}

pub struct KeyParser {
    state: ParserState,
    count: Option<usize>,
    count_str: String,
}

impl KeyParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Ready,
            count: None,
            count_str: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = ParserState::Ready;
        self.count = None;
        self.count_str.clear();
    }

    pub fn pending_keys(&self) -> &str {
        &self.count_str
    }

    fn get_count(&self) -> usize {
        self.count.unwrap_or(1)
    }

    pub fn parse(&mut self, event: &KeyEvent, mode: Mode) -> ParsedCommand {
        if event.key == Key::Escape {
            self.reset();
            return ParsedCommand::Escape;
        }

        match mode {
            Mode::Normal => self.parse_normal(event),
            Mode::VisualCharacterwise | Mode::VisualLinewise => self.parse_visual(event),
            Mode::Insert => ParsedCommand::Invalid,
        }
    }

    fn parse_normal(&mut self, event: &KeyEvent) -> ParsedCommand {
        let ch = match &event.key {
            Key::Char(c) => Some(*c),
            Key::Return => {
                let cmd = match self.state {
                    ParserState::WaitingZPrefix => {
                        self.reset();
                        return ParsedCommand::Motion(
                            Motion::ScrollCursorTopFirstNonBlank,
                            self.get_count(),
                        );
                    }
                    _ => ParsedCommand::Motion(Motion::Return, self.get_count()),
                };
                self.reset();
                return cmd;
            }
            Key::Backspace => {
                let cmd = ParsedCommand::Motion(Motion::Left, self.get_count());
                self.reset();
                return cmd;
            }
            Key::Delete => {
                let cmd = ParsedCommand::OperatorMotion(Operator::Delete, Motion::Right, self.get_count());
                self.reset();
                return cmd;
            }
            _ => None,
        };

        let ch = match ch {
            Some(c) => c,
            None => {
                if event.has_modifier(Modifier::Control) {
                    if let Key::Char(c) = &event.key {
                        match c {
                            'b' => {
                                let cmd =
                                    ParsedCommand::Motion(Motion::ScrollPageUp, self.get_count());
                                self.reset();
                                return cmd;
                            }
                            'd' => {
                                let cmd = ParsedCommand::Motion(
                                    Motion::ScrollHalfPageDown,
                                    self.get_count(),
                                );
                                self.reset();
                                return cmd;
                            }
                            'f' => {
                                let cmd = ParsedCommand::Motion(
                                    Motion::ScrollPageDown,
                                    self.get_count(),
                                );
                                self.reset();
                                return cmd;
                            }
                            'u' => {
                                let cmd = ParsedCommand::Motion(
                                    Motion::ScrollHalfPageUp,
                                    self.get_count(),
                                );
                                self.reset();
                                return cmd;
                            }
                            _ => {}
                        }
                    }
                }
                self.reset();
                return ParsedCommand::Invalid;
            }
        };

        match self.state {
            ParserState::Ready => self.parse_normal_ready(ch, event),
            ParserState::WaitingOperatorMotion(op) => self.parse_operator_motion(op, ch, event),
            ParserState::WaitingGPrefix => self.parse_g_prefix(ch),
            ParserState::WaitingZPrefix => self.parse_z_prefix(ch),
            ParserState::WaitingOpenBracket => self.parse_open_bracket(ch),
            ParserState::WaitingCloseBracket => self.parse_close_bracket(ch),
            ParserState::WaitingFindChar(kind) => {
                let motion = match kind {
                    FindCharKind::Find => Motion::FindChar(ch),
                    FindCharKind::FindBack => Motion::FindCharBack(ch),
                    FindCharKind::Til => Motion::TilChar(ch),
                    FindCharKind::TilBack => Motion::TilCharBack(ch),
                };
                let cmd = ParsedCommand::Motion(motion, self.get_count());
                self.reset();
                cmd
            }
            ParserState::WaitingReplaceChar => {
                let cmd = ParsedCommand::Replace(ch);
                self.reset();
                cmd
            }
            ParserState::WaitingOperatorGPrefix(op) => self.parse_operator_g_prefix(op, ch),
            ParserState::WaitingOperatorFindChar(op, kind) => {
                let motion = match kind {
                    FindCharKind::Find => Motion::FindChar(ch),
                    FindCharKind::FindBack => Motion::FindCharBack(ch),
                    FindCharKind::Til => Motion::TilChar(ch),
                    FindCharKind::TilBack => Motion::TilCharBack(ch),
                };
                let cmd = ParsedCommand::OperatorMotion(op, motion, self.get_count());
                self.reset();
                cmd
            }
            ParserState::WaitingOperatorTextObjectKind(op, is_inner) => {
                self.parse_text_object_kind(op, is_inner, ch)
            }
        }
    }

    fn parse_normal_ready(&mut self, ch: char, event: &KeyEvent) -> ParsedCommand {
        // Handle ctrl+ keys
        if event.has_modifier(Modifier::Control) {
            match ch {
                'b' => {
                    let cmd = ParsedCommand::Motion(Motion::ScrollPageUp, self.get_count());
                    self.reset();
                    return cmd;
                }
                'd' => {
                    let cmd = ParsedCommand::Motion(Motion::ScrollHalfPageDown, self.get_count());
                    self.reset();
                    return cmd;
                }
                'f' => {
                    let cmd = ParsedCommand::Motion(Motion::ScrollPageDown, self.get_count());
                    self.reset();
                    return cmd;
                }
                'u' => {
                    let cmd = ParsedCommand::Motion(Motion::ScrollHalfPageUp, self.get_count());
                    self.reset();
                    return cmd;
                }
                'r' => {
                    let cmd = ParsedCommand::Redo;
                    self.reset();
                    return cmd;
                }
                _ => {
                    self.reset();
                    return ParsedCommand::Invalid;
                }
            }
        }

        // Count accumulation
        if ch.is_ascii_digit() && (ch != '0' || self.count.is_some()) {
            self.count_str.push(ch);
            let digit = ch as usize - '0' as usize;
            self.count = Some(self.count.unwrap_or(0) * 10 + digit);
            return ParsedCommand::Incomplete;
        }

        let count = self.get_count();

        let cmd = match ch {
            // Basic navigation
            'h' => ParsedCommand::Motion(Motion::Left, count),
            'l' => ParsedCommand::Motion(Motion::Right, count),
            'j' => ParsedCommand::Motion(Motion::Down, count),
            'k' => ParsedCommand::Motion(Motion::Up, count),
            '0' => ParsedCommand::Motion(Motion::LineStart, count),
            '$' => ParsedCommand::Motion(Motion::LineEnd, count),
            '^' => ParsedCommand::Motion(Motion::FirstNonBlank, count),
            '_' => ParsedCommand::Motion(Motion::LastNonBlank, count),
            '-' => ParsedCommand::Motion(Motion::LinePrevFirstNonBlank, count),
            'w' => ParsedCommand::Motion(Motion::WordForward, count),
            'W' => ParsedCommand::Motion(Motion::WordForwardBig, count),
            'b' => ParsedCommand::Motion(Motion::WordBackward, count),
            'B' => ParsedCommand::Motion(Motion::WordBackwardBig, count),
            'e' => ParsedCommand::Motion(Motion::WordEnd, count),
            'E' => ParsedCommand::Motion(Motion::WordEndBig, count),
            ';' => ParsedCommand::Motion(Motion::RepeatFind, count),
            ',' => ParsedCommand::Motion(Motion::RepeatFindReverse, count),
            'n' => ParsedCommand::Motion(Motion::NextSearch, count),
            'N' => ParsedCommand::Motion(Motion::PrevSearch, count),
            'G' => {
                if self.count.is_some() {
                    ParsedCommand::Motion(Motion::GoToLine, count)
                } else {
                    ParsedCommand::Motion(Motion::GoToLastLine, 1)
                }
            }
            'H' => ParsedCommand::Motion(Motion::ScreenTop, count),
            'M' => ParsedCommand::Motion(Motion::ScreenMiddle, count),
            'L' => ParsedCommand::Motion(Motion::ScreenBottom, count),
            '%' => ParsedCommand::Motion(Motion::MatchBracket, count),
            '(' => ParsedCommand::Motion(Motion::SentenceBackward, count),
            ')' => ParsedCommand::Motion(Motion::SentenceForward, count),
            '{' => ParsedCommand::Motion(Motion::ParagraphBackward, count),
            '}' => ParsedCommand::Motion(Motion::ParagraphForward, count),
            '/' => ParsedCommand::Motion(Motion::SearchForward, count),
            '?' => ParsedCommand::Motion(Motion::SearchBackward, count),

            // Find char
            'f' => {
                self.state = ParserState::WaitingFindChar(FindCharKind::Find);
                return ParsedCommand::Incomplete;
            }
            'F' => {
                self.state = ParserState::WaitingFindChar(FindCharKind::FindBack);
                return ParsedCommand::Incomplete;
            }
            't' => {
                self.state = ParserState::WaitingFindChar(FindCharKind::Til);
                return ParsedCommand::Incomplete;
            }
            'T' => {
                self.state = ParserState::WaitingFindChar(FindCharKind::TilBack);
                return ParsedCommand::Incomplete;
            }

            // Prefix commands
            'g' => {
                self.state = ParserState::WaitingGPrefix;
                return ParsedCommand::Incomplete;
            }
            'z' => {
                self.state = ParserState::WaitingZPrefix;
                return ParsedCommand::Incomplete;
            }
            '[' => {
                self.state = ParserState::WaitingOpenBracket;
                return ParsedCommand::Incomplete;
            }
            ']' => {
                self.state = ParserState::WaitingCloseBracket;
                return ParsedCommand::Incomplete;
            }

            // Operators
            'd' => {
                self.state = ParserState::WaitingOperatorMotion(Operator::Delete);
                return ParsedCommand::Incomplete;
            }
            'c' => {
                self.state = ParserState::WaitingOperatorMotion(Operator::Change);
                return ParsedCommand::Incomplete;
            }
            'y' => {
                self.state = ParserState::WaitingOperatorMotion(Operator::Yank);
                return ParsedCommand::Incomplete;
            }

            // Shortcuts
            'D' => ParsedCommand::OperatorMotion(Operator::Delete, Motion::LineEnd, 1),
            'C' => ParsedCommand::OperatorMotion(Operator::Change, Motion::LineEnd, 1),
            'Y' => ParsedCommand::OperatorLine(Operator::Yank, count),

            // Insert mode entries
            'i' => ParsedCommand::EnterInsert(crate::modes::InsertVariant::I),
            'I' => ParsedCommand::EnterInsert(crate::modes::InsertVariant::BigI),
            'a' => ParsedCommand::EnterInsert(crate::modes::InsertVariant::A),
            'A' => ParsedCommand::EnterInsert(crate::modes::InsertVariant::BigA),
            'o' => ParsedCommand::EnterInsert(crate::modes::InsertVariant::O),
            'O' => ParsedCommand::EnterInsert(crate::modes::InsertVariant::BigO),

            // Single-char editing
            'x' => ParsedCommand::OperatorMotion(Operator::Delete, Motion::Right, count),
            'X' => ParsedCommand::OperatorMotion(Operator::Delete, Motion::Left, count),
            's' => ParsedCommand::OperatorMotion(Operator::Change, Motion::Right, count),
            'r' => {
                self.state = ParserState::WaitingReplaceChar;
                return ParsedCommand::Incomplete;
            }
            '~' => ParsedCommand::ToggleCase,
            'J' => ParsedCommand::JoinLines,
            'p' => ParsedCommand::PasteAfter,
            'P' => ParsedCommand::PasteBefore,

            // Indent
            '<' => {
                self.state = ParserState::WaitingOperatorMotion(Operator::Outdent);
                return ParsedCommand::Incomplete;
            }
            '>' => {
                self.state = ParserState::WaitingOperatorMotion(Operator::Indent);
                return ParsedCommand::Incomplete;
            }

            // Line substitute (like cc)
            'S' => ParsedCommand::OperatorLine(Operator::Change, count),

            // Dot repeat
            '.' => ParsedCommand::RepeatLastChange,

            // Undo
            'u' => ParsedCommand::Undo,

            // Visual mode
            'v' => ParsedCommand::EnterVisualCharacterwise,
            'V' => ParsedCommand::EnterVisualLinewise,

            _ => ParsedCommand::Invalid,
        };

        self.reset();
        cmd
    }

    fn parse_operator_motion(
        &mut self,
        op: Operator,
        ch: char,
        event: &KeyEvent,
    ) -> ParsedCommand {
        if event.has_modifier(Modifier::Control) {
            self.reset();
            return ParsedCommand::Invalid;
        }

        // Count after operator (e.g., d2w)
        if ch.is_ascii_digit() && (ch != '0' || self.count.is_some()) {
            self.count_str.push(ch);
            let digit = ch as usize - '0' as usize;
            self.count = Some(self.count.unwrap_or(0) * 10 + digit);
            return ParsedCommand::Incomplete;
        }

        let count = self.get_count();

        // Double operator = line operation (dd, cc, yy, <<, >>)
        let op_char = match op {
            Operator::Delete => 'd',
            Operator::Change => 'c',
            Operator::Yank => 'y',
            Operator::Indent => '>',
            Operator::Outdent => '<',
            _ => '\0',
        };
        if ch == op_char {
            let cmd = ParsedCommand::OperatorLine(op, count);
            self.reset();
            return cmd;
        }

        // Text objects (i/a prefix)
        if ch == 'i' {
            self.state = ParserState::WaitingOperatorTextObjectKind(op, true);
            return ParsedCommand::Incomplete;
        }
        if ch == 'a' {
            self.state = ParserState::WaitingOperatorTextObjectKind(op, false);
            return ParsedCommand::Incomplete;
        }

        // Find char within operator
        match ch {
            'f' => {
                self.state = ParserState::WaitingOperatorFindChar(op, FindCharKind::Find);
                return ParsedCommand::Incomplete;
            }
            'F' => {
                self.state = ParserState::WaitingOperatorFindChar(op, FindCharKind::FindBack);
                return ParsedCommand::Incomplete;
            }
            't' => {
                self.state = ParserState::WaitingOperatorFindChar(op, FindCharKind::Til);
                return ParsedCommand::Incomplete;
            }
            'T' => {
                self.state = ParserState::WaitingOperatorFindChar(op, FindCharKind::TilBack);
                return ParsedCommand::Incomplete;
            }
            _ => {}
        }

        // g prefix within operator
        if ch == 'g' {
            self.state = ParserState::WaitingOperatorGPrefix(op);
            return ParsedCommand::Incomplete;
        }

        let motion = match ch {
            'h' => Motion::Left,
            'l' => Motion::Right,
            'j' => Motion::Down,
            'k' => Motion::Up,
            '0' => Motion::LineStart,
            '$' => Motion::LineEnd,
            '^' => Motion::FirstNonBlank,
            'w' => Motion::WordForward,
            'W' => Motion::WordForwardBig,
            'b' => Motion::WordBackward,
            'B' => Motion::WordBackwardBig,
            'e' => Motion::WordEnd,
            'E' => Motion::WordEndBig,
            'G' => {
                let m = if self.count.is_some() { Motion::GoToLine } else { Motion::GoToLastLine };
                let cmd = ParsedCommand::OperatorMotion(op, m, count);
                self.reset();
                return cmd;
            }
            _ => {
                self.reset();
                return ParsedCommand::Invalid;
            }
        };

        let cmd = ParsedCommand::OperatorMotion(op, motion, count);
        self.reset();
        cmd
    }

    fn parse_g_prefix(&mut self, ch: char) -> ParsedCommand {
        let count = self.get_count();
        let cmd = match ch {
            'g' => {
                if self.count.is_some() {
                    ParsedCommand::Motion(Motion::GoToLine, count)
                } else {
                    ParsedCommand::Motion(Motion::GoToFirstLine, count)
                }
            }
            '0' => ParsedCommand::Motion(Motion::DisplayLineStart, count),
            '$' => ParsedCommand::Motion(Motion::DisplayLineEnd, count),
            '^' => ParsedCommand::Motion(Motion::DisplayFirstNonBlank, count),
            '_' => ParsedCommand::Motion(Motion::DisplayLastNonBlank, count),
            'j' => ParsedCommand::Motion(Motion::DisplayDown, count),
            'k' => ParsedCommand::Motion(Motion::DisplayUp, count),
            'm' => ParsedCommand::Motion(Motion::DisplayMiddle, count),
            'e' => ParsedCommand::Motion(Motion::WordEndBackward, count),
            'E' => ParsedCommand::Motion(Motion::WordEndBackwardBig, count),
            'I' => ParsedCommand::Motion(Motion::InsertLineStart, count),
            'x' => ParsedCommand::OpenUrl,
            _ => ParsedCommand::Invalid,
        };
        self.reset();
        cmd
    }

    fn parse_z_prefix(&mut self, ch: char) -> ParsedCommand {
        let count = self.get_count();
        let cmd = match ch {
            't' => ParsedCommand::Motion(Motion::ScrollCursorTop, count),
            'z' => ParsedCommand::Motion(Motion::ScrollCursorCenter, count),
            'b' => ParsedCommand::Motion(Motion::ScrollCursorBottom, count),
            '.' => ParsedCommand::Motion(Motion::ScrollCursorCenterFirstNonBlank, count),
            '-' => ParsedCommand::Motion(Motion::ScrollCursorBottomFirstNonBlank, count),
            _ => ParsedCommand::Invalid,
        };
        self.reset();
        cmd
    }

    fn parse_open_bracket(&mut self, ch: char) -> ParsedCommand {
        let count = self.get_count();
        let cmd = match ch {
            '(' => ParsedCommand::Motion(Motion::UnmatchedParenBackward, count),
            '{' => ParsedCommand::Motion(Motion::UnmatchedBraceBackward, count),
            _ => ParsedCommand::Invalid,
        };
        self.reset();
        cmd
    }

    fn parse_close_bracket(&mut self, ch: char) -> ParsedCommand {
        let count = self.get_count();
        let cmd = match ch {
            ')' => ParsedCommand::Motion(Motion::UnmatchedParenForward, count),
            '}' => ParsedCommand::Motion(Motion::UnmatchedBraceForward, count),
            _ => ParsedCommand::Invalid,
        };
        self.reset();
        cmd
    }

    fn parse_operator_g_prefix(&mut self, op: Operator, ch: char) -> ParsedCommand {
        let count = self.get_count();
        let motion = match ch {
            'g' => Motion::GoToFirstLine,
            '0' => Motion::DisplayLineStart,
            '$' => Motion::DisplayLineEnd,
            _ => {
                self.reset();
                return ParsedCommand::Invalid;
            }
        };
        let cmd = ParsedCommand::OperatorMotion(op, motion, count);
        self.reset();
        cmd
    }

    fn parse_text_object_kind(
        &mut self,
        op: Operator,
        is_inner: bool,
        ch: char,
    ) -> ParsedCommand {
        let obj = match ch {
            'w' => {
                if is_inner {
                    TextObject::InnerWord
                } else {
                    TextObject::AWord
                }
            }
            'W' => {
                if is_inner {
                    TextObject::InnerWordBig
                } else {
                    TextObject::AWordBig
                }
            }
            's' => {
                if is_inner {
                    TextObject::InnerSentence
                } else {
                    TextObject::ASentence
                }
            }
            'p' => {
                if is_inner {
                    TextObject::InnerParagraph
                } else {
                    TextObject::AParagraph
                }
            }
            '(' | 'b' => {
                if is_inner {
                    TextObject::InnerParen
                } else {
                    TextObject::AParen
                }
            }
            '{' | 'B' => {
                if is_inner {
                    TextObject::InnerBrace
                } else {
                    TextObject::ABrace
                }
            }
            '[' => {
                if is_inner {
                    TextObject::InnerBracket
                } else {
                    TextObject::ABracket
                }
            }
            '<' => {
                if is_inner {
                    TextObject::InnerAngle
                } else {
                    TextObject::AAngle
                }
            }
            '"' => {
                if is_inner {
                    TextObject::InnerDoubleQuote
                } else {
                    TextObject::ADoubleQuote
                }
            }
            '\'' => {
                if is_inner {
                    TextObject::InnerSingleQuote
                } else {
                    TextObject::ASingleQuote
                }
            }
            '`' => {
                if is_inner {
                    TextObject::InnerBacktick
                } else {
                    TextObject::ABacktick
                }
            }
            ')' => {
                if is_inner {
                    TextObject::InnerParen
                } else {
                    TextObject::AParen
                }
            }
            '}' => {
                if is_inner {
                    TextObject::InnerBrace
                } else {
                    TextObject::ABrace
                }
            }
            ']' => {
                if is_inner {
                    TextObject::InnerBracket
                } else {
                    TextObject::ABracket
                }
            }
            '>' => {
                if is_inner {
                    TextObject::InnerAngle
                } else {
                    TextObject::AAngle
                }
            }
            _ => {
                self.reset();
                return ParsedCommand::Invalid;
            }
        };
        let cmd = ParsedCommand::OperatorTextObject(op, obj, self.get_count());
        self.reset();
        cmd
    }

    fn parse_visual(&mut self, event: &KeyEvent) -> ParsedCommand {
        let ch = match &event.key {
            Key::Char(c) => *c,
            Key::Return => {
                self.reset();
                return ParsedCommand::Motion(Motion::Return, self.get_count());
            }
            Key::Backspace => {
                let cmd = ParsedCommand::Motion(Motion::Left, self.get_count());
                self.reset();
                return cmd;
            }
            _ => {
                self.reset();
                return ParsedCommand::Invalid;
            }
        };

        if event.has_modifier(Modifier::Control) {
            self.reset();
            return ParsedCommand::Invalid;
        }

        // Count accumulation
        if ch.is_ascii_digit() && (ch != '0' || self.count.is_some()) {
            self.count_str.push(ch);
            let digit = ch as usize - '0' as usize;
            self.count = Some(self.count.unwrap_or(0) * 10 + digit);
            return ParsedCommand::Incomplete;
        }

        let count = self.get_count();

        let cmd = match self.state {
            ParserState::WaitingFindChar(kind) => {
                let motion = match kind {
                    FindCharKind::Find => Motion::FindChar(ch),
                    FindCharKind::FindBack => Motion::FindCharBack(ch),
                    FindCharKind::Til => Motion::TilChar(ch),
                    FindCharKind::TilBack => Motion::TilCharBack(ch),
                };
                ParsedCommand::Motion(motion, count)
            }
            ParserState::WaitingGPrefix => match ch {
                'g' => ParsedCommand::Motion(Motion::GoToFirstLine, count),
                '$' => ParsedCommand::Motion(Motion::DisplayLineEnd, count),
                '_' => ParsedCommand::Motion(Motion::DisplayLastNonBlank, count),
                'e' => ParsedCommand::Motion(Motion::WordEndBackward, count),
                'E' => ParsedCommand::Motion(Motion::WordEndBackwardBig, count),
                'I' => ParsedCommand::Motion(Motion::InsertLineStart, count),
                'j' => ParsedCommand::Motion(Motion::DisplayDown, count),
                'k' => ParsedCommand::Motion(Motion::DisplayUp, count),
                'x' => ParsedCommand::OpenUrl,
                _ => ParsedCommand::Invalid,
            },
            _ => {
                match ch {
                    // Navigation
                    'h' => ParsedCommand::Motion(Motion::Left, count),
                    'l' => ParsedCommand::Motion(Motion::Right, count),
                    'j' => ParsedCommand::Motion(Motion::Down, count),
                    'k' => ParsedCommand::Motion(Motion::Up, count),
                    '0' => ParsedCommand::Motion(Motion::LineStart, count),
                    '$' => ParsedCommand::Motion(Motion::LineEnd, count),
                    '^' => ParsedCommand::Motion(Motion::FirstNonBlank, count),
                    '_' => ParsedCommand::Motion(Motion::LastNonBlank, count),
                    '-' => ParsedCommand::Motion(Motion::LinePrevFirstNonBlank, count),
                    'w' => ParsedCommand::Motion(Motion::WordForward, count),
                    'W' => ParsedCommand::Motion(Motion::WordForwardBig, count),
                    'b' => ParsedCommand::Motion(Motion::WordBackward, count),
                    'B' => ParsedCommand::Motion(Motion::WordBackwardBig, count),
                    'e' => ParsedCommand::Motion(Motion::WordEnd, count),
                    'E' => ParsedCommand::Motion(Motion::WordEndBig, count),
                    ';' => ParsedCommand::Motion(Motion::RepeatFind, count),
                    ',' => ParsedCommand::Motion(Motion::RepeatFindReverse, count),
                    '(' => ParsedCommand::Motion(Motion::SentenceBackward, count),
                    ')' => ParsedCommand::Motion(Motion::SentenceForward, count),
                    '{' => ParsedCommand::Motion(Motion::ParagraphBackward, count),
                    '}' => ParsedCommand::Motion(Motion::ParagraphForward, count),
                    '%' => ParsedCommand::Motion(Motion::MatchBracket, count),
                    'n' => ParsedCommand::Motion(Motion::NextSearch, count),
                    'N' => ParsedCommand::Motion(Motion::PrevSearch, count),
                    'G' => {
                        if self.count.is_some() {
                            ParsedCommand::Motion(Motion::GoToLine, count)
                        } else {
                            ParsedCommand::Motion(Motion::GoToLastLine, 1)
                        }
                    }

                    // Find char
                    'f' => {
                        self.state = ParserState::WaitingFindChar(FindCharKind::Find);
                        return ParsedCommand::Incomplete;
                    }
                    'F' => {
                        self.state = ParserState::WaitingFindChar(FindCharKind::FindBack);
                        return ParsedCommand::Incomplete;
                    }
                    't' => {
                        self.state = ParserState::WaitingFindChar(FindCharKind::Til);
                        return ParsedCommand::Incomplete;
                    }
                    'T' => {
                        self.state = ParserState::WaitingFindChar(FindCharKind::TilBack);
                        return ParsedCommand::Incomplete;
                    }
                    'g' => {
                        self.state = ParserState::WaitingGPrefix;
                        return ParsedCommand::Incomplete;
                    }

                    // Operations
                    'x' => ParsedCommand::VisualOperation(Operator::Delete),
                    'c' => ParsedCommand::VisualOperation(Operator::Change),
                    'd' => ParsedCommand::VisualOperation(Operator::Delete),
                    'y' => ParsedCommand::VisualOperation(Operator::Yank),
                    '<' => ParsedCommand::VisualOperation(Operator::Outdent),
                    '>' => ParsedCommand::VisualOperation(Operator::Indent),
                    '~' => ParsedCommand::VisualOperation(Operator::ToggleCase),
                    'u' => ParsedCommand::VisualOperation(Operator::Lowercase),
                    'U' => ParsedCommand::VisualOperation(Operator::Uppercase),
                    'C' => ParsedCommand::VisualOperation(Operator::Change),
                    'D' => ParsedCommand::VisualOperation(Operator::Delete),
                    'S' => ParsedCommand::VisualOperation(Operator::Change),
                    'R' => ParsedCommand::VisualOperation(Operator::Change),
                    'Y' => ParsedCommand::VisualOperation(Operator::Yank),
                    'J' => ParsedCommand::JoinLines,

                    // Mode switching
                    'o' => ParsedCommand::VisualSwapAnchor,
                    'v' => ParsedCommand::EnterVisualCharacterwise,
                    'V' => ParsedCommand::EnterVisualLinewise,

                    _ => ParsedCommand::Invalid,
                }
            }
        };

        self.reset();
        cmd
    }
}

impl Default for KeyParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_keys(keys: &str) -> ParsedCommand {
        let mut parser = KeyParser::new();
        let mut result = ParsedCommand::Incomplete;
        for ch in keys.chars() {
            result = parser.parse(&KeyEvent::char(ch), Mode::Normal);
            if result != ParsedCommand::Incomplete {
                return result;
            }
        }
        result
    }

    fn parse_keys_visual(keys: &str) -> ParsedCommand {
        let mut parser = KeyParser::new();
        let mut result = ParsedCommand::Incomplete;
        for ch in keys.chars() {
            result = parser.parse(&KeyEvent::char(ch), Mode::VisualCharacterwise);
            if result != ParsedCommand::Incomplete {
                return result;
            }
        }
        result
    }

    #[test]
    fn simple_motions() {
        assert_eq!(parse_keys("h"), ParsedCommand::Motion(Motion::Left, 1));
        assert_eq!(parse_keys("j"), ParsedCommand::Motion(Motion::Down, 1));
        assert_eq!(parse_keys("k"), ParsedCommand::Motion(Motion::Up, 1));
        assert_eq!(parse_keys("l"), ParsedCommand::Motion(Motion::Right, 1));
        assert_eq!(parse_keys("w"), ParsedCommand::Motion(Motion::WordForward, 1));
        assert_eq!(parse_keys("$"), ParsedCommand::Motion(Motion::LineEnd, 1));
        assert_eq!(parse_keys("0"), ParsedCommand::Motion(Motion::LineStart, 1));
    }

    #[test]
    fn count_prefix() {
        assert_eq!(parse_keys("3w"), ParsedCommand::Motion(Motion::WordForward, 3));
        assert_eq!(parse_keys("15j"), ParsedCommand::Motion(Motion::Down, 15));
        assert_eq!(parse_keys("5j"), ParsedCommand::Motion(Motion::Down, 5));
    }

    #[test]
    fn operator_motion() {
        assert_eq!(
            parse_keys("dw"),
            ParsedCommand::OperatorMotion(Operator::Delete, Motion::WordForward, 1)
        );
        assert_eq!(
            parse_keys("d$"),
            ParsedCommand::OperatorMotion(Operator::Delete, Motion::LineEnd, 1)
        );
        assert_eq!(
            parse_keys("3dw"),
            ParsedCommand::OperatorMotion(Operator::Delete, Motion::WordForward, 3)
        );
    }

    #[test]
    fn operator_line() {
        assert_eq!(
            parse_keys("dd"),
            ParsedCommand::OperatorLine(Operator::Delete, 1)
        );
        assert_eq!(
            parse_keys("cc"),
            ParsedCommand::OperatorLine(Operator::Change, 1)
        );
        assert_eq!(
            parse_keys("yy"),
            ParsedCommand::OperatorLine(Operator::Yank, 1)
        );
        assert_eq!(
            parse_keys("3dd"),
            ParsedCommand::OperatorLine(Operator::Delete, 3)
        );
    }

    #[test]
    fn operator_text_object() {
        assert_eq!(
            parse_keys("ciw"),
            ParsedCommand::OperatorTextObject(Operator::Change, TextObject::InnerWord, 1)
        );
        assert_eq!(
            parse_keys("dap"),
            ParsedCommand::OperatorTextObject(Operator::Delete, TextObject::AParagraph, 1)
        );
        assert_eq!(
            parse_keys("yi\""),
            ParsedCommand::OperatorTextObject(Operator::Yank, TextObject::InnerDoubleQuote, 1)
        );
        assert_eq!(
            parse_keys("ci("),
            ParsedCommand::OperatorTextObject(Operator::Change, TextObject::InnerParen, 1)
        );
        assert_eq!(
            parse_keys("da{"),
            ParsedCommand::OperatorTextObject(Operator::Delete, TextObject::ABrace, 1)
        );
    }

    #[test]
    fn find_char() {
        assert_eq!(
            parse_keys("fa"),
            ParsedCommand::Motion(Motion::FindChar('a'), 1)
        );
        assert_eq!(
            parse_keys("Fx"),
            ParsedCommand::Motion(Motion::FindCharBack('x'), 1)
        );
        assert_eq!(
            parse_keys("ta"),
            ParsedCommand::Motion(Motion::TilChar('a'), 1)
        );
        assert_eq!(
            parse_keys("2fa"),
            ParsedCommand::Motion(Motion::FindChar('a'), 2)
        );
    }

    #[test]
    fn operator_find_char() {
        assert_eq!(
            parse_keys("dfa"),
            ParsedCommand::OperatorMotion(Operator::Delete, Motion::FindChar('a'), 1)
        );
        assert_eq!(
            parse_keys("cTx"),
            ParsedCommand::OperatorMotion(Operator::Change, Motion::TilCharBack('x'), 1)
        );
    }

    #[test]
    fn g_prefix() {
        assert_eq!(
            parse_keys("gg"),
            ParsedCommand::Motion(Motion::GoToFirstLine, 1)
        );
        assert_eq!(
            parse_keys("g$"),
            ParsedCommand::Motion(Motion::DisplayLineEnd, 1)
        );
        assert_eq!(
            parse_keys("gj"),
            ParsedCommand::Motion(Motion::DisplayDown, 1)
        );
        assert_eq!(parse_keys("gx"), ParsedCommand::OpenUrl);
    }

    #[test]
    fn z_prefix() {
        assert_eq!(
            parse_keys("zt"),
            ParsedCommand::Motion(Motion::ScrollCursorTop, 1)
        );
        assert_eq!(
            parse_keys("zz"),
            ParsedCommand::Motion(Motion::ScrollCursorCenter, 1)
        );
        assert_eq!(
            parse_keys("zb"),
            ParsedCommand::Motion(Motion::ScrollCursorBottom, 1)
        );
    }

    #[test]
    fn bracket_prefix() {
        assert_eq!(
            parse_keys("[("),
            ParsedCommand::Motion(Motion::UnmatchedParenBackward, 1)
        );
        assert_eq!(
            parse_keys("[{"),
            ParsedCommand::Motion(Motion::UnmatchedBraceBackward, 1)
        );
        assert_eq!(
            parse_keys("])"),
            ParsedCommand::Motion(Motion::UnmatchedParenForward, 1)
        );
        assert_eq!(
            parse_keys("]}"),
            ParsedCommand::Motion(Motion::UnmatchedBraceForward, 1)
        );
    }

    #[test]
    fn operator_g_prefix() {
        assert_eq!(
            parse_keys("dgg"),
            ParsedCommand::OperatorMotion(Operator::Delete, Motion::GoToFirstLine, 1)
        );
        assert_eq!(
            parse_keys("cg$"),
            ParsedCommand::OperatorMotion(Operator::Change, Motion::DisplayLineEnd, 1)
        );
    }

    #[test]
    fn insert_commands() {
        assert_eq!(
            parse_keys("i"),
            ParsedCommand::EnterInsert(crate::modes::InsertVariant::I)
        );
        assert_eq!(
            parse_keys("A"),
            ParsedCommand::EnterInsert(crate::modes::InsertVariant::BigA)
        );
        assert_eq!(
            parse_keys("o"),
            ParsedCommand::EnterInsert(crate::modes::InsertVariant::O)
        );
    }

    #[test]
    fn editing_commands() {
        assert_eq!(parse_keys("~"), ParsedCommand::ToggleCase);
        assert_eq!(parse_keys("J"), ParsedCommand::JoinLines);
        assert_eq!(parse_keys("p"), ParsedCommand::PasteAfter);
        assert_eq!(parse_keys("P"), ParsedCommand::PasteBefore);
    }

    #[test]
    fn replace_char() {
        assert_eq!(parse_keys("rx"), ParsedCommand::Replace('x'));
    }

    #[test]
    fn indent_outdent() {
        assert_eq!(
            parse_keys("<<"),
            ParsedCommand::OperatorLine(Operator::Outdent, 1)
        );
        assert_eq!(
            parse_keys(">>"),
            ParsedCommand::OperatorLine(Operator::Indent, 1)
        );
    }

    #[test]
    fn shortcuts() {
        assert_eq!(
            parse_keys("D"),
            ParsedCommand::OperatorMotion(Operator::Delete, Motion::LineEnd, 1)
        );
        assert_eq!(
            parse_keys("C"),
            ParsedCommand::OperatorMotion(Operator::Change, Motion::LineEnd, 1)
        );
    }

    #[test]
    fn visual_mode_operations() {
        assert_eq!(
            parse_keys_visual("d"),
            ParsedCommand::VisualOperation(Operator::Delete)
        );
        assert_eq!(
            parse_keys_visual("c"),
            ParsedCommand::VisualOperation(Operator::Change)
        );
        assert_eq!(
            parse_keys_visual("y"),
            ParsedCommand::VisualOperation(Operator::Yank)
        );
        assert_eq!(parse_keys_visual("o"), ParsedCommand::VisualSwapAnchor);
    }

    #[test]
    fn visual_mode_navigation() {
        assert_eq!(
            parse_keys_visual("w"),
            ParsedCommand::Motion(Motion::WordForward, 1)
        );
        assert_eq!(
            parse_keys_visual("gg"),
            ParsedCommand::Motion(Motion::GoToFirstLine, 1)
        );
    }

    #[test]
    fn ctrl_keys() {
        let mut parser = KeyParser::new();
        let result = parser.parse(&KeyEvent::ctrl('d'), Mode::Normal);
        assert_eq!(result, ParsedCommand::Motion(Motion::ScrollHalfPageDown, 1));
    }

    #[test]
    fn escape_resets() {
        let mut parser = KeyParser::new();
        parser.parse(&KeyEvent::char('d'), Mode::Normal); // start operator
        let result = parser.parse(&KeyEvent::escape(), Mode::Normal);
        assert_eq!(result, ParsedCommand::Escape);
        // Parser should be reset, next key should parse fresh
        let result = parser.parse(&KeyEvent::char('w'), Mode::Normal);
        assert_eq!(result, ParsedCommand::Motion(Motion::WordForward, 1));
    }

    #[test]
    fn d2fw() {
        // d2f" style
        let mut parser = KeyParser::new();
        let r1 = parser.parse(&KeyEvent::char('d'), Mode::Normal);
        assert_eq!(r1, ParsedCommand::Incomplete);
        let r2 = parser.parse(&KeyEvent::char('2'), Mode::Normal);
        assert_eq!(r2, ParsedCommand::Incomplete);
        let r3 = parser.parse(&KeyEvent::char('f'), Mode::Normal);
        assert_eq!(r3, ParsedCommand::Incomplete);
        let r4 = parser.parse(&KeyEvent::char('"'), Mode::Normal);
        assert_eq!(
            r4,
            ParsedCommand::OperatorMotion(Operator::Delete, Motion::FindChar('"'), 2)
        );
    }
}
