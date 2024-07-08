use crate::engine::tokens::Token;
use indicatif::ProgressBar;
use itertools::enumerate;
use ndarray::{s, Array, Array1, Array2};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::{HashMap, LinkedList};
use std::fs::{self, OpenOptions};
use std::io::Write;

fn analyze_single_token<F>(
    token: &Token,
    processes: &HashMap<String, (usize, Option<F>)>,
    max_time: usize,
) -> (Array1<f64>, Array2<u32>, Array2<u32>)
where
    F: Fn() -> usize + Sync,
{
    let mut time: usize = 0;
    let mut token_reentrances: Array2<u32> = Array::zeros((processes.len(), max_time));
    let mut token_occupencies: Array2<u32> = Array::zeros((processes.len(), max_time));
    let mut token_lifetimes: Array1<f64> = Array::zeros(processes.len());
    for code in token.timeline.clone() {
        if processes.contains_key(&code) {
            let (index, delay_sampler) = processes.get(&code).unwrap();
            let index = index.clone();
            token_reentrances[[index, time]] += 1;
            if let Some(delay_sampler) = delay_sampler {
                let delay = delay_sampler();
                let new_time = time + delay;
                token_lifetimes[index] += delay as f64;
                let mut s = token_occupencies.slice_mut(s![index, time..new_time]);
                s += 1;
                time = new_time;
            }
        }
    }
    return (token_lifetimes, token_reentrances, token_occupencies);
}

pub fn analyze_timeline<F>(
    tokens: LinkedList<Token>,
    processes: &HashMap<String, (usize, Option<F>)>,
    max_time: usize,
    logs_folder: &str,
) where
    F: Fn() -> usize + Sync,
{
    let bar = ProgressBar::new(tokens.len() as u64);
    let (sum_lifetimes, all_reentrances, all_occupencies) = tokens
        .par_iter()
        .fold(
            || {
                let sum_lifetimes: Array1<f64> = Array::zeros(processes.len());
                let all_reentrances: Array2<u32> = Array::zeros((processes.len(), max_time));
                let all_occupencies: Array2<u32> = Array::zeros((processes.len(), max_time));
                (sum_lifetimes, all_reentrances, all_occupencies)
            },
            |(sum_lifetimes, all_reentrances, all_occupencies), token| {
                let (token_lifetimes, token_reentrances, token_occupencies) =
                    analyze_single_token(token, processes, max_time);
                bar.inc(1);
                return (
                    sum_lifetimes + token_lifetimes,
                    all_reentrances + token_reentrances,
                    all_occupencies + token_occupencies,
                );
            },
        )
        .reduce(
            || {
                let acc_lifetimes: Array1<f64> = Array::zeros(processes.len());
                let acc_reentrances: Array2<u32> = Array::zeros((processes.len(), max_time));
                let acc_occupencies: Array2<u32> = Array::zeros((processes.len(), max_time));
                (acc_lifetimes, acc_reentrances, acc_occupencies)
            },
            |(acc_lifetimes, acc_reentrances, acc_occupencies),
             (sum_lifetimes, all_reentrances, all_occupencies)| {
                (
                    acc_lifetimes + sum_lifetimes,
                    acc_reentrances + all_reentrances,
                    acc_occupencies + all_occupencies,
                )
            },
        );
    println!("{}", sum_lifetimes);
    let sum_lifetimes = sum_lifetimes / (tokens.len() as f64);
    for (code, (index, delay)) in processes {
        println!("lifetime {}: {}", code, sum_lifetimes[index.clone()]);
        fs::create_dir_all(format!("{}/{}", logs_folder, code)).unwrap();
        let mut reentrance_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(format!("{}/{}/reentrances.csv", logs_folder, code))
            .unwrap();
        writeln!(reentrance_file, "time,quantity").unwrap();
        for (time, quantity) in enumerate(all_reentrances.slice(s![index.clone(), ..])) {
            writeln!(reentrance_file, "{},{}", time, quantity).unwrap();
        }
        if let Some(_) = delay {
            let mut occupency_file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .open(format!("{}/{}/occupency.csv", logs_folder, code))
                .unwrap();
            writeln!(occupency_file, "time,quantity").unwrap();
            for (time, quantity) in enumerate(all_occupencies.slice(s![index.clone(), ..])) {
                writeln!(occupency_file, "{},{}", time, quantity).unwrap();
            }
        }
    }
}
