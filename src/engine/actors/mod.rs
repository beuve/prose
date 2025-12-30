use bimap::BiMap;
use serde_yaml::Value;

use crate::engine::scheduler::Scheduler;
use crate::parser::yaml_parser::Result;
use std::collections::LinkedList;
use std::sync::{Arc, Mutex};

use super::tokens::Token;

#[allow(unused)] // Used only in docstring
use crate::engine::fifo::Fifo;

/// Associates a product code with a supply quantity
pub struct SupplyOffer(pub String, pub u32);
pub type AMActor = Arc<Mutex<dyn Actor + Send + Sync + 'static>>;
pub type AMSource = Arc<Mutex<dyn Source + Send + Sync + 'static>>;

/// [Actors][Actor] are nodes in a components flow graph. They produce new [Tokens][Token]
/// from [Tokens][Token] stored in their storage, represented by [Fifos][Fifo].
pub trait Actor {
    fn parse(
        doc: &Value,
        code: u16,
        components: &BiMap<String, u16>,
        scheduler: Scheduler,
        dt: f64,
    ) -> Result<AMActor>
    where
        Self: Sized;
    /// Import tokens
    fn import(&mut self, code_product: u16, tokens: LinkedList<Token>, time: u64);

    /// Register a client callback for the specified product
    fn register(&mut self, code: u16, code_product: u16, quantity: u32, actor: AMActor);

    /// Resets the actor for a new run.
    fn reset(&mut self);

    /// Prints a repport at the end of the simulation.
    fn report(&self, log_folder: &str);

    fn code(&self) -> u16;

    fn total(&self) -> u64;

    fn as_source(&mut self) -> &mut dyn Source;

    fn tokens(&mut self) -> LinkedList<Token>;
}

pub trait Source: Actor {
    fn supply(&mut self, time: u64) -> bool;

    fn delay(&self) -> u64;
}

mod broadcast;
mod simple_actor;
mod simple_sink;
mod simple_source;

pub use simple_actor::SimpleActor;
pub use simple_sink::SimpleSink;
pub use simple_source::SimpleSource;
