use rand::thread_rng;
use rand_distr::{Distribution, LogNormal};
use serde_yaml::Value;

use crate::analyzer::Sampler;

use super::yaml_parser::Result;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

type TimeCallback = fn(&Value, f64) -> Result<Sampler>;
pub static TIME_CALLBACK: LazyLock<Arc<Mutex<HashMap<String, TimeCallback>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn add_time_callback_implementation(
    label: String,
    callback: fn(&Value, f64) -> Result<Sampler>,
) {
    TIME_CALLBACK.lock().unwrap().insert(label, callback);
}

pub fn import_default_time_callbacks() {
    add_time_callback_implementation(String::from("log_normal"), parse_lognormal);
    add_time_callback_implementation(String::from("constant"), constant);
}

fn parse_lognormal(doc: &Value, dt: f64) -> Result<Sampler> {
    let mean = doc.get("mean").expect("haha").as_f64().expect("");
    let std = doc.get("std").expect("haha").as_f64().expect("");
    Ok(Box::new(move || {
        let mut rng = thread_rng();
        (LogNormal::from_mean_cv(mean, std / mean)
            .unwrap()
            .sample(&mut rng) as f32
            / dt as f32)
            .round() as usize
    }) as Box<dyn Fn() -> usize + Sync + Send>)
}

fn constant(doc: &Value, dt: f64) -> Result<Sampler> {
    let value = doc.get("value").expect("haha").as_f64().expect("");
    Ok(Box::new(move || (value / dt) as usize) as Box<dyn Fn() -> usize + Sync + Send>)
}
