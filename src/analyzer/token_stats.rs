use ndarray::{Array, Array1, Array2};

pub struct TokenStats {
    pub lifetimes: Array1<f64>,
    pub lifetimes_sq: Array1<f64>,
    pub reentrances: Array2<u32>,
    pub occupancies: Array2<u32>,
}

impl TokenStats {
    pub fn zeros(process_count: usize, max_time: usize) -> Self {
        Self {
            lifetimes: Array::zeros(process_count),
            lifetimes_sq: Array::zeros(process_count),
            reentrances: Array::zeros((process_count, max_time)),
            occupancies: Array::zeros((process_count, max_time)),
        }
    }

    pub fn accumulate(self, other: Self) -> Self {
        Self {
            lifetimes: self.lifetimes + other.lifetimes,
            lifetimes_sq: self.lifetimes_sq + other.lifetimes_sq,
            reentrances: self.reentrances + other.reentrances,
            occupancies: self.occupancies + other.occupancies,
        }
    }
}
