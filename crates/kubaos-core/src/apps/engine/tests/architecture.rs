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
icon = "fixture"

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
icon = "fixture"

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
icon = "fixture"

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

/// The game's i386 package pulling in i386's C library, which leaves its configuration behind when
/// removed, and its compiler runtime, which apt marks Protected, as Ubuntu's `libgcc-s1` and
/// `libc6` are.
fn with_i386_runtime(world: &World) {
    let mut state = world.host.state.borrow_mut();
    state.depends.insert(
        "libgame:i386".to_owned(),
        ["libgcc-s1:i386".to_owned(), "libc6:i386".to_owned()].into(),
    );
    state
        .depends
        .insert("libgcc-s1:i386".to_owned(), ["libc6:i386".to_owned()].into());
    state.protected.insert("libgcc-s1:i386".to_owned());
    state.conffiles.insert(
        "libc6:i386".to_owned(),
        vec!["/etc/ld.so.conf.d/i386-linux-gnu.conf".to_owned()],
    );
}

#[test]
fn a_protected_dependency_stays_at_the_package_removal_and_goes_with_the_architecture() {
    let world = World::new(&[("game", GAME)]);
    with_i386_runtime(&world);
    let before = world.host.snapshot();
    world.install("game", true).0.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("libgcc-s1:i386"));

    let (result, events) = world.remove("game");

    result.unwrap();
    assert!(!takes_i386(&world));
    assert!(
        world.host.state.borrow().rc.is_empty(),
        "the configuration left is purged"
    );
    assert_eq!(world.host.snapshot(), before);
    assert!(
        !events.0.iter().any(|e| matches!(e, UpdateEvent::Note { .. })),
        "{:#?}",
        events.0
    );
}

#[test]
fn an_architecture_package_the_app_didnt_bring_keeps_it_with_its_configuration_said() {
    let world = World::new(&[("game", GAME)]);
    with_i386_runtime(&world);
    world.install("game", true).0.unwrap();
    world.host.state.borrow_mut().rc.insert("libmine:i386".to_owned());

    let (result, events) = world.remove("game");

    result.unwrap();
    assert!(takes_i386(&world));
    assert_eq!(
        world.host.state.borrow().rc,
        ["libmine:i386".to_owned()].into(),
        "only what the app brought is purged"
    );
    assert!(
        events.0.contains(&UpdateEvent::Note {
            text: "dpkg keeps taking i386 packages, as libmine:i386's configuration is still there.".to_owned()
        }),
        "{:#?}",
        events.0
    );
}

#[test]
fn a_dependency_the_user_installed_by_name_since_stays_and_keeps_the_architecture() {
    let world = World::new(&[("game", GAME)]);
    with_i386_runtime(&world);
    world.install("game", true).0.unwrap();
    world.host.state.borrow_mut().manual.insert("libc6:i386".to_owned());

    let (result, events) = world.remove("game");

    result.unwrap();
    let state = world.host.state.borrow();
    assert!(state.dpkg.contains_key("libc6:i386"));
    assert!(state.foreign.contains("i386"));
    drop(state);
    assert!(
        events
            .0
            .iter()
            .any(|e| matches!(e, UpdateEvent::Note { text } if text.contains("libc6:i386"))),
        "{:#?}",
        events.0
    );
}

#[test]
fn a_package_the_user_installed_on_the_runtime_keeps_it_and_the_architecture() {
    let world = World::new(&[("game", GAME)]);
    with_i386_runtime(&world);
    world.install("game", true).0.unwrap();
    {
        let mut state = world.host.state.borrow_mut();
        state.dpkg.insert("libmine:i386".to_owned(), "1.0".to_owned());
        state.manual.insert("libmine:i386".to_owned());
        state
            .depends
            .insert("libmine:i386".to_owned(), ["libc6:i386".to_owned()].into());
    }

    let (result, events) = world.remove("game");

    result.unwrap();
    let state = world.host.state.borrow();
    for package in ["libmine:i386", "libc6:i386", "libgcc-s1:i386"] {
        assert!(state.dpkg.contains_key(package), "{package}");
    }
    assert!(state.foreign.contains("i386"));
    drop(state);
    assert!(
        events.0.contains(&UpdateEvent::Note {
            text: "dpkg keeps taking i386 packages, as 3 of them are still installed: libc6:i386, libgcc-s1:i386, libmine:i386."
                .to_owned()
        }),
        "{:#?}",
        events.0
    );
}

#[test]
fn a_removal_stopped_between_the_packages_and_the_architecture_finishes_both() {
    let world = World::new(&[("game", GAME)]);
    with_i386_runtime(&world);
    let before = world.host.snapshot();
    world.install("game", true).0.unwrap();
    let mut root = world.root();
    let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        world.run(
            &mut root,
            &mut super::StopAt("Let dpkg install 32-bit packages"),
            true,
            |e| e.remove("game"),
        )
    }));
    assert!(stopped.is_err());
    assert!(
        world.host.state.borrow().dpkg.contains_key("libgcc-s1:i386"),
        "left by the packages' undo"
    );

    world.remove("game").0.unwrap();

    assert!(!takes_i386(&world));
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("game"), (None, None));
}

#[test]
fn the_machines_own_architecture_is_never_taken_away() {
    use crate::apps::action::Action;
    use crate::apps::action::perform;
    use crate::apps::host::fake::FakeHost;
    use crate::apps::journal::Claim;

    let dir = crate::sandbox::tempdir().unwrap();
    let mut host = FakeHost::new(dir.path());
    let table: toml::Table =
        toml::from_str("name = \"X\"\nkind = \"architecture\"\narchitecture = \"amd64\"\n").unwrap();
    let action = Action::from_folder(table).unwrap();

    let note = perform::undo(
        &action,
        &mut host,
        &[Claim::new("arch:amd64", true)],
        &crate::apps::journal::Prior::default(),
        perform::Undoing::default(),
    )
    .unwrap();

    assert_eq!(note, None);
    assert!(host.state.borrow().log.is_empty(), "{:#?}", host.state.borrow().log);
}
