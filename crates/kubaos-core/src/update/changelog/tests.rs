use super::Change;
use super::ChangeKind;
use super::Changelog;
use super::Section;
use super::github_repository;
use crate::update::fixture;
use crate::update::git::GitRefs;

fn parsed(message: &str) -> (ChangeKind, Option<String>, String, bool, Option<u32>) {
    let change = Change::parse("c0ffee", message, None);
    (change.kind, change.scope, change.subject, change.breaking, change.pr)
}

#[test]
fn a_conventional_subject_gives_its_kind_scope_and_description() {
    assert_eq!(
        parsed("feat(menu): run a shell command"),
        (
            ChangeKind::Feat,
            Some("menu".into()),
            "run a shell command".into(),
            false,
            None
        )
    );
    assert_eq!(
        parsed("perf: cache the icon lookups"),
        (ChangeKind::Perf, None, "cache the icon lookups".into(), false, None)
    );
}

#[test]
fn a_bang_or_a_breaking_change_footer_marks_a_change_breaking() {
    assert!(parsed("fix(session)!: restart a dead shell").3);
    assert!(parsed("refactor!: drop the old config").3);
    assert!(parsed("refactor: move the config\n\nBREAKING CHANGE: the old path is gone").3);
    assert!(parsed("Rework the bar\n\nBREAKING-CHANGE: the old modules are gone").3);
    assert!(!parsed("docs: describe BREAKING CHANGE: footers").3);
}

#[test]
fn a_subject_that_is_not_conventional_is_other_and_whole() {
    for subject in [
        "Update the README",
        "wip(menu): try a thing",
        "feat(): no scope inside",
        "feat:",
        "Revert \"feat(menu): run a shell command\"",
    ] {
        assert_eq!(
            parsed(subject),
            (ChangeKind::Other, None, subject.into(), false, None),
            "{subject}"
        );
    }
}

#[test]
fn a_squash_merge_subject_names_its_pull_request() {
    assert_eq!(
        parsed("feat(bar): warn before an X11 title (#393)"),
        (
            ChangeKind::Feat,
            Some("bar".into()),
            "warn before an X11 title".into(),
            false,
            Some(393)
        )
    );
    assert_eq!(Change::parse("c", "fix: x (#5)", Some(9)).pr, Some(9));
}

#[test]
fn each_kind_is_listed_under_its_section_and_breaking_ones_under_none() {
    let section = |message: &str| Change::parse("c", message, None).section();

    assert_eq!(section("feat: a"), Some(Section::New));
    assert_eq!(section("fix: a"), Some(Section::Fixed));
    assert_eq!(section("perf: a"), Some(Section::Faster));
    assert_eq!(section("build: a"), Some(Section::Build));
    assert_eq!(section("style: a"), Some(Section::Other));
    assert_eq!(section("A plain subject"), Some(Section::Other));
    for internal in ["docs: a", "test: a", "ci: a", "chore: a"] {
        assert_eq!(section(internal), Some(Section::Internal), "{internal}");
    }
    assert_eq!(section("chore!: a"), None);
}

#[test]
fn github_remotes_over_https_and_ssh_give_the_repository_and_others_none() {
    let repo = Some("https://github.com/SuplenC/kubaos".to_owned());

    assert_eq!(github_repository("https://github.com/SuplenC/kubaos.git"), repo);
    assert_eq!(github_repository("https://github.com/SuplenC/kubaos/"), repo);
    assert_eq!(github_repository("git@github.com:SuplenC/kubaos.git"), repo);
    assert_eq!(github_repository("ssh://git@github.com/SuplenC/kubaos"), repo);
    assert_eq!(github_repository("https://gitlab.com/SuplenC/kubaos.git"), None);
    assert_eq!(github_repository("/srv/git/kubaos.git"), None);
    assert_eq!(github_repository("https://github.com/SuplenC"), None);
    assert_eq!(github_repository("https://github.com/a/b/c"), None);
}

#[test]
fn the_limit_leaves_breaking_changes_in_and_counts_what_it_leaves_out() {
    let changes = ["feat: a", "fix!: b", "fix: c", "docs: d", "perf: e"]
        .into_iter()
        .map(|m| Change::parse("c", m, Some(1)))
        .collect();

    let log = Changelog::new(changes, 1, Some("https://github.com/o/r".into()));

    let subjects: Vec<&str> = log.changes.iter().map(|c| c.subject.as_str()).collect();
    assert_eq!(subjects, ["a", "b"]);
    assert_eq!((log.more_changes, log.internal_changes, log.count()), (2, 1, 4));
    assert_eq!(log.pr_url(7).as_deref(), Some("https://github.com/o/r/pull/7"));
}

#[test]
fn merges_collapse_into_the_commits_their_pull_requests_brought_in() {
    let tmp = tempfile::tempdir().unwrap();
    fixture::origin(tmp.path(), "0.1.0\n", &[]);
    let work = tmp.path().join("work");
    let from = fixture::git(&work, &["rev-parse", "HEAD"]);
    fixture::commit(&work, "docs: describe the groups", &[("docs", "1")], None);
    fixture::git(&work, &["switch", "-q", "-c", "session"]);
    fixture::commit(&work, "fix(session)!: restart a dead shell", &[("session", "1")], None);
    fixture::git(&work, &["switch", "-q", "main"]);
    fixture::git(&work, &["switch", "-q", "-c", "menu"]);
    fixture::commit(&work, "feat(menu): run a shell command", &[("menu", "1")], None);
    fixture::commit(&work, "fix(menu): quote the command", &[("menu", "2")], None);
    fixture::git(&work, &["switch", "-q", "main"]);
    let merge = ["merge", "-q", "--no-ff", "-m"];
    fixture::git(
        &work,
        &[&merge[..], &["Merge pull request #396 from o/menu", "menu"]].concat(),
    );
    fixture::git(&work, &["switch", "-q", "session"]);
    fixture::git(
        &work,
        &[&merge[..], &["Merge branch 'main' into session", "main"]].concat(),
    );
    fixture::git(&work, &["switch", "-q", "main"]);
    fixture::git(
        &work,
        &[&merge[..], &["Merge pull request #395 from o/session", "session"]].concat(),
    );
    fixture::git(&work, &["remote", "set-url", "origin", "git@github.com:o/r.git"]);

    let log = GitRefs::new(&work, &tmp.path().join("missing")).changelog(&from, "HEAD", 20);

    let changes: Vec<_> = log
        .changes
        .iter()
        .map(|c| (c.kind, c.subject.as_str(), c.breaking, c.pr))
        .collect();
    assert_eq!(
        changes,
        [
            (ChangeKind::Fix, "restart a dead shell", true, Some(395)),
            (ChangeKind::Fix, "quote the command", false, Some(396)),
            (ChangeKind::Feat, "run a shell command", false, Some(396)),
        ]
    );
    assert_eq!(log.internal_changes, 1);
    assert_eq!(log.repository.as_deref(), Some("https://github.com/o/r"));
    assert!(GitRefs::new(&work, &work).changelog("HEAD", "HEAD", 20).is_empty());
}
