use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    VisualCharacterwise,
    VisualLinewise,
}

#[derive(Debug, Clone)]
pub struct ModeEntryConfig {
    pub escape_key: bool,
    pub double_escape_sends_real: bool,
    pub custom_sequence: Option<[char; 2]>,
    pub control_bracket: bool,
    pub double_escape_timeout_ms: u64,
    pub sequence_timeout_ms: u64,
}

impl Default for ModeEntryConfig {
    fn default() -> Self {
        Self {
            escape_key: true,
            double_escape_sends_real: true,
            custom_sequence: None,
            control_bracket: true,
            double_escape_timeout_ms: 300,
            sequence_timeout_ms: 200,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeTransition {
    None,
    To(Mode),
    SendEscape,
}

pub struct ModeStateMachine {
    current: Mode,
    config: ModeEntryConfig,
    last_escape_time: Option<Instant>,
    pending_sequence_char: Option<(char, Instant)>,
}

impl ModeStateMachine {
    pub fn new(config: ModeEntryConfig) -> Self {
        Self {
            current: Mode::Normal,
            config,
            last_escape_time: None,
            pending_sequence_char: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.current
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.current = mode;
    }

    pub fn pending_sequence_char(&self) -> Option<char> {
        self.pending_sequence_char.map(|(c, _)| c)
    }

    pub fn handle_escape(&mut self) -> ModeTransition {
        match self.current {
            Mode::Insert => {
                if self.config.double_escape_sends_real {
                    if let Some(last) = self.last_escape_time {
                        if last.elapsed().as_millis() < self.config.double_escape_timeout_ms as u128
                        {
                            self.last_escape_time = None;
                            return ModeTransition::SendEscape;
                        }
                    }
                    self.last_escape_time = Some(Instant::now());
                }
                self.current = Mode::Normal;
                ModeTransition::To(Mode::Normal)
            }
            Mode::VisualCharacterwise | Mode::VisualLinewise => {
                self.current = Mode::Normal;
                ModeTransition::To(Mode::Normal)
            }
            Mode::Normal => {
                if self.config.double_escape_sends_real {
                    if let Some(last) = self.last_escape_time {
                        if last.elapsed().as_millis() < self.config.double_escape_timeout_ms as u128
                        {
                            self.last_escape_time = None;
                            return ModeTransition::SendEscape;
                        }
                    }
                    self.last_escape_time = Some(Instant::now());
                }
                ModeTransition::None
            }
        }
    }

    pub fn handle_control_bracket(&mut self) -> ModeTransition {
        if !self.config.control_bracket {
            return ModeTransition::None;
        }
        self.handle_escape()
    }

    pub fn handle_insert_char(&mut self, ch: char) -> ModeTransition {
        if self.current != Mode::Insert {
            return ModeTransition::None;
        }

        let seq = match self.config.custom_sequence {
            Some(s) => s,
            None => return ModeTransition::None,
        };

        if let Some((first_char, time)) = self.pending_sequence_char {
            self.pending_sequence_char = None;
            if first_char == seq[0]
                && ch == seq[1]
                && time.elapsed().as_millis() < self.config.sequence_timeout_ms as u128
            {
                self.current = Mode::Normal;
                return ModeTransition::To(Mode::Normal);
            }
        }

        if ch == seq[0] {
            self.pending_sequence_char = Some((ch, Instant::now()));
        }

        ModeTransition::None
    }

    pub fn enter_insert(&mut self, _variant: InsertVariant) -> ModeTransition {
        if self.current == Mode::Normal {
            self.current = Mode::Insert;
            ModeTransition::To(Mode::Insert)
        } else {
            ModeTransition::None
        }
    }

    pub fn enter_visual_characterwise(&mut self) -> ModeTransition {
        match self.current {
            Mode::Normal => {
                self.current = Mode::VisualCharacterwise;
                ModeTransition::To(Mode::VisualCharacterwise)
            }
            Mode::VisualCharacterwise => {
                self.current = Mode::Normal;
                ModeTransition::To(Mode::Normal)
            }
            Mode::VisualLinewise => {
                self.current = Mode::VisualCharacterwise;
                ModeTransition::To(Mode::VisualCharacterwise)
            }
            Mode::Insert => ModeTransition::None,
        }
    }

    pub fn enter_visual_linewise(&mut self) -> ModeTransition {
        match self.current {
            Mode::Normal => {
                self.current = Mode::VisualLinewise;
                ModeTransition::To(Mode::VisualLinewise)
            }
            Mode::VisualLinewise => {
                self.current = Mode::Normal;
                ModeTransition::To(Mode::Normal)
            }
            Mode::VisualCharacterwise => {
                self.current = Mode::VisualLinewise;
                ModeTransition::To(Mode::VisualLinewise)
            }
            Mode::Insert => ModeTransition::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertVariant {
    I,
    A,
    O,
    BigI,
    BigA,
    BigO,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sm() -> ModeStateMachine {
        ModeStateMachine::new(ModeEntryConfig::default())
    }

    fn make_sm_with_sequence(seq: [char; 2]) -> ModeStateMachine {
        ModeStateMachine::new(ModeEntryConfig {
            custom_sequence: Some(seq),
            ..Default::default()
        })
    }

    #[test]
    fn starts_in_normal_mode() {
        let sm = make_sm();
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn normal_to_insert_via_i() {
        let mut sm = make_sm();
        let t = sm.enter_insert(InsertVariant::I);
        assert_eq!(t, ModeTransition::To(Mode::Insert));
        assert_eq!(sm.mode(), Mode::Insert);
    }

    #[test]
    fn insert_to_normal_via_escape() {
        let mut sm = make_sm();
        sm.enter_insert(InsertVariant::I);
        let t = sm.handle_escape();
        assert_eq!(t, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn normal_to_visual_characterwise() {
        let mut sm = make_sm();
        let t = sm.enter_visual_characterwise();
        assert_eq!(t, ModeTransition::To(Mode::VisualCharacterwise));
        assert_eq!(sm.mode(), Mode::VisualCharacterwise);
    }

    #[test]
    fn normal_to_visual_linewise() {
        let mut sm = make_sm();
        let t = sm.enter_visual_linewise();
        assert_eq!(t, ModeTransition::To(Mode::VisualLinewise));
        assert_eq!(sm.mode(), Mode::VisualLinewise);
    }

    #[test]
    fn visual_characterwise_toggle_off() {
        let mut sm = make_sm();
        sm.enter_visual_characterwise();
        let t = sm.enter_visual_characterwise();
        assert_eq!(t, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn visual_linewise_toggle_off() {
        let mut sm = make_sm();
        sm.enter_visual_linewise();
        let t = sm.enter_visual_linewise();
        assert_eq!(t, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn visual_switch_characterwise_to_linewise() {
        let mut sm = make_sm();
        sm.enter_visual_characterwise();
        let t = sm.enter_visual_linewise();
        assert_eq!(t, ModeTransition::To(Mode::VisualLinewise));
        assert_eq!(sm.mode(), Mode::VisualLinewise);
    }

    #[test]
    fn visual_switch_linewise_to_characterwise() {
        let mut sm = make_sm();
        sm.enter_visual_linewise();
        let t = sm.enter_visual_characterwise();
        assert_eq!(t, ModeTransition::To(Mode::VisualCharacterwise));
        assert_eq!(sm.mode(), Mode::VisualCharacterwise);
    }

    #[test]
    fn visual_to_normal_via_escape() {
        let mut sm = make_sm();
        sm.enter_visual_characterwise();
        let t = sm.handle_escape();
        assert_eq!(t, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn control_bracket_exits_insert() {
        let mut sm = make_sm();
        sm.enter_insert(InsertVariant::I);
        let t = sm.handle_control_bracket();
        assert_eq!(t, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn custom_sequence_exits_insert() {
        let mut sm = make_sm_with_sequence(['j', 'k']);
        sm.enter_insert(InsertVariant::I);
        let t1 = sm.handle_insert_char('j');
        assert_eq!(t1, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Insert);
        let t2 = sm.handle_insert_char('k');
        assert_eq!(t2, ModeTransition::To(Mode::Normal));
        assert_eq!(sm.mode(), Mode::Normal);
    }

    #[test]
    fn custom_sequence_wrong_second_char() {
        let mut sm = make_sm_with_sequence(['j', 'k']);
        sm.enter_insert(InsertVariant::I);
        sm.handle_insert_char('j');
        let t = sm.handle_insert_char('j');
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::Insert);
    }

    #[test]
    fn enter_insert_ignored_when_not_normal() {
        let mut sm = make_sm();
        sm.enter_visual_characterwise();
        let t = sm.enter_insert(InsertVariant::I);
        assert_eq!(t, ModeTransition::None);
        assert_eq!(sm.mode(), Mode::VisualCharacterwise);
    }

    #[test]
    fn all_insert_variants() {
        for variant in [
            InsertVariant::I,
            InsertVariant::A,
            InsertVariant::O,
            InsertVariant::BigI,
            InsertVariant::BigA,
            InsertVariant::BigO,
        ] {
            let mut sm = make_sm();
            let t = sm.enter_insert(variant);
            assert_eq!(t, ModeTransition::To(Mode::Insert));
            assert_eq!(sm.mode(), Mode::Insert);
            sm.handle_escape();
        }
    }
}
