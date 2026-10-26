use std::path::Path;

#[test]
fn the_checked_in_outputs_match_the_tokens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tokens = kubaos_tokens::load(&root).expect("design/tokens.toml holds every rule");

    let stale = kubaos_tokens::target::stale(&root, &tokens).expect("read the outputs");

    assert!(
        stale.is_empty(),
        "stale, run `cargo run -p kubaos-tokens -- write`: {stale:?}"
    );
}
