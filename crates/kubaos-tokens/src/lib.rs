//! KubaOS's design tokens: `design/tokens.toml` read, checked against the design language's rules,
//! and written out as each toolkit's copy.
//!
//! ```text
//! design/tokens.toml ─▶ parse ─▶ rules::check ─▶ Tokens ─▶ Target::render ─▶ crates/kubaos-ui/src/tokens.rs
//!                                                                         ├▶ qml/KubaOS/Components/Tokens.qml
//!                                                                         ├▶ qml/KubaOS/Components/Symbols.qml
//!                                                                         ├▶ config/{ghostty,foot,fuzzel}, config/tmux/colors.conf
//!                                                                         ├▶ config/matugen/templates/{ghostty,foot.ini,tmux.conf,…}
//!                                                                         └▶ (boot: to come)
//! ```
//!
//! # Owns
//!
//! - Reading the tokens ([`model`]), their rules ([`rules`]), and the outputs ([`target`]), each a
//!   [`Target`] rendering the same [`Tokens`].
//!
//! # Not here
//!
//! - The values: `design/tokens.toml`. Using them in iced: `kubaos-ui`; in QML,
//!   `KubaOS.Components`.

pub mod contrast;
pub mod model;
pub mod qml;
pub mod rules;
pub mod rust;
pub mod target;
pub mod terminal;

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub use self::model::Tokens;
pub use self::rules::Violation;
pub use self::target::Target;

/// Where the tokens live, relative to the repo's root.
pub const TOKENS_PATH: &str = "design/tokens.toml";

/// Why the tokens couldn't be loaded.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("parsing the tokens")]
    Parse(#[source] toml::de::Error),
    #[error("the tokens break {} rule(s)", .0.len())]
    Invalid(Vec<Violation>),
}

/// Reads the tokens from `text` and checks every rule.
///
/// # Errors
///
/// [`Error::Parse`] when `text` isn't the tokens' shape, [`Error::Invalid`] with every broken rule
/// when it is but breaks some.
pub fn parse(text: &str) -> Result<Tokens, Error> {
    let tokens: Tokens = toml::from_str(text).map_err(Error::Parse)?;
    let violations = rules::check(&tokens);
    if violations.is_empty() {
        Ok(tokens)
    } else {
        Err(Error::Invalid(violations))
    }
}

/// Reads and checks the tokens of the repo at `root`.
///
/// # Errors
///
/// [`Error::Read`] when the file can't be read; otherwise as [`parse`].
pub fn load(root: &Path) -> Result<Tokens, Error> {
    let path = root.join(TOKENS_PATH);
    let text = fs::read_to_string(&path).map_err(|source| Error::Read { path, source })?;
    parse(&text)
}
