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
