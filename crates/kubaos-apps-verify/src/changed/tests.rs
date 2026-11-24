use kubaos_core::sandbox;

use super::*;
use crate::check;
use crate::check::Context;
use crate::check::Outcome;
use crate::stub::Reply;
use crate::stub::Stub;

/// A fixture app, `tool`, with `actions`; written here, so no test follows the live catalog.
fn app_toml(actions: &str) -> String {
    format!(
        r#"name = "Tool"
category = "ai"
tags = ["tool"]
summary = "A tool for the tests."
description = """\
A tool for the tests."""
description_from = "https://example.com/README.md"
icon = "openai"
keeps = """\
Nothing stays."""

{actions}"#
    )
}

/// A `download` action of two architectures, each at `url` followed by its architecture.
fn download(url: &str) -> String {
    let sha256 = "0".repeat(64);
    app_toml(&format!(
        r#"[[action]]
name = "Download the tool"
kind = "download"
to = "/usr/local/bin/tool"
version = "1.0"
unpack = "file"
amd64 = {{ url = "{url}amd64", sha256 = "{sha256}" }}
arm64 = {{ url = "{url}arm64", sha256 = "{sha256}" }}
"#
    ))
}

/// An `apt` action of `packages` from the vendor repository `vendor`.
fn apt(packages: &str) -> String {
    app_toml(&format!(
        r#"[[action]]
name = "Install the tool"
kind = "apt"
packages = [{packages}]
repo = "vendor"
"#
    ))
}

/// A package manifest pinning the repository `vendor`, its key at `key_url`.
fn manifest(key_url: &str) -> String {
    format!(
        "[repo.vendor]\nkey_url = \"{key_url}\"\nfingerprint = \"9DC858229FC7DD38854AE2D88D81803C0EBFCD88\"\nkeyring = \"/etc/apt/keyrings/vendor.gpg\"\n"
    )
}

/// A checkout with `base` as the app's `app.toml` and `manifest` as its package manifest,
/// committed, then `head` as the app's `app.toml` in the working tree; its root and the base
/// commit.
fn checkout(base: &str, head: &str, manifest: &str) -> (tempfile::TempDir, String) {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("apps/tool");
    fs::create_dir_all(&app).unwrap();
    fs::create_dir_all(root.path().join(SOURCES)).unwrap();
    fs::write(
        root.path().join(SOURCES).join("vendor.sources"),
        "Types: deb\nURIs: https://example.com/deb\nSuites: stable\nComponents: main\n",
    )
    .unwrap();
    fs::write(root.path().join("system/packages.toml"), manifest).unwrap();
    fs::write(app.join("app.toml"), base).unwrap();
    sandbox::git(root.path(), &["init", "-q"]);
    sandbox::git(root.path(), &["add", "."]);
    sandbox::git(root.path(), &["commit", "-q", "-m", "base"]);
    let commit = sandbox::git(root.path(), &["rev-parse", "HEAD"]);
    fs::write(app.join("app.toml"), head).unwrap();
    (root, commit)
}

/// The rows of checking the app in `root` against what changed since `base`.
fn rows(root: &Path, base: &str) -> Vec<check::Row> {
    let catalog = Catalog::load(&root.join(CATALOG)).unwrap();
    let manifest = Manifest::parse(&fs::read_to_string(root.join("system/packages.toml")).unwrap()).unwrap();
    let changed = Changed::read(root, base, &manifest);
    let context = Context::new(root.to_path_buf(), manifest, "resolute".to_owned()).with_changed(changed);
    check::app(&context, catalog.get("tool").unwrap())
}

/// An HTTPS URL on loopback whose TLS handshake fails, an outage, at once.
fn tls_down(stub: &Stub) -> String {
    stub.url("/tool-").replacen("http://", "https://", 1)
}

#[test]
fn an_outage_on_a_download_the_branch_changed_is_an_error() {
    let stub = Stub::always(Reply::Body(Vec::new()));
    let (root, base) = checkout(
        &download("https://example.com/old/tool-"),
        &download(&tls_down(&stub)),
        "",
    );

    let rows = rows(root.path(), &base);

    let sha256: Vec<_> = rows.iter().filter(|row| row.check == "sha256").collect();
    assert_eq!(sha256.len(), 2);
    for row in sha256 {
        assert_eq!(row.outcome, Outcome::Error, "{}", row.detail);
        assert!(row.detail.contains("this branch changes the pin"), "{}", row.detail);
    }
}

#[test]
fn an_outage_on_a_download_the_branch_kept_is_a_warning() {
    let stub = Stub::always(Reply::Body(Vec::new()));
    let app = download(&tls_down(&stub));
    let (root, base) = checkout(&app, &app, "");

    let rows = rows(root.path(), &base);

    assert!(rows.iter().any(|row| row.check == "sha256"));
    for row in rows.iter().filter(|row| row.check == "sha256") {
        assert_eq!(row.outcome, Outcome::Warning, "{}", row.detail);
    }
}

#[test]
fn a_new_package_from_a_kept_repository_that_is_down_fails() {
    let stub = Stub::always(Reply::Status(503));
    let (root, base) = checkout(
        &apt(r#""tool""#),
        &apt(r#""tool", "tool-typo""#),
        &manifest(&stub.url("/gpg")),
    );

    let rows = rows(root.path(), &base);

    let key = rows.iter().find(|row| row.check == "key").unwrap();
    assert_eq!(key.outcome, Outcome::Warning, "{}", key.detail);
    let unchecked = rows.iter().find(|row| row.check == "candidate").unwrap();
    assert_eq!(unchecked.outcome, Outcome::Error, "{}", unchecked.detail);
    assert_eq!(unchecked.subject, "tool tool-typo");
}

#[test]
fn a_kept_package_from_a_kept_repository_that_is_down_only_warns() {
    let stub = Stub::always(Reply::Status(503));
    let app = apt(r#""tool""#);
    let (root, base) = checkout(&app, &app, &manifest(&stub.url("/gpg")));

    let rows = rows(root.path(), &base);

    assert!(!rows.is_empty());
    assert!(rows.iter().all(|row| row.outcome == Outcome::Warning), "{rows:?}");
}

#[test]
fn an_app_new_on_the_branch_or_a_base_that_cant_be_read_counts_as_changed() {
    let app = download("https://example.com/a/tool-");
    let (root, base) = checkout(&app, &app, "");
    let kept = Changed::read(root.path(), &base, &Manifest::default());
    let unreadable = Changed::read(root.path(), "no-such-commit", &Manifest::default());
    let catalog = Catalog::load(&root.path().join(CATALOG)).unwrap();
    let kind = &catalog.get("tool").unwrap().actions[0].kind;

    assert!(!kept.action("tool", kind));
    assert!(kept.action("another", kind));
    assert!(unreadable.action("tool", kind));
}
