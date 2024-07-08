use std::collections::LinkedList;

use super::tokens::Token;

#[derive(Clone)]
pub struct Fifo {
    pub code: String,
    pub tokens: LinkedList<Token>,
    pub log: bool,
}

impl Fifo {
    pub fn new(code: String, log: bool) -> Fifo {
        Fifo {
            code,
            tokens: LinkedList::new(),
            log,
        }
    }

    pub fn available_tokens(&self) -> u32 {
        self.tokens.len() as u32
    }

    pub fn put(&mut self, mut new_tokens: LinkedList<Token>) {
        new_tokens.append(&mut self.tokens);
        (*self).tokens = new_tokens;
    }

    pub fn get(&mut self, quantity: u32) -> LinkedList<Token> {
        if quantity == 0 {
            return LinkedList::new();
        }
        let mut sent_tokens = self
            .tokens
            .split_off(self.tokens.len() - (quantity as usize));
        if self.log {
            for t in sent_tokens.iter_mut() {
                t.age(self.code.clone())
            }
        }
        return sent_tokens;
    }

    pub fn get_all(&mut self) -> LinkedList<Token> {
        let mut sent_tokens = self.tokens.split_off(0);
        if self.log {
            for t in sent_tokens.iter_mut() {
                t.age(self.code.clone())
            }
        }
        return sent_tokens;
    }

    pub fn reset(&mut self) {
        self.tokens = LinkedList::new();
    }
}
