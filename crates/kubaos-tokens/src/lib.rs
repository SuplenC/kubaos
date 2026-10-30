//! KubaOS's design tokens: `design/tokens.toml` read, checked against the design language's rules,
//! and written out as each toolkit's copy.
//!
//! ```text
//! design/tokens.toml ─▶ parse ─▶ rules::check ─▶ Tokens ─▶ Target::render
//!   ├▶ by each crate's build script, into its OUT_DIR: kubaos-ui's tokens.rs, Kōbō's done/neon.rs,
//!   │  kubaos-cursor's tokens.rs
//!   ├▶ by the Nix build, or `just tokens` (and the mockups' Vite build) in a checkout:
//!   │  qml/KubaOS/Components/{Tokens,Symbols}.qml,
//!   │  config/{ghostty,foot,fuzzel}, config/tmux/colors.conf, config/matugen/templates/{ghostty,…},
//!   │  config/hypr/tokens.lua, design/mockups/src/tokens.css
//!   └▶ checked in: the Plymouth script's tokens, kubaos.plymouth, GRUB's theme.txt,
//!      config/matugen/config.toml's hues
//! ```
//!
//! # Owns
//!
//! - Reading the tokens ([`model`]), their rules ([`rules`]), and the outputs ([`target`]), each a
//!   [`Target`] rendering the same [`Tokens`], and what a crate's build script does with its own
//!   ([`build_script`]).
//!
//! # Not here
//!
//! - The values: `design/tokens.toml`. Using them in iced: `kubaos-ui`; in QML,
//!   `KubaOS.Components`.

pub mod boot;
pub mod contrast;
pub mod css;
pub mod cursor;
pub mod hypr;
pub mod model;
pub mod neon;
pub mod oklch;
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

/// Why a crate's build script couldn't write its tokens module.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BuildScriptError {
    #[error("cargo didn't set {0}")]
    Env(&'static str),
    #[error("loading the tokens")]
    Load(#[source] Error),
    #[error("{0} names no file")]
    NoFileName(&'static str),
    #[error("rendering {path}")]
    Render {
        path: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// What a crate's build script does for its [`target::Kept::OutDir`] output: reads the tokens from
/// the repo the crate is in, renders `target` and writes it into `OUT_DIR` under its file's name,
/// and asks cargo to run the script again when the tokens change.
///
/// # Errors
///
/// [`BuildScriptError`]: cargo didn't set `CARGO_MANIFEST_DIR` or `OUT_DIR`, the tokens can't be
/// loaded from the repo two directories up, or the output can't be rendered or written.
pub fn build_script(target: &dyn Target) -> Result<(), BuildScriptError> {
    let var = |name: &'static str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .ok_or(BuildScriptError::Env(name))
    };
    let root = var("CARGO_MANIFEST_DIR")?.join("../..");
    rerun_if_changed(&root.join(TOKENS_PATH));
    let tokens = load(&root).map_err(BuildScriptError::Load)?;
    let name = Path::new(target.path())
        .file_name()
        .ok_or(BuildScriptError::NoFileName(target.path()))?;
    let text = target
        .render(&tokens, &root)
        .map_err(|source| BuildScriptError::Render {
            path: target.path(),
            source,
        })?;
    let path = var("OUT_DIR")?.join(name);
    fs::write(&path, text).map_err(|source| BuildScriptError::Write { path, source })
}

#[expect(clippy::print_stdout, reason = "A build script talks to cargo on stdout.")]
fn rerun_if_changed(path: &Path) {
    println!("cargo::rerun-if-changed={}", path.display());
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
