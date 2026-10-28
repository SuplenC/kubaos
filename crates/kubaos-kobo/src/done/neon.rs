//! The neon sign's tokens: how a tube strikes, as the boot splash's sign does, and the クバ mark's
//! strokes, which the logo's SVGs draw too. `build.rs` writes them from `design/tokens.toml` with
//! `kubaos-tokens`.

include!(concat!(env!("OUT_DIR"), "/neon.rs"));
