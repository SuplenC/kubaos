//! Writes the pointer's tokens, which `src/tokens.rs` includes, from `design/tokens.toml`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    kubaos_tokens::build_script(&kubaos_tokens::cursor::CursorTokens)
}
