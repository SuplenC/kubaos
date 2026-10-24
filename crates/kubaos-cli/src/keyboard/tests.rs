use crate::job::chain;

use super::logged;
use super::refusal;

#[test]
fn a_polkit_refusal_says_polkit_refused() {
    let e = refusal(
        "Error executing command as another user: Not authorized\n\nThis incident has been reported.\n",
        "exit status: 127",
    );

    assert_eq!(chain(&e), "polkit refused it");
}

#[test]
fn a_dismissed_password_prompt_says_so() {
    let e = refusal("Error executing command as another user: Request dismissed\n", "exit status: 126");

    assert_eq!(chain(&e), "the password prompt was dismissed");
}

#[test]
fn a_missing_helper_names_the_script_that_installs_it() {
    let missing = "/usr/local/libexec/kubaos-keyboard-write isn't installed; system/50-hyprland.sh installs it";

    for said in [
        "Error accessing /usr/local/libexec/kubaos-keyboard-write: No such file or directory\n",
        "Cannot run program /usr/local/libexec/kubaos-keyboard-write: No such file or directory\n",
    ] {
        assert_eq!(chain(&refusal(said, "exit status: 127")), missing);
    }
}

#[test]
fn any_other_refusal_keeps_the_helpers_reason() {
    let e = refusal("kubaos-keyboard-write: 'us\"' isn't an XKB name list\n", "exit status: 2");

    assert_eq!(chain(&e), "kubaos-keyboard-write: 'us\"' isn't an XKB name list");
}

#[test]
fn a_silent_failure_names_the_exit_status() {
    let e = refusal("", "exit status: 3");

    assert_eq!(chain(&e), "pkexec failed (exit status: 3)");
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
