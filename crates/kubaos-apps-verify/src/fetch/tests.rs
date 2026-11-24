use super::*;
use crate::stub;
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
        assert_eq!(stub.requests(), TRIES as usize, "HTTP {status}");
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
    assert_eq!(class_of(&stub::refused("/key.asc")), (Class::Outage, Some(7)));
}

#[test]
fn a_connection_closed_with_no_answer_is_an_outage() {
    let stub = Stub::always(Reply::Close);

    assert_eq!(class_of(&stub.url("/key.asc")), (Class::Outage, Some(52)));
}

#[test]
fn a_timeout_is_an_outage() {
    let stub = Stub::always(Reply::Stall);

    assert_eq!(class_of(&stub.url("/key.asc")), (Class::Outage, Some(28)));
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
    assert_eq!(down.requests(), TRIES as usize);
    assert_eq!(status(&looping.url("/")).unwrap_err().class(), Class::Outage);
}
