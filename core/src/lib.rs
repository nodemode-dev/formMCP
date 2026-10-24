pub mod family;
pub mod mcp;

pub use family::{App, Family, Source};

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
