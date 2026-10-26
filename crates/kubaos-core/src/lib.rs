//! kubaos-core: the engine and its types. The contract between the
//! engine, the CLI and Kōbō is docs/INSTALL-INTERNALS.md.

pub mod apt;
pub mod bitwarden;
pub mod config;
pub mod demo;
pub mod engine;
pub mod event;
pub mod keyboard;
pub mod packages;
pub mod probe;
pub mod setup;
pub mod state;
pub mod step;
pub mod suites;
pub mod switch;
pub mod update;

pub use config::{InstallConfig, Secrets};
pub use kubaos_phone as phone;
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
