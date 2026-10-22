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
