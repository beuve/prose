extern crate flow2; // not needed since Rust edition 2018

use rand::thread_rng;
use rand_distr::{Distribution, LogNormal};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use flow2::{
    analyzer::timeline::analyze_timeline,
    engine::actor::{Actor, Broadcast, SimpleActor, SimpleSink, SimpleSource},
};
use threadpool::ThreadPool;

pub fn main() {
    let pool = ThreadPool::new(7);

    let production_actor = Arc::new(Mutex::new(SimpleSource::new(
        "production".to_string(),
        "plastic".to_string(),
        (394288, 1),
        143.9152E6 as u32,
        pool.clone(),
    )));
    let use_actor = Arc::new(Mutex::new(SimpleActor::new(
        "use".to_string(),
        "plastic".to_string(),
        pool.clone(),
    )));
    let use_broadcast = Arc::new(Mutex::new(Broadcast::new(
        "use_broadcast".to_string(),
        "plastic".to_string(),
        pool.clone(),
    )));
    let recycling_actor = Arc::new(Mutex::new(SimpleActor::new(
        "recycling".to_string(),
        "plastic".to_string(),
        pool.clone(),
    )));
    let discard_actor = Arc::new(Mutex::new(SimpleSink::new(
        "discard".to_string(),
        "plastic".to_string(),
    )));
    let incineration_actor = Arc::new(Mutex::new(SimpleSink::new(
        "incineration".to_string(),
        "plastic".to_string(),
    )));

    production_actor.clone().lock().unwrap().register(
        "use_broadcast".to_string(),
        "plastic".to_string(),
        1,
        use_actor.clone(),
    );
    recycling_actor.clone().lock().unwrap().register(
        "use_actor".to_string(),
        "plastic".to_string(),
        1,
        use_actor.clone(),
    );
    use_actor.clone().lock().unwrap().register(
        "use_broadcast".to_string(),
        "plastic".to_string(),
        1,
        use_broadcast.clone(),
    );
    use_broadcast.clone().lock().unwrap().register(
        "use_actor".to_string(),
        "plastic".to_string(),
        29,
        use_actor.clone(),
    );
    use_broadcast.clone().lock().unwrap().register(
        "recycling_actor".to_string(),
        "plastic".to_string(),
        7,
        recycling_actor.clone(),
    );
    use_broadcast.clone().lock().unwrap().register(
        "incineration_actor".to_string(),
        "plastic".to_string(),
        9,
        incineration_actor.clone(),
    );
    use_broadcast.clone().lock().unwrap().register(
        "discard_actor".to_string(),
        "plastic".to_string(),
        55,
        discard_actor.clone(),
    );
    let now = Instant::now();
    pool.execute(move || {
        production_actor
            .lock()
            .unwrap()
            .supply(production_actor.clone())
    });
    pool.join();
    println!("Elapsed: {:.2?}", now.elapsed());
    println!("#use: {}", use_actor.lock().unwrap().total);
    println!("#recycling: {}", recycling_actor.lock().unwrap().total);
    println!(
        "#discard: {}",
        discard_actor.lock().unwrap().import_fifo.available_tokens()
    );
    println!(
        "#incineration: {}",
        incineration_actor
            .lock()
            .unwrap()
            .import_fifo
            .available_tokens()
    );
    let now = Instant::now();
    let mut tokens = discard_actor
        .lock()
        .unwrap()
        .import_fifo
        .tokens
        .split_off(0);
    tokens.append(&mut incineration_actor.lock().unwrap().import_fifo.tokens);
    println!("{:?}", LogNormal::from_mean_cv(8., 2. / 8.).unwrap());
    let use_delay: &'static (dyn Fn() -> usize + Sync) = &|| {
        let mut rng = thread_rng();
        (LogNormal::from_mean_cv(8., 2. / 8.)
            .unwrap()
            .sample(&mut rng) as f32
            * 365.)
            .round() as usize
    };
    let prod_delay: &'static (dyn Fn() -> usize + Sync) = &|| 1;
    analyze_timeline(
        tokens,
        &HashMap::from([
            ("use/plastic".to_string(), (0, Some(use_delay))),
            ("production/plastic".to_string(), (2, Some(prod_delay))),
            ("recycling/plastic".to_string(), (1, None)),
        ]),
        200 * 365,
        "./logs/log1",
    );
    println!("Elapsed: {:.2?}", now.elapsed());
}
