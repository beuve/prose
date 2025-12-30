use clap::Parser;
use std::{collections::LinkedList, path::Path};

use componentflow::{
    analyzer::analyze_timeline,
    engine::{actors::AMActor, tokens::Token},
    parser::{
        actors_parser::import_default_actors,
        time_distribution_parser::import_default_time_callbacks,
        yaml_parser::{parse_config, Result},
    },
};

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

pub fn supply_loop(source: AMActor, times: u32) {
    let need_new_supply = {
        // This is usefull to release source to avoid deadlocks
        let mut _actor = source.lock().unwrap();
        let _source = _actor.as_source();
        let delay = _source.delay();
        _source.supply(delay * times as u64)
    };

    if need_new_supply {
        supply_loop(source.clone(), times + 1);
    }
}

pub fn main() -> Result<()> {
    import_default_actors();
    import_default_time_callbacks();
    let args = Arguments::parse();
    let config = parse_config(Path::new(&args.config));
    let sources: Vec<AMActor> = config
        .init_sources
        .iter()
        .map(|a| config.actors.get(a).unwrap().clone())
        .collect();
    sources.into_iter().for_each(|a| supply_loop(a, 0));
    config.scheduler.run();
    let mut tokens: LinkedList<Token> = LinkedList::new();
    for (label, actor) in config.actors {
        println!("#{label}: {}", actor.lock().unwrap().total());
        tokens.append(&mut actor.lock().unwrap().tokens());
    }
    analyze_timeline(
        tokens,
        (config.global.time_window as f64 / config.global.dt) as usize,
        config.loging_config,
        args.output,
        config.global.dt,
    );
    Ok(())
}
