pub mod catalog;
pub mod child;
pub mod clients;
pub mod extensions;
pub mod family;
pub mod hooks;
pub mod http;
pub mod log;
pub mod mcp;
pub mod reach;
pub mod redact;
pub mod secrets;
pub mod settings;
pub mod suggest;
pub mod telemetry;
pub mod toolcache;
pub mod verify;

pub use family::{App, Family, Source};
pub use reach::{Places, Reach, Route};

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
