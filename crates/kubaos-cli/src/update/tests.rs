use super::notifies;

#[test]
fn a_run_with_a_terminal_on_stdout_or_stderr_shows_its_summary_there_and_doesnt_notify() {
    assert!(!notifies(false, true, true));
    assert!(!notifies(false, false, true));
    assert!(!notifies(false, true, false));
}

#[test]
fn a_json_run_leaves_the_summary_to_its_front_end() {
    assert!(!notifies(true, false, false));
}

#[test]
fn a_run_without_a_terminal_or_a_front_end_notifies() {
    assert!(notifies(false, false, false));
}
