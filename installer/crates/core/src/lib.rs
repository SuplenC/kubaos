//! dotinstall-core: the installer's types and engine. The contract between
//! the engine, the CLI and the GUI is docs/INSTALLER.md.

pub mod config;
pub mod demo;
pub mod engine;
pub mod event;
pub mod probe;
pub mod setup;
pub mod step;

pub use config::{InstallConfig, Secrets};
pub use event::{Event, Level, StepInfo};
pub use probe::{Manifest, Probe};
pub use step::StepId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid config: {0}")]
    Config(String),
    #[error("{step:?} failed: {message}")]
    Step {
        step: StepId,
        message: String,
        hint: Option<String>,
    },
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}
