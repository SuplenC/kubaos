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
fn a_step_a_release_renames_is_undone_and_done_again_under_its_new_name() {
    let world = twins();
    world.install("twins", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    let renamed = TWO_RUNS
        .replace("Start the first", "Start one")
        .replace("Start the second", "Start two");
    world.write_app("twins", &put_in_front(&renamed, THIRD_RUN));

    world.install("twins", true).0.unwrap();

    let log = world.log_since(from);
    for line in ["run ./up-c", "run ./down-a", "run ./down-b", "run ./up-a", "run ./up-b"] {
        assert_eq!(log.iter().filter(|l| *l == line).count(), 1, "{line}: {log:#?}");
    }
    let at = |line: &str| log.iter().position(|l| l == line).unwrap();
    assert!(
        at("run ./down-a") < at("run ./up-a"),
        "the old name's undo ran last: {log:#?}"
    );
    assert!(mark(&world, "twins-a").exists() && mark(&world, "twins-b").exists());
    let done = |step, name: &str| (step, name.to_owned(), State::Done, true);
    assert_eq!(
        root_entries(&world),
        [done(1, "Start the third"), done(3, "Start one"), done(4, "Start two")]
    );
}

/// A root `apt` step of `name` that installs `package`, as a journal records it.
fn apt_entry(step: usize, name: &str, package: &str) -> Entry {
    let action: Action = toml::from_str(&format!(
        "name = \"{name}\"\nkind = \"apt\"\npackages = [\"{package}\"]\n"
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

fn numbered(journal: &Journal) -> Vec<(usize, String)> {
    journal
        .entries
        .iter()
        .map(|e| (e.step, format!("{:?}", e.action.kind)))
        .collect()
}

#[test]
fn an_entry_goes_with_the_step_of_its_name_whatever_its_action_and_its_place() {
    let mut journal = Journal::new("x", None);
    journal.entries = vec![
        apt_entry(1, "A", "pa"),
        apt_entry(2, "B", "pb"),
        apt_entry(3, "C", "pc"),
    ];

    journal.align(&[(1, "New"), (2, "C"), (4, "A")], 4);

    let names: Vec<(usize, &str)> = journal
        .entries
        .iter()
        .map(|e| (e.step, e.action.name.as_str()))
        .collect();
    assert_eq!(names, [(2, "C"), (4, "A"), (5, "B")]);
}

#[test]
fn entries_of_one_name_from_an_older_catalog_align_the_same_every_time() {
    let mut journal = Journal::new("x", None);
    journal.entries = vec![apt_entry(1, "N", "pa"), apt_entry(2, "N", "pb")];
    let steps = [(1, "M"), (2, "N")];

    journal.align(&steps, 2);
    let once = numbered(&journal);
    journal.align(&steps, 2);

    assert_eq!(numbered(&journal), once);
    assert_eq!(once.iter().map(|(n, _)| *n).collect::<Vec<_>>(), [2, 3]);
    assert!(
        once[0].1.contains("pa"),
        "the first of the name keeps the step: {once:?}"
    );
}

#[test]
fn taking_an_entry_out_never_moves_another() {
    let mut journal = Journal::new("x", None);
    journal.entries = vec![
        apt_entry(1, "Kept", "pk"),
        apt_entry(2, "Gone", "pg"),
        apt_entry(3, "Also gone", "pa"),
    ];
    let steps = [(1, "Kept")];
    journal.align(&steps, 1);
    let before = numbered(&journal);

    journal.drop_step(before[1].0);
    journal.align(&steps, 1);

    assert_eq!(numbered(&journal), [before[0].clone(), before[2].clone()]);
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
        assert!(
            state.dpkg.contains_key("merged-tool") && state.dpkg.contains_key("merged-plugin"),
            "{at}"
        );
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
    assert_eq!(
        reported(&events, "Write a"),
        [ActionState::Undoing, ActionState::Undone]
    );
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

/// The packages of an `apt` step of `name`, as TOML.
fn apt_step(name: &str, packages: &str) -> String {
    format!("[[action]]\nname = \"{name}\"\nkind = \"apt\"\npackages = [{packages}]\n\n")
}

#[test]
fn an_apt_step_a_release_splits_in_two_keeps_both_packages_whichever_comes_first() {
    let one = super::app(&apt_step("Install the tools", "\"tool-a\", \"tool-b\""));
    let kept = apt_step("Install the tools", "\"tool-a\"");
    let split = apt_step("Install tool b", "\"tool-b\"");
    for (at, text) in [
        ("after", super::app(&format!("{kept}{split}"))),
        ("before", super::app(&format!("{split}{kept}"))),
    ] {
        let world = World::new(&[("split", &one)]);
        world.install("split", true).0.unwrap();
        let from = world.host.state.borrow().log.len();
        world.write_app("split", &text);

        world.install("split", true).0.unwrap();

        let log = world.log_since(from);
        assert!(!log.iter().any(|l| l.starts_with("apt")), "{at}: {log:#?}");
        world.remove("split").0.unwrap();
        let state = world.host.state.borrow();
        assert!(
            !state.dpkg.contains_key("tool-a") && !state.dpkg.contains_key("tool-b"),
            "{at}: the removal left {:?}",
            state.dpkg
        );
    }
}

#[test]
fn a_run_a_release_moves_from_root_to_the_user_keeps_its_stack_until_the_removal() {
    let engine = apt_step("Install the engine", "\"moved-engine\"");
    let run = "[[action]]\nname = \"Start the stack\"\nkind = \"run\"\nroot = true\ndo = [\"./up\"]\n\
               check = [\"./up-to-date\"]\nundo = [\"./down\"]\n";
    let world = World::new(&[("moved", &super::app(&format!("{engine}{run}")))]);
    stack_scripts(&world, "moved", "");
    world.install("moved", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    world.write_app(
        "moved",
        &super::app(&format!("{engine}{}", run.replace("root = true\n", ""))),
    );

    let events = world.install("moved", true).1;

    assert!(mark(&world, "moved").exists(), "the update tore the stack down");
    assert!(!touched_the_stack(&world.log_since(from)));
    assert!(!undid_anything(&events), "{:?}", events.0);

    world.remove("moved").0.unwrap();
    let downs = world.log_since(from).iter().filter(|l| *l == "run ./down").count();
    assert_eq!(downs, 1);
    assert!(!mark(&world, "moved").exists());
}

#[test]
fn a_run_that_only_undoes_a_release_takes_out_while_it_adds_another_keeps_its_own_undo() {
    let file = "[[action]]\nname = \"Write\"\nkind = \"file\"\nto = \"~/.config/uo/x\"\ntext = \"x\\n\"\n\n";
    let off = |name: &str, script: &str| {
        format!("[[action]]\nname = \"{name}\"\nkind = \"run\"\nundo = [\"./{script}\"]\n\n")
    };
    let world = World::new(&[("uo", &super::app(&format!("{file}{}", off("Turn A off", "off-a"))))]);
    world.script("uo", "off-a", "systemctl stop a.service");
    world.script("uo", "off-b", "systemctl stop b.service");
    world.install("uo", true).0.unwrap();
    world.write_app("uo", &super::app(&format!("{file}{}", off("Turn B off", "off-b"))));

    world.install("uo", true).0.unwrap();

    let calls = world.host.sandbox().calls();
    assert!(!calls.iter().any(|c| c.starts_with("systemctl stop")), "{calls:?}");
    world.remove("uo").0.unwrap();
    let calls = world.host.sandbox().calls();
    for stop in ["systemctl stop a.service", "systemctl stop b.service"] {
        assert_eq!(calls.iter().filter(|c| *c == stop).count(), 1, "{stop}: {calls:?}");
    }
}

#[test]
fn root_refuses_to_take_out_a_step_its_catalog_still_has_whatever_the_user_side_asks() {
    let files = "[[action]]\nname = \"One\"\nkind = \"file\"\nto = \"/etc/tk/one\"\ntext = \"1\\n\"\n\n\
                 [[action]]\nname = \"Two\"\nkind = \"file\"\nto = \"/etc/tk/two\"\ntext = \"2\\n\"\n";
    let world = World::new(&[(
        "tk",
        &super::app(&format!("{}{files}", apt_step("Install tk", "\"tk\""))),
    )]);
    world.install("tk", true).0.unwrap();
    let mut root = world.root();

    for step in [1, 2, 3, 9] {
        let reply = root.serve(&format!(r#"{{"op":"take_out","app":"tk","step":{step}}}"#));
        assert!(
            matches!(
                reply,
                crate::apps::root::Reply::Refused { .. } | crate::apps::root::Reply::Step { .. }
            ),
            "{step}: {reply:?}"
        );
    }

    assert!(
        world.system_path("/etc/tk/one").is_file(),
        "root took out a step the catalog has"
    );
    assert!(
        world.system_path("/etc/tk/two").is_file(),
        "root took out a step the catalog has"
    );
    assert!(world.host.state.borrow().dpkg.contains_key("tk"));
    let reply = root.serve(r#"{"op":"align","app":"tk"}"#);
    assert!(matches!(reply, crate::apps::root::Reply::Refused { .. }), "{reply:?}");
}

#[test]
fn a_catalog_refuses_two_steps_of_one_install_under_one_name_but_not_across_setups() {
    let twice = super::app(&format!(
        "{}{}",
        apt_step("Install the packages", "\"pa\""),
        apt_step("Install the packages", "\"pb\"")
    ));
    let world = World::new(&[("flip", &twice)]);

    let refused = format!("{:?}", crate::apps::Catalog::load(&world.catalog).unwrap_err());

    assert!(refused.contains("DuplicateStepName"), "{refused}");
    let setups = super::app(&format!(
        "{}[setup.a]\nname = \"A\"\nsummary = \"A\"\n\n[[setup.a.action]]\nname = \"Write it\"\nkind = \"file\"\n\
         to = \"~/a\"\ntext = \"a\\n\"\n\n[setup.b]\nname = \"B\"\nsummary = \"B\"\n\n[[setup.b.action]]\n\
         name = \"Write it\"\nkind = \"file\"\nto = \"~/b\"\ntext = \"b\\n\"\n",
        apt_step("Install the packages", "\"pa\"")
    ));
    let world = World::new(&[("both", &setups)]);
    crate::apps::Catalog::load(&world.catalog).unwrap();
}

#[test]
fn an_older_journal_of_two_steps_under_one_name_updates_keeping_every_package() {
    let world = World::new(&[(
        "flip",
        &super::app(&format!(
            "{}{}",
            apt_step("Install the packages", "\"pa\""),
            apt_step("Install more", "\"pb\"")
        )),
    )]);
    world.install("flip", true).0.unwrap();
    // As a catalog before names were one step's alone left it: both entries of one name.
    let (_, root) = world.journals("flip");
    let mut root = root.unwrap();
    root.entries[1].action.name = "Install the packages".to_owned();
    root.save(&world.root_journals).unwrap();
    let from = world.host.state.borrow().log.len();
    world.write_app(
        "flip",
        &super::app(&format!(
            "{}{}",
            apt_step("Install more", "\"pb\", \"px\""),
            apt_step("Install the packages", "\"pa\", \"py\"")
        )),
    );

    world.install("flip", true).0.unwrap();

    let log = world.log_since(from);
    assert!(!log.iter().any(|l| l.contains("remove")), "{log:#?}");
    {
        let state = world.host.state.borrow();
        for package in ["pa", "pb", "px", "py"] {
            assert!(state.dpkg.contains_key(package), "{package}: {:?}", state.dpkg);
        }
    }
    world.remove("flip").0.unwrap();
    let state = world.host.state.borrow();
    for package in ["pa", "pb", "px", "py"] {
        assert!(!state.dpkg.contains_key(package), "{package} stays: {:?}", state.dpkg);
    }
}

#[test]
fn an_update_killed_partway_through_a_release_that_renumbers_its_steps_finishes_on_the_next() {
    let engine = apt_step("Install the engine", "\"crash-engine\"");
    let run = "[[action]]\nname = \"Start the stack\"\nkind = \"run\"\nroot = true\ndo = [\"./up\"]\n\
               check = [\"./up-to-date\"]\nundo = [\"./down\"]\n\n";
    let file = "[[action]]\nname = \"Write\"\nkind = \"file\"\nto = \"~/.config/crash/x\"\ntext = \"x\\n\"\n";
    let world = World::new(&[("crash", &super::app(&format!("{engine}{run}{file}")))]);
    stack_scripts(&world, "crash", "");
    world.install("crash", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    let front = "[[action]]\nname = \"Front\"\nkind = \"file\"\nto = \"/etc/crash/front\"\ntext = \"f\\n\"\n\n";
    world.write_app("crash", &super::app(&format!("{front}{engine}{run}")));
    let mut root = world.root();
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        world.run(&mut root, &mut super::StopAt("Start the stack"), true, |e| {
            e.install("crash", None)
        })
    }));
    assert!(killed.is_err(), "the run wasn't killed");

    world.install("crash", true).0.unwrap();

    let log = world.log_since(from);
    assert!(
        !touched_the_stack(&log) && !log.iter().any(|l| l.starts_with("apt")),
        "{log:#?}"
    );
    assert!(mark(&world, "crash").exists());
    assert!(world.system_path("/etc/crash/front").is_file());
    assert!(
        !world.host_home().join(".config/crash/x").exists(),
        "the file taken out stays"
    );
}

#[test]
fn a_found_step_a_release_takes_out_asks_for_no_root_and_reports_no_undo() {
    let file = "[[action]]\nname = \"Write\"\nkind = \"file\"\nto = \"~/.config/found/x\"\ntext = \"x\\n\"\n";
    let world = World::new(&[(
        "found",
        &super::app(&format!("{}{file}", apt_step("Install it", "\"pre-there\""))),
    )]);
    world
        .host
        .state
        .borrow_mut()
        .dpkg
        .insert("pre-there".to_owned(), "1".to_owned());
    world.install("found", true).0.unwrap();
    world.write_app("found", &super::app(file));
    let mut root = world.root();
    let mut events = Events::default();

    world
        .run(&mut root, &mut events, true, |e| e.install("found", None))
        .unwrap();

    assert!(!root.started, "root was asked for");
    assert!(!undid_anything(&events), "{:?}", events.0);
    assert!(world.host.state.borrow().dpkg.contains_key("pre-there"));

    world.remove("found").0.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("pre-there"));
}

#[test]
fn a_step_a_release_merges_in_part_keeps_what_apt_pulled_in_for_its_undo() {
    let world = World::new(&[(
        "partial",
        &super::app(&format!(
            "{}{}",
            apt_step("Install the tool", "\"p-tool\""),
            apt_step("Install its plugins", "\"p-plugin\", \"p-extra\"")
        )),
    )]);
    world
        .host
        .state
        .borrow_mut()
        .depends
        .insert("p-extra".to_owned(), ["libextra".to_owned()].into());
    world.install("partial", true).0.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("libextra"));
    world.write_app(
        "partial",
        &super::app(&apt_step("Install the tool", "\"p-tool\", \"p-plugin\"")),
    );

    world.install("partial", true).0.unwrap();

    let (_, root) = world.journals("partial");
    let root = root.unwrap();
    assert_eq!(root.entries.len(), 1, "{root:?}");
    let owned: Vec<&str> = root.entries[0]
        .claims
        .iter()
        .filter(|c| c.owned)
        .map(|c| c.claim.as_str())
        .collect();
    assert!(owned.contains(&"package:p-plugin"), "{owned:?}");
    assert!(!owned.iter().any(|c| c.starts_with("dep:")), "{owned:?}");
    let state = world.host.state.borrow();
    assert!(state.dpkg.contains_key("p-plugin"));
    assert!(
        !state.dpkg.contains_key("p-extra") && !state.dpkg.contains_key("libextra"),
        "{:?}",
        state.dpkg
    );
}

/// A `file` step of `name` that writes `text` to `to`, as TOML.
fn file_step(name: &str, to: &str, text: &str) -> String {
    format!("[[action]]\nname = \"{name}\"\nkind = \"file\"\nto = \"{to}\"\ntext = \"{text}\\n\"\n\n")
}

/// An app of `shared` steps and the setups `a` and `b` of their own steps.
fn two_setups(shared: &str, a: &str, b: &str) -> String {
    let own = |id: &str, steps: &str| {
        format!(
            "[setup.{id}]\nname = \"{id}\"\nsummary = \"{id}\"\n\n{}",
            steps.replace("[[action]]", &format!("[[setup.{id}.action]]"))
        )
    };
    super::app(&format!("{shared}{}{}", own("a", a), own("b", b)))
}

#[test]
fn a_step_changed_to_another_path_whose_old_one_a_new_step_before_it_writes_keeps_that_file() {
    let world = World::new(&[("pp", &super::app(&file_step("Write config", "~/.config/pp/a", "a")))]);
    world.install("pp", true).0.unwrap();
    let a = world.host_home().join(".config/pp/a");
    world.write_app(
        "pp",
        &super::app(&format!(
            "{}{}",
            file_step("Write a", "~/.config/pp/a", "a"),
            file_step("Write config", "~/.config/pp/b", "b")
        )),
    );

    world.install("pp", true).0.unwrap();

    assert!(a.is_file(), "the file the new step writes is gone");
    assert!(world.host_home().join(".config/pp/b").is_file());
    world.remove("pp").0.unwrap();
    assert!(!a.exists() && !world.host_home().join(".config/pp").exists());
}

#[test]
fn a_step_changed_to_another_kind_whose_package_a_new_step_before_it_installs_keeps_that_package() {
    let world = World::new(&[("kc", &super::app(&apt_step("Install it", "\"kc-a\", \"kc-b\"")))]);
    world.install("kc", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    world.write_app(
        "kc",
        &super::app(&format!(
            "{}{}",
            apt_step("Install a", "\"kc-a\""),
            file_step("Install it", "/etc/kc/conf", "x")
        )),
    );

    world.install("kc", true).0.unwrap();

    let log = world.log_since(from);
    assert!(log.contains(&"apt remove kc-b".to_owned()), "{log:#?}");
    {
        let state = world.host.state.borrow();
        assert!(state.dpkg.contains_key("kc-a"), "the new step's package was removed");
        assert!(!state.dpkg.contains_key("kc-b"));
    }
    world.remove("kc").0.unwrap();
    assert!(!world.host.state.borrow().dpkg.contains_key("kc-a"));
}

#[test]
fn a_step_a_release_merges_in_part_hands_over_what_apt_pulled_in_for_the_package_it_hands_over() {
    let world = World::new(&[(
        "pm",
        &super::app(&format!(
            "{}{}",
            apt_step("Install the tool", "\"pm-tool\""),
            apt_step("Install its plugins", "\"pm-plugin\", \"pm-extra\"")
        )),
    )]);
    world
        .host
        .state
        .borrow_mut()
        .depends
        .insert("pm-plugin".to_owned(), ["libplugin".to_owned()].into());
    world.install("pm", true).0.unwrap();
    world.write_app(
        "pm",
        &super::app(&apt_step("Install the tool", "\"pm-tool\", \"pm-plugin\"")),
    );

    world.install("pm", true).0.unwrap();

    assert!(world.host.state.borrow().dpkg.contains_key("libplugin"));
    world.remove("pm").0.unwrap();
    let state = world.host.state.borrow();
    assert!(
        !state.dpkg.contains_key("libplugin"),
        "left with no owner: {:?}",
        state.dpkg
    );
}

#[test]
fn a_change_a_failed_update_left_pending_and_a_second_change_both_go_with_the_app() {
    let world = World::new(&[(
        "rr",
        &super::app(&format!(
            "{}{}",
            apt_step("Install", "\"rr-p\", \"rr-q\", \"rr-r\""),
            apt_step("Other", "\"rr-z\"")
        )),
    )]);
    world.install("rr", true).0.unwrap();
    world.write_app(
        "rr",
        &super::app(&format!(
            "{}{}",
            apt_step("Install", "\"rr-p\", \"rr-q\""),
            apt_step("Other", "\"rr-z\", \"rr-y\"")
        )),
    );
    world.host.state.borrow_mut().fail.insert("apt:rr-y".to_owned());
    assert!(world.install("rr", true).0.is_err());
    world.host.state.borrow_mut().fail.clear();
    world.write_app(
        "rr",
        &super::app(&format!(
            "{}{}",
            apt_step("Install", "\"rr-p\""),
            apt_step("Other", "\"rr-z\"")
        )),
    );

    world.install("rr", true).0.unwrap();

    world.remove("rr").0.unwrap();
    let state = world.host.state.borrow();
    for package in ["rr-p", "rr-q", "rr-r", "rr-z"] {
        assert!(!state.dpkg.contains_key(package), "{package} stays: {:?}", state.dpkg);
    }
}

#[test]
fn a_file_step_renamed_with_new_text_goes_whole_with_the_app() {
    let world = World::new(&[("rn", &super::app(&file_step("Write it", "~/.config/rn/x", "old")))]);
    world.install("rn", true).0.unwrap();
    let x = world.host_home().join(".config/rn/x");
    world.write_app(
        "rn",
        &super::app(&file_step("Write the config", "~/.config/rn/x", "new")),
    );

    world.install("rn", true).0.unwrap();

    assert_eq!(fs::read_to_string(&x).unwrap(), "new\n");
    world.remove("rn").0.unwrap();
    assert!(!x.exists() && !world.host_home().join(".config/rn").exists());
}

#[test]
fn an_apt_step_renamed_alone_runs_no_apt_and_its_package_goes_with_the_app() {
    let world = World::new(&[("ra", &super::app(&apt_step("Install it", "\"ra-p\"")))]);
    world.install("ra", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    world.write_app("ra", &super::app(&apt_step("Install the app", "\"ra-p\"")));

    world.install("ra", true).0.unwrap();

    assert!(!world.log_since(from).iter().any(|l| l.starts_with("apt")));
    world.remove("ra").0.unwrap();
    assert!(!world.host.state.borrow().dpkg.contains_key("ra-p"));
}

#[test]
fn a_file_step_renamed_then_changed_writes_the_new_text_and_goes_with_the_app() {
    let world = World::new(&[("rs", &super::app(&file_step("Write it", "~/.config/rs/x", "old")))]);
    world.install("rs", true).0.unwrap();
    let x = world.host_home().join(".config/rs/x");
    world.write_app(
        "rs",
        &super::app(&file_step("Write the config", "~/.config/rs/x", "old")),
    );
    world.install("rs", true).0.unwrap();
    world.write_app(
        "rs",
        &super::app(&file_step("Write the config", "~/.config/rs/x", "new")),
    );

    world.install("rs", true).0.unwrap();

    assert_eq!(fs::read_to_string(&x).unwrap(), "new\n");
    world.remove("rs").0.unwrap();
    assert!(!x.exists());
}

#[test]
fn a_run_renamed_alone_is_undone_and_done_again() {
    let run = |name: &str| {
        format!(
            "[[action]]\nname = \"{name}\"\nkind = \"run\"\nroot = true\ndo = [\"./up\"]\n\
             check = [\"./up-to-date\"]\nundo = [\"./down\"]\n\n"
        )
    };
    let engine = apt_step("Install", "\"rr2-e\"");
    let world = World::new(&[("rr2", &super::app(&format!("{engine}{}", run("Start the stack"))))]);
    stack_scripts(&world, "rr2", "");
    world.install("rr2", true).0.unwrap();
    let from = world.host.state.borrow().log.len();
    world.write_app("rr2", &super::app(&format!("{engine}{}", run("Start the service"))));

    world.install("rr2", true).0.unwrap();

    let log = world.log_since(from);
    for line in ["run ./down", "run ./up"] {
        assert_eq!(log.iter().filter(|l| *l == line).count(), 1, "{line}: {log:#?}");
    }
    assert!(mark(&world, "rr2").exists(), "the stack isn't back");
}

#[test]
fn a_shared_root_step_a_release_takes_out_after_a_setup_switch_is_taken_out() {
    let shared = format!(
        "{}{}",
        apt_step("Install", "\"st-e\""),
        file_step("Extra", "/etc/st/extra", "x")
    );
    let a = file_step("A", "~/.config/st/a", "a");
    let b = format!(
        "{}{}{}",
        file_step("B1", "~/.config/st/b1", "b"),
        file_step("B2", "~/.config/st/b2", "b"),
        file_step("B3", "~/.config/st/b3", "b")
    );
    let world = World::new(&[("st", &two_setups(&shared, &a, &b))]);
    install(&world, "st", "a");
    install(&world, "st", "b");
    assert_eq!(world.journals("st").1.unwrap().setup.as_deref(), Some("b"));
    world.write_app("st", &two_setups(&apt_step("Install", "\"st-e\""), &a, &b));

    install(&world, "st", "b");

    assert!(
        !world.system_path("/etc/st/extra").exists(),
        "the step taken out stands"
    );
}

#[test]
fn a_switch_to_a_longer_setup_keeps_a_run_that_only_undoes_a_release_took_out_for_the_removal() {
    let shared = apt_step("Install", "\"sw-e\"");
    let off = "[[action]]\nname = \"Turn it off\"\nkind = \"run\"\nundo = [\"./off\"]\n\n";
    let b = format!(
        "{}{}{}",
        file_step("B1", "~/.config/sw/b1", "b"),
        file_step("B2", "~/.config/sw/b2", "b"),
        file_step("B3", "~/.config/sw/b3", "b")
    );
    let world = World::new(&[(
        "sw",
        &two_setups(&shared, &format!("{}{off}", file_step("A", "~/.config/sw/a", "a")), &b),
    )]);
    world.script("sw", "off", "systemctl stop sw.service");
    install(&world, "sw", "a");
    world.write_app("sw", &two_setups(&shared, &file_step("A", "~/.config/sw/a", "a"), &b));
    install(&world, "sw", "a");

    install(&world, "sw", "b");

    let stops = || {
        world
            .host
            .sandbox()
            .calls()
            .iter()
            .filter(|c| *c == "systemctl stop sw.service")
            .count()
    };
    assert_eq!(stops(), 0, "the switch ran the removal's undo");
    world.remove("sw").0.unwrap();
    assert_eq!(stops(), 1, "the removal forgot it");
}

#[test]
fn an_older_replaced_part_keeps_the_package_a_new_step_holds() {
    let world = World::new(&[(
        "ch",
        &super::app(&format!(
            "{}{}",
            apt_step("Install", "\"ch-a1\", \"ch-a2\""),
            apt_step("Other", "\"ch-z\"")
        )),
    )]);
    world.install("ch", true).0.unwrap();
    world.write_app(
        "ch",
        &super::app(&format!(
            "{}{}",
            apt_step("Install", "\"ch-a1\""),
            apt_step("Other", "\"ch-z\", \"ch-y\"")
        )),
    );
    world.host.state.borrow_mut().fail.insert("apt:ch-y".to_owned());
    assert!(world.install("ch", true).0.is_err());
    world.host.state.borrow_mut().fail.clear();
    world.write_app(
        "ch",
        &super::app(&format!(
            "{}{}{}",
            apt_step("Keep", "\"ch-a2\""),
            apt_step("Install", "\"ch-a3\""),
            apt_step("Other", "\"ch-z\"")
        )),
    );
    let from = world.host.state.borrow().log.len();

    world.install("ch", true).0.unwrap();

    let log = world.log_since(from);
    assert!(world.host.state.borrow().dpkg.contains_key("ch-a2"));
    assert!(
        !log.iter().any(|l| l.starts_with("apt remove") && l.contains("ch-a2")),
        "{log:#?}"
    );
    world.remove("ch").0.unwrap();
    let state = world.host.state.borrow();
    for package in ["ch-a1", "ch-a2", "ch-a3", "ch-z"] {
        assert!(!state.dpkg.contains_key(package), "{package} stays: {:?}", state.dpkg);
    }
}

#[test]
fn an_older_replaced_part_keeps_the_file_a_new_step_writes() {
    let world = World::new(&[(
        "cf",
        &super::app(&format!(
            "{}{}",
            file_step("Write", "~/.config/cf/a", "a"),
            apt_step("Other", "\"cf-z\"")
        )),
    )]);
    world.install("cf", true).0.unwrap();
    world.write_app(
        "cf",
        &super::app(&format!(
            "{}{}",
            file_step("Write", "~/.config/cf/b", "b"),
            apt_step("Other", "\"cf-z\", \"cf-y\"")
        )),
    );
    world.host.state.borrow_mut().fail.insert("apt:cf-y".to_owned());
    assert!(world.install("cf", true).0.is_err());
    world.host.state.borrow_mut().fail.clear();
    world.write_app(
        "cf",
        &super::app(&format!(
            "{}{}{}",
            file_step("Keep a", "~/.config/cf/a", "a"),
            file_step("Write", "~/.config/cf/c", "c"),
            apt_step("Other", "\"cf-z\"")
        )),
    );
    let a = world.host_home().join(".config/cf/a");

    let events = world.install("cf", true).1;

    assert!(a.is_file(), "the file the new step writes is gone");
    assert_eq!(reported(&events, "Keep a").len(), 1, "{:?}", events.steps());
    assert!(!world.host_home().join(".config/cf/b").exists());
}

#[test]
fn a_step_merged_in_part_leaves_what_apt_pulled_in_to_another_app_that_found_it() {
    let world = World::new(&[
        (
            "pm",
            &super::app(&format!(
                "{}{}",
                apt_step("Install the tool", "\"pm-tool\""),
                apt_step("Install its plugins", "\"pm-plugin\", \"pm-extra\"")
            )),
        ),
        ("oth", &super::app(&apt_step("Install lib", "\"libplugin\""))),
    ]);
    world
        .host
        .state
        .borrow_mut()
        .depends
        .insert("pm-plugin".to_owned(), ["libplugin".to_owned()].into());
    world.install("pm", true).0.unwrap();
    world.install("oth", true).0.unwrap();
    world.write_app(
        "pm",
        &super::app(&apt_step("Install the tool", "\"pm-tool\", \"pm-plugin\"")),
    );

    world.install("pm", true).0.unwrap();

    let owners: Vec<&str> = ["pm", "oth"]
        .into_iter()
        .filter(|id| {
            world.journals(id).1.unwrap().entries.iter().any(|e| {
                e.claims
                    .iter()
                    .any(|c| c.owned && (c.claim == "dep:libplugin" || c.claim == "package:libplugin"))
            })
        })
        .collect();
    assert_eq!(owners, ["oth"]);
    world.remove("pm").0.unwrap();
    assert!(
        world.host.state.borrow().dpkg.contains_key("libplugin"),
        "the other app's package went"
    );
}

#[test]
fn a_step_merged_in_part_killed_after_its_undo_hands_over_what_apt_pulled_in_on_the_next_run() {
    let world = World::new(&[(
        "pk",
        &super::app(&format!(
            "{}{}",
            apt_step("Install the tool", "\"pk-tool\""),
            apt_step("Install its plugins", "\"pk-plugin\", \"pk-extra\"")
        )),
    )]);
    world
        .host
        .state
        .borrow_mut()
        .depends
        .insert("pk-plugin".to_owned(), ["libpk".to_owned()].into());
    world.install("pk", true).0.unwrap();
    world.write_app(
        "pk",
        &super::app(&apt_step("Install the tool", "\"pk-tool\", \"pk-plugin\"")),
    );
    // Killed as the undo of the merged step's other package ends, before its journal is written.
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:pk-extra".to_owned());
    assert!(world.install("pk", true).0.is_err());
    world.host.state.borrow_mut().fail.clear();

    world.install("pk", true).0.unwrap();

    world.remove("pk").0.unwrap();
    let state = world.host.state.borrow();
    for package in ["pk-tool", "pk-plugin", "pk-extra", "libpk"] {
        assert!(!state.dpkg.contains_key(package), "{package} stays: {:?}", state.dpkg);
    }
}
