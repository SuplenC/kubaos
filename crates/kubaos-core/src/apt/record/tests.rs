use super::Record;

#[test]
fn the_record_round_trips_sorted_under_its_header() {
    let mut record = Record::default();
    record.add("nautilus", "files");
    record.add("curl", "tools");

    let text = record.to_text();

    assert!(text.starts_with("# "));
    assert!(text.ends_with("curl\ttools\nnautilus\tfiles\n"));
    assert_eq!(Record::parse(&text), record);
}

#[test]
fn a_package_appended_twice_keeps_its_first_group() {
    let record = Record::parse("curl\ttools\ncurl\tnix\nzsh\n");

    assert_eq!(record.group_of("curl"), Some("tools"));
    assert_eq!(record.group_of("zsh"), Some(""));
    assert!(!record.has("git"));
}

#[test]
fn a_removed_package_is_forgotten() {
    let mut record = Record::parse("curl\ttools\n");

    record.remove("curl");

    assert!(!record.has("curl"));
}
