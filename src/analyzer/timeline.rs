use crate::engine::tokens::Token;
use crate::parser::yaml_parser::ActorLogInfos;
use indicatif::ProgressBar;
use itertools::enumerate;
use ndarray::{s, Array, Array1, Array2};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::{HashMap, LinkedList};
use std::fs::{self, OpenOptions};
use std::io::Write;

fn analyze_single_token(
    token: &Token,
    processes: &HashMap<u16, ActorLogInfos>,
    max_time: usize,
) -> (Array1<f64>, Array1<f64>, Array2<u32>, Array2<u32>) {
    let mut time: usize = 0;
    let mut token_reentrances: Array2<u32> = Array::zeros((processes.len(), max_time));
    let mut token_occupencies: Array2<u32> = Array::zeros((processes.len(), max_time));
    let mut token_lifetimes: Array1<f64> = Array::zeros(processes.len());
    for (index, code) in token.timeline.iter().enumerate() {
        if index == 0 {
            // The first value contains the number of executions of
            // the production actor before this token was created
            time += *code as usize;
        } else if processes.contains_key(code) {
            let actor_log_infos = processes.get(code).unwrap();
            token_reentrances[[actor_log_infos.index, time]] += 1;
            if let Some(delay_sampler) = &actor_log_infos.time_sampler {
                let delay = delay_sampler();
                let mut new_time = time + delay;
                if new_time >= max_time {
                    new_time = max_time - 1;
                }
                token_lifetimes[actor_log_infos.index] += delay as f64;
                let mut s = token_occupencies.slice_mut(s![actor_log_infos.index, time..new_time]);
                s += 1;
                time = new_time;
            }
        }
    }
    (
        token_lifetimes.clone(),
        token_lifetimes.map(|x| x.powi(2)),
        token_reentrances,
        token_occupencies,
    )
}

pub fn analyze_timeline(
    tokens: LinkedList<Token>,
    processes: &HashMap<u16, ActorLogInfos>,
    max_time: usize,
    logs_folder: String,
    dt: f64,
) {
    let bar = ProgressBar::new(tokens.len() as u64);
    let (sum_lifetimes, sum_lifetimes_s, all_reentrances, all_occupencies) = tokens
        .par_iter()
        .fold(
            || {
                let sum_lifetimes: Array1<f64> = Array::zeros(processes.len());
                let sum_lifetimes_s: Array1<f64> = Array::zeros(processes.len());
                let all_reentrances: Array2<u32> = Array::zeros((processes.len(), max_time));
                let all_occupencies: Array2<u32> = Array::zeros((processes.len(), max_time));
                (
                    sum_lifetimes,
                    sum_lifetimes_s,
                    all_reentrances,
                    all_occupencies,
                )
            },
            |(sum_lifetimes, sum_lifetimes_s, all_reentrances, all_occupencies), token| {
                let (token_lifetimes, token_lifetimes_s, token_reentrances, token_occupencies) =
                    analyze_single_token(token, processes, max_time);
                bar.inc(1);
                (
                    sum_lifetimes + token_lifetimes,
                    sum_lifetimes_s + token_lifetimes_s,
                    all_reentrances + token_reentrances,
                    all_occupencies + token_occupencies,
                )
            },
        )
        .reduce(
            || {
                let acc_lifetimes: Array1<f64> = Array::zeros(processes.len());
                let acc_lifetimes_s: Array1<f64> = Array::zeros(processes.len());
                let acc_reentrances: Array2<u32> = Array::zeros((processes.len(), max_time));
                let acc_occupencies: Array2<u32> = Array::zeros((processes.len(), max_time));
                (
                    acc_lifetimes,
                    acc_lifetimes_s,
                    acc_reentrances,
                    acc_occupencies,
                )
            },
            |(acc_lifetimes, acc_lifetimes_s, acc_reentrances, acc_occupencies),
             (sum_lifetimes, sum_lifetimes_s, all_reentrances, all_occupencies)| {
                (
                    acc_lifetimes + sum_lifetimes,
                    acc_lifetimes_s + sum_lifetimes_s,
                    acc_reentrances + all_reentrances,
                    acc_occupencies + all_occupencies,
                )
            },
        );
    let n = tokens.len() as f64;
    let mean_lifetimes = sum_lifetimes.clone() / n;
    let var_lifetimes = sum_lifetimes_s / n - mean_lifetimes.map(|x| x.powi(2));
    for actor_log_infos in processes.values() {
        println!(
            "lifetime {}: {}±{}",
            actor_log_infos.product_code,
            mean_lifetimes[actor_log_infos.index] * dt,
            (var_lifetimes[actor_log_infos.index] / n).sqrt() * dt
        );
        fs::create_dir_all(format!("{}/{}", logs_folder, actor_log_infos.product_code)).unwrap();
        let mut reentrance_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(format!(
                "{}/{}/reentrances.csv",
                logs_folder, actor_log_infos.product_code
            ))
            .unwrap();
        writeln!(reentrance_file, "time,quantity").unwrap();
        for (time, quantity) in enumerate(all_reentrances.slice(s![actor_log_infos.index, ..])) {
            writeln!(reentrance_file, "{},{}", time, quantity).unwrap();
        }
        if actor_log_infos.time_sampler.is_some() {
            let mut occupency_file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .open(format!(
                    "{}/{}/occupency.csv",
                    logs_folder, actor_log_infos.product_code
                ))
                .unwrap();
            writeln!(occupency_file, "time,quantity").unwrap();
            for (time, quantity) in enumerate(all_occupencies.slice(s![actor_log_infos.index, ..]))
            {
                writeln!(occupency_file, "{},{}", time, quantity).unwrap();
            }
        }
    }
}
