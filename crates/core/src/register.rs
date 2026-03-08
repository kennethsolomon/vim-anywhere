use crate::buffer::YankStyle;

#[derive(Debug, Clone)]
pub struct RegisterEntry {
    pub content: String,
    pub style: YankStyle,
}

pub struct RegisterManager {
    unnamed: Option<RegisterEntry>,
    last_find: Option<FindRecord>,
    last_search: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FindRecord {
    pub char: char,
    pub forward: bool,
    pub til: bool,
}

impl RegisterManager {
    pub fn new() -> Self {
        Self {
            unnamed: None,
            last_find: None,
            last_search: None,
        }
    }

    pub fn yank(&mut self, content: String, style: YankStyle) {
        self.unnamed = Some(RegisterEntry { content, style });
    }

    pub fn get_unnamed(&self) -> Option<&RegisterEntry> {
        self.unnamed.as_ref()
    }

    pub fn set_last_find(&mut self, record: FindRecord) {
        self.last_find = Some(record);
    }

    pub fn get_last_find(&self) -> Option<&FindRecord> {
        self.last_find.as_ref()
    }

    pub fn set_last_search(&mut self, pattern: String) {
        self.last_search = Some(pattern);
    }

    pub fn get_last_search(&self) -> Option<&str> {
        self.last_search.as_deref()
    }
}

impl Default for RegisterManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yank_and_retrieve() {
        let mut rm = RegisterManager::new();
        assert!(rm.get_unnamed().is_none());

        rm.yank("hello".to_string(), YankStyle::Characterwise);
        let entry = rm.get_unnamed().unwrap();
        assert_eq!(entry.content, "hello");
        assert_eq!(entry.style, YankStyle::Characterwise);
    }

    #[test]
    fn yank_overwrites() {
        let mut rm = RegisterManager::new();
        rm.yank("first".to_string(), YankStyle::Characterwise);
        rm.yank("second".to_string(), YankStyle::Linewise);
        let entry = rm.get_unnamed().unwrap();
        assert_eq!(entry.content, "second");
        assert_eq!(entry.style, YankStyle::Linewise);
    }

    #[test]
    fn last_find() {
        let mut rm = RegisterManager::new();
        assert!(rm.get_last_find().is_none());
        rm.set_last_find(FindRecord {
            char: 'x',
            forward: true,
            til: false,
        });
        let f = rm.get_last_find().unwrap();
        assert_eq!(f.char, 'x');
        assert!(f.forward);
    }

    #[test]
    fn last_search() {
        let mut rm = RegisterManager::new();
        assert!(rm.get_last_search().is_none());
        rm.set_last_search("pattern".to_string());
        assert_eq!(rm.get_last_search(), Some("pattern"));
    }
}
