use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use bimap::BiMap;
use serde_yaml::Value;

use crate::engine::{
    actors::{AMActor, Actor, SimpleActor, SimpleSink, SimpleSource},
    scheduler::Scheduler,
};

use super::yaml_parser::Result;

type ActorCallback = fn(&Value, u16, &BiMap<String, u16>, Scheduler, f64) -> Result<AMActor>;
pub static ACTORS: LazyLock<Arc<Mutex<HashMap<String, ActorCallback>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn add_actor_implementation(label: String, callback: ActorCallback) {
    ACTORS.lock().unwrap().insert(label, callback);
}

pub fn import_default_actors() {
    add_actor_implementation(String::from("SimpleActor"), SimpleActor::parse);
    add_actor_implementation(String::from("SimpleSink"), SimpleSink::parse);
    add_actor_implementation(String::from("SimpleSource"), SimpleSource::parse);
}
