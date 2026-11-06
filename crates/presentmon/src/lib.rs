mod session;
mod error;
mod loader;
mod sample;

pub use error::PmError;
pub use sample::{FpsQuery, FpsSample};
pub use session::{Session, TrackedProcess};

pub const DEFAULT_WINDOW_MS: f64 = 1000.0;

/// ETW flush period used for realtime polling, in ms (allowed range 8..1000).
pub const ETW_FLUSH_PERIOD_MS: u32 = 50;