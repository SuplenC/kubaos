use std::path::PathBuf;

use super::Channel;
use super::HistoryEntry;
use super::Installed;
use super::RecordError;
use super::parse_history;
use crate::update::version::Version;

fn installed() -> Installed {
    Installed {
        version: Version::parse("0.1.0").unwrap(),
        codename: Some("shinjuku".into()),
        commit: "1a2b3c4d5e6f".into(),
        channel: Channel::Edge,
        date: "2026-10-22T10:00:00Z".into(),
    }
}

#[test]
fn a_release_record_reads_back_as_written() {
    let record = installed();

    let back = Installed::parse(&record.to_text()).unwrap();

    assert_eq!(back, record);
    assert_eq!(record.describe(), "Shinjuku 0.1.0 (edge, 1a2b3c4)");
}

#[test]
fn a_record_without_a_commit_is_refused_and_unknown_keys_are_not() {
    let missing = Installed::parse("version=0.1.0\nchannel=stable\n");
    let extra = Installed::parse("version=0.1.0\nchannel=stable\ncommit=abc\nfuture=1\n").unwrap();

    assert!(matches!(missing, Err(RecordError::Missing("commit"))));
    assert_eq!(extra.codename, None);
    assert_eq!(extra.channel, Channel::Stable);
}

#[test]
fn a_channel_is_stable_or_edge() {
    assert_eq!("edge\n".parse::<Channel>().unwrap(), Channel::Edge);
    assert_eq!("stable".parse::<Channel>().unwrap(), Channel::Stable);
    assert!("beta".parse::<Channel>().is_err());
}

#[test]
fn history_lines_read_back_and_junk_is_skipped() {
    let entry = HistoryEntry {
        date: "2026-10-22T10:00:00Z".into(),
        from_commit: "aaa".into(),
        from_version: "0.1.0".into(),
        to_commit: "bbb".into(),
        to_version: "0.1.1".into(),
        lock: Some(PathBuf::from("/home/kuba/.local/state/kubaos/locks/1.lock")),
    };
    let no_lock = HistoryEntry { lock: None, ..entry.clone() };
    let text = format!("{}garbage\n\n{}", entry.line(), no_lock.line());

    assert_eq!(parse_history(&text), [entry, no_lock]);
}
