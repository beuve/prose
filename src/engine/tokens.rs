use std::collections::{HashMap, LinkedList};
use std::vec::Vec;

#[derive(Debug, Clone)]
pub struct Token {
    pub code: String,
    pub timeline: LinkedList<String>,
    pub parts: HashMap<String, Vec<Token>>,
    pub token_time: u64,
}

impl Token {
    pub fn new(code: String, time: u64) -> Token {
        Token {
            code,
            timeline: LinkedList::new(),
            parts: HashMap::new(),
            token_time: time,
        }
    }

    pub fn age(&mut self, code: String) {
        self.timeline.push_back(code);
    }
}
