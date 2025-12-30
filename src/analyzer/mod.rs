mod analyze_token;
mod csv;
mod token_stats;

pub type Sampler = Box<dyn Fn() -> usize + Sync + Send>;
use std::{collections::LinkedList, fs};

use crate::{engine::tokens::Token, parser::yaml_parser::LogingConfig};
use analyze_token::analyze_single_token;
use csv::write_csv;
use indicatif::ProgressBar;
use ndarray::s;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use token_stats::TokenStats;

pub fn analyze_timeline(
    tokens: LinkedList<Token>,
    max_time: usize,
    loging_config: LogingConfig,
    logs_folder: String,
    dt: f64,
) {
    let bar = ProgressBar::new(tokens.len() as u64);
    let process_count = loging_config.actors_indices.len();

    let total_stats = tokens
        .par_iter()
        .fold(
            || TokenStats::zeros(process_count, max_time),
            |acc, token| {
                let stats = analyze_single_token(token, &loging_config, max_time);
                bar.inc(1);
                acc.accumulate(stats)
            },
        )
        .reduce(
            || TokenStats::zeros(process_count, max_time),
            |a, b| a.accumulate(b),
        );

    let n = tokens.len() as f64;

    let mean_lifetimes = &total_stats.lifetimes / n;
    let var_lifetimes = &total_stats.lifetimes_sq / n - mean_lifetimes.map(|x| x.powi(2));

    for (code, idx) in loging_config.actors_indices.iter() {
        let name = loging_config.name_from_code(*code);
        println!(
            "lifetime {}: {}±{}",
            name,
            mean_lifetimes[*idx] * dt,
            (var_lifetimes[*idx] / n).sqrt() * dt
        );

        let actor_dir = format!("{}/{}", logs_folder, name);
        fs::create_dir_all(&actor_dir).unwrap();

        write_csv(
            format!("{}/reentrances.csv", actor_dir),
            &total_stats.reentrances.slice(s![*idx, ..]),
        );
        write_csv(
            format!("{}/occupency.csv", actor_dir),
            &total_stats.occupancies.slice(s![*idx, ..]),
        );
    }
}
