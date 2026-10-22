use super::GitRefs;
use super::RemoteRefs;
use crate::update::fixture;
use crate::update::policy::Refs;
use crate::update::policy::Verified;

#[test]
fn a_signed_commit_and_tag_verify_and_unsigned_ones_do_not() {
    let tmp = tempfile::tempdir().unwrap();
    let key = fixture::key(&tmp.path().join("keys"));
    let origin = fixture::origin(tmp.path(), "0.1.0\n", &[]);
    let work = tmp.path().join("work");
    let unsigned = fixture::git(&work, &["rev-parse", "HEAD"]);
    fixture::tag(&work, "v0.1.0", None);
    let signed = fixture::commit(&work, "fix", &[("a", "1")], Some(&key));
    fixture::tag(&work, "v0.1.1", Some(&key));
    fixture::git(&work, &["push", "-q", "origin", "main", "--tags"]);
    let checkout = tmp.path().join("kubaos");
    fixture::checkout(&origin, &checkout);

    let refs = GitRefs::new(&checkout, &key.allowed_signers);
    let unkeyed = GitRefs::new(&checkout, &tmp.path().join("missing"));

    assert_eq!(refs.main_head().unwrap().as_deref(), Some(signed.as_str()));
    assert_eq!(refs.verify_commit(&signed).unwrap(), Verified::Good);
    assert_eq!(refs.verify_commit(&unsigned).unwrap(), Verified::Bad);
    assert_eq!(refs.verify_tag("v0.1.1").unwrap(), Verified::Good);
    assert_eq!(refs.verify_tag("v0.1.0").unwrap(), Verified::Bad);
    assert_eq!(unkeyed.verify_commit(&signed).unwrap(), Verified::NoKeys);
    let mut tags = refs.tags().unwrap();
    tags.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(
        tags.iter().map(|t| (t.name.as_str(), t.commit.as_str())).collect::<Vec<_>>(),
        [("v0.1.0", unsigned.as_str()), ("v0.1.1", signed.as_str())]
    );
    assert_eq!(refs.file_at(&unsigned, "VERSION").as_deref(), Some("0.1.0\n"));
    assert_eq!(refs.log(&unsigned, &signed, 10).len(), 1);
}

#[test]
fn ls_remote_listings_give_main_and_the_peeled_commit_of_each_tag() {
    let listing = "aaaa\trefs/heads/main\n\
                   t111\trefs/tags/v0.1.0\n\
                   c111\trefs/tags/v0.1.0^{}\n\
                   c222\trefs/tags/v0.1.1\n";
    let tmp = tempfile::tempdir().unwrap();

    let refs = RemoteRefs::parse(listing, GitRefs::new(tmp.path(), tmp.path()));

    assert_eq!(refs.main_head().unwrap().as_deref(), Some("aaaa"));
    let tags: Vec<_> = refs.tags().unwrap().into_iter().map(|t| (t.name, t.commit)).collect();
    assert_eq!(tags, [("v0.1.0".into(), "c111".into()), ("v0.1.1".into(), "c222".into())]);
    assert_eq!(refs.verify_tag("v0.1.1").unwrap(), Verified::Unchecked);
}
