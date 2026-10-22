use std::collections::BTreeMap;

use super::Edge;
use super::NO_KEYS_EDGE;
use super::NO_KEYS_STABLE;
use super::Policy;
use super::PolicyError;
use super::Refs;
use super::RefsError;
use super::Stable;
use super::Tag;
use super::Verified;
use crate::update::record::Channel;
use crate::update::record::Installed;
use crate::update::version::Version;

/// Refs a test lays out: a main head, tags, and how each verifies.
#[derive(Default)]
struct FakeRefs {
    main: Option<String>,
    tags: Vec<Tag>,
    verdicts: BTreeMap<String, Verified>,
    files: BTreeMap<(String, String), String>,
}

impl FakeRefs {
    fn tag(mut self, name: &str, commit: &str, verdict: Verified) -> Self {
        self.tags.push(Tag {
            name: name.into(),
            commit: commit.into(),
        });
        self.verdicts.insert(name.into(), verdict);
        self
    }

    fn file(mut self, commit: &str, path: &str, text: &str) -> Self {
        self.files.insert((commit.into(), path.into()), text.into());
        self
    }
}

impl Refs for FakeRefs {
    fn main_head(&self) -> Result<Option<String>, RefsError> {
        Ok(self.main.clone())
    }

    fn tags(&self) -> Result<Vec<Tag>, RefsError> {
        Ok(self.tags.clone())
    }

    fn verify_commit(&self, commit: &str) -> Result<Verified, RefsError> {
        Ok(self.verdicts.get(commit).copied().unwrap_or(Verified::Bad))
    }

    fn verify_tag(&self, name: &str) -> Result<Verified, RefsError> {
        Ok(self.verdicts.get(name).copied().unwrap_or(Verified::Bad))
    }

    fn file_at(&self, commit: &str, path: &str) -> Option<String> {
        self.files.get(&(commit.into(), path.into())).cloned()
    }
}

fn installed(version: &str, commit: &str, channel: Channel) -> Installed {
    Installed {
        version: Version::parse(version).unwrap(),
        codename: Some("shinjuku".into()),
        commit: commit.into(),
        channel,
        date: String::new(),
    }
}

fn edge_refs(head: &str, verdict: Verified) -> FakeRefs {
    let mut refs = FakeRefs {
        main: Some(head.into()),
        ..FakeRefs::default()
    };
    refs.verdicts.insert(head.into(), verdict);
    refs.file(head, "VERSION", "0.1.0\n")
        .file(head, "CODENAME", "shinjuku\n")
}

#[test]
fn edge_moves_to_a_new_signed_head_of_main() {
    let refs = edge_refs("bbbbbbbbbb", Verified::Good);

    let pick = Edge.pick(&installed("0.1.0", "aaaa", Channel::Edge), &refs).unwrap();

    let target = pick.update.unwrap();
    assert_eq!(target.commit, "bbbbbbbbbb");
    assert_eq!(target.label, "main at bbbbbbb (0.1.0)");
    assert_eq!(target.codename.as_deref(), Some("shinjuku"));
    assert!(pick.notes.is_empty());
}

#[test]
fn edge_on_mains_head_has_nothing_to_do() {
    let refs = edge_refs("aaaa", Verified::Bad);

    let pick = Edge.pick(&installed("0.1.0", "aaaa", Channel::Edge), &refs).unwrap();

    assert_eq!(pick.update, None);
}

#[test]
fn edge_refuses_a_head_signed_by_no_allowed_key() {
    let refs = edge_refs("bbbb", Verified::Bad);

    let err = Edge
        .pick(&installed("0.1.0", "aaaa", Channel::Edge), &refs)
        .unwrap_err();

    assert!(matches!(err, PolicyError::Unsigned { commit } if commit == "bbbb"));
}

#[test]
fn edge_without_allowed_signers_takes_the_head_with_one_warning() {
    let refs = edge_refs("bbbb", Verified::NoKeys);

    let pick = Edge.pick(&installed("0.1.0", "aaaa", Channel::Edge), &refs).unwrap();

    assert_eq!(pick.update.unwrap().commit, "bbbb");
    assert_eq!(pick.notes, [NO_KEYS_EDGE]);
}

#[test]
fn stable_takes_the_newest_verified_patch_of_its_series() {
    let refs = FakeRefs::default()
        .tag("v0.1.0", "c0", Verified::Good)
        .tag("v0.1.1", "c1", Verified::Good)
        .tag("v0.1.3", "c3", Verified::Bad)
        .tag("v0.1.2", "c2", Verified::Good)
        .tag("v0.1.4-rc.1", "c4", Verified::Good)
        .tag("v0.2.0", "d0", Verified::Good)
        .tag("v0.2.1", "d1", Verified::Good)
        .tag("v0.3.0", "e0", Verified::Good)
        .file("d1", "CODENAME", "akihabara\n");

    let pick = Stable.pick(&installed("0.1.0", "c0", Channel::Stable), &refs).unwrap();

    assert_eq!(pick.update.unwrap().commit, "c2");
    let upgrade = pick.upgrade.unwrap();
    assert_eq!((upgrade.commit.as_str(), upgrade.label.as_str()), ("d1", "0.2.1"));
    assert_eq!(upgrade.codename.as_deref(), Some("akihabara"));
    assert_eq!(pick.notes, ["v0.1.3 is not signed by a KubaOS key: skipped"]);
}

#[test]
fn stable_without_tags_or_keys_updates_only_packages() {
    let none = FakeRefs::default();
    let unkeyed = FakeRefs::default().tag("v0.1.1", "c1", Verified::NoKeys);

    let no_tags = Stable.pick(&installed("0.1.0", "c0", Channel::Stable), &none).unwrap();
    let no_keys = Stable
        .pick(&installed("0.1.0", "c0", Channel::Stable), &unkeyed)
        .unwrap();

    assert_eq!(no_tags, super::Pick::default());
    assert_eq!((no_keys.update, no_keys.upgrade), (None, None));
    assert_eq!(no_keys.notes, [NO_KEYS_STABLE]);
}
