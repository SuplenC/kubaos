//! Tor Browser of the shipped catalog, in each of its setups, on the fake machine.

use std::fs;
use std::path::PathBuf;

use super::super::EngineError;
use super::World;
use super::ai::run;
use super::ai::world;
use crate::apps::picks::Picks;

/// The menu entries the X11 setup writes, with the ids of the package's own, and the line each
/// starts Tor Browser's launcher with.
const ENTRIES: [(&str, &str); 2] = [
    (
        ".local/share/applications/org.torproject.torbrowser-launcher.desktop",
        "Exec=env MOZ_ENABLE_WAYLAND=0 torbrowser-launcher %u",
    ),
    (
        ".local/share/applications/org.torproject.torbrowser-launcher.settings.desktop",
        "Exec=env MOZ_ENABLE_WAYLAND=0 torbrowser-launcher --settings",
    ),
];

/// Tor Browser picked in `setup`.
fn picked(setup: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add("tor-browser");
    picks.add_setup("tor-browser", setup);
    picks
}

fn add(world: &World, setup: &str) -> Result<(), EngineError> {
    run(world, |e| e.add("tor-browser", Some(setup), &picked(setup))).map(|_| ())
}

fn entries(world: &World) -> Vec<PathBuf> {
    ENTRIES.iter().map(|(path, _)| world.host_home().join(path)).collect()
}

/// The apt runs that took the launcher away.
fn launcher_removals(world: &World) -> usize {
    let state = world.host.state.borrow();
    state
        .log
        .iter()
        .filter(|l| l.starts_with("apt ") && !l.starts_with("apt install") && l.contains("torbrowser-launcher"))
        .count()
}

#[test]
fn tor_browser_installs_in_each_setup_and_its_removal_leaves_the_machine_as_before() {
    for setup in ["anonymity", "wayland"] {
        let world = world("tor-browser");
        let before = world.host.snapshot();

        add(&world, setup).unwrap_or_else(|e| panic!("{setup}: {e}"));

        assert!(
            world.host.state.borrow().dpkg.contains_key("torbrowser-launcher"),
            "{setup}"
        );
        for ((_, exec), entry) in ENTRIES.iter().zip(entries(&world)) {
            if setup == "anonymity" {
                let text = fs::read_to_string(&entry).unwrap();
                assert!(text.lines().any(|l| l == *exec), "{text}");
                assert_eq!(text.lines().filter(|l| l.starts_with("Exec=")).count(), 1, "{text}");
            } else {
                assert!(!entry.exists(), "Wayland is the session's: {}", entry.display());
            }
        }

        run(&world, |e| e.remove_with("tor-browser", &Picks::default())).unwrap_or_else(|e| panic!("{setup}: {e}"));

        assert_eq!(world.host.snapshot(), before, "{setup}");
        assert_eq!(world.journals("tor-browser"), (None, None), "{setup}");
    }
}

#[test]
fn switching_tor_browser_between_x11_and_wayland_moves_its_menu_entries_alone() {
    let world = world("tor-browser");
    add(&world, "anonymity").unwrap();

    add(&world, "wayland").unwrap();

    assert!(
        entries(&world).iter().all(|e| !e.exists()),
        "the package's entries are the menu's again"
    );
    assert!(world.host.state.borrow().dpkg.contains_key("torbrowser-launcher"));

    add(&world, "anonymity").unwrap();

    for ((_, exec), entry) in ENTRIES.iter().zip(entries(&world)) {
        let text = fs::read_to_string(&entry).unwrap();
        assert!(text.lines().any(|l| l == *exec), "{text}");
    }
    assert!(world.host.state.borrow().dpkg.contains_key("torbrowser-launcher"));
    assert_eq!(launcher_removals(&world), 0, "a switch keeps the launcher");
}

#[test]
fn a_tor_browser_menu_entry_of_the_users_own_refuses_the_x11_setup_and_is_never_written_over() {
    for at in 0..ENTRIES.len() {
        let world = world("tor-browser");
        let entry = &entries(&world)[at];
        fs::create_dir_all(entry.parent().unwrap()).unwrap();
        fs::write(entry, "[Desktop Entry]\nName=Mine\n").unwrap();
        let before = world.host.snapshot();

        let result = add(&world, "anonymity");

        let Err(EngineError::Refused { why, yours, .. }) = result else {
            panic!("Tor Browser was installed over the user's entry: {result:?}");
        };
        assert!(yours, "the refusal is over a file of the user's");
        assert!(
            why.iter()
                .any(|w| w.contains("is yours") && w.contains("delete or rename it")),
            "{why:?}"
        );
        assert_eq!(world.host.snapshot(), before);
        assert_eq!(fs::read_to_string(entry).unwrap(), "[Desktop Entry]\nName=Mine\n");
    }
}

#[test]
fn the_wayland_setup_installs_beside_a_tor_browser_menu_entry_of_the_users_own_and_leaves_it() {
    let world = world("tor-browser");
    let entry = &entries(&world)[0];
    fs::create_dir_all(entry.parent().unwrap()).unwrap();
    fs::write(entry, "[Desktop Entry]\nName=Mine\n").unwrap();

    add(&world, "wayland").unwrap();
    run(&world, |e| e.remove_with("tor-browser", &Picks::default())).unwrap();

    assert_eq!(fs::read_to_string(entry).unwrap(), "[Desktop Entry]\nName=Mine\n");
}

#[test]
fn switching_to_x11_over_a_menu_entry_of_the_users_own_is_refused_and_keeps_wayland() {
    let world = world("tor-browser");
    add(&world, "wayland").unwrap();
    let entry = &entries(&world)[0];
    fs::create_dir_all(entry.parent().unwrap()).unwrap();
    fs::write(entry, "[Desktop Entry]\nName=Mine\n").unwrap();

    let result = add(&world, "anonymity");

    assert!(
        matches!(result, Err(EngineError::Refused { yours: true, .. })),
        "{result:?}"
    );
    assert_eq!(fs::read_to_string(entry).unwrap(), "[Desktop Entry]\nName=Mine\n");
    assert!(world.host.state.borrow().dpkg.contains_key("torbrowser-launcher"));
    let (user, _) = world.journals("tor-browser");
    assert_eq!(user.and_then(|j| j.setup).as_deref(), Some("wayland"));
}
