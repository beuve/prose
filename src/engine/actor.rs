use threadpool::ThreadPool;

use crate::engine::fifo::Fifo;
use std::cmp::{max, min};
use std::collections::{HashMap, LinkedList};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

use super::tokens::Token;

/// Associates a product code with a supply quantity
pub struct SupplyOffer(pub String, pub u32);
pub type AMActor = Arc<Mutex<dyn Actor + Send + Sync + 'static>>;

/// [Actors][Actor] are nodes in a components flow graph. They produce new [Tokens][Token]
/// from [Tokens][Token] stored in their storage, represented by [Fifos][Fifo].
pub trait Actor {
    /// Import tokens
    fn import(&mut self, code_product: &String, tokens: LinkedList<Token>);

    /// Register a client callback for the specified product
    fn register(&mut self, code: String, code_product: String, quantity: u32, actor: AMActor);

    /// Resets the actor for a new run.
    fn reset(&mut self);

    /// Prints a repport at the end of the simulation.
    fn report(&self, log_folder: &String);
}

pub struct SimpleActor {
    pub code: String,
    pub code_product: String,
    import_fifo: Fifo,
    client: Option<AMActor>,
    pool: ThreadPool,
    pub total: u64,
}

impl SimpleActor {
    pub fn new(code: String, code_product: String, pool: ThreadPool) -> SimpleActor {
        Self {
            code: code.clone(),
            code_product: code_product.clone(),
            import_fifo: Fifo::new(code.clone() + "/" + code_product.as_str(), true),
            client: None,
            pool,
            total: 0,
        }
    }

    pub fn check_requirements(&mut self) {
        if let Some(client) = self.client.as_ref() {
            if self.import_fifo.available_tokens() > 0 {
                let tokens = self.import_fifo.get_all();
                let code_product = self.code_product.clone();
                let client = client.clone();
                self.pool.execute(move || {
                    (client.lock().unwrap()).import(&code_product, tokens);
                });
            }
        }
    }
}

impl Actor for SimpleActor {
    fn import(&mut self, _: &String, tokens: LinkedList<Token>) {
        self.total += tokens.len() as u64;
        self.import_fifo.put(tokens);
        self.check_requirements();
    }

    fn register(&mut self, _: String, _: String, _: u32, actor: AMActor) {
        self.client = Some(actor);
    }

    fn reset(&mut self) {
        self.import_fifo.reset();
    }

    fn report(&self, _: &String) {}
}

pub struct SimpleSource {
    pub code: String,
    pub code_product: String,
    pub speed: (u32, u32),
    pub num_executions: usize,
    pub total: u32,
    pub max_production: u32,
    pub client: Option<AMActor>,
    pool: ThreadPool,
}

impl SimpleSource {
    pub fn new(
        code: String,
        code_product: String,
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
            client: None,
            pool,
        }
    }

    pub fn supply(&mut self, source: Arc<Mutex<SimpleSource>>) {
        let quantity = min(self.max_production - self.total, self.speed.0);
        if quantity == 0 {
            return;
        }
        if let Some(client) = self.client.as_ref() {
            self.total += quantity;
            let client = client.clone();
            let code_product = self.code_product.clone();
            self.pool.execute(move || {
                (client.lock().unwrap()).import(
                    &code_product,
                    LinkedList::from_iter(
                        vec![Token::new(code_product.clone(), None); quantity as usize].into_iter(),
                    ),
                )
            });
            self.pool
                .execute(move || source.lock().unwrap().supply(source.clone()));
            self.num_executions += 1;
        }
    }
}

impl Actor for SimpleSource {
    fn import(&mut self, _: &String, _: LinkedList<Token>) {
        panic!("A source should not be supplied")
    }

    fn register(&mut self, _: String, _: String, _: u32, actor: AMActor) {
        self.client = Some(actor);
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
    pub code: String,
    pub code_product: String,
    pub import_fifo: Fifo,
}

impl SimpleSink {
    pub fn new(code: String, code_product: String) -> Self {
        Self {
            code: code.clone(),
            code_product: code_product.clone(),
            import_fifo: Fifo::new(code + "/" + code_product.as_str(), true),
        }
    }
}

impl Actor for SimpleSink {
    fn import(&mut self, _: &String, tokens: LinkedList<Token>) {
        self.import_fifo.put(tokens);
    }

    fn register(&mut self, _: String, _: String, _: u32, _: AMActor) {
        panic!("Sink have no outputs");
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
    pub code: String,
    pub code_product: String,
    pub import_fifo: Fifo,
    clients: HashMap<String, (u32, AMActor)>,
    quantity: u32,
    pool: ThreadPool,
    rolling_sequence: Vec<String>,
    rolling_index: usize,
}

impl Broadcast {
    pub fn new(code: String, code_product: String, pool: ThreadPool) -> Broadcast {
        Self {
            code: code.clone(),
            code_product: code_product.clone(),
            import_fifo: Fifo::new(code.clone() + "/" + code_product.as_str(), false),
            clients: HashMap::new(),
            quantity: 0,
            pool,
            rolling_sequence: vec![],
            rolling_index: 0,
        }
    }

    pub fn create_rolling_sequence(&mut self) {
        let actors: Vec<String> = self
            .clients
            .keys()
            .map(|c| c.clone())
            .collect::<Vec<String>>();
        let mut counts = vec![0; actors.len()];
        let total = 100;
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
                .collect::<Vec<&String>>()
                .len() as u32;

            let tokens = self
                .import_fifo
                .get(q * num_full_activations + remaining_number);
            let code_product = self.code_product.clone();
            self.pool
                .execute(move || a.lock().unwrap().import(&code_product, tokens));
        }
        self.rolling_index =
            (self.rolling_index + remaining_tokens as usize) % self.rolling_sequence.len();
    }
}

impl Actor for Broadcast {
    fn import(&mut self, _: &String, tokens: LinkedList<Token>) {
        self.import_fifo.put(tokens);
        self.check_requirements();
    }

    fn register(&mut self, code: String, _: String, quantity: u32, actor: AMActor) {
        let old = self.clients.insert(code, (quantity, actor));
        if let Some((q, _)) = old {
            self.quantity -= q;
        }
        self.quantity += quantity;
        self.create_rolling_sequence();
    }

    fn reset(&mut self) {
        self.import_fifo.reset();
    }

    fn report(&self, _: &String) {}
}
