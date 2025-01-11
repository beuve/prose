use threadpool::ThreadPool;
use yaml_rust2::Yaml;

use crate::engine::fifo::Fifo;
use crate::parser::yaml_parser::ParseError::UnknownComponent;
use crate::parser::yaml_parser::{Result, YamlParser};
use std::cmp::{max, min};
use std::collections::{HashMap, LinkedList};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::u32;

use super::tokens::Token;

/// Associates a product code with a supply quantity
pub struct SupplyOffer(pub String, pub u32);
pub type AMActor = Arc<Mutex<dyn Actor + Send + Sync + 'static>>;
pub type AMSource = Arc<Mutex<dyn Source + Send + Sync + 'static>>;

/// [Actors][Actor] are nodes in a components flow graph. They produce new [Tokens][Token]
/// from [Tokens][Token] stored in their storage, represented by [Fifos][Fifo].
pub trait Actor {
    fn parse(
        doc: &Yaml,
        code: u16,
        components: HashMap<String, u16>,
        pool: ThreadPool,
    ) -> Result<AMActor>
    where
        Self: Sized;
    /// Import tokens
    fn import(&mut self, code_product: u16, tokens: LinkedList<Token>);

    /// Register a client callback for the specified product
    fn register(&mut self, code: u16, code_product: u16, quantity: u32, actor: AMActor);

    /// Resets the actor for a new run.
    fn reset(&mut self);

    /// Prints a repport at the end of the simulation.
    fn report(&self, log_folder: &String);

    fn code(&self) -> u16;

    fn total(&self) -> u64;

    fn as_source(&mut self) -> &mut dyn Source;

    fn tokens(&mut self) -> LinkedList<Token>;
}

pub trait Source: Actor {
    fn supply(&mut self) -> bool;
}

pub struct SimpleActor {
    pub code: u16,
    pub code_product: u16,
    pub import_fifo: Fifo,
    client: AMActor,
    pool: ThreadPool,
    pub total: u64,
}

impl SimpleActor {
    pub fn new(code: u16, code_product: u16, pool: ThreadPool) -> SimpleActor {
        let fifo: Fifo = Fifo::new(code, true);
        SimpleActor {
            code: code,
            code_product: code_product,
            import_fifo: fifo,
            client: Broadcast::new(code, code_product, pool.clone()),
            pool,
            total: 0,
        }
    }

    pub fn check_requirements(&mut self) {
        if self.import_fifo.available_tokens() > 0 {
            let tokens = self.import_fifo.get_all();
            let code_product = self.code_product;
            let client = self.client.clone();
            self.pool.execute(move || {
                (client.lock().unwrap()).import(code_product, tokens);
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
        doc: &Yaml,
        code: u16,
        components: HashMap<String, u16>,
        pool: ThreadPool,
    ) -> Result<AMActor>
    where
        Self: Sized,
    {
        let component = doc.get("component")?.str()?;
        let code_product = components
            .get(component)
            .ok_or_else(|| UnknownComponent(String::from(component)))?;
        return Ok(Arc::new(Mutex::new(SimpleActor::new(
            code,
            code_product.clone(),
            pool,
        ))));
    }

    fn as_source(&mut self) -> &mut dyn Source {
        panic!("SimpleActor is not a source");
    }

    fn import(&mut self, _: u16, tokens: LinkedList<Token>) {
        self.total += tokens.len() as u64;
        self.import_fifo.put(tokens);
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

    fn report(&self, _: &String) {}
}

pub struct SimpleSource {
    pub code: u16,
    pub code_product: u16,
    pub speed: (u32, u32),
    pub num_executions: u16,
    pub total: u32,
    pub max_production: u32,
    pub client: AMActor,
    pool: ThreadPool,
}

impl SimpleSource {
    pub fn new(
        code: u16,
        code_product: u16,
        speed: (u32, u32),
        max_production: u32,
        pool: ThreadPool,
    ) -> Self {
        Self {
            code,
            code_product,
            max_production,
            speed,
            num_executions: 0,
            total: 0u32,
            client: Broadcast::new(code, code_product, pool.clone()),
            pool,
        }
    }
}

impl Source for SimpleSource {
    fn supply(&mut self) -> bool {
        let quantity = min(self.max_production - self.total, self.speed.0);
        if quantity == 0 {
            return false;
        }
        self.total += quantity;
        let client = self.client.clone();
        let code_product = self.code_product;
        let num_executions = self.num_executions;
        self.pool.execute(move || {
            (client.lock().unwrap()).import(
                code_product,
                LinkedList::from_iter(
                    vec![Token::new(code_product, Some(num_executions)); quantity as usize]
                        .into_iter(),
                ),
            )
        });
        self.num_executions += 1;
        return true;
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
        doc: &Yaml,
        code: u16,
        components: HashMap<String, u16>,
        pool: ThreadPool,
    ) -> Result<AMActor>
    where
        Self: Sized,
    {
        let component = doc.get("component")?.str()?;
        let code_product = components
            .get(component)
            .ok_or_else(|| UnknownComponent(String::from(component)))?;
        let speed = {
            let speed_doc = doc.get("speed")?;
            let time = speed_doc.get("time")?.int()? as u32;
            let quantity = speed_doc.get("quantity")?.int()? as u32;
            (quantity, time)
        };
        let max_production = doc.get("max_production")?.int()? as u32;
        return Ok(Arc::new(Mutex::new(SimpleSource::new(
            code,
            code_product.clone(),
            speed,
            max_production,
            pool,
        ))));
    }

    fn as_source(&mut self) -> &mut dyn Source {
        return self;
    }

    fn import(&mut self, _: u16, _: LinkedList<Token>) {
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
        self.num_executions = 0;
    }

    fn report(&self, log_folder: &String) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(format!("{}/logs.csv", log_folder))
            .unwrap();
        writeln!(file, "{};{};{{}}", self.code, self.total).unwrap();
    }
}

pub struct SimpleSink {
    pub code: u16,
    pub code_product: u16,
    pub import_fifo: Fifo,
}

impl SimpleSink {
    pub fn new(code: u16, code_product: u16) -> Self {
        Self {
            code: code,
            code_product: code_product,
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

    fn parse(
        doc: &Yaml,
        code: u16,
        components: HashMap<String, u16>,
        _: ThreadPool,
    ) -> Result<AMActor>
    where
        Self: Sized,
    {
        let component = doc.get("component")?.str()?;
        let code_product = components
            .get(component)
            .ok_or_else(|| UnknownComponent(String::from(component)))?;
        return Ok(Arc::new(Mutex::new(SimpleSink::new(
            code,
            code_product.clone(),
        ))));
    }

    fn as_source(&mut self) -> &mut dyn Source {
        panic!("SimpleActor is not a source");
    }

    fn import(&mut self, _: u16, tokens: LinkedList<Token>) {
        self.import_fifo.put(tokens);
    }

    fn register(&mut self, _: u16, _: u16, _: u32, _: AMActor) {
        panic!("Sink have no output");
    }

    fn reset(&mut self) {
        self.import_fifo.reset();
    }

    fn report(&self, log_folder: &String) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(format!("{}/logs.csv", log_folder))
            .unwrap();
        writeln!(file, "{};{}", self.code, self.import_fifo.tokens.len()).unwrap();
    }
}

pub struct Broadcast {
    pub code: u16,
    pub code_product: u16,
    pub import_fifo: Fifo,
    clients: HashMap<u16, (u32, AMActor)>,
    quantity: u32,
    pool: ThreadPool,
    rolling_sequence: Vec<u16>,
    rolling_index: usize,
}

impl Broadcast {
    pub fn new(code: u16, code_product: u16, pool: ThreadPool) -> Arc<Mutex<Broadcast>> {
        Arc::new(Mutex::new(Self {
            code: code,
            code_product: code_product,
            import_fifo: Fifo::new(code, false),
            clients: HashMap::new(),
            quantity: 0,
            pool,
            rolling_sequence: vec![],
            rolling_index: 0,
        }))
    }

    pub fn create_rolling_sequence(&mut self) {
        let actors: Vec<u16> = self.clients.keys().map(|c| c.clone()).collect();
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
            sequence.push(actors[index].clone());
            counts[index] += 1;
        }
        self.rolling_sequence = sequence;
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
            self.pool
                .execute(move || client.lock().unwrap().import(code_product, tokens));
            return;
        }
        let num_full_activations =
            self.import_fifo.available_tokens() / self.rolling_sequence.len() as u32;
        let remaining_tokens =
            self.import_fifo.available_tokens() % self.rolling_sequence.len() as u32;

        for (code, (q, a)) in self.clients.iter() {
            let a = a.clone();
            let remaining_number = self
                .rolling_sequence
                .iter()
                .cycle()
                .skip(self.rolling_index)
                .take(remaining_tokens as usize)
                .filter(|c| c == &code)
                .collect::<Vec<&u16>>()
                .len() as u32;

            let tokens = self
                .import_fifo
                .get(q * num_full_activations + remaining_number);
            let code_product = self.code_product;
            self.pool
                .execute(move || a.lock().unwrap().import(code_product, tokens));
        }
        self.rolling_index =
            (self.rolling_index + remaining_tokens as usize) % self.rolling_sequence.len();
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

    fn parse(_: &Yaml, _: u16, _: HashMap<String, u16>, _: ThreadPool) -> Result<AMActor>
    where
        Self: Sized,
    {
        panic!("Broadcast should not be called directly by users");
    }

    fn as_source(&mut self) -> &mut dyn Source {
        panic!("SimpleActor is not a source");
    }

    fn import(&mut self, _: u16, tokens: LinkedList<Token>) {
        self.import_fifo.put(tokens);
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

    fn report(&self, _: &String) {}
}
