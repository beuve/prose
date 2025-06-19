use clap::Parser;
use std::collections::LinkedList;

use componentflow::{
    analyzer::timeline::analyze_timeline,
    engine::{actor::AMActor, tokens::Token},
    parser::{
        actors_parser::import_default_actors,
        time_distribution_parser::import_default_time_callbacks,
        yaml_parser::{parse_config, Result},
    },
};
use threadpool::ThreadPool;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// Path to the Yaml configuration file
    #[arg(short, long)]
    pub config: String,

    /// Path to the output folder
    #[arg(short, long)]
    pub output: String,
}

pub fn supply_loop(source: AMActor, pool: ThreadPool, times: u32) {
    let need_new_supply = source.lock().unwrap().as_source().supply();
    if need_new_supply {
        supply_loop(source, pool, times + 1);
    }
}

pub fn main() -> Result<()> {
    import_default_actors();
    import_default_time_callbacks();
    let args = Arguments::parse();
    let pool = ThreadPool::new(1);
    let config = parse_config(args.config, pool.clone())?;
    let sources: Vec<AMActor> = config
        .init_sources
        .iter()
        .map(|a| config.actors.get(a).unwrap().clone())
        .collect();
    pool.clone().execute(move || {
        sources
            .into_iter()
            .for_each(|a| supply_loop(a, config.pool.clone(), 0))
    });
    pool.join();
    let mut tokens: LinkedList<Token> = LinkedList::new();
    for (label, actor) in config.actors {
        println!("#{label}: {}", actor.lock().unwrap().total());
        tokens.append(&mut actor.lock().unwrap().tokens());
    }
    analyze_timeline(
        tokens,
        &config.logs,
        (config.global.time_window as f64 / config.global.dt) as usize,
        args.output,
        config.global.dt,
    );
    Ok(())
}
