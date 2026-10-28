use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use kubaos_tokens::target::Kept;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_checked_in_outputs_match_the_tokens() {
    let root = root();
    let tokens = kubaos_tokens::load(&root).expect("design/tokens.toml holds every rule");

    let stale = kubaos_tokens::target::stale(&root, &tokens).expect("render every output");

    assert!(stale.is_empty(), "stale, run `just tokens`: {stale:?}");
}

#[test]
fn git_ignores_the_built_outputs_and_no_other() {
    let ignore = fs::read_to_string(root().join(".gitignore")).expect("read .gitignore");
    let ignored: BTreeSet<&str> = ignore.lines().filter_map(|line| line.strip_prefix('/')).collect();

    for target in kubaos_tokens::target::targets() {
        let listed = ignored.contains(target.path());
        assert_eq!(
            listed,
            target.kept() == Kept::Built,
            "{} is kept {:?}: list it in .gitignore as `/{}` exactly when it is built",
            target.path(),
            target.kept(),
            target.path()
        );
    }
}
