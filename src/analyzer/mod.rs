pub mod plot;
pub mod timeline;

pub type TimeCallback = Box<dyn Fn() -> usize + Sync>;
