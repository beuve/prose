use std::fs::OpenOptions;

use itertools::enumerate;
use std::io::Write;

pub fn write_csv(path: String, data: &ndarray::ArrayView1<u32>) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();

    writeln!(file, "time,quantity").unwrap();

    for (time, quantity) in enumerate(data) {
        writeln!(file, "{},{}", time, quantity).unwrap();
    }
}
