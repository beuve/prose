use rand::thread_rng;
use rand_distr::{Distribution, LogNormal};
use yaml_rust2::Yaml;

use crate::analyzer::TimeCallback;

use super::yaml_parser::{Result, YamlParser};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

pub static TIME_CALLBACK: LazyLock<
    Arc<Mutex<HashMap<String, fn(&Yaml, f64) -> Result<TimeCallback>>>>,
> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn add_time_callback_implementation(
    label: String,
    callback: fn(&Yaml, f64) -> Result<TimeCallback>,
) {
    TIME_CALLBACK.lock().unwrap().insert(label, callback);
}

pub fn import_default_time_callbacks() {
    add_time_callback_implementation(String::from("log_normal"), parse_lognormal);
    add_time_callback_implementation(String::from("constant"), constant);
}

fn parse_lognormal(doc: &Yaml, dt: f64) -> Result<TimeCallback> {
    let mean = doc.get("mean")?.float()?;
    let std = doc.get("std")?.float()?;
    Ok(Box::new(move || {
        let mut rng = thread_rng();
        (LogNormal::from_mean_cv(mean, std / mean)
            .unwrap()
            .sample(&mut rng) as f32
            / dt as f32)
            .round() as usize
    }) as Box<(dyn Fn() -> usize + Sync)>)
}

fn constant(doc: &Yaml, dt: f64) -> Result<TimeCallback> {
    let value = doc.get("value")?.float()?;
    Ok(Box::new(move || (value / dt) as usize) as Box<(dyn Fn() -> usize + Sync)>)
}
