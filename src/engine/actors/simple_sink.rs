use bimap::BiMap;
use serde_yaml::Value;

use crate::engine::actors::{AMActor, Actor, Source};
use crate::engine::fifo::Fifo;
use crate::engine::scheduler::Scheduler;
use crate::parser::yaml_parser::Result;
use std::collections::LinkedList;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

use super::super::tokens::Token;

pub struct SimpleSink {
    pub code: u16,
    pub import_fifo: Fifo,
}

impl SimpleSink {
    pub fn new(code: u16) -> Self {
        Self {
            code,
            import_fifo: Fifo::new(code, true),
        }
    }
}

impl Actor for SimpleSink {
    fn code(&self) -> u16 {
        self.code
    }

    fn total(&self) -> u64 {
        self.import_fifo.available_tokens() as u64
    }

    fn tokens(&mut self) -> LinkedList<Token> {
        self.import_fifo.tokens.split_off(0)
    }

    fn parse(_: &Value, code: u16, _: &BiMap<String, u16>, _: Scheduler, _: f64) -> Result<AMActor>
    where
        Self: Sized,
    {
        Ok(Arc::new(Mutex::new(SimpleSink::new(code))))
    }

    fn as_source(&mut self) -> &mut dyn Source {
        panic!("SimpleActor is not a source");
    }

    fn import(&mut self, _: u16, tokens: LinkedList<Token>, time: u64) {
        self.import_fifo.put(tokens, time);
    }

    fn register(&mut self, _: u16, _: u16, _: u32, _: AMActor) {
        panic!("Sink have no output");
    }

    fn reset(&mut self) {
        self.import_fifo.reset();
    }

    fn report(&self, log_folder: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/logs.csv", log_folder))
            .unwrap();
        writeln!(file, "{};{}", self.code, self.import_fifo.tokens.len()).unwrap();
    }
}
