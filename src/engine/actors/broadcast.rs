use bimap::BiMap;
use serde_yaml::Value;

use crate::engine::actors::{AMActor, Actor, Source};
use crate::engine::fifo::Fifo;
use crate::engine::scheduler::Scheduler;
use crate::parser::yaml_parser::Result;
use crate::utils::distributions::CyclicSampler;
use std::cmp::max;
use std::collections::{HashMap, LinkedList};
use std::sync::{Arc, Mutex};

use super::super::tokens::Token;

pub struct Broadcast {
    pub code: u16,
    pub code_product: u16,
    pub import_fifo: Fifo,
    clients: HashMap<u16, (u32, AMActor)>,
    quantity: u32,
    scheduler: Scheduler,
    output_sampler: CyclicSampler<u16>,
}

impl Broadcast {
    pub fn new(code: u16, code_product: u16, scheduler: Scheduler) -> Arc<Mutex<Broadcast>> {
        Arc::new(Mutex::new(Self {
            code,
            code_product,
            import_fifo: Fifo::new(code, false),
            clients: HashMap::new(),
            quantity: 0,
            scheduler,
            output_sampler: CyclicSampler::new(),
        }))
    }

    pub fn create_rolling_sequence(&mut self) {
        let actors: Vec<u16> = self.clients.keys().copied().collect();
        let mut counts = vec![0; actors.len()];
        let total = self.quantity as i32;
        let mut sequence = vec![];
        for _ in 0..total {
            let (index, _) = actors
                .iter()
                .enumerate()
                .map(|(index, code)| {
                    let probability = self.clients.get(code).unwrap().0 as i32;
                    let sequence_length = max(1, sequence.len()) as i32;
                    let count = counts[index];
                    (index, probability * sequence_length - total * count)
                })
                .max_by(|(_, a), (_, b)| a.cmp(b))
                .unwrap();
            sequence.push(actors[index]);
            counts[index] += 1;
        }

        self.output_sampler.set_samples(sequence);
    }

    pub fn check_requirements(&mut self) {
        if self.import_fifo.available_tokens() == 0 {
            return;
        }
        if self.clients.len() == 1 {
            let key = self.clients.keys().next().unwrap();
            let (_, client) = self.clients.get(key).unwrap().clone();
            let tokens = self.import_fifo.get_all();
            let code_product = self.code_product;
            self.scheduler.schedule(0, move |time| {
                client.lock().unwrap().import(code_product, tokens, time)
            });
            return;
        }
        let tokens_per_output = self
            .output_sampler
            .freq(self.import_fifo.available_tokens() as usize);

        for (code, (_, a)) in self.clients.iter() {
            if !tokens_per_output.contains_key(code) {
                continue;
            }
            let a = a.clone();

            let tokens = self
                .import_fifo
                .get(*tokens_per_output.get(code).unwrap() as u32);
            let code_product = self.code_product;
            self.scheduler.schedule(0, move |time| {
                a.lock().unwrap().import(code_product, tokens, time)
            });
        }
    }
}

impl Actor for Broadcast {
    fn code(&self) -> u16 {
        self.code
    }

    fn total(&self) -> u64 {
        self.import_fifo.available_tokens() as u64
    }

    fn tokens(&mut self) -> LinkedList<Token> {
        self.import_fifo.tokens.split_off(0)
    }

    fn parse(_: &Value, _: u16, _: &BiMap<String, u16>, _: Scheduler, _: f64) -> Result<AMActor>
    where
        Self: Sized,
    {
        panic!("Broadcast should not be called directly by users");
    }

    fn as_source(&mut self) -> &mut dyn Source {
        panic!("SimpleActor is not a source");
    }

    fn import(&mut self, _: u16, tokens: LinkedList<Token>, time: u64) {
        self.import_fifo.put(tokens, time);
        self.check_requirements();
    }

    fn register(&mut self, code: u16, _: u16, quantity: u32, actor: AMActor) {
        let old = self.clients.insert(code, (quantity, actor));
        if let Some((q, _)) = old {
            self.quantity -= q;
        }
        self.quantity += quantity;
        if self.clients.len() > 1 {
            self.create_rolling_sequence();
        }
    }

    fn reset(&mut self) {
        self.import_fifo.reset();
    }

    fn report(&self, _: &str) {}
}
