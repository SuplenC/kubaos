use super::Baseline;
use super::Problem;
use super::compare;
use super::shrink;
use crate::lint::Rule;
use crate::lint::Scan;
use crate::lint::Violation;

fn violation(path: &str, line: usize, rule: Rule) -> Violation {
    Violation {
        path: path.to_owned(),
        line,
        column: 5,
        rule,
        what: "size: 22".to_owned(),
        note: None,
    }
}

fn scan(violations: Vec<Violation>) -> Scan {
    Scan {
        violations,
        ..Scan::default()
    }
}

fn baseline(text: &str) -> Baseline {
    Baseline::parse(text).expect("a baseline")
}

#[test]
fn a_file_at_its_count_passes() {
    let found = scan(vec![
        violation("A.qml", 3, Rule::Size),
        violation("A.qml", 9, Rule::Size),
    ]);

    assert_eq!(compare(&found, &baseline("[\"A.qml\"]\nsize = 2\n")), []);
}

#[test]
fn a_new_violation_puts_its_file_over_and_lists_every_one_of_its_rule() {
    let found = scan(vec![
        violation("A.qml", 3, Rule::Size),
        violation("A.qml", 9, Rule::Size),
        violation("B.qml", 1, Rule::Colour),
    ]);

    let problems = compare(&found, &baseline("[\"A.qml\"]\nsize = 1\n"));

    assert_eq!(problems.len(), 2);
    assert!(matches!(&problems[0], Problem::Over { path: "A.qml", found, .. } if found.len() == 2));
    assert!(matches!(
        &problems[1],
        Problem::Over {
            path: "B.qml",
            allowed: 0,
            ..
        }
    ));
}

#[test]
fn a_fixed_violation_leaves_the_baseline_stale_until_it_shrinks() {
    let found = scan(vec![violation("A.qml", 3, Rule::Size)]);
    let before = baseline("[\"A.qml\"]\nsize = 2\nradius = 1\n");

    let problems = compare(&found, &before);
    let after = shrink(&found, &before).expect("nothing grew");

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert_eq!(after, baseline("[\"A.qml\"]\nsize = 1\n"));
    assert_eq!(compare(&found, &after), []);
}

#[test]
fn shrinking_refuses_a_file_over_its_count() {
    let found = scan(vec![
        violation("A.qml", 3, Rule::Size),
        violation("A.qml", 4, Rule::Size),
    ]);

    assert!(shrink(&found, &baseline("[\"A.qml\"]\nsize = 1\n")).is_err());
}

#[test]
fn a_baseline_reads_back_from_what_it_renders() {
    let written = baseline("[\"config/a b/A.qml\"]\nsize = 2\ncolour = 1\n");

    assert_eq!(Baseline::parse(&written.render()).expect("rendered"), written);
}
