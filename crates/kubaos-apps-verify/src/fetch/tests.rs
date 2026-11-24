use super::*;
use crate::stub::Refused;
use crate::stub::Reply;
use crate::stub::Stub;

/// The class of fetching `url` and curl's exit code for it.
fn class_of(url: &str) -> (Class, Option<i32>) {
    let error = get(url).unwrap_err();
    let exit = match error.cause {
        Cause::Curl { exit, .. } => exit,
        _ => None,
    };
    (error.class(), exit)
}

#[test]
fn a_server_error_or_too_many_requests_is_an_outage_after_every_try() {
    for status in [500, 502, 503, 504, 429] {
        let stub = Stub::always(Reply::Status(status));

        let error = get(&stub.url("/InRelease")).unwrap_err();

        assert_eq!(error.class(), Class::Outage, "HTTP {status}");
        assert_eq!(stub.requests(), RETRY.tries as usize, "HTTP {status}");
        assert!(crate::check::chain(&error).ends_with(&format!("HTTP {status}")));
    }
}

#[test]
fn a_redirect_loop_is_an_outage() {
    let stub = Stub::always(Reply::Redirect("/loop".to_owned()));

    assert_eq!(class_of(&stub.url("/loop")), (Class::Outage, Some(47)));
}

#[test]
fn a_refused_connection_is_an_outage() {
    let refused = Refused::new();

    assert_eq!(class_of(&refused.url("/key.asc")), (Class::Outage, Some(7)));
}

#[test]
fn a_connection_closed_with_no_answer_is_an_outage() {
    let stub = Stub::always(Reply::Close);

    assert_eq!(class_of(&stub.url("/key.asc")), (Class::Outage, Some(52)));
}

#[test]
fn a_timeout_is_an_outage_tried_again_once_and_its_host_isnt_tried_again() {
    let stub = Stub::always(Reply::Stall);

    let first = class_of(&stub.url("/key.asc"));
    let later = get(&stub.url("/InRelease")).unwrap_err();

    assert_eq!(first, (Class::Outage, Some(28)));
    assert_eq!(stub.requests(), 2);
    assert!(matches!(later.cause, Cause::HostDown(_)), "{later:?}");
    assert_eq!(later.class(), Class::Outage);
}

#[test]
fn a_server_error_leaves_the_host_to_be_tried_again() {
    let stub = Stub::always(Reply::Status(503));

    let _ = get(&stub.url("/InRelease"));
    let _ = get(&stub.url("/Release"));

    assert_eq!(stub.requests(), 2 * RETRY.tries as usize);
}

#[test]
fn the_tries_wait_twice_as_long_each_time_within_the_budget() {
    let waits: Vec<_> = (2..=8)
        .map_while(|next| Retry::RUNS.wait(next, Duration::ZERO, 0))
        .map(|wait| wait.as_secs())
        .collect();

    assert_eq!(waits, [1, 2, 4, 8, 16, 32]);
    assert_eq!(Retry::RUNS.wait(4, Duration::from_secs(179), 0), None);
    assert_eq!(Retry::RUNS.wait(2, Duration::ZERO, 1), Some(Duration::from_secs(1)));
    assert_eq!(Retry::RUNS.wait(3, Duration::ZERO, 2), None);
}

#[test]
fn a_stream_or_connection_broken_off_is_an_outage() {
    for exit in [16, 18, 55, 56, 92] {
        let cause = Cause::Curl {
            exit: Some(exit),
            said: String::new(),
        };

        assert_eq!(cause.class(), Class::Outage, "exit {exit}");
    }
}

#[test]
fn a_failed_tls_handshake_is_an_outage() {
    let stub = Stub::always(Reply::Body(b"plain".to_vec()));
    let url = stub.url("/key.asc").replacen("http://", "https://", 1);

    assert_eq!(class_of(&url), (Class::Outage, Some(35)));
}

#[test]
fn a_name_that_doesnt_resolve_is_an_outage() {
    for exit in [5, 6] {
        let cause = Cause::Curl {
            exit: Some(exit),
            said: String::new(),
        };

        assert_eq!(cause.class(), Class::Outage);
    }
}

#[test]
fn a_gone_or_missing_url_fails_at_once() {
    for status in [404, 410] {
        let stub = Stub::always(Reply::Status(status));

        let error = get(&stub.url("/app.deb")).unwrap_err();

        assert_eq!(error.class(), Class::Failure, "HTTP {status}");
        assert_eq!(stub.requests(), 1, "HTTP {status}");
    }
}

#[test]
fn an_outage_that_passes_within_the_tries_is_no_error() {
    let stub = Stub::new(|_, nth| {
        if nth == 0 {
            Reply::Status(503)
        } else {
            Reply::Body(b"served".to_vec())
        }
    });

    let body = get(&stub.url("/InRelease")).unwrap();

    assert_eq!(body, b"served");
    assert_eq!(stub.requests(), 2);
}

#[test]
fn a_download_is_hashed_as_served_and_an_outage_while_hashing_is_one() {
    let served = Stub::always(Reply::Body(b"abc".to_vec()));
    let down = Stub::always(Reply::Status(503));

    let hashed = sha256(&served.url("/app.deb")).unwrap();
    let error = sha256(&down.url("/app.deb")).unwrap_err();

    assert_eq!(
        hashed,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(error.class(), Class::Outage);
}

#[test]
fn a_web_apps_status_is_its_answer_and_a_down_one_is_an_outage() {
    let missing = Stub::always(Reply::Status(404));
    let down = Stub::always(Reply::Status(503));
    let looping = Stub::always(Reply::Redirect("/".to_owned()));

    let answer = status(&missing.url("/")).unwrap();

    assert_eq!(answer.status, 404);
    assert_eq!(status(&down.url("/")).unwrap_err().class(), Class::Outage);
    assert_eq!(down.requests(), RETRY.tries as usize);
    assert_eq!(status(&looping.url("/")).unwrap_err().class(), Class::Outage);
}
