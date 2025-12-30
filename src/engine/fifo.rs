use std::collections::LinkedList;

use super::tokens::Token;

#[derive(Clone)]
pub struct Fifo {
    pub code: u16,
    pub tokens: LinkedList<Token>,
    pub log: bool,
}

impl Fifo {
    pub fn new(code: u16, log: bool) -> Fifo {
        Fifo {
            code,
            tokens: LinkedList::new(),
            log,
        }
    }

    pub fn available_tokens(&self) -> u32 {
        self.tokens.len() as u32
    }

    pub fn put(&mut self, mut new_tokens: LinkedList<Token>, time: u64) {
        if new_tokens.is_empty() {
            return;
        }
        if self.log {
            for t in new_tokens.iter_mut() {
                t.age(self.code, time)
            }
        }
        new_tokens.append(&mut self.tokens);
        self.tokens = new_tokens;
    }

    pub fn get(&mut self, quantity: u32) -> LinkedList<Token> {
        if quantity == 0 {
            return LinkedList::new();
        }
        self.tokens
            .split_off(self.tokens.len() - (quantity as usize))
    }

    pub fn get_all(&mut self) -> LinkedList<Token> {
        self.tokens.split_off(0)
    }

    pub fn reset(&mut self) {
        self.tokens = LinkedList::new();
    }
}
