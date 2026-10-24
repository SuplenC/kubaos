use crate::job::chain;

use super::logged;
use super::refusal;

#[test]
fn a_refused_authorization_reads_as_a_sentence_without_pkexecs_words() {
    let e = refusal(
        "Error executing command as another user: Not authorized\n\nThis incident has been reported.\n",
        "exit status: 127",
    );

    assert_eq!(chain(&e), "the keyboard layouts weren't changed: permission was refused");
}

#[test]
fn a_dismissed_password_prompt_is_a_refused_permission_too() {
    let e = refusal("Error executing command as another user: Request dismissed\n", "exit status: 126");

    assert_eq!(chain(&e), "the keyboard layouts weren't changed: permission was refused");
}

#[test]
fn a_missing_helper_names_the_script_that_installs_it() {
    let e = refusal(
        "Cannot run program /usr/local/libexec/kubaos-keyboard-write: No such file or directory\n",
        "exit status: 127",
    );

    assert_eq!(
        chain(&e),
        "the keyboard layouts weren't changed: /usr/local/libexec/kubaos-keyboard-write isn't installed; \
         system/50-hyprland.sh installs it"
    );
}

#[test]
fn any_other_refusal_keeps_the_helpers_reason() {
    let e = refusal("kubaos-keyboard-write: 'us\"' isn't an XKB name list\n", "exit status: 2");

    assert_eq!(
        chain(&e),
        "the keyboard layouts weren't changed: kubaos-keyboard-write: 'us\"' isn't an XKB name list"
    );
}

#[test]
fn a_silent_failure_names_the_exit_status() {
    let e = refusal("", "exit status: 3");

    assert_eq!(chain(&e), "the keyboard layouts weren't changed: pkexec failed (exit status: 3)");
}

#[test]
fn the_log_line_carries_everything_pkexec_said_on_one_line() {
    let line = logged(
        "Error executing command as another user: Not authorized\n\nThis incident has been reported.\n",
        "exit status: 127",
    );

    assert_eq!(
        line,
        "pkexec kubaos-keyboard-write failed (exit status: 127): \
         Error executing command as another user: Not authorized / This incident has been reported."
    );
}
