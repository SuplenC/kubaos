use super::*;

#[test]
fn it_holds_until_its_end_or_forever_while_on() {
    assert!(State::on(0).holds(1_000));
    assert!(State::on(2_000).holds(1_999));
    assert!(!State::on(2_000).holds(2_000));
    assert!(!State::default().holds(0));
}

#[test]
fn the_state_file_reads_back_and_a_missing_or_broken_one_is_off() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());

    assert_eq!(State::read(&file), State::default());
    State::on(1_793_000_000).write(&file).expect("write");
    assert_eq!(State::read(&file), State::on(1_793_000_000));
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        "{\"active\":true,\"until\":1793000000}\n"
    );
    std::fs::write(&file, "{half").expect("break it");
    assert_eq!(State::read(&file), State::default());
}
