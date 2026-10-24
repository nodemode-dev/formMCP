pub mod catalog;
pub mod child;
pub mod clients;
pub mod family;
pub mod log;
pub mod mcp;
pub mod reach;
pub mod settings;
pub mod suggest;

pub use family::{App, Family, Source};
pub use reach::{Places, Reach, Route};

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
