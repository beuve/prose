use bimap::BiMap;
use serde_yaml::Value;

use crate::analyzer::Sampler;
use crate::engine::actors::broadcast::Broadcast;
use crate::engine::actors::{AMActor, Actor, Source};
use crate::engine::fifo::Fifo;
use crate::engine::scheduler::Scheduler;
use crate::parser::time_distribution_parser::TIME_CALLBACK;
use crate::parser::yaml_parser::Result;
use std::collections::LinkedList;
use std::sync::{Arc, Mutex};

use super::super::tokens::Token;

pub struct SimpleActor {
    pub code: u16,
    pub code_product: u16,
    pub import_fifo: Fifo,
    client: AMActor,
    scheduler: Scheduler,
    delay_sampler: Option<Sampler>,
    pub total: u64,
}

impl SimpleActor {
    pub fn new(
        code: u16,
        code_product: u16,
        scheduler: Scheduler,
        delay_sampler: Option<Sampler>,
    ) -> SimpleActor {
        let fifo: Fifo = Fifo::new(code, true);
        SimpleActor {
            code,
            code_product,
            import_fifo: fifo,
            client: Broadcast::new(code, code_product, scheduler.clone()),
            scheduler,
            delay_sampler,
            total: 0,
        }
    }

    pub fn check_requirements(&mut self) {
        if self.import_fifo.available_tokens() > 0 {
            let tokens = self.import_fifo.get_all();
            let client = self.client.clone();
            let code_product = self.code_product;
            let time = match &self.delay_sampler {
                Some(s) => s(),
                None => 0,
            };
            self.scheduler.schedule(time as u64, move |time| {
                (client.lock().unwrap()).import(code_product, tokens, time);
            });
        }
    }
}

impl Actor for SimpleActor {
    fn code(&self) -> u16 {
        self.code
    }

    fn total(&self) -> u64 {
        self.total
    }

    fn tokens(&mut self) -> LinkedList<Token> {
        self.import_fifo.tokens.split_off(0)
    }

    fn parse(
        doc: &Value,
        code: u16,
        components: &BiMap<String, u16>,
        scheduler: Scheduler,
        dt: f64,
    ) -> Result<AMActor>
    where
        Self: Sized,
    {
        let config = doc.get("config").expect("").as_mapping().expect("msg");
        let product = config.get("product").expect("msg").as_str().expect("msg");
        let code_product = components.get_by_left(product).expect("msg");
        let log_config = config.get("log");
        let delay_sampler = if log_config.is_some_and(|l| l.is_null()) {
            None
        } else {
            log_config.map(|logs| {
                let logs = logs.as_mapping().expect("msg");
                assert!(logs.len() == 1);
                let (distribution_name, distribution_config) = logs.iter().last().expect("");
                let distribution_name = distribution_name.as_str().expect("msg");
                let delay_callback = TIME_CALLBACK.lock().unwrap()[distribution_name];
                delay_callback(distribution_config, dt).expect("msg")
            })
        };
        Ok(Arc::new(Mutex::new(SimpleActor::new(
            code,
            *code_product,
            scheduler,
            delay_sampler,
        ))))
    }

    fn as_source(&mut self) -> &mut dyn Source {
        panic!("SimpleActor is not a source");
    }

    fn import(&mut self, _: u16, tokens: LinkedList<Token>, time: u64) {
        self.total += tokens.len() as u64;
        self.import_fifo.put(tokens, time);
        self.check_requirements();
    }

    fn register(&mut self, code: u16, code_product: u16, quantity: u32, actor: AMActor) {
        self.client
            .lock()
            .unwrap()
            .register(code, code_product, quantity, actor);
    }

    fn reset(&mut self) {
        self.import_fifo.reset();
    }

    fn report(&self, _: &str) {
        println!("{} : {}", self.code(), self.import_fifo.available_tokens());
    }
}
