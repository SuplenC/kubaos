use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use super::Checkout;
use super::LeaveError;
use super::Notice;
use super::PlanError;
use super::Preview;
use super::PreviewFile;
use super::Request;
use super::ResolveError;
use super::flake_ref;
use super::leave;
use super::plan;
use super::resolve;
use super::worktree_for;
use crate::switch::Host;
use crate::switch::HostError;
use crate::switch::JobId;
use crate::switch::Target;

/// A host whose checkouts are described and evaluated from its lists, recording what it was
/// asked.
#[derive(Default)]
struct FakeHost {
    not_checkouts: Vec<PathBuf>,
    not_evaluating: Vec<PathBuf>,
    evaluated: RefCell<Vec<PathBuf>>,
}

impl Host for FakeHost {
    fn worker_unit(&self, job: &JobId) -> String {
        format!("fake-{job}")
    }

    fn start_worker(&self, _job: &JobId) -> Result<(), HostError> {
        Ok(())
    }

    fn worker_alive(&self, _unit: &str) -> Result<bool, HostError> {
        Ok(true)
    }

    fn switch(&self, _log: File, _from: Option<&Path>) -> Result<(), HostError> {
        panic!("planning never switches")
    }

    fn describe(&self, path: &Path) -> Result<Checkout, HostError> {
        if self.not_checkouts.iter().any(|p| p == path) {
            return Err(HostError::exited("git", Some(128)));
        }
        Ok(Checkout {
            branch: Some("feature/x".to_owned()),
            commit: "abcdef0123".to_owned(),
            dirty: false,
        })
    }

    fn evaluate(&self, path: &Path) -> Result<(), HostError> {
        self.evaluated.borrow_mut().push(path.to_path_buf());
        if self.not_evaluating.iter().any(|p| p == path) {
            return Err(HostError::exited("nix", Some(1)));
        }
        Ok(())
    }
}

fn preview(path: &str) -> Preview {
    Preview {
        path: path.into(),
        branch: Some("feature/a".to_owned()),
        commit: "0123456789abcdef".to_owned(),
        dirty: true,
        since: 1_793_227_524_000,
    }
}

fn from(path: &str) -> Request {
    Request::From(path.into())
}

#[test]
fn a_preview_file_round_trips_and_empty_or_missing_is_no_preview() {
    let tmp = tempfile::tempdir().unwrap();
    let file = PreviewFile::new(tmp.path().join("state/preview.json"));
    assert_eq!(file.read().unwrap(), None);

    file.write(&preview("/w/a")).unwrap();
    assert_eq!(file.read().unwrap(), Some(preview("/w/a")));

    file.clear().unwrap();
    assert_eq!(file.read().unwrap(), None);
    assert!(file.path().exists(), "cleared, the file stays for its watchers");
}

#[test]
fn a_preview_file_that_is_not_one_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let file = PreviewFile::new(tmp.path().join("preview.json"));
    fs::write(file.path(), "{ not json").unwrap();

    assert!(file.read().is_err());
}

#[test]
fn a_preview_names_its_branch_or_else_its_short_commit() {
    let mut detached = preview("/w/a");
    assert_eq!(detached.name(), "feature/a");

    detached.branch = None;

    assert_eq!(detached.name(), "0123456");
}

#[test]
fn a_plain_switch_without_a_preview_is_the_home_as_it_stands() {
    let plan = plan(Request::Plain, None, &FakeHost::default()).unwrap();

    assert_eq!(plan.target, Target::Current);
    assert_eq!(plan.notice, None);
}

#[test]
fn a_plain_switch_during_a_preview_is_refused_naming_both_ways_out() {
    let active = preview("/w/a");

    let err = plan(Request::Plain, Some(&active), &FakeHost::default()).unwrap_err();

    let message = err.to_string();
    assert!(matches!(err, PlanError::Active { .. }), "{err:?}");
    assert!(message.contains("feature/a"), "{message}");
    assert!(message.contains("kubaos switch --from /w/a"), "{message}");
    assert!(message.contains("kubaos switch --release"), "{message}");
}

#[test]
fn a_switch_to_the_release_leaves_the_preview_and_says_so() {
    let active = preview("/w/a");

    let plan = plan(Request::Release, Some(&active), &FakeHost::default()).unwrap();

    assert_eq!(plan.target, Target::Release);
    assert_eq!(plan.notice, Some(Notice::Leaves(active)));
}

#[test]
fn a_switch_to_the_release_without_a_preview_still_switches_and_says_none_was_on() {
    let plan = plan(Request::Release, None, &FakeHost::default()).unwrap();

    assert_eq!(plan.target, Target::Release);
    assert_eq!(plan.notice, Some(Notice::NoPreview));
}

#[test]
fn a_checkout_is_evaluated_before_its_switch_is_planned() {
    let host = FakeHost::default();

    let plan = plan(from("/w/b"), None, &host).unwrap();

    assert_eq!(plan.target, Target::Checkout { path: "/w/b".into() });
    assert_eq!(plan.notice, None);
    assert_eq!(*host.evaluated.borrow(), [PathBuf::from("/w/b")]);
}

#[test]
fn a_checkout_that_does_not_evaluate_is_refused() {
    let host = FakeHost {
        not_evaluating: vec!["/w/broken".into()],
        ..FakeHost::default()
    };

    let err = plan(from("/w/broken"), None, &host).unwrap_err();

    assert!(matches!(err, PlanError::DoesNotEvaluate { .. }), "{err:?}");
    assert!(err.to_string().contains("nothing was switched"));
}

#[test]
fn a_folder_that_is_no_checkout_is_refused_before_evaluating() {
    let host = FakeHost {
        not_checkouts: vec!["/tmp/x".into()],
        ..FakeHost::default()
    };

    let err = plan(from("/tmp/x"), None, &host).unwrap_err();

    assert!(matches!(err, PlanError::NotACheckout { .. }), "{err:?}");
    assert!(host.evaluated.borrow().is_empty());
}

#[test]
fn one_preview_at_a_time_a_second_checkout_replaces_the_first_and_says_so() {
    let active = preview("/w/a");

    let plan = plan(from("/w/b"), Some(&active), &FakeHost::default()).unwrap();

    assert_eq!(plan.target, Target::Checkout { path: "/w/b".into() });
    assert_eq!(plan.notice, Some(Notice::Replaces(active)));
}

#[test]
fn the_same_checkout_again_rebuilds_the_preview_without_a_notice() {
    let active = preview("/w/a");

    let plan = plan(from("/w/a"), Some(&active), &FakeHost::default()).unwrap();

    assert_eq!(plan.notice, None);
}

const WORKTREES: &str = "worktree /h/Projects/kubaos
HEAD 1111111111111111111111111111111111111111
branch refs/heads/main

worktree /h/Projects/kubaos/.claude/worktrees/agent-1
HEAD 2222222222222222222222222222222222222222
branch refs/heads/feature/issue-228-live-preview

worktree /h/Projects/kubaos-detached
HEAD 3333333333333333333333333333333333333333
detached

";

#[test]
fn a_branch_is_found_in_the_worktree_listing() {
    assert_eq!(
        worktree_for(WORKTREES, "feature/issue-228-live-preview"),
        Some(PathBuf::from("/h/Projects/kubaos/.claude/worktrees/agent-1"))
    );
    assert_eq!(
        worktree_for(WORKTREES, "main"),
        Some(PathBuf::from("/h/Projects/kubaos"))
    );
    assert_eq!(worktree_for(WORKTREES, "feature/issue"), None);
    assert_eq!(worktree_for(WORKTREES, "detached"), None);
}

#[test]
fn a_folder_given_to_from_is_taken_as_it_is_without_listing_worktrees() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join("checkout")).unwrap();

    let path = resolve("checkout", tmp.path(), Path::new("/nowhere"), |_| {
        panic!("a folder needs no worktree listing")
    })
    .unwrap();

    assert_eq!(path, tmp.path().join("checkout").canonicalize().unwrap());
}

#[test]
fn a_branch_given_to_from_is_the_main_checkouts_worktree_for_it() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("kubaos");
    fs::create_dir(&repo).unwrap();

    let path = resolve("feature/issue-228-live-preview", tmp.path(), &repo, |listed| {
        assert_eq!(listed, repo);
        Ok(WORKTREES.to_owned())
    })
    .unwrap();

    assert_eq!(path, Path::new("/h/Projects/kubaos/.claude/worktrees/agent-1"));
}

#[test]
fn a_branch_no_worktree_has_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("kubaos");
    fs::create_dir(&repo).unwrap();

    let err = resolve("feature/gone", tmp.path(), &repo, |_| Ok(WORKTREES.to_owned())).unwrap_err();

    assert!(matches!(err, ResolveError::NoCheckout { .. }), "{err:?}");
    assert!(err.to_string().contains("feature/gone"));
}

#[test]
fn without_a_main_checkout_a_branch_is_refused() {
    let tmp = tempfile::tempdir().unwrap();

    let err = resolve("feature/x", tmp.path(), &tmp.path().join("kubaos"), |_| {
        panic!("no main checkout to list")
    })
    .unwrap_err();

    assert!(matches!(err, ResolveError::NoCheckout { .. }), "{err:?}");
}

#[test]
fn leaving_without_a_preview_switches_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let file = PreviewFile::new(tmp.path().join("preview.json"));

    let left = leave(&file, |_| panic!("no preview, no switch")).unwrap();

    assert_eq!(left, None);
}

#[test]
fn leaving_a_preview_switches_back_to_the_release_first() {
    let tmp = tempfile::tempdir().unwrap();
    let file = PreviewFile::new(tmp.path().join("preview.json"));
    file.write(&preview("/w/a")).unwrap();

    let left = leave(&file, |_| {
        file.clear().unwrap();
        Ok(())
    })
    .unwrap();

    assert_eq!(left, Some(preview("/w/a")));
}

#[test]
fn a_failed_switch_back_stops_the_update() {
    let tmp = tempfile::tempdir().unwrap();
    let file = PreviewFile::new(tmp.path().join("preview.json"));
    file.write(&preview("/w/a")).unwrap();

    let err = leave(&file, |_| Err("home-manager exited with code 1".to_owned())).unwrap_err();

    assert!(matches!(err, LeaveError::Switch { .. }), "{err:?}");
    assert!(err.to_string().contains("feature/a"));
}

#[test]
fn a_preview_still_recorded_after_switching_back_stops_the_update() {
    let tmp = tempfile::tempdir().unwrap();
    let file = PreviewFile::new(tmp.path().join("preview.json"));
    file.write(&preview("/w/a")).unwrap();

    let err = leave(&file, |_| Ok(())).unwrap_err();

    assert!(matches!(err, LeaveError::StillActive { .. }), "{err:?}");
}

#[test]
fn a_checkouts_flake_reference_is_git_and_escapes_what_a_url_reads() {
    assert_eq!(
        flake_ref(Path::new("/h/Projects/kubaos")),
        "git+file:///h/Projects/kubaos"
    );
    assert_eq!(flake_ref(Path::new("/h/my work#1?")), "git+file:///h/my%20work%231%3F");
}
