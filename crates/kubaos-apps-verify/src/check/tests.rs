use super::*;
use crate::stub::Refused;
use crate::stub::Reply;
use crate::stub::Stub;

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

#[test]
fn only_a_loopback_host_counts_as_this_machine() {
    for url in [
        "localhost:9001/",
        "127.0.0.1",
        "127.8.0.1:80/x",
        "[::1]:3000/",
        "LOCALHOST",
        "u@localhost:1",
    ] {
        assert!(loopback(url), "{url}");
    }
    for url in [
        "example.com/",
        "localhost.example.com",
        "10.0.0.1:80",
        "[::2]/",
        "127.0.0.1.nip.io",
    ] {
        assert!(!loopback(url), "{url}");
    }
}

const HELM_KEYS: &[u8] = include_bytes!("../../keys/helm.asc");
const HELM_SUMS: &[u8] = include_bytes!("../bump/source/fixtures/helm-v4.3.2-linux-amd64.tar.gz.sha256sum");
const HELM_SIGNATURE: &[u8] = include_bytes!("../bump/source/fixtures/helm-v4.3.2-linux-amd64.tar.gz.sha256sum.asc");
const OTHER_SIGNATURE: &[u8] =
    include_bytes!("../bump/source/fixtures/helm-v4.3.2-linux-amd64.tar.gz.sha256sum.other.asc");

/// The sha256 of `b"served"`.
const SERVED: &str = "8fe88bf015b53b37b5fa84dc726cd93aacca97f601185be11f5fb74fa3d77802";

fn pinned_rows(url: &str, sha256: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut row = |check, subject, outcome, detail| {
        rows.push(Row {
            app: "tool".to_owned(),
            check,
            subject,
            outcome,
            detail,
        });
    };
    pinned("(amd64)", url, sha256, &mut row);
    rows
}

fn sha256_row(rows: &[Row]) -> &Row {
    rows.iter().find(|row| row.check == "sha256").unwrap()
}

#[test]
fn every_outage_of_a_pinned_download_is_a_warning_that_passes() {
    let tls = Stub::always(Reply::Body(Vec::new()));
    let refused = Refused::new();
    let urls = [
        Stub::always(Reply::Status(503)).url("/tool.deb"),
        Stub::always(Reply::Status(500)).url("/tool.deb"),
        Stub::always(Reply::Status(429)).url("/tool.deb"),
        Stub::always(Reply::Redirect("/tool.deb".to_owned())).url("/tool.deb"),
        Stub::always(Reply::Close).url("/tool.deb"),
        // Only the download stalls, so the digests looked for beside it don't wait out the timeout.
        Stub::new(|path, _| match path {
            "/tool.deb" => Reply::Stall,
            _ => Reply::Status(404),
        })
        .url("/tool.deb"),
        refused.url("/tool.deb"),
        tls.url("/tool.deb").replacen("http://", "https://", 1),
    ];

    for url in urls {
        let rows = pinned_rows(&url, SERVED);

        let row = sha256_row(&rows);
        assert_eq!(row.outcome, Outcome::Warning, "{url}: {}", row.detail);
        assert!(row.detail.contains(&url), "{url}: {}", row.detail);
        assert_eq!(crate::exit_code(&rows, false), 0, "{url}");
    }
}

#[test]
fn a_sha256_mismatch_or_a_gone_download_fails() {
    let served = Stub::always(Reply::Body(b"served".to_vec()));
    let missing = Stub::always(Reply::Status(404));
    let gone = Stub::always(Reply::Status(410));

    let mismatch = pinned_rows(&served.url("/tool.deb"), &"0".repeat(64));
    let not_found = pinned_rows(&missing.url("/tool.deb"), SERVED);
    let removed = pinned_rows(&gone.url("/tool.deb"), SERVED);

    assert_eq!(sha256_row(&mismatch).outcome, Outcome::Mismatch);
    assert_eq!(sha256_row(&not_found).outcome, Outcome::Error);
    assert_eq!(sha256_row(&removed).outcome, Outcome::Error);
    for rows in [mismatch, not_found, removed] {
        assert_eq!(crate::exit_code(&rows, false), 1);
    }
}

#[test]
fn an_outage_that_passes_within_the_tries_leaves_no_warning() {
    let stub = Stub::new(|path, nth| match path {
        "/tool.deb" if nth == 0 => Reply::Status(503),
        "/tool.deb" => Reply::Body(b"served".to_vec()),
        _ => Reply::Status(404),
    });

    let rows = pinned_rows(&stub.url("/tool.deb"), SERVED);

    assert_eq!(sha256_row(&rows).outcome, Outcome::Ok);
    assert!(rows.iter().all(|row| row.outcome != Outcome::Warning), "{rows:?}");
    assert_eq!(crate::exit_code(&rows, false), 0);
}

#[test]
fn a_missing_package_candidate_fails() {
    let context = Context::new(PathBuf::from("."), Manifest::default(), "resolute".to_owned());
    let _ = context.ubuntu.get_or_init("amd64", || Ok(Versions::new()));
    let mut rows = Vec::new();

    candidate(
        &context,
        None,
        "no-such-tool",
        "amd64",
        None,
        &mut |_, _, outcome, _| {
            rows.push(outcome);
        },
    );

    assert_eq!(rows, [Outcome::Mismatch]);
}

#[test]
fn a_repository_down_on_every_try_is_a_warning_and_a_missing_key_an_error() {
    let down = Stub::always(Reply::Status(503));
    let missing = Stub::always(Reply::Status(404));

    let outage = Problem::apt(&apt::key(&down.url("/key.asc")).unwrap_err());
    let no_key = Problem::apt(&apt::key(&missing.url("/key.asc")).unwrap_err());
    let no_release = Problem::apt(&apt::release(&down.url("/dists/stable"), HELM_KEYS).unwrap_err());

    assert_eq!(outage.outcome, Outcome::Warning);
    assert_eq!(no_key.outcome, Outcome::Error);
    assert_eq!(no_release.outcome, Outcome::Warning);
}

/// A repository whose suite has no `InRelease`, only a `Release` and its `Release.gpg`.
fn detached(signature: &'static [u8]) -> Stub {
    Stub::new(move |path, _| match path {
        "/dists/stable/Release" => Reply::Body(HELM_SUMS.to_vec()),
        "/dists/stable/Release.gpg" => Reply::Body(signature.to_vec()),
        _ => Reply::Status(404),
    })
}

#[test]
fn a_release_signed_by_a_key_that_isnt_pinned_fails() {
    let signed = detached(HELM_SIGNATURE);
    let other = detached(OTHER_SIGNATURE);

    let verified = apt::release(&signed.url("/dists/stable"), HELM_KEYS);
    let refused = Problem::apt(&apt::release(&other.url("/dists/stable"), HELM_KEYS).unwrap_err());

    assert!(verified.is_ok(), "{verified:?}");
    assert_eq!(refused.outcome, Outcome::Error);
}

#[test]
fn a_warning_is_a_github_annotation_and_nothing_else_is() {
    let row = |outcome, detail: &str| Row {
        app: "tool".to_owned(),
        check: "sha256",
        subject: "tool.deb (amd64)".to_owned(),
        outcome,
        detail: detail.to_owned(),
    };
    let rows = [
        row(Outcome::Ok, "fine"),
        row(Outcome::Warning, "fetching https://x/tool.deb: HTTP 503\n100%"),
    ];

    let text = crate::annotations(&rows);

    assert_eq!(
        text,
        "::warning title=Upstream down::tool sha256 tool.deb (amd64): fetching https://x/tool.deb: HTTP 503%0A100%25\n"
    );
}

#[test]
fn a_down_checksum_file_beside_a_served_download_is_a_warning() {
    let stub = Stub::new(|path, _| match path {
        "/tool.deb" => Reply::Body(b"served".to_vec()),
        _ => Reply::Status(503),
    });

    let rows = pinned_rows(&stub.url("/tool.deb"), SERVED);

    let published = rows.iter().find(|row| row.check == "published").unwrap();
    assert_eq!(published.outcome, Outcome::Warning, "{}", published.detail);
    assert!(published.detail.contains("/tool.deb.sha256"), "{}", published.detail);
    assert_eq!(stub.requests(), 1 + crate::fetch::tries());
}

#[test]
fn a_download_that_is_down_skips_the_vendors_digests() {
    let stub = Stub::always(Reply::Status(503));

    let rows = pinned_rows(&stub.url("/tool.deb"), SERVED);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].outcome, Outcome::Warning);
    assert_eq!(stub.requests(), crate::fetch::tries());
}

#[test]
fn an_inrelease_that_is_down_isnt_followed_by_the_detached_files() {
    let down = Stub::always(Reply::Status(503));

    let problem = Problem::apt(&apt::release(&down.url("/dists/stable"), HELM_KEYS).unwrap_err());

    assert_eq!(problem.outcome, Outcome::Warning);
    assert_eq!(down.requests(), crate::fetch::tries());
}

#[test]
fn a_release_gpg_that_is_down_is_a_warning_naming_it() {
    let stub = Stub::new(|path, _| match path {
        "/dists/stable/Release" => Reply::Body(HELM_SUMS.to_vec()),
        "/dists/stable/Release.gpg" => Reply::Status(503),
        _ => Reply::Status(404),
    });

    let problem = Problem::apt(&apt::release(&stub.url("/dists/stable"), HELM_KEYS).unwrap_err());

    assert_eq!(problem.outcome, Outcome::Warning, "{}", problem.detail);
    assert!(problem.detail.contains("Release.gpg"), "{}", problem.detail);
}

#[test]
fn a_forbidden_or_unauthorized_download_fails() {
    for status in [401, 403, 451] {
        let stub = Stub::always(Reply::Status(status));

        let rows = pinned_rows(&stub.url("/tool.deb"), SERVED);

        assert_eq!(sha256_row(&rows).outcome, Outcome::Error, "HTTP {status}");
    }
}

#[test]
fn an_outage_on_a_pin_the_branch_changed_is_an_error() {
    let (fresh, fresh_detail) = escalated(Outcome::Warning, "HTTP 503".to_owned(), true);
    let (kept, _) = escalated(Outcome::Warning, "HTTP 503".to_owned(), false);
    let (mismatch, _) = escalated(Outcome::Mismatch, "pinned a".to_owned(), true);

    assert_eq!(fresh, Outcome::Error);
    assert!(fresh_detail.starts_with("HTTP 503; "), "{fresh_detail}");
    assert_eq!(kept, Outcome::Warning);
    assert_eq!(mismatch, Outcome::Mismatch);
}
