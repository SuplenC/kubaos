use crate::job::chain;

use super::refusal;

#[test]
fn access_denied_reads_as_a_sentence_without_localectls_words() {
    let e = refusal("Failed to set keymap: Access denied\n", "exit status: 1");

    assert_eq!(chain(&e), "the keyboard layouts weren't changed: permission was refused");
}

#[test]
fn a_refused_interactive_authentication_is_a_refused_permission_too() {
    let e = refusal("Failed to set keymap: Interactive authentication required.\n", "exit status: 1");

    assert_eq!(chain(&e), "the keyboard layouts weren't changed: permission was refused");
}

#[test]
fn any_other_refusal_keeps_localeds_reason() {
    let e = refusal(
        "Failed to set keymap: Specified keymap cannot be compiled, refusing as invalid.\n",
        "exit status: 1",
    );

    assert_eq!(
        chain(&e),
        "the keyboard layouts weren't changed: Specified keymap cannot be compiled, refusing as invalid."
    );
}

#[test]
fn a_silent_failure_names_the_exit_status() {
    let e = refusal("", "exit status: 3");

    assert_eq!(chain(&e), "the keyboard layouts weren't changed: localectl failed (exit status: 3)");
}
