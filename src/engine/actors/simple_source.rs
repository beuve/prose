use bimap::BiMap;
use serde_yaml::Value;

use crate::engine::actors::broadcast::Broadcast;
use crate::engine::actors::{AMActor, Actor, Source};
use crate::engine::scheduler::Scheduler;
use crate::parser::yaml_parser::Result;
use std::cmp::min;
use std::collections::LinkedList;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

use super::super::tokens::Token;

pub struct SimpleSource {
    pub code: u16,
    pub code_product: u16,
    pub speed: (u32, u32),
    pub time_of_last_execution: Option<u64>,
    pub total: u32,
    pub max_production: u32,
    pub client: AMActor,
    scheduler: Scheduler,
}

impl SimpleSource {
    pub fn new(
        code: u16,
        code_product: u16,
        speed: (u32, u32),
        max_production: u32,
        scheduler: Scheduler,
    ) -> Self {
        Self {
            code,
            code_product,
            max_production,
            speed,
            time_of_last_execution: None,
            total: 0u32,
            client: Broadcast::new(code, code_product, scheduler.clone()),
            scheduler,
        }
    }
}

impl Source for SimpleSource {
    fn supply(&mut self, time: u64) -> bool {
        let quantity = min(self.max_production - self.total, self.speed.0);
        if quantity == 0 {
            return false;
        }
        self.total += quantity;
        let client = self.client.clone();
        let code_product = self.code_product;
        let tokens = LinkedList::from_iter(vec![Token::new(code_product, None); quantity as usize]);
        self.scheduler.schedule(time, move |time| {
            (client.lock().unwrap()).import(code_product, tokens, time)
        });
        self.time_of_last_execution = Some(time);
        true
    }

    fn delay(&self) -> u64 {
        self.speed.1 as u64
    }
}

impl Actor for SimpleSource {
    fn code(&self) -> u16 {
        self.code
    }

    fn total(&self) -> u64 {
        self.total as u64
    }

    fn tokens(&mut self) -> LinkedList<Token> {
        LinkedList::new()
    }

    fn parse(
        doc: &Value,
        code: u16,
        components: &BiMap<String, u16>,
        scheduler: Scheduler,
        _: f64,
    ) -> Result<AMActor>
    where
        Self: Sized,
    {
        let config = doc.get("config").expect("").as_mapping().expect("msg");
        let product = config.get("product").expect("msg").as_str().expect("msg");
        let code_product = components.get_by_left(product).expect("msg");
        let speed = {
            let speed_doc = config.get("speed").expect("msg");
            let time = speed_doc.get("time").expect("msg").as_u64().expect("msg") as u32;
            let quantity = speed_doc
                .get("quantity")
                .expect("msg")
                .as_u64()
                .expect("msg") as u32;
            (quantity, time)
        };
        let max_production = config
            .get("max_production")
            .expect("msg")
            .as_u64()
            .expect("msg") as u32;
        Ok(Arc::new(Mutex::new(SimpleSource::new(
            code,
            *code_product,
            speed,
            max_production,
            scheduler,
        ))))
    }

    fn as_source(&mut self) -> &mut dyn Source {
        self
    }

    fn import(&mut self, _: u16, _: LinkedList<Token>, _: u64) {
        panic!("A source should not be supplied")
    }

    fn register(&mut self, code: u16, code_product: u16, quantity: u32, actor: AMActor) {
        self.client
            .lock()
            .unwrap()
            .register(code, code_product, quantity, actor);
    }

    fn reset(&mut self) {
        self.total = 0;
    }

    fn report(&self, log_folder: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/logs.csv", log_folder))
            .unwrap();
        writeln!(file, "{};{};{{}}", self.code, self.total).unwrap();
    }
}
