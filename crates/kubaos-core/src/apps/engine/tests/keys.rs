//! The keys of the vendor repositories the apps set up, fetched again as they near their expiry.

use super::super::Engine;
use super::super::Keeper;
use super::Events;
use super::FIXTURE;
use super::World;
use crate::apps::action::Side;
use crate::apps::host::fake::FakeRoot;
use crate::apps::root::Reply;
use crate::update::event::UpdateEvent;

/// Runs [`Engine::refresh_keys`] on `world` with `root`: the repositories it refreshed, and the
/// run's events.
fn refresh(world: &World, root: &mut FakeRoot) -> (Vec<String>, Events) {
    let mut events = Events::default();
    let mut user_host = world.host.clone();
    let keeper = Keeper::new(&mut user_host, Side::User, &world.catalog, &world.user_journals);
    let mut engine = Engine::new(keeper, root, &world.root_journals, &world.lock, &mut events, false);
    let refreshed = engine.refresh_keys().unwrap();
    (refreshed, events)
}

fn warnings(events: &Events) -> Vec<String> {
    events
        .0
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Warning { text } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_key_nearing_its_expiry_of_a_repository_an_app_set_up_is_fetched_again_through_root() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    world.host.state.borrow_mut().keys_due.insert("fixture".to_owned());
    let mut root = world.root();

    let (refreshed, events) = refresh(&world, &mut root);

    assert_eq!(refreshed, ["fixture"]);
    assert!(warnings(&events).is_empty(), "{:?}", warnings(&events));
    assert_eq!(root.requests, [r#"{"op":"refresh","app":"fixture","step":1}"#]);
    assert!(world.host.state.borrow().keys_due.is_empty());
}

#[test]
fn no_key_nearing_its_expiry_asks_for_no_root() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    // A repository no app's journal records is no app's to refresh.
    world.host.state.borrow_mut().keys_due.insert("other".to_owned());
    let mut root = world.root();

    let (refreshed, events) = refresh(&world, &mut root);

    assert!(refreshed.is_empty());
    assert!(!root.started, "root was asked for with no key due");
    assert!(events.0.is_empty(), "{:?}", events.0);
}

#[test]
fn a_key_that_couldnt_be_fetched_again_is_a_warning_and_stays_due() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    {
        let mut state = world.host.state.borrow_mut();
        state.keys_due.insert("fixture".to_owned());
        state.fail.insert("apt-repo:fixture".to_owned());
    }
    let mut root = world.root();

    let (refreshed, events) = refresh(&world, &mut root);

    assert!(refreshed.is_empty());
    let warnings = warnings(&events);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].starts_with("the fixture repository's signing key nears its expiry"));
    assert!(world.host.state.borrow().keys_due.contains("fixture"));
}

#[test]
fn the_root_helper_refreshes_only_a_repository_a_settled_apt_step_of_its_journal_set_up() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    let mut root = world.root();
    let refused = |reply: &Reply| matches!(reply, Reply::Refused { .. });

    for line in [
        r#"{"op":"refresh","app":"fixture","step":2}"#,
        r#"{"op":"refresh","app":"fixture","step":9}"#,
        r#"{"op":"refresh","app":"nothing","step":1}"#,
        r#"{"op":"refresh","app":"../etc","step":1}"#,
        r#"{"op":"refresh","app":"fixture","step":1,"repo":"evil"}"#,
    ] {
        let reply = root.serve(line);
        assert!(refused(&reply), "{line} was taken: {reply:?}");
    }

    let reply = root.serve(r#"{"op":"refresh","app":"fixture","step":1}"#);
    assert_eq!(reply, Reply::Ready);
}
