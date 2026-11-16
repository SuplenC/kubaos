use super::*;

const DIGEST: &str = "02cb6bc6a429d8d0da9fdf9d23739358165cf7b98a245783d6f641526f75b0a7";

#[test]
fn a_checksum_file_gives_the_digest_on_the_line_naming_the_file() {
    let text = format!("{} other.deb\n{DIGEST} *tool.deb\n", "a".repeat(64));

    assert_eq!(digest_for(&text, "tool.deb").as_deref(), Some(DIGEST));
}

#[test]
fn a_checksum_file_of_one_line_gives_its_digest_whatever_it_names() {
    assert_eq!(
        digest_for(&format!("{}\n", DIGEST.to_uppercase()), "tool.deb").as_deref(),
        Some(DIGEST)
    );
}

#[test]
fn a_checksum_file_naming_other_files_gives_nothing() {
    let text = format!("{DIGEST}  a.deb\n{DIGEST}  b.deb\n");

    assert_eq!(digest_for(&text, "tool.deb"), None);
}

#[test]
fn only_a_mismatch_or_an_error_fails() {
    assert!(!Outcome::Ok.fails());
    assert!(!Outcome::Note.fails());
    assert!(Outcome::Mismatch.fails());
    assert!(Outcome::Error.fails());
}
