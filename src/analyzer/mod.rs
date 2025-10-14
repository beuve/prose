pub mod plot;
pub mod timeline;

pub type Sampler = Box<dyn Fn() -> usize + Sync>;
