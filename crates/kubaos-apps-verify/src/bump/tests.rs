use toml_edit::DocumentMut;

use super::Found;
use super::Step;
use super::actions;
use super::agree;
use super::pins;
use super::write;

const APP: &str = r#"# The app's own comment stays.
name = "T3 Code"

[[action]]
name = "Install T3 Code"
kind = "deb"
package = "t3code"
version = "0.1.9"
amd64 = { url = "https://example.com/v0.1.9/a.deb", sha256 = "aa" }
arm64 = { url = "https://example.com/v0.1.9/b.deb", sha256 = "bb" }

# Between the actions.
[[action]]
name = "Start it"
kind = "unit"

[[setup.local.action]]
name = "Download the runtime"
kind = "download"
version = "0.1.9"
amd64 = { url = "https://example.com/v0.1.9/c.tar.gz", sha256 = "cc" }
"#;

fn newer(version: &str, url: &str, sha256: &str) -> Found {
    Found::Newer {
        version: version.into(),
        url: url.into(),
        sha256: sha256.into(),
    }
}

#[test]
fn the_pins_are_every_deb_and_download_in_every_setup() {
    let mut doc: DocumentMut = APP.parse().unwrap();

    let found: Vec<_> = actions(&mut doc).into_iter().filter_map(|table| pins(table)).collect();

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].name, "Install T3 Code");
    assert_eq!(found[0].arches.len(), 2);
    assert_eq!(found[1].name, "Download the runtime");
    assert_eq!(
        found[1].arches,
        [(
            "amd64",
            "https://example.com/v0.1.9/c.tar.gz".to_owned(),
            "cc".to_owned()
        )]
    );
}

#[test]
fn a_move_rewrites_its_values_and_keeps_the_rest_of_the_file() {
    let mut doc: DocumentMut = APP.parse().unwrap();

    let mut tables = actions(&mut doc);
    write(
        tables[0],
        "0.1.11",
        &[
            ("amd64", "https://example.com/v0.1.11/a.deb".into(), "a1".into()),
            ("arm64", "https://example.com/v0.1.11/b.deb".into(), "b1".into()),
        ],
    );

    assert_eq!(
        doc.to_string(),
        APP.replace(
            r#"version = "0.1.9"
amd64 = { url = "https://example.com/v0.1.9/a.deb", sha256 = "aa" }
arm64 = { url = "https://example.com/v0.1.9/b.deb", sha256 = "bb" }"#,
            r#"version = "0.1.11"
amd64 = { url = "https://example.com/v0.1.11/a.deb", sha256 = "a1" }
arm64 = { url = "https://example.com/v0.1.11/b.deb", sha256 = "b1" }"#
        )
    );
}

#[test]
fn architectures_that_move_together_are_one_move() {
    let step = agree(
        "0.1.9",
        &[
            ("amd64", newer("0.1.11", "https://a", "a1")),
            ("arm64", newer("0.1.11", "https://b", "b1")),
        ],
    );

    assert_eq!(
        step,
        Step::Moved {
            from: "0.1.9".into(),
            to: "0.1.11".into(),
            arches: vec![
                ("amd64", "https://a".into(), "a1".into()),
                ("arm64", "https://b".into(), "b1".into()),
            ],
        }
    );
}

#[test]
fn architectures_that_disagree_are_a_failure() {
    let one_behind = agree(
        "0.1.9",
        &[("amd64", newer("0.1.11", "https://a", "a1")), ("arm64", Found::Current)],
    );
    let apart = agree(
        "0.1.9",
        &[
            ("amd64", newer("0.1.11", "https://a", "a1")),
            ("arm64", newer("0.1.10", "https://b", "b1")),
        ],
    );

    assert!(matches!(one_behind, Step::Failed(_)));
    assert!(matches!(apart, Step::Failed(_)));
}

#[test]
fn every_architecture_current_is_current() {
    let step = agree("0.1.9", &[("amd64", Found::Current), ("arm64", Found::Current)]);

    assert_eq!(step, Step::Current("0.1.9".into()));
}
