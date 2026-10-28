//! The design tokens: every length, type style, duration, curve and colour role, as closed enums
//! and constants. A length is a variant, never a number, so nothing off the scale can be written.
//!
//! # Owns
//!
//! - The values, which `build.rs` writes from `design/tokens.toml` with `kubaos-tokens`.
//!
//! # Not here
//!
//! - Turning them into iced values at a display scale: the rest of `kubaos-ui`.

include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
