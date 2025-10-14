use std::collections::{HashMap, LinkedList};

#[derive(Debug, Clone)]
pub struct Token {
    pub code: u16,
    pub timeline: LinkedList<u16>,
    pub parts: HashMap<u16, LinkedList<Token>>,
}

impl Token {
    pub fn new(code: u16, init: Option<u16>) -> Token {
        let mut timeline = LinkedList::new();
        if let Some(code) = init {
            timeline.push_back(code);
        }
        Token {
            code,
            timeline,
            parts: HashMap::new(),
        }
    }

    pub fn add_part(&mut self, code: u16, mut tokens: LinkedList<Token>) {
        if let Some(l) = self.parts.get_mut(&code) {
            l.append(&mut tokens);
        } else {
            self.parts.insert(code, tokens);
        }
    }

    pub fn age(&mut self, code: u16) {
        self.timeline.push_back(code + self.code);
        for (_, tokens) in self.parts.iter_mut() {
            for t in tokens {
                t.age(code);
            }
        }
    }
}
