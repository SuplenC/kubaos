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

/// What an entry records, whatever its number: whether it is root's, its action, its state and
/// its claims.
type Recorded = (bool, Action, State, Vec<(String, bool)>);

/// Every entry of both journals of `id`.
fn recorded(world: &World, id: &str) -> Vec<Recorded> {
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

/// What the run reported of the step named `name`, in order.
fn reported(events: &Events, name: &str) -> Vec<ActionState> {
    events
        .0
        .iter()
        .filter_map(|e| match e {
            crate::update::event::UpdateEvent::Action(a) if a.name == name => Some(a.state),
            _ => None,
        })
        .collect()
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

    let events = install(&world, "layered", "local");

    assert!(!stack.exists(), "the step taken out stands");
    assert_eq!(
        reported(&events, "Put the stack in place"),
        [ActionState::Undoing, ActionState::Undone]
    );
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
fn steps_of_the_same_thing_a_release_renames_and_changes_keep_their_entries_by_their_undo_when_another_comes_in_front()
{
    let world = twins();
    world.install("twins", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    // Neither name nor action is the one recorded: each run goes by what it undoes.
    let renamed = TWO_RUNS
        .replace("Start the first", "Start one")
        .replace("Start the second", "Start two")
        .replace("[\"./up-a\"]", "[\"./up-a\", \"again\"]")
        .replace("[\"./up-b\"]", "[\"./up-b\", \"again\"]");
    world.write_app("twins", &put_in_front(&renamed, THIRD_RUN));

    let (installed, events) = world.install("twins", true);

    installed.unwrap();
    assert!(!undid_anything(&events), "{:?}", events.0);
    let log = world.log_since(from);
    assert!(log.contains(&"run ./up-c".to_owned()), "{log:#?}");
    assert!(!log.iter().any(|l| l.starts_with("run ./down")), "{log:#?}");
    let done = |step, name: &str| (step, name.to_owned(), State::Done, true);
    assert_eq!(
        root_entries(&world),
        [done(1, "Start the third"), done(3, "Start one"), done(4, "Start two")]
    );
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
    assert_eq!(
        undoes,
        [
            vec!["./down-c".to_owned()],
            vec!["./down-a".to_owned()],
            vec!["./down-b".to_owned()]
        ]
    );

    world.remove("twins").0.unwrap();
    let log = world.log_since(from);
    for down in ["run ./down-a", "run ./down-b", "run ./down-c"] {
        assert_eq!(log.iter().filter(|l| *l == down).count(), 1, "{down}: {log:#?}");
    }
}

/// A root `run` of `name` that starts `up` and undoes with `down`, as a journal records it.
fn run_entry(step: usize, name: &str, up: &str, down: &str) -> Entry {
    let action: Action = toml::from_str(&format!(
        "name = \"{name}\"\nkind = \"run\"\nroot = true\ndo = [\"{up}\"]\ncheck = [\"{up}-check\"]\nundo = [\"{down}\"]\n"
    ))
    .unwrap();
    Entry {
        step,
        state: State::Done,
        claims: Vec::new(),
        prior: crate::apps::journal::Prior::default(),
        action,
        replaced: None,
    }
}

#[test]
fn steps_of_the_same_thing_with_nothing_else_in_common_go_with_their_entries_in_order() {
    let mut journal = Journal::new("twins", None);
    journal.entries = vec![
        run_entry(1, "a", "./a", "./down-a"),
        run_entry(2, "b", "./b", "./down-b"),
    ];
    let steps: Vec<(usize, Option<Action>)> = [
        ("c", "./c", "./down-c"),
        ("d", "./d", "./down-d"),
        ("e", "./e", "./down-e"),
    ]
    .iter()
    .enumerate()
    .map(|(i, (name, up, down))| (i + 1, Some(run_entry(0, name, up, down).action)))
    .collect();

    journal.align(&steps, 3);

    let names: Vec<(usize, &str)> = journal
        .entries
        .iter()
        .map(|e| (e.step, e.action.name.as_str()))
        .collect();
    assert_eq!(names, [(1, "a"), (2, "b")]);
}

/// A package, then two root runs: the images pulled, then the stack started, each undone apart.
const PULLED: &str = r#"
name = "Pulled"
summary = "An app whose stack is pulled, then started"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

[[action]]
name = "Install the engine"
kind = "apt"
packages = ["pulled-engine"]

[[action]]
name = "Pull the images"
kind = "run"
root = true
do = ["./up-a"]
check = ["./up-to-date-a"]
undo = ["./down-a"]

[[action]]
name = "Start the stack"
kind = "run"
root = true
do = ["./up-b"]
check = ["./up-to-date-b"]
undo = ["./down-b"]
"#;

#[test]
fn a_removal_before_the_update_undoes_a_step_the_release_changed_to_another_kind() {
    let world = World::new(&[("pulled", PULLED)]);
    for suffix in ["-a", "-b"] {
        stack_scripts(&world, "pulled", suffix);
    }
    world.install("pulled", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    let changed = PULLED.replace(
        "name = \"Pull the images\"\nkind = \"run\"\nroot = true\ndo = [\"./up-a\"]\ncheck = [\"./up-to-date-a\"]\nundo = [\"./down-a\"]",
        "name = \"Put the stack in place\"\nkind = \"file\"\nto = \"/etc/pulled/stack.yaml\"\ntext = \"stack\\n\"",
    );
    assert_ne!(changed, PULLED);
    world.write_app("pulled", &changed);

    world.remove("pulled").0.unwrap();

    let log = world.log_since(from);
    for down in ["run ./down-a", "run ./down-b", "apt remove pulled-engine"] {
        assert_eq!(log.iter().filter(|l| *l == down).count(), 1, "{down}: {log:#?}");
    }
    assert_eq!(world.journals("pulled"), (None, None));
}

/// Two packages, each a step, which a release makes one step.
const TWO_PACKAGES: &str = r#"
name = "Merged"
summary = "An app of two packages"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

[[action]]
name = "Install the tool"
kind = "apt"
packages = ["merged-tool"]

[[action]]
name = "Install its plugin"
kind = "apt"
packages = ["merged-plugin"]
"#;

#[test]
fn two_steps_a_release_makes_one_are_neither_undone_nor_done_again() {
    let world = World::new(&[("merged", TWO_PACKAGES)]);
    world.install("merged", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    let one = TWO_PACKAGES.replace(
        "packages = [\"merged-tool\"]\n\n[[action]]\nname = \"Install its plugin\"\nkind = \"apt\"\npackages = [\"merged-plugin\"]",
        "packages = [\"merged-tool\", \"merged-plugin\"]",
    );
    assert_ne!(one, TWO_PACKAGES);
    world.write_app("merged", &one);

    // The plugin's step leaves the journal, its package to the merged step: nothing runs.
    let (installed, events) = world.install("merged", true);

    installed.unwrap();
    let log = world.log_since(from);
    assert!(!log.iter().any(|l| l.starts_with("apt")), "{log:#?}");
    assert_eq!(reported(&events, "Install its plugin"), []);
    let (_, root) = world.journals("merged");
    let root = root.unwrap();
    assert_eq!(root.entries.len(), 1, "{root:?}");
    let owned: Vec<&str> = root.entries[0]
        .claims
        .iter()
        .filter(|c| c.owned)
        .map(|c| c.claim.as_str())
        .collect();
    assert_eq!(owned, ["package:merged-tool", "package:merged-plugin"]);

    world.remove("merged").0.unwrap();
    let state = world.host.state.borrow();
    assert!(!state.dpkg.contains_key("merged-tool") && !state.dpkg.contains_key("merged-plugin"));
}

#[test]
fn a_rolled_back_removal_puts_back_a_step_a_release_took_out_and_the_next_update_takes_it_out() {
    let world = layered();
    install(&world, "layered", "local");
    let stack = world.system_path("/etc/layered/stack.yaml");
    let without = LAYERED.replace(
        "[[setup.local.action]]\nname = \"Put the stack in place\"\nkind = \"file\"\nto = \"/etc/layered/stack.yaml\"\ntext = \"stack\\n\"\n\n",
        "",
    );
    world.write_app("layered", &without);
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:layered-engine".to_owned());

    let (removed, events) = world.remove("layered");

    assert!(
        matches!(removed, Err(super::EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    let warned: Vec<_> = events
        .0
        .iter()
        .filter(|e| matches!(e, crate::update::event::UpdateEvent::Warning { .. }))
        .collect();
    assert!(warned.is_empty(), "{warned:?}");
    let (user, root) = world.journals("layered");
    assert!(user.unwrap().complete);
    assert!(
        root.unwrap()
            .entries
            .iter()
            .any(|e| e.action.name == "Put the stack in place"),
        "the step taken out wasn't put back"
    );

    world.host.state.borrow_mut().fail.clear();
    install(&world, "layered", "local");
    assert!(!stack.exists());
    let names: Vec<String> = recorded(&world, "layered").into_iter().map(|r| r.1.name).collect();
    assert_eq!(names, ["Write its config", "Install the engine", "Start the stack"]);
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

/// A new root step, which a release puts in beside the merge.
const NEW_ROOT_FILE: &str = "[[action]]\nname = \"Write its config\"\nkind = \"file\"\n\
                             to = \"/etc/merged/new.conf\"\ntext = \"new\\n\"\n\n";

#[test]
fn two_steps_a_release_makes_one_beside_a_new_step_of_their_side_are_neither_undone_nor_done_again() {
    let merged = TWO_PACKAGES.replace(
        "packages = [\"merged-tool\"]\n\n[[action]]\nname = \"Install its plugin\"\nkind = \"apt\"\npackages = [\"merged-plugin\"]",
        "packages = [\"merged-tool\", \"merged-plugin\"]",
    );
    let after = format!("{merged}\n{}", NEW_ROOT_FILE.trim_end());
    let before = put_in_front(&merged, NEW_ROOT_FILE);
    for (at, text) in [("after", after), ("before", before)] {
        let world = World::new(&[("merged", TWO_PACKAGES)]);
        world.install("merged", true).0.unwrap();
        let from = world.host.state.borrow().log.len();
        world.write_app("merged", &text);

        world.install("merged", true).0.unwrap();

        let log = world.log_since(from);
        assert!(!log.iter().any(|l| l.starts_with("apt")), "{at}: {log:#?}");
        assert!(world.system_path("/etc/merged/new.conf").is_file(), "{at}");
        let state = world.host.state.borrow();
        assert!(state.dpkg.contains_key("merged-tool") && state.dpkg.contains_key("merged-plugin"), "{at}");
    }
}

/// A package, then two files of one folder: only the first's entry made the folder.
const FILES: &str = r#"
name = "Files"
summary = "An app of two files"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

[[action]]
name = "Install the tool"
kind = "apt"
packages = ["files-tool"]

[[action]]
name = "Write a"
kind = "file"
to = "~/.config/filesapp/a"
text = "a\n"

[[action]]
name = "Write b"
kind = "file"
to = "~/.config/filesapp/b"
text = "b\n"
"#;

/// The `[[action]]` of the step named `name` in `text`, taken out.
fn without(text: &str, name: &str) -> String {
    let at = text.find(&format!("[[action]]\nname = \"{name}\"")).unwrap();
    let end = text[at + 1..].find("[[action]]").map_or(text.len(), |n| at + 1 + n);
    format!("{}{}", &text[..at], &text[end..])
}

#[test]
fn a_users_step_taken_out_in_front_of_another_of_its_side_is_the_one_undone() {
    let world = World::new(&[("files", FILES)]);
    world.install("files", true).0.unwrap();
    let (a, b) = (
        world.host_home().join(".config/filesapp/a"),
        world.host_home().join(".config/filesapp/b"),
    );
    world.write_app("files", &without(FILES, "Write a"));

    let events = world.install("files", true).1;

    assert!(!a.exists(), "the step taken out stands");
    assert!(b.is_file(), "the step that stands was undone");
    assert_eq!(reported(&events, "Write a"), [ActionState::Undoing, ActionState::Undone]);
    assert_eq!(reported(&events, "Write b"), [ActionState::Done]);

    world.remove("files").0.unwrap();
    assert!(
        !world.host_home().join(".config/filesapp").exists(),
        "the folder the step taken out made stays"
    );
}

/// Root's: a config, then the package.
const ROOTY: &str = r#"
name = "Rooty"
summary = "An app of root's steps alone"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

[[action]]
name = "Write its old config"
kind = "file"
to = "/etc/rooty/old.conf"
text = "old\n"

[[action]]
name = "Install the engine"
kind = "apt"
packages = ["rooty-engine"]
"#;

#[test]
fn a_roots_step_taken_out_in_front_of_another_of_an_app_of_root_alone_is_the_one_undone() {
    let world = World::new(&[("rooty", ROOTY)]);
    world.install("rooty", true).0.unwrap();
    let old = world.system_path("/etc/rooty/old.conf");
    let from = world.host.state.borrow().log.len();
    world.write_app("rooty", &without(ROOTY, "Write its old config"));

    let events = world.install("rooty", true).1;

    assert!(!old.exists(), "the step taken out stands");
    let log = world.log_since(from);
    assert!(!log.iter().any(|l| l.starts_with("apt")), "{log:#?}");
    assert!(world.host.state.borrow().dpkg.contains_key("rooty-engine"));
    assert_eq!(
        reported(&events, "Write its old config"),
        [ActionState::Undoing, ActionState::Undone]
    );
}
