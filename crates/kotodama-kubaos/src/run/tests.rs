use super::*;

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|&word| word.to_owned()).collect()
}

#[test]
fn output_is_what_the_program_wrote_to_stdout() {
    let output = Programs.output(&argv(&["sh", "-c", "echo hello"])).unwrap();

    assert_eq!(output, "hello\n");
}

#[test]
fn a_program_that_fails_says_its_last_line_on_stderr() {
    let err = Programs
        .output(&argv(&["sh", "-c", "echo first >&2; echo 'no such sink' >&2; exit 3"]))
        .unwrap_err();

    assert_eq!(err.to_string(), "sh failed: no such sink");
}

#[test]
fn a_program_that_fails_silently_says_how_it_exited() {
    let err = Programs.output(&argv(&["sh", "-c", "exit 3"])).unwrap_err();

    assert!(err.to_string().starts_with("sh failed: exit"), "{err}");
}

#[test]
fn a_missing_program_cannot_start() {
    let err = Programs.output(&argv(&["kubaos-no-such-program"])).unwrap_err();

    assert!(matches!(err, RunError::Start { .. }), "{err:?}");
    assert_eq!(err.to_string(), "couldn't start kubaos-no-such-program");
}

#[test]
fn a_delayed_launch_starts_the_program_after_the_delay() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("ran");

    Programs
        .launch(
            &argv(&["sh", "-c", &format!("touch '{}'", marker.display())]),
            Duration::from_millis(50),
        )
        .unwrap();

    assert!(!marker.exists(), "it ran before the delay");
    for _ in 0..100 {
        if marker.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("the program never ran");
}

/// Runs `script` in the background and gives each line it wrote, then whether it succeeded.
fn background(script: &str) -> (Vec<String>, bool) {
    background_taking(script, Duration::ZERO)
}

/// [`background`], taking `each` over each line, as a busy protocol loop may.
fn background_taking(script: &str, each: Duration) -> (Vec<String>, bool) {
    let (sender, received) = std::sync::mpsc::channel();
    let ended = sender.clone();
    Programs
        .background(
            &argv(&["sh", "-c", script]),
            Box::new(move |line| {
                thread::sleep(each);
                sender.send(Ok(line)).unwrap();
            }),
            Box::new(move |succeeded| ended.send(Err(succeeded)).unwrap()),
        )
        .unwrap();
    let mut lines = Vec::new();
    loop {
        match received.recv_timeout(Duration::from_secs(5)).expect("the program ends") {
            Ok(line) => lines.push(line),
            Err(succeeded) => return (lines, succeeded),
        }
    }
}

#[test]
fn a_background_program_reports_each_line_of_stdout_and_stderr_then_its_end() {
    let (lines, succeeded) = background("echo one; echo two >&2; echo three");

    assert_eq!(lines, ["one", "two", "three"]);
    assert!(succeeded);
}

#[test]
fn a_background_program_that_fails_ends_so() {
    let (lines, succeeded) = background("echo 'no root'; exit 126");

    assert_eq!(lines, ["no root"]);
    assert!(!succeeded);
}

#[test]
fn a_background_programs_bytes_that_arent_utf8_are_replaced_and_the_lines_after_them_still_come() {
    let (lines, succeeded) = background(r"printf 'caf\351\n'; echo after");

    assert_eq!(lines, ["caf\u{FFFD}", "after"]);
    assert!(succeeded);
}

#[test]
fn a_background_programs_overlong_line_is_cut_and_the_lines_after_it_still_come() {
    let (lines, _) = background("head -c 1000000 /dev/zero | tr '\\0' a; echo; echo after");

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].len(), LINE_CAP);
    assert_eq!(lines[1], "after");
}

#[test]
fn a_background_programs_last_line_counts_without_a_newline() {
    let (lines, _) = background("echo first; printf last");

    assert_eq!(lines, ["first", "last"]);
}

#[test]
fn a_background_programs_end_comes_after_every_line_even_with_its_output_held_open() {
    let script = "for i in $(seq 300); do echo \"$i\"; done; sleep 30 & exit 0";
    let begun = std::time::Instant::now();

    let (lines, succeeded) = background_taking(script, Duration::from_millis(5));

    assert_eq!(lines.len(), 300);
    assert_eq!(lines.last().map(String::as_str), Some("300"));
    assert!(succeeded);
    assert!(
        begun.elapsed() < Duration::from_secs(10),
        "it waited for the held output"
    );
}

#[test]
fn a_background_program_that_cant_start_is_an_error() {
    let started = Programs.background(&argv(&["kubaos-no-such-program"]), Box::new(|_| {}), Box::new(|_| {}));

    assert!(matches!(started, Err(RunError::Start { .. })), "{started:?}");
}
