//! A release that puts a step into an installed app, takes one out or changes one: each journal
//! entry stays with the step it records, whatever number the step has now.

use std::fs;
use std::path::PathBuf;

use super::Events;
use super::World;
use super::ai::run;
use super::ai::world;
use crate::apps::action::Action;
use crate::apps::journal::Entry;
use crate::apps::journal::Journal;
use crate::apps::journal::State;
use crate::update::event::ActionState;

/// An app with a shared package and two setups: `local` puts a stack in place and starts it, as
/// root, then writes the user's config; `web` writes a file of its own.
const LAYERED: &str = r#"
name = "Layered"
summary = "An app with setups"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

[[action]]
name = "Install the engine"
kind = "apt"
packages = ["layered-engine"]

[setup.local]
name = "Layered on this machine"
summary = "Its own stack"

[[setup.local.action]]
name = "Put the stack in place"
kind = "file"
to = "/etc/layered/stack.yaml"
text = "stack\n"

[[setup.local.action]]
name = "Start the stack"
kind = "run"
root = true
do = ["./up"]
check = ["./up-to-date"]
undo = ["./down"]

[[setup.local.action]]
name = "Write its config"
kind = "file"
to = "~/.config/layered/config"
text = "local\n"

[setup.web]
name = "Layered on the web"
summary = "In a window"

[[setup.web.action]]
name = "Write its page"
kind = "file"
to = "~/.config/layered/web"
text = "web\n"
"#;

/// The shared step a release puts in front of every other.
const ICON: &str = "[[action]]\nname = \"Put its icon in place\"\nkind = \"file\"\n\
                    to = \"~/.local/share/icons/layered.svg\"\ntext = \"<svg/>\\n\"\n\n";

/// `text` with `step`, an action's TOML, put in before its first action.
fn put_in_front(text: &str, step: &str) -> String {
    text.replacen("[[action]]", &format!("{step}[[action]]"), 1)
}

/// The file the stand-in `name` script of `id` keeps to say it ran, which its `-up-to-date`
/// check looks for and its `down` takes away.
fn mark(world: &World, name: &str) -> PathBuf {
    world
        .catalog
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("marks")
        .join(name)
}

/// Stand-ins for a stack's `up`, `up-to-date` and `down` scripts of the app `id`, under the
/// names `up{suffix}` and so on, that keep a mark instead of starting anything.
fn stack_scripts(world: &World, id: &str, suffix: &str) {
    let mark = mark(world, &format!("{id}{suffix}"));
    fs::create_dir_all(mark.parent().unwrap()).unwrap();
    let mark = mark.display();
    world.script(id, &format!("up{suffix}"), &format!("touch '{mark}'"));
    world.script(id, &format!("up-to-date{suffix}"), &format!("[ -e '{mark}' ]"));
    world.script(id, &format!("down{suffix}"), &format!("rm '{mark}'"));
}

fn layered() -> World {
    let world = World::new(&[("layered", LAYERED)]);
    stack_scripts(&world, "layered", "");
    world
}

fn install(world: &World, id: &str, setup: &str) -> Events {
    let mut events = Events::default();
    let mut root = world.root();
    world
        .run(&mut root, &mut events, true, |e| e.install(id, Some(setup)))
        .unwrap();
    events
}

/// An entry's claims as `(claim, owned)`, in order: a settled step may record them in another.
fn claims(entry: &Entry) -> Vec<(String, bool)> {
    let mut claims: Vec<(String, bool)> = entry.claims.iter().map(|c| (c.claim.clone(), c.owned)).collect();
    claims.sort();
    claims
}

/// Every entry of both journals of `id`, as `(root, action, state, claims)`: what each records,
/// whatever its number.
fn recorded(world: &World, id: &str) -> Vec<(bool, Action, State, Vec<(String, bool)>)> {
    let (user, root) = world.journals(id);
    let mut all: Vec<_> = [(false, user), (true, root)]
        .into_iter()
        .flat_map(|(side, j)| {
            j.into_iter().flat_map(move |j| {
                j.entries
                    .into_iter()
                    .map(move |e| (side, e.action.clone(), e.state, claims(&e)))
            })
        })
        .collect();
    all.sort_by(|a, b| (a.0, &a.1.name).cmp(&(b.0, &b.1.name)));
    all
}

/// Whether the run logged in `log` started or tore down a stack.
fn touched_the_stack(log: &[String]) -> bool {
    log.iter()
        .any(|l| l.starts_with("run ./up") && !l.starts_with("run ./up-to-date") || l.starts_with("run ./down"))
}

fn undid_anything(events: &Events) -> bool {
    events
        .steps()
        .iter()
        .any(|(_, state)| matches!(state, ActionState::Undoing | ActionState::Undone))
}

#[test]
fn a_shared_step_put_in_front_of_an_installed_setup_runs_and_every_other_step_stays_done_and_owned() {
    let world = layered();
    install(&world, "layered", "local");
    let before = recorded(&world, "layered");
    let from = world.host.state.borrow().log.len();
    world.write_app("layered", &put_in_front(LAYERED, ICON));

    let events = install(&world, "layered", "local");

    let log = world.log_since(from);
    assert!(!touched_the_stack(&log), "{log:#?}");
    assert!(!log.iter().any(|l| l.starts_with("apt")), "{log:#?}");
    assert!(!undid_anything(&events), "{:?}", events.0);
    let mut after = recorded(&world, "layered");
    let icon = after.iter().position(|r| r.1.name == "Put its icon in place").unwrap();
    let (root, _, state, claims) = after.remove(icon);
    assert!(!root);
    assert_eq!(state, State::Done);
    assert!(
        claims.iter().any(|(c, owned)| *owned && c.ends_with("layered.svg")),
        "{claims:?}"
    );
    assert_eq!(after, before);
    let (user, root) = world.journals("layered");
    let numbers = |j: Option<Journal>| j.unwrap().entries.iter().map(|e| e.step).collect::<Vec<_>>();
    assert_eq!((numbers(user), numbers(root)), (vec![1, 5], vec![2, 3, 4]));
    assert!(world.host_home().join(".local/share/icons/layered.svg").is_file());
}

#[test]
fn a_step_a_release_takes_out_is_undone_and_the_steps_after_it_stay() {
    let world = layered();
    install(&world, "layered", "local");
    let stack = world.system_path("/etc/layered/stack.yaml");
    assert!(stack.is_file());
    let from = world.host.state.borrow().log.len();
    let without = LAYERED.replace(
        "[[setup.local.action]]\nname = \"Put the stack in place\"\nkind = \"file\"\nto = \"/etc/layered/stack.yaml\"\ntext = \"stack\\n\"\n\n",
        "",
    );
    assert_ne!(without, LAYERED);
    world.write_app("layered", &without);

    install(&world, "layered", "local");

    assert!(!stack.exists(), "the step taken out stands");
    let log = world.log_since(from);
    assert!(!touched_the_stack(&log), "{log:#?}");
    let names: Vec<String> = recorded(&world, "layered").into_iter().map(|r| r.1.name).collect();
    assert_eq!(names, ["Write its config", "Install the engine", "Start the stack"]);
    assert!(world.journals("layered").0.unwrap().complete);
}

#[test]
fn a_step_changed_to_another_path_beside_a_step_put_in_front_is_undone_and_done_again() {
    let world = layered();
    install(&world, "layered", "local");
    let old = world.host_home().join(".config/layered/config");
    assert!(old.is_file());
    let from = world.host.state.borrow().log.len();
    let moved = put_in_front(LAYERED, ICON).replace("~/.config/layered/config", "~/.config/layered/settings");
    world.write_app("layered", &moved);

    install(&world, "layered", "local");

    assert!(!old.exists(), "the old step's file stays");
    assert!(world.host_home().join(".config/layered/settings").is_file());
    let log = world.log_since(from);
    assert!(!touched_the_stack(&log), "{log:#?}");
    let (user, _) = world.journals("layered");
    let user = user.unwrap();
    let written: Vec<(usize, &str)> = user
        .entries
        .iter()
        .map(|e| (e.step, e.action.path().unwrap_or_default()))
        .collect();
    assert_eq!(
        written,
        [
            (1, "~/.local/share/icons/layered.svg"),
            (5, "~/.config/layered/settings")
        ]
    );
    assert!(user.entries.iter().all(|e| e.state == State::Done));
}

#[test]
fn the_other_setup_still_loses_its_own_steps_after_a_step_is_put_in_front() {
    let world = layered();
    install(&world, "layered", "local");
    world.write_app("layered", &put_in_front(LAYERED, ICON));

    install(&world, "layered", "web");

    assert!(!world.system_path("/etc/layered/stack.yaml").exists());
    assert!(!mark(&world, "layered").exists(), "the stack wasn't torn down");
    assert!(!world.host_home().join(".config/layered/config").exists());
    assert!(world.host_home().join(".config/layered/web").is_file());
    let names: Vec<String> = recorded(&world, "layered").into_iter().map(|r| r.1.name).collect();
    assert_eq!(names, ["Put its icon in place", "Write its page", "Install the engine"]);
}

/// Two root runs of one app, each its own stack: the same thing to [`Action::same_thing`].
const TWO_RUNS: &str = r#"
name = "Twins"
summary = "An app with two runs"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

[[action]]
name = "Install the twins"
kind = "apt"
packages = ["twins"]

[[action]]
name = "Start the first"
kind = "run"
root = true
do = ["./up-a"]
check = ["./up-to-date-a"]
undo = ["./down-a"]

[[action]]
name = "Start the second"
kind = "run"
root = true
do = ["./up-b"]
check = ["./up-to-date-b"]
undo = ["./down-b"]
"#;

/// A third run, which a release puts in front of the two.
const THIRD_RUN: &str = "[[action]]\nname = \"Start the third\"\nkind = \"run\"\nroot = true\n\
                         do = [\"./up-c\"]\ncheck = [\"./up-to-date-c\"]\nundo = [\"./down-c\"]\n\n";

fn twins() -> World {
    let world = World::new(&[("twins", TWO_RUNS)]);
    for suffix in ["-a", "-b", "-c"] {
        stack_scripts(&world, "twins", suffix);
    }
    world
}

#[test]
fn steps_of_the_same_thing_keep_their_own_entries_by_their_action_when_another_comes_in_front() {
    let world = twins();
    world.install("twins", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    world.write_app("twins", &put_in_front(TWO_RUNS, THIRD_RUN));

    let (installed, events) = world.install("twins", true);

    installed.unwrap();
    let log = world.log_since(from);
    assert!(log.contains(&"run ./up-c".to_owned()), "{log:#?}");
    assert!(
        !log.iter()
            .any(|l| ["run ./up-a", "run ./up-b", "run ./down-a", "run ./down-b"].contains(&l.as_str())),
        "{log:#?}"
    );
    assert!(!undid_anything(&events), "{:?}", events.0);
    let done = |step, name: &str| (step, name.to_owned(), State::Done, true);
    assert_eq!(
        root_entries(&world),
        [
            done(1, "Start the third"),
            done(3, "Start the first"),
            done(4, "Start the second")
        ]
    );
}

/// Root's entries of the twins' runs, each `(step, name, state, whether it owns its run)`.
fn root_entries(world: &World) -> Vec<(usize, String, State, bool)> {
    world
        .journals("twins")
        .1
        .unwrap()
        .entries
        .iter()
        .filter(|e| matches!(e.action.kind, crate::apps::action::Kind::Run { .. }))
        .map(|e| {
            let owns = e.claims.iter().any(|c| c.claim == "run" && c.owned);
            (e.step, e.action.name.clone(), e.state, owns)
        })
        .collect()
}

#[test]
fn steps_of_the_same_thing_a_release_changes_keep_their_entries_by_name_when_another_comes_in_front() {
    let world = twins();
    world.install("twins", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    // Neither action is the one recorded: each goes by its name among the runs.
    let changed = TWO_RUNS
        .replace("[\"./up-a\"]", "[\"./up-a\", \"again\"]")
        .replace("[\"./up-b\"]", "[\"./up-b\", \"again\"]");
    assert_ne!(changed, TWO_RUNS);
    world.write_app("twins", &put_in_front(&changed, THIRD_RUN));

    let (installed, events) = world.install("twins", true);

    installed.unwrap();
    let log = world.log_since(from);
    assert!(log.contains(&"run ./up-c".to_owned()), "{log:#?}");
    assert!(!log.iter().any(|l| l.starts_with("run ./down")), "{log:#?}");
    assert!(!undid_anything(&events), "{:?}", events.0);
    let done = |step, name: &str| (step, name.to_owned(), State::Done, true);
    assert_eq!(
        root_entries(&world),
        [
            done(1, "Start the third"),
            done(3, "Start the first"),
            done(4, "Start the second")
        ]
    );

    // Each still undoes its own stack.
    world.remove("twins").0.unwrap();
    let log = world.log_since(from);
    for down in ["run ./down-a", "run ./down-b", "run ./down-c"] {
        assert_eq!(log.iter().filter(|l| *l == down).count(), 1, "{down}: {log:#?}");
    }
}

#[test]
fn steps_of_the_same_thing_a_release_renames_and_changes_keep_their_entries_in_their_order() {
    let world = twins();
    world.install("twins", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    let renamed = TWO_RUNS
        .replace("Start the first", "Start one")
        .replace("Start the second", "Start two")
        .replace("[\"./up-a\"]", "[\"./up-a\", \"again\"]")
        .replace("[\"./up-b\"]", "[\"./up-b\", \"again\"]");
    world.write_app("twins", &renamed);

    let (installed, events) = world.install("twins", true);

    installed.unwrap();
    assert!(!undid_anything(&events), "{:?}", events.0);
    let done = |step, name: &str| (step, name.to_owned(), State::Done, true);
    assert_eq!(root_entries(&world), [done(2, "Start one"), done(3, "Start two")]);
    let (_, root) = world.journals("twins");
    let undoes: Vec<Vec<String>> = root
        .unwrap()
        .entries
        .iter()
        .filter_map(|e| match &e.action.kind {
            crate::apps::action::Kind::Run { undo, .. } => Some(undo.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(undoes, [vec!["./down-a".to_owned()], vec!["./down-b".to_owned()]]);
    assert!(!world.log_since(from).iter().any(|l| l.starts_with("run ./down")));
}

#[test]
fn penpots_local_setup_installed_before_its_icon_step_keeps_its_stack_when_the_icon_step_comes() {
    let world = world("penpot");
    stack_scripts(&world, "penpot", "");
    install(&world, "penpot", "local");
    let icon = world
        .host_home()
        .join(".local/share/icons/hicolor/scalable/apps/penpot.svg");
    assert!(icon.is_file());
    // The journals as a release before the icon step wrote them: the same entries, each a number
    // lower, and no icon.
    let (user, root) = world.journals("penpot");
    let (mut user, mut root) = (user.unwrap(), root.unwrap());
    let first = user.entries.remove(0);
    assert_eq!(first.action.name, "Put Penpot's icon in place");
    for entry in user.entries.iter_mut().chain(&mut root.entries) {
        entry.step -= 1;
    }
    user.save(&world.user_journals).unwrap();
    root.save(&world.root_journals).unwrap();
    fs::remove_file(&icon).unwrap();
    let stack: Vec<Entry> = root.entries.clone();
    let from = world.host.state.borrow().log.len();

    let events = install(&world, "penpot", "local");

    let log = world.log_since(from);
    assert!(!touched_the_stack(&log), "{log:#?}");
    assert!(!undid_anything(&events), "{:?}", events.0);
    assert!(icon.is_file());
    assert!(mark(&world, "penpot").exists(), "the stack was torn down");
    assert!(world.system_path("/etc/kubaos/penpot/docker-compose.yaml").is_file());
    let (user, root) = world.journals("penpot");
    let (user, root) = (user.unwrap(), root.unwrap());
    assert!(user.complete);
    assert_eq!(user.entries[0].action.name, "Put Penpot's icon in place");
    assert_eq!(user.entries[0].step, 1);
    assert_eq!(user.entries[0].state, State::Done);
    assert_eq!(root.entries.len(), stack.len());
    for (now, was) in root.entries.iter().zip(&stack) {
        assert_eq!(now.step, was.step + 1, "{}", now.action.name);
        assert_eq!(
            (&now.action, now.state, claims(now), &now.prior),
            (&was.action, was.state, claims(was), &was.prior)
        );
        assert!(now.claims.iter().all(|c| c.owned), "{now:?}");
    }

    // Its removal still takes the stack down, once.
    let from = world.host.state.borrow().log.len();
    run(&world, |e| e.remove("penpot").map(|_| Vec::new())).unwrap();
    let downs = world.log_since(from).iter().filter(|l| *l == "run ./down").count();
    assert_eq!(downs, 1);
}
