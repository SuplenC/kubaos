//! Writes the neon sign's tokens, which `src/done/neon.rs` includes, from `design/tokens.toml`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    kubaos_tokens::build_script(&kubaos_tokens::neon::KoboNeon)
}
