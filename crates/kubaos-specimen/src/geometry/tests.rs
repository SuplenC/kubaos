use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use kubaos_geometry::Fixed;
use kubaos_geometry::Role;

use super::frames;
use super::rules;
use crate::shot;

fn count(node: &kubaos_geometry::Node, matches: &dyn Fn(&Role) -> bool) -> usize {
    usize::from(matches(&node.role)) + node.children.iter().map(|c| count(c, matches)).sum::<usize>()
}

#[test]
fn every_page_holds_to_the_geometry_rules_at_every_scale_in_both_modes() {
    shot::load_fonts();
    let frames = frames().expect("lay every page out");

    let findings = kubaos_geometry::check(&frames, &rules());

    let report: Vec<String> = findings.iter().take(40).map(ToString::to_string).collect();
    assert!(
        findings.is_empty(),
        "{} geometry finding(s), `cargo run -p kubaos-specimen -- check` lists them:\n{}",
        findings.len(),
        report.join("\n")
    );
}

#[test]
fn the_probes_report_the_components_pages_buttons_texts_and_icons() {
    shot::load_fonts();
    let frames = frames().expect("lay every page out");
    let components = frames
        .iter()
        .find(|f| f.page == "components")
        .expect("the components page");

    let buttons = count(
        &components.root,
        &|role| matches!(role, Role::Component { name, .. } if name == "Button" || name == "IconButton"),
    );
    let texts = count(
        &components.root,
        &|role| matches!(role, Role::Text { content, .. } if !content.is_empty()),
    );
    let icons = count(&components.root, &|role| matches!(role, Role::Icon { .. }));

    assert!(buttons > 100, "{buttons} buttons");
    assert!(texts > 100, "{texts} texts");
    assert!(icons > 50, "{icons} icons");
}

/// DESIGN.md's table of variants: each component name, the words its variant may use, and which
/// measures it fixes.
fn documented_variants() -> BTreeMap<String, (BTreeSet<String>, Fixed)> {
    let design = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/DESIGN.md"))
        .expect("read docs/DESIGN.md");
    let table = design
        .split("<!-- variants:start -->")
        .nth(1)
        .and_then(|rest| rest.split("<!-- variants:end -->").next())
        .expect("DESIGN.md's variants table");
    let backticked = |cell: &str| -> Vec<String> { cell.split('`').skip(1).step_by(2).map(str::to_owned).collect() };
    let mut documented = BTreeMap::new();
    for row in table.lines().filter(|l| l.starts_with("| `")) {
        let cells: Vec<&str> = row.split('|').map(str::trim).collect();
        let fixed = match cells[3] {
            "both" => Fixed::BOTH,
            "height" => Fixed::HEIGHT,
            "width" => Fixed {
                width: true,
                height: false,
            },
            "none" => Fixed::NONE,
            other => panic!("{other:?} isn't a fixed measure"),
        };
        let words: BTreeSet<String> = backticked(cells[2]).into_iter().collect();
        for name in backticked(cells[1]) {
            documented.insert(name, (words.clone(), fixed));
        }
    }
    documented
}

#[test]
fn every_component_reports_the_variant_words_and_fixed_measures_design_md_documents() {
    shot::load_fonts();
    let frames = frames().expect("lay every page out");
    let documented = documented_variants();
    let mut reported: BTreeMap<String, (BTreeSet<String>, Fixed)> = BTreeMap::new();
    let mut pending: Vec<&kubaos_geometry::Node> = frames.iter().map(|f| &f.root).collect();
    while let Some(node) = pending.pop() {
        if let Role::Component { name, variant, fixed } = &node.role {
            let entry = reported
                .entry(name.clone())
                .or_insert_with(|| (BTreeSet::new(), *fixed));
            assert_eq!(entry.1, *fixed, "{name} reports two sets of fixed measures");
            entry.0.extend(variant.split_whitespace().map(str::to_owned));
        }
        pending.extend(&node.children);
    }

    for (name, (words, fixed)) in &reported {
        let (allowed, documented_fixed) = documented
            .get(name)
            .unwrap_or_else(|| panic!("{name} isn't in DESIGN.md's variants table"));
        assert_eq!(fixed, documented_fixed, "{name}'s fixed measures");
        let undocumented: Vec<&String> = words.difference(allowed).collect();
        assert!(
            undocumented.is_empty(),
            "{name} uses undocumented variant words {undocumented:?}"
        );
    }
    let missing: Vec<&String> = documented.keys().filter(|name| !reported.contains_key(*name)).collect();
    assert!(
        missing.is_empty(),
        "DESIGN.md documents {missing:?}, which no page reports"
    );
}
