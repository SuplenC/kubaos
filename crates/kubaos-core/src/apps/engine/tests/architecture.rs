//! A foreign architecture: dpkg taking i386 packages for an app's 32-bit libraries, shared by
//! claim between the apps that need it.

use super::EngineError;
use super::World;
use crate::apps::journal::Journal;
use crate::apps::journal::State;
use crate::update::event::UpdateEvent;

/// i386 taken, then a package of the machine's and one of i386's, as Steam's install is.
const GAME: &str = r#"
name = "Game"
summary = "An app with 32-bit libraries"
category = "games"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

[[action]]
name = "Let dpkg install 32-bit packages"
kind = "architecture"
architecture = "i386"

[[action]]
name = "Install the game"
kind = "apt"
packages = ["game", "libgame:i386"]
"#;

/// Another app that needs i386, as Wine's 32-bit libraries would.
const EMULATOR: &str = r#"
name = "Emulator"
summary = "Another app with 32-bit libraries"
category = "games"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

[[action]]
name = "Let dpkg install 32-bit packages"
kind = "architecture"
architecture = "i386"

[[action]]
name = "Install the emulator"
kind = "apt"
packages = ["libemu:i386"]
"#;

/// A group first, whose removal the tests can make fail after i386 was taken away.
const GROUPED: &str = r#"
name = "Grouped"
summary = "An app with a group before its architecture"
category = "games"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

[[action]]
name = "Let you use it"
kind = "group"
group = "players"

[[action]]
name = "Let dpkg install 32-bit packages"
kind = "architecture"
architecture = "i386"

[[action]]
name = "Install the game"
kind = "apt"
packages = ["libgame:i386"]
"#;

fn takes_i386(world: &World) -> bool {
    world.host.state.borrow().foreign.contains("i386")
}

/// The claim of `id`'s architecture step in root's journal, whether it is owned.
fn owns_i386(world: &World, id: &str) -> Option<bool> {
    let root = Journal::read(&world.root_journals, id).unwrap()?;
    root.entries
        .iter()
        .flat_map(|e| &e.claims)
        .find(|c| c.claim == "arch:i386")
        .map(|c| c.owned)
}

#[test]
fn an_install_adds_the_architecture_before_its_packages_and_owns_it() {
    let world = World::new(&[("game", GAME)]);

    world.install("game", true).0.unwrap();

    assert!(takes_i386(&world));
    assert!(world.host.state.borrow().dpkg.contains_key("libgame:i386"));
    assert_eq!(owns_i386(&world, "game"), Some(true));
    let log = world.host.state.borrow().log.clone();
    let at = |what: &str| log.iter().position(|l| l.starts_with(what));
    assert!(
        at("arch i386 true") < at("apt update") && at("apt update") < at("apt install"),
        "{log:#?}"
    );
}

#[test]
fn an_architecture_there_already_is_found_and_stays_after_the_removal() {
    let world = World::new(&[("game", GAME)]);
    world.host.state.borrow_mut().foreign.insert("i386".to_owned());

    world.install("game", true).0.unwrap();

    let root = Journal::read(&world.root_journals, "game").unwrap().unwrap();
    assert_eq!(root.entry(1).unwrap().state, State::Found);
    assert_eq!(owns_i386(&world, "game"), Some(false));

    world.remove("game").0.unwrap();

    assert!(takes_i386(&world));
    assert!(!world.host.state.borrow().dpkg.contains_key("libgame:i386"));
}

#[test]
fn removing_the_app_takes_its_packages_then_the_architecture() {
    let world = World::new(&[("game", GAME)]);
    let before = world.host.snapshot();
    world.install("game", true).0.unwrap();

    world.remove("game").0.unwrap();

    assert!(!takes_i386(&world));
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("game"), (None, None));
}

#[test]
fn a_package_of_the_architecture_left_keeps_it_and_says_which() {
    let world = World::new(&[("game", GAME)]);
    world.install("game", true).0.unwrap();
    world
        .host
        .state
        .borrow_mut()
        .dpkg
        .insert("libmine:i386".to_owned(), "1.0".to_owned());

    let (result, events) = world.remove("game");

    result.unwrap();
    assert!(takes_i386(&world));
    assert_eq!(world.journals("game"), (None, None));
    assert!(
        events.0.contains(&UpdateEvent::Note {
            text: "dpkg keeps taking i386 packages, as libmine:i386 is still installed.".to_owned()
        }),
        "{:#?}",
        events.0
    );
}

#[test]
fn two_apps_share_the_architecture_and_the_last_to_go_takes_it() {
    let world = World::new(&[("game", GAME), ("emulator", EMULATOR)]);
    world.install("game", true).0.unwrap();
    world.install("emulator", true).0.unwrap();
    assert_eq!(owns_i386(&world, "emulator"), Some(false));

    world.remove("game").0.unwrap();

    assert!(takes_i386(&world));
    assert_eq!(owns_i386(&world, "emulator"), Some(true));

    world.remove("emulator").0.unwrap();

    assert!(!takes_i386(&world));
}

#[test]
fn the_app_that_found_the_architecture_leaves_it_to_the_one_that_added_it() {
    let world = World::new(&[("game", GAME), ("emulator", EMULATOR)]);
    world.install("game", true).0.unwrap();
    world.install("emulator", true).0.unwrap();

    world.remove("emulator").0.unwrap();

    assert!(takes_i386(&world));
    assert_eq!(owns_i386(&world, "game"), Some(true));

    world.remove("game").0.unwrap();

    assert!(!takes_i386(&world));
}

#[test]
fn a_removal_that_fails_after_the_architecture_went_puts_it_back_with_its_packages() {
    let world = World::new(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    let installed = world.host.snapshot();
    world.host.state.borrow_mut().fail.insert("groupdel:players".to_owned());

    let (result, _) = world.remove("grouped");

    assert!(
        matches!(result, Err(EngineError::RemovalRolledBack { ref name, .. }) if name == "Let you use it"),
        "{result:?}"
    );
    assert!(takes_i386(&world));
    assert_eq!(world.host.snapshot(), installed);
    assert_eq!(owns_i386(&world, "grouped"), Some(true));
}

#[test]
fn an_install_that_fails_after_adding_the_architecture_takes_it_away_again() {
    let world = World::new(&[("game", GAME)]);
    world.host.state.borrow_mut().fail.insert("apt:game".to_owned());
    let before = world.host.snapshot();

    let (result, _) = world.install("game", true);

    assert!(matches!(result, Err(EngineError::Step { .. })), "{result:?}");
    assert!(!takes_i386(&world));
    assert_eq!(world.host.snapshot(), before);
}
