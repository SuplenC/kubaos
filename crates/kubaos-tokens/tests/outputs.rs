use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use kubaos_tokens::target::Kept;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_checked_in_outputs_match_the_tokens() {
    let root = root();
    let tokens = kubaos_tokens::load(&root).expect("design/tokens.toml holds every rule");

    let stale = kubaos_tokens::target::stale(&root, &tokens).expect("render every output");

    assert!(stale.is_empty(), "stale, run `just tokens`: {stale:?}");
}

/// The home links every `/config/` line of `.gitignore` as a built output, and the e2e checks
/// each one, so the list must be the built outputs exactly, both ways; the mockups' CSS is one too.
#[test]
fn git_ignores_the_built_outputs_and_no_other() {
    let ignore = fs::read_to_string(root().join(".gitignore")).expect("read .gitignore");
    let ignored: BTreeSet<&str> = ignore
        .lines()
        .filter_map(|line| line.strip_prefix('/'))
        .filter(|path| {
            ["config/", "qml/", "design/mockups/src/"]
                .iter()
                .any(|dir| path.starts_with(dir))
        })
        .collect();
    let built: BTreeSet<&str> = kubaos_tokens::target::targets()
        .filter(|target| target.kept() == Kept::Built)
        .map(kubaos_tokens::Target::path)
        .collect();

    assert_eq!(
        ignored, built,
        "the /config/, /qml/ and /design/mockups/src/ lines of .gitignore are the built outputs, each as `/PATH`"
    );
}
