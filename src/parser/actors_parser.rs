use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use threadpool::ThreadPool;
use yaml_rust2::Yaml;

use crate::engine::actor::{AMActor, Actor, SimpleActor, SimpleSink, SimpleSource};

use super::yaml_parser::Result;
pub static ACTORS: LazyLock<
    Arc<
        Mutex<HashMap<String, fn(&Yaml, u16, HashMap<String, u16>, ThreadPool) -> Result<AMActor>>>,
    >,
> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn add_actor_implementation(
    label: String,
    callback: fn(&Yaml, u16, HashMap<String, u16>, ThreadPool) -> Result<AMActor>,
) {
    ACTORS.lock().unwrap().insert(label, callback);
}

pub fn import_default_actors() {
    add_actor_implementation(String::from("SimpleActor"), SimpleActor::parse);
    add_actor_implementation(String::from("SimpleSink"), SimpleSink::parse);
    add_actor_implementation(String::from("SimpleSource"), SimpleSource::parse);
}
