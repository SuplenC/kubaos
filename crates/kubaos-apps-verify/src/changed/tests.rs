use std::path::PathBuf;

use super::*;
use crate::check;
use crate::check::Context;
use crate::check::Outcome;
use crate::stub::Reply;
use crate::stub::Stub;

/// The repository's own Argo CD app, one `download` action of two architectures.
fn argocd() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/argocd")
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = kubaos_git::at(root)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@example.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// A checkout holding Argo CD with its downloads at `base_url`, committed, then at `head_url` in
/// the working tree; its root and the base commit.
fn checkout(base_url: &str, head_url: &str) -> (tempfile::TempDir, String) {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("apps/argocd");
    fs::create_dir_all(&app).unwrap();
    fs::copy(argocd().join("argocd.svg"), app.join("argocd.svg")).unwrap();
    let text = fs::read_to_string(argocd().join("app.toml")).unwrap();
    let pinned = |url: &str| {
        text.replace(
            "https://github.com/argoproj/argo-cd/releases/download/v3.6.2/argocd-linux-",
            url,
        )
    };
    fs::write(app.join("app.toml"), pinned(base_url)).unwrap();
    git(root.path(), &["init", "-q"]);
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-q", "-m", "base"]);
    let base = git(root.path(), &["rev-parse", "HEAD"]);
    fs::write(app.join("app.toml"), pinned(head_url)).unwrap();
    (root, base)
}

/// The rows of checking Argo CD in `root` against what changed since `base`.
fn rows(root: &Path, base: &str) -> Vec<check::Row> {
    let catalog = Catalog::load(&root.join(CATALOG)).unwrap();
    let manifest = Manifest::default();
    let changed = Changed::read(root, base, &manifest);
    let context = Context::new(root.to_path_buf(), manifest, "resolute".to_owned()).with_changed(changed);
    check::app(&context, catalog.get("argocd").unwrap())
}

/// An HTTPS URL on loopback whose TLS handshake fails, an outage, at once.
fn down(stub: &Stub) -> String {
    stub.url("/argocd-linux-").replacen("http://", "https://", 1)
}

#[test]
fn an_outage_on_a_download_the_branch_changed_is_an_error() {
    let stub = Stub::always(Reply::Body(Vec::new()));
    let (root, base) = checkout("https://example.com/old/argocd-linux-", &down(&stub));

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
    let url = down(&stub);
    let (root, base) = checkout(&url, &url);

    let rows = rows(root.path(), &base);

    assert!(rows.iter().any(|row| row.check == "sha256"));
    for row in rows.iter().filter(|row| row.check == "sha256") {
        assert_eq!(row.outcome, Outcome::Warning, "{}", row.detail);
    }
}

#[test]
fn an_app_new_on_the_branch_or_a_base_that_cant_be_read_counts_as_changed() {
    let (root, base) = checkout(
        "https://example.com/a/argocd-linux-",
        "https://example.com/a/argocd-linux-",
    );
    let kept = Changed::read(root.path(), &base, &Manifest::default());
    let unreadable = Changed::read(root.path(), "no-such-commit", &Manifest::default());
    let catalog = Catalog::load(&root.path().join(CATALOG)).unwrap();
    let kind = &catalog.get("argocd").unwrap().actions[0].kind;

    assert!(!kept.action("argocd", kind));
    assert!(kept.action("another", kind));
    assert!(unreadable.action("argocd", kind));
}
