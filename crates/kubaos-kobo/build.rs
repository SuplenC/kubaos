//! Writes the neon sign's tokens, which `src/done/neon.rs` includes, from `design/tokens.toml`.

fn main() -> Result<(), kubaos_tokens::BuildScriptError> {
    kubaos_tokens::build_script(&kubaos_tokens::neon::KoboNeon)
}
