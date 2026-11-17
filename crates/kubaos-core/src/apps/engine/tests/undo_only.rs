//! A `run` with an `undo` alone: nothing at install, the app's from the run that records it, and
//! its undo on removal.

use super::Events;
use super::World;
use crate::apps::Catalog;
use crate::apps::journal::Journal;
use crate::apps::journal::State;

/// A package, then root's `run` that only undoes: `./off` stops the package's service.
const GUARD: &str = r#"
name = "Guard"
summary = "An app whose removal turns something off"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

[[action]]
name = "Install the guard"
kind = "apt"
packages = ["guard"]

[[action]]
name = "Turn the guard off when it goes"
kind = "run"
root = true
undo = ["./off"]
"#;

fn guard() -> World {
    let world = World::new(&[("guard", GUARD)]);
    world.script("guard", "off", "systemctl stop guard.service");
    world
}

/// Whether the stand-in `systemctl` was asked to stop the guard: `./off` ran.
fn turned_off(world: &World) -> bool {
    world
        .host
        .sandbox()
        .calls()
        .contains(&"systemctl stop guard.service".to_owned())
}

/// Root's entry of the run, step 2.
fn the_run(world: &World) -> crate::apps::journal::Entry {
    let root = Journal::read(&world.root_journals, "guard").unwrap().unwrap();
    root.entry(2).unwrap().clone()
}

#[test]
fn a_run_that_only_undoes_runs_nothing_on_install_and_its_undo_before_the_package_goes() {
    let world = guard();

    world.install("guard", true).0.unwrap();

    assert!(!turned_off(&world));
    let entry = the_run(&world);
    assert_eq!(entry.state, State::Done);
    assert!(entry.claims.iter().all(|c| c.claim == "run" && c.owned), "{entry:?}");
    let from = world.host.state.borrow().log.len();

    world.remove("guard").0.unwrap();

    assert!(turned_off(&world));
    let log = world.host.state.borrow().log[from..].to_vec();
    let at = |what: &str| log.iter().position(|l| l.starts_with(what));
    assert!(
        at("run ./off").is_some_and(|off| at("apt remove").is_some_and(|gone| off < gone)),
        "{log:#?}"
    );
}

#[test]
fn a_run_that_only_undoes_needs_no_root_once_recorded() {
    let world = guard();
    world.install("guard", true).0.unwrap();
    let mut root = world.root();
    let mut events = Events::default();

    world
        .run(&mut root, &mut events, true, |e| e.install("guard", None))
        .unwrap();

    assert!(!root.started, "root was asked for");
    assert!(!turned_off(&world));
}

#[test]
fn an_adopted_app_owns_its_run_that_only_undoes_so_its_removal_runs_it() {
    let world = guard();
    world
        .host
        .state
        .borrow_mut()
        .dpkg
        .insert("guard".to_owned(), "1.0".to_owned());
    let mut root = world.root();
    let mut events = Events::default();
    world
        .run(&mut root, &mut events, true, |e| {
            assert!(e.adopt("guard", true, None).unwrap().is_some(), "not taken in");
            Ok(super::super::Finished::default())
        })
        .unwrap();

    world.remove("guard").0.unwrap();

    assert!(turned_off(&world));
    assert!(!world.host.state.borrow().dpkg.contains_key("guard"));
}

#[test]
fn a_run_with_do_but_no_check_or_with_no_undo_is_refused() {
    let half = GUARD.replace(r#"undo = ["./off"]"#, "do = [\"./off\"]\nundo = [\"./off\"]");
    let none = GUARD.replace("undo = [\"./off\"]\n", "");
    for (text, want) in [(half, "RunHalf"), (none, "undo")] {
        let world = World::new(&[("guard", &text)]);
        world.script("guard", "off", "true");

        let refused = format!("{:?}", Catalog::load(&world.catalog).unwrap_err());

        assert!(refused.contains(want), "{refused}");
    }
}
