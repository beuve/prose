use std::collections::{HashMap, LinkedList};
use std::vec::Vec;

#[derive(Debug, Clone)]
pub struct Token {
    pub code: String,
    pub timeline: LinkedList<String>,
    pub parts: HashMap<String, Vec<Token>>,
}

impl Token {
    pub fn new(code: String, timeline: Option<LinkedList<String>>) -> Token {
        Token {
            code,
            timeline: timeline.unwrap_or(LinkedList::new()),
            parts: HashMap::new(),
        }
    }

    pub fn age(&mut self, code: String) {
        self.timeline.push_back(code);
    }
}
