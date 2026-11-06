use std::os::unix::fs::PermissionsExt;

use super::*;

fn sample(at: i64) -> Entry {
    Entry::Sample(Sample {
        at,
        percent: Some(64.0),
        stop: "power-saver".to_owned(),
        ..Sample::default()
    })
}

fn full_sample() -> Entry {
    Entry::Sample(Sample {
        at: 1_793_000_000,
        percent: Some(64.0),
        wh: Some(51.2),
        watts: Some(7.1),
        stop: "power-saver".to_owned(),
        lid_closed: Some(true),
        panel_on: Some(false),
        brightness: Some(40),
        keep_awake: true,
        docked: false,
        cpu: Some(3.2),
        wakers: Some(Wakers {
            secs: 600,
            irqs: vec![("LOC (Local timer interrupts)".to_owned(), 412.5)],
            procs: vec![("qs".to_owned(), 12.3)],
        }),
    })
}

fn sleep() -> Entry {
    Entry::Sleep(Sleep {
        at: 1_793_000_000,
        end: 1_793_005_100,
        trigger: Some("lid".to_owned()),
        percent_before: Some(100.0),
        percent_after: Some(96.0),
        wh_before: Some(80.0),
        wh_after: Some(76.8),
        charger: false,
        watts: Some(2.26),
        percent_per_hour: Some(2.82),
        hw_sleep_us: Some(0),
        last_hw_sleep_us: Some(0),
        hw_share: Some(0.0),
    })
}

fn event() -> Entry {
    Entry::Event(Event {
        at: 1_793_000_000,
        what: What::Stop,
        detail: Some("power-saver".to_owned()),
        percent: Some(64.0),
    })
}

fn file() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("power").join(FILE);
    (dir, path)
}

#[test]
fn every_entry_survives_the_round_trip_through_its_line() {
    for entry in [full_sample(), sample(1), sleep(), event()] {
        let line = entry.line();

        let read: Entry = serde_json::from_str(&line).expect("reads back");

        assert_eq!(read, entry);
        assert_eq!(line.matches('\n').count(), 1);
    }
}

#[test]
fn the_lines_are_the_ones_the_contract_documents() {
    assert!(
        event()
            .line()
            .starts_with(r#"{"type":"event","at":1793000000,"what":"stop","detail":"power-saver""#)
    );
    assert!(
        sleep()
            .line()
            .starts_with(r#"{"type":"sleep","at":1793000000,"end":1793005100,"trigger":"lid""#)
    );
    assert!(
        sample(5)
            .line()
            .starts_with(r#"{"type":"sample","at":5,"percent":64.0,"stop":"power-saver""#)
    );
}

#[test]
fn a_max_hw_sleep_sized_number_reads_back_whole() {
    let entry = Entry::Sleep(Sleep {
        hw_sleep_us: Some(u64::MAX),
        ..Sleep::default()
    });

    let read: Entry = serde_json::from_str(&entry.line()).expect("reads back");

    assert_eq!(read, entry);
}

#[test]
fn appends_make_the_folder_and_keep_the_order_and_the_file_private() {
    let (_dir, path) = file();

    append(&path, &sample(1)).expect("the first");
    append(&path, &sample(2)).expect("the second");

    assert_eq!(read(&path, 0), [sample(1), sample(2)]);
    let mode = fs::metadata(&path).expect("stat").permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn a_line_that_does_not_read_is_skipped_and_a_missing_file_has_none() {
    let (dir, path) = file();
    assert_eq!(read(&path, 0), []);
    fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
    let later = r#"{"type":"from-a-later-release","at":3}"#;
    fs::write(
        &path,
        format!("{}{later}\n{{\"type\":\"sam\n{}", sample(1).line(), sample(2).line()),
    )
    .expect("write");

    assert_eq!(read(&path, 0), [sample(1), sample(2)]);
    drop(dir);
}

#[test]
fn reading_since_leaves_out_what_is_older() {
    let (_dir, path) = file();
    append(&path, &sample(1)).expect("append");
    append(&path, &sample(5)).expect("append");

    assert_eq!(read(&path, 5), [sample(5)]);
}

#[test]
fn a_tail_starts_at_the_first_whole_line() {
    let (_dir, path) = file();
    for at in 1..=3 {
        append(&path, &sample(at)).expect("append");
    }
    let line = sample(3).line().len() as u64;

    assert_eq!(read_tail(&path, line + 5), [sample(3)]);
    assert_eq!(read_tail(&path, 1 << 20), [sample(1), sample(2), sample(3)]);
}

#[test]
fn a_line_a_crash_cut_is_ended_so_the_next_append_is_whole() {
    let (_dir, path) = file();
    append(&path, &sample(1)).expect("append");
    let mut cut = OpenOptions::new().append(true).open(&path).expect("open");
    cut.write_all(b"{\"type\":\"sam").expect("cut");

    end_last_line(&path).expect("end it");
    end_last_line(&path).expect("nothing more to do");
    append(&path, &sample(2)).expect("append");

    assert_eq!(read(&path, 0), [sample(1), sample(2)]);
    assert_eq!(fs::read_to_string(&path).expect("read").lines().count(), 3);
}

#[test]
fn ending_the_last_line_of_a_missing_or_empty_file_does_nothing() {
    let (_dir, path) = file();

    end_last_line(&path).expect("missing");
    fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
    fs::write(&path, "").expect("write");
    end_last_line(&path).expect("empty");

    assert_eq!(fs::read_to_string(&path).expect("read"), "");
}

#[test]
fn a_file_within_the_retention_and_its_slack_is_left_untouched() {
    let (_dir, path) = file();
    let now = 1_000 * DAY;
    append(&path, &sample(now - (RETENTION_DAYS + PRUNE_SLACK_DAYS) * DAY)).expect("append");
    let before = fs::metadata(&path).expect("stat").modified().expect("a time");
    let inode = std::os::unix::fs::MetadataExt::ino(&fs::metadata(&path).expect("stat"));

    assert!(!prune(&path, now).expect("prune"));

    assert_eq!(fs::metadata(&path).expect("stat").modified().expect("a time"), before);
    assert_eq!(
        std::os::unix::fs::MetadataExt::ino(&fs::metadata(&path).expect("stat")),
        inode
    );
}

#[test]
fn past_the_slack_only_what_is_past_the_retention_goes() {
    let (_dir, path) = file();
    let now = 1_000 * DAY;
    let older = now - (RETENTION_DAYS + PRUNE_SLACK_DAYS + 1) * DAY;
    let just_past = now - RETENTION_DAYS * DAY - 1;
    let kept = now - RETENTION_DAYS * DAY;
    for at in [older, just_past, kept, now] {
        append(&path, &sample(at)).expect("append");
    }

    assert!(prune(&path, now).expect("prune"));

    assert_eq!(read(&path, 0), [sample(kept), sample(now)]);
    let mode = fs::metadata(&path).expect("stat").permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    assert!(!path.with_extension("new").exists());
}

#[test]
fn pruning_a_missing_or_empty_file_does_nothing() {
    let (_dir, path) = file();

    assert!(!prune(&path, 1_000 * DAY).expect("missing"));
    fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
    fs::write(&path, "").expect("write");
    assert!(!prune(&path, 1_000 * DAY).expect("empty"));
}

#[test]
fn a_broken_first_line_counts_as_old_and_goes_at_the_prune() {
    let (_dir, path) = file();
    let now = 1_000 * DAY;
    fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
    fs::write(&path, format!("broken\n{}", sample(now).line())).expect("write");

    assert!(prune(&path, now).expect("prune"));

    assert_eq!(fs::read_to_string(&path).expect("read"), sample(now).line());
}

#[test]
fn round_keeps_the_places_asked_for() {
    assert!((round(7.126, 2) - 7.13).abs() < 1e-9);
    assert!((round(84.4, 0) - 84.0).abs() < 1e-9);
}
