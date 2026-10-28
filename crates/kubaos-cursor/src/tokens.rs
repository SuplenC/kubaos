//! The design tokens the pointer is sized by, which `build.rs` writes from `design/tokens.toml`
//! with `kubaos-tokens`.

include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
