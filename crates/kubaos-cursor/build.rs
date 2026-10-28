//! Writes the pointer's tokens, which `src/tokens.rs` includes, from `design/tokens.toml`.

fn main() -> Result<(), kubaos_tokens::BuildScriptError> {
    kubaos_tokens::build_script(&kubaos_tokens::cursor::CursorTokens)
}
