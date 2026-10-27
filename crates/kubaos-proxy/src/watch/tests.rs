use super::*;

#[test]
fn a_tunnel_dropped_while_awake_is_reconnected() {
    assert_eq!(respond(Event::NetworkChanged, true, None), Response::Reconcile);
}

#[test]
fn a_resume_reconnects_whatever_failed_before() {
    assert_eq!(
        respond(Event::Resumed, true, Some(Duration::from_secs(1))),
        Response::Reconcile
    );
}

#[test]
fn a_change_right_after_a_failed_reconnect_only_republishes() {
    assert_eq!(
        respond(Event::NetworkChanged, true, Some(Duration::from_secs(5))),
        Response::Publish
    );
    assert_eq!(
        respond(Event::NetworkChanged, true, Some(RETRY_AFTER + Duration::from_secs(1))),
        Response::Reconcile
    );
}

#[test]
fn with_the_proxy_off_a_change_only_republishes() {
    assert_eq!(respond(Event::NetworkChanged, false, None), Response::Publish);
    assert_eq!(respond(Event::Resumed, false, None), Response::Publish);
}
