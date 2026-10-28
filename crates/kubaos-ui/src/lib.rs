//! KubaOS's iced components: every Rust front end (Kōbō, and the apps to come) draws with these,
//! so they look and behave like one thing with the shell.
//!
//! A view is built from the layout primitives and the closed component set, all of which take
//! tokens and a [`Look`], never a length, colour or duration of their own:
//!
//! ```text
//! design/tokens.toml ─kubaos-tokens─▶ tokens ─▶ Look (colours, mode, scale)
//!                                                 │
//!                       layout (page, section, row, stack, grid) + component::* ─▶ iced Element
//! ```
//!
//! # Owns
//!
//! - The tokens as Rust ([`tokens`], generated), display scales ([`scale`]), colours and contrast
//!   ([`color`]), motion ([`motion`]), type ([`text`]) and icons ([`icon`]).
//! - Layout on the grid ([`layout`], [`grid`], [`scroll`]), the focus tube ([`focus`]) and the
//!   components ([`component`]).
//! - What each box is, for the geometry check ([`probe`]).
//!
//! # Not here
//!
//! - The values: `design/tokens.toml`. The shell's and the menu's components: `KubaOS.Components`
//!   in QML, from the same tokens.

pub mod color;
pub mod component;
pub mod focus;
pub mod grid;
pub mod icon;
pub mod layout;
pub mod look;
pub mod motion;
pub mod probe;
pub mod scale;
pub mod scroll;
pub mod snap;
pub mod text;
#[rustfmt::skip]
#[expect(
    clippy::match_same_arms,
    clippy::too_many_lines,
    reason = "Generated: one arm per token, even where two share a value, and one per symbol."
)]
pub mod tokens;

pub use self::look::Look;
pub use self::scale::Scale;
pub use self::tokens::Mode;
