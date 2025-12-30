use statrs::distribution::{ContinuousCDF, LogNormal};
use std::{
    collections::hash_map::Entry::Vacant,
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct CyclicSampler<T> {
    samples: Vec<T>,
    index: AtomicUsize,
}

impl<T> Default for CyclicSampler<T>
where
    T: Copy + std::cmp::Eq + std::hash::Hash + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CyclicSampler<T>
where
    T: Copy + std::cmp::Eq + std::hash::Hash + std::fmt::Debug,
{
    pub fn new_from_vec(samples: Vec<T>) -> Self {
        Self {
            samples,
            index: AtomicUsize::new(0),
        }
    }

    pub fn new() -> Self {
        Self {
            samples: vec![],
            index: AtomicUsize::new(0),
        }
    }

    pub fn set_samples(&mut self, samples: Vec<T>) {
        self.samples = samples;
        self.index.store(0, Ordering::Relaxed);
    }

    pub fn sample(&self) -> T {
        let index = self.index.fetch_add(1, Ordering::Relaxed);
        self.samples[index % self.samples.len()]
    }

    pub fn sample_n(&self, n: usize) -> Vec<T> {
        let mut res = vec![];
        for _ in 1..n {
            let s = self.sample();
            res.push(s);
        }
        res
    }

    pub fn freq(&self, n: usize) -> HashMap<T, usize> {
        let mut res = HashMap::new();
        for _ in 0..n {
            let s = self.sample();
            if let Vacant(e) = res.entry(s) {
                e.insert(1);
            } else {
                let freq = res.get_mut(&s).unwrap();
                *freq += 1;
            }
        }
        res
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

pub trait DistributionSampler {
    fn new_from_distribution(dist: impl ContinuousCDF<f64, f64>, len: usize) -> Self;

    fn lognormal(mean: f64, std: f64, len: usize) -> Self
    where
        Self: std::marker::Sized,
    {
        let variance = std * std;
        let sigma_sq = (1.0 + variance / (mean * mean)).ln();
        let sigma = sigma_sq.sqrt();

        let mu = mean.ln() - sigma_sq / 2.0;
        let dist = LogNormal::new(mu, sigma).unwrap();
        Self::new_from_distribution(dist, len)
    }
}

impl DistributionSampler for CyclicSampler<f64> {
    fn new_from_distribution(dist: impl ContinuousCDF<f64, f64>, len: usize) -> Self {
        let samples = halton_sequence(len)
            .iter()
            .map(|v| dist.inverse_cdf(*v))
            .collect();
        Self {
            samples,
            index: AtomicUsize::new(0),
        }
    }
}

fn halton_sequence(len: usize) -> Vec<f64> {
    let mut n = 0u64;
    let mut d = 1u64;
    let b = 2; // Base 2
    let mut res = vec![0f64; len];
    for v in res.iter_mut() {
        let x = d - n;
        if x == 1 {
            n = 1;
            d *= b;
        } else {
            let mut y = d / b;
            while x <= y {
                y /= b;
            }
            n = (b + 1) * y - x;
        }
        *v = n as f64 / d as f64;
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halton() {
        let seq = halton_sequence(7);
        assert_eq!(
            seq,
            vec![
                1f64 / 2f64,
                1f64 / 4f64,
                3f64 / 4f64,
                1f64 / 8f64,
                5f64 / 8f64,
                3f64 / 8f64,
                7f64 / 8f64
            ]
        );
    }
}
