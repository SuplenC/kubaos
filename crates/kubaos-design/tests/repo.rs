use std::path::Path;

use kubaos_design::baseline;
use kubaos_design::baseline::Baseline;
use kubaos_design::lint;

#[test]
fn the_repo_holds_to_its_lint_baseline() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scan = lint::scan(&root).expect("scan the repo");
    let baseline = Baseline::load(&root).expect("read the baseline");

    let problems: Vec<String> = baseline::compare(&scan, &baseline)
        .iter()
        .map(ToString::to_string)
        .collect();
    let markers: Vec<String> = scan.bad_markers.iter().map(ToString::to_string).collect();

    assert!(
        problems.is_empty() && markers.is_empty(),
        "the design lint (`just design-check`) fails:\n{}\n{}",
        problems.join("\n"),
        markers.join("\n")
    );
}
