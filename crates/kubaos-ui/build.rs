//! Writes the design tokens module, which `src/tokens.rs` includes, from `design/tokens.toml`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    kubaos_tokens::build_script(&kubaos_tokens::rust::RustModule)
}
