use std::fs;
use std::path::PathBuf;

use super::super::Engine;
use super::super::EngineError;
use super::super::Keeper;
use super::super::Ran;
use super::Events;
use super::World;
use crate::apps::action::Side;
use crate::apps::host::fake::FakeRoot;
use crate::apps::journal::Journal;
use crate::apps::picks::Picks;
use crate::update::event::UpdateEvent;

const ABOUT: &str = "summary = \"An app for the tests\"\ncategory = \"development\"\ntags = [\"editor\"]\n\
                     description = \"An app the tests install.\"\ndescription_from = \"https://example.com/test\"\n";

/// An editor from apt, with a file of its own as root.
fn neovim() -> String {
    format!(
        r#"name = "Neovim"
{ABOUT}
[[action]]
name = "Install Neovim"
kind = "apt"
packages = ["neovim"]

[[action]]
name = "Write its system-wide defaults"
kind = "file"
to = "/etc/xdg/nvim/sysinit.vim"
text = "set nocompatible\n"
"#
    )
}

/// Neovim's distribution: only its own part, its config and the KubaOS defaults it reads, a
/// configure step.
fn lazyvim(requires: &str) -> String {
    format!(
        r#"name = "LazyVim"
{ABOUT}requires = [{requires}]

[[action]]
name = "Write its config"
kind = "file"
to = "~/.config/nvim/init.lua"
text = "require('config.lazy')\n"

[[action]]
name = "Write KubaOS's defaults"
kind = "file"
stage = "configure"
to = "~/.config/kubaos/lazyvim/defaults.lua"
text = "vim.o.number = true\n"

[[action]]
name = "Sync its plugins"
kind = "run"
do = ["./sync"]
check = ["./synced"]
undo = ["./unsync"]
"#
    )
}

/// Another app on Neovim, to share it with LazyVim.
fn notes() -> String {
    format!(
        r#"name = "Notes"
{ABOUT}requires = ["neovim"]

[[action]]
name = "Write the notes plugin"
kind = "file"
to = "~/.config/notes/notes.lua"
text = "return {{}}\n"
"#
    )
}

fn world(requires: &str) -> World {
    let (neovim, lazyvim, notes) = (neovim(), lazyvim(requires), notes());
    let world = World::new(&[("neovim", &neovim), ("lazyvim", &lazyvim), ("notes", &notes)]);
    world.script("lazyvim", "sync", r#"touch "$HOME/synced""#);
    world.script("lazyvim", "synced", r#"[ -e "$HOME/synced" ]"#);
    world.script("lazyvim", "unsync", r#"rm -f "$HOME/synced""#);
    world
        .host
        .state
        .borrow_mut()
        .candidates
        .insert("neovim".to_owned(), "0.10.2-1".to_owned());
    world
}

fn picks(ids: &[&str]) -> Picks {
    let mut picks = Picks::default();
    for id in ids {
        picks.add(id);
    }
    picks
}

impl World {
    /// Runs `go` with an engine of this world, reporting to `events`.
    fn with<T>(
        &self,
        events: &mut Events,
        go: impl FnOnce(&mut Engine<'_>) -> Result<T, EngineError>,
    ) -> (Result<T, EngineError>, FakeRoot) {
        let mut root = self.root();
        let mut user_host = self.host.clone();
        let keeper = Keeper::new(&mut user_host, Side::User, &self.catalog, &self.user_journals);
        let mut engine = Engine::new(keeper, &mut root, &self.root_journals, &self.lock, events, true);
        let result = go(&mut engine);
        (result, root)
    }

    fn add(&self, id: &str, picked: &[&str]) -> (Result<Vec<Ran>, EngineError>, Events) {
        let mut events = Events::default();
        let (result, _) = self.with(&mut events, |e| e.add(id, None, &picks(picked)));
        (result, events)
    }

    fn remove_with(&self, id: &str, picked: &[&str]) -> (Result<Vec<Ran>, EngineError>, Events) {
        let mut events = Events::default();
        let (result, _) = self.with(&mut events, |e| e.remove_with(id, &picks(picked)));
        (result, events)
    }

    fn home_file(&self, path: &str) -> PathBuf {
        self.host_home().join(path)
    }
}

impl Events {
    /// The apps the run went through, as `name n/of`.
    fn apps(&self) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|e| match e {
                UpdateEvent::App(app) => Some(format!("{} {}/{}", app.name, app.n, app.of)),
                _ => None,
            })
            .collect()
    }
}

fn names(ran: &[Ran]) -> Vec<&str> {
    ran.iter().map(|r| r.name.as_str()).collect()
}

#[test]
fn installing_lazyvim_installs_neovim_first_through_its_own_folder_and_journal() {
    let world = world("{ id = \"neovim\", min = \"0.10\" }");

    let (result, events) = world.add("lazyvim", &["lazyvim"]);

    assert_eq!(names(&result.unwrap()), ["Neovim", "LazyVim"]);
    assert_eq!(events.apps(), ["Neovim 1/2", "LazyVim 2/2"]);
    assert_eq!(world.host.state.borrow().dpkg["neovim"], "1.0");
    assert!(world.home_file(".config/nvim/init.lua").is_file());
    let (neovim, neovim_root) = world.journals("neovim");
    let neovim = neovim.unwrap();
    assert!(neovim.complete);
    assert_eq!(neovim.brought_by, ["lazyvim"]);
    assert!(neovim.entries.is_empty(), "Neovim's steps are root's: {neovim:?}");
    assert_eq!(neovim_root.unwrap().entries.len(), 2);
    let lazyvim = world.journals("lazyvim").0.unwrap();
    assert!(lazyvim.complete);
    assert!(lazyvim.brought_by.is_empty());
    assert!(lazyvim.entries.iter().all(|e| e.action.name != "Install Neovim"));
}

#[test]
fn a_min_the_required_apps_source_cant_meet_is_refused_before_anything_runs() {
    let world = world("{ id = \"neovim\", min = \"0.11\" }");
    let before = world.host.snapshot();
    let mut events = Events::default();

    let (result, root) = world.with(&mut events, |e| e.add("lazyvim", None, &picks(&["lazyvim"])));

    let Err(EngineError::Refused { app, why }) = result else {
        panic!("the install went on: {result:?}");
    };
    assert_eq!(app, "LazyVim");
    assert_eq!(
        why,
        ["it needs Neovim 0.11 or newer, and this machine would get 0.10.2-1"]
    );
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("neovim"), (None, None));
    assert_eq!(world.journals("lazyvim"), (None, None));
    assert!(root.calls.borrow().is_empty(), "{:?}", root.calls.borrow());
}

#[test]
fn a_min_is_met_by_the_version_the_machine_has_already() {
    let world = world("{ id = \"neovim\", min = \"0.11\" }");
    world
        .host
        .state
        .borrow_mut()
        .dpkg
        .insert("neovim".to_owned(), "0.11.1".to_owned());

    let (result, _) = world.add("lazyvim", &["lazyvim"]);

    assert_eq!(names(&result.unwrap()), ["Neovim", "LazyVim"]);
}

#[test]
fn removing_neovim_while_lazyvim_is_installed_is_refused_naming_lazyvim() {
    let world = world("\"neovim\"");
    world.add("lazyvim", &["lazyvim"]).0.unwrap();
    let before = world.host.snapshot();

    let (result, events) = world.remove_with("neovim", &["lazyvim"]);

    let Err(e) = result else {
        panic!("Neovim went");
    };
    assert_eq!(e.to_string(), "Neovim is needed by LazyVim: remove that first");
    assert!(events.0.is_empty(), "{:?}", events.0);
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn removing_lazyvim_takes_the_neovim_it_brought_and_leaves_the_machine_as_before() {
    let world = world("\"neovim\"");
    let before = world.host.snapshot();
    world.add("lazyvim", &["lazyvim"]).0.unwrap();

    let (result, events) = world.remove_with("lazyvim", &[]);

    assert_eq!(names(&result.unwrap()), ["LazyVim", "Neovim"]);
    assert_eq!(events.apps(), ["LazyVim 1/2", "Neovim 2/2"]);
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("neovim"), (None, None));
    assert_eq!(world.journals("lazyvim"), (None, None));
}

#[test]
fn a_required_app_the_user_picked_stays_when_the_app_that_required_it_goes() {
    let world = world("\"neovim\"");
    world.add("neovim", &["neovim"]).0.unwrap();
    world.add("lazyvim", &["neovim", "lazyvim"]).0.unwrap();

    let (result, _) = world.remove_with("lazyvim", &["neovim"]);

    assert_eq!(names(&result.unwrap()), ["LazyVim"]);
    let neovim = world.journals("neovim").0.unwrap();
    assert!(neovim.complete && neovim.brought_by.is_empty(), "{neovim:?}");
    assert!(world.host.state.borrow().dpkg.contains_key("neovim"));
}

#[test]
fn a_required_app_the_user_picks_after_it_came_stays_with_them() {
    let world = world("\"neovim\"");
    world.add("lazyvim", &["lazyvim"]).0.unwrap();

    let (result, _) = world.remove_with("lazyvim", &["neovim"]);

    assert_eq!(names(&result.unwrap()), ["LazyVim"]);
    assert!(world.host.state.borrow().dpkg.contains_key("neovim"));
}

#[test]
fn a_required_app_two_apps_brought_goes_with_the_last_of_them() {
    let world = world("\"neovim\"");
    fs::create_dir_all(world.home_file(".config")).unwrap();
    let before = world.host.snapshot();
    world.add("lazyvim", &["lazyvim"]).0.unwrap();
    world.add("notes", &["lazyvim", "notes"]).0.unwrap();
    assert_eq!(world.journals("neovim").0.unwrap().brought_by, ["lazyvim", "notes"]);

    let (first, _) = world.remove_with("lazyvim", &["notes"]);

    assert_eq!(names(&first.unwrap()), ["LazyVim"]);
    assert_eq!(world.journals("neovim").0.unwrap().brought_by, ["notes"]);
    let (last, _) = world.remove_with("notes", &[]);
    assert_eq!(names(&last.unwrap()), ["Notes", "Neovim"]);
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_failed_install_takes_back_the_required_app_it_brought() {
    let world = world("\"neovim\"");
    world.script("lazyvim", "sync", "exit 1");
    let before = world.host.snapshot();

    let (result, _) = world.add("lazyvim", &["lazyvim"]);

    let Err(EngineError::Step { name, .. }) = result else {
        panic!("the install went through: {result:?}");
    };
    assert_eq!(name, "Sync its plugins");
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("neovim"), (None, None));
    assert_eq!(world.journals("lazyvim"), (None, None));
}

#[test]
fn a_failed_install_keeps_a_required_app_that_was_there() {
    let world = world("\"neovim\"");
    world.add("neovim", &["neovim"]).0.unwrap();
    world.script("lazyvim", "sync", "exit 1");

    let (result, _) = world.add("lazyvim", &["neovim", "lazyvim"]);

    assert!(result.is_err());
    assert!(world.journals("neovim").0.unwrap().complete);
    assert!(world.host.state.borrow().dpkg.contains_key("neovim"));
}

#[test]
fn the_configure_stage_runs_after_the_install_and_alone_again_on_its_own() {
    let world = world("\"neovim\"");
    let defaults = world.home_file(".config/kubaos/lazyvim/defaults.lua");

    let (_, events) = world.add("lazyvim", &["lazyvim"]);

    let steps: Vec<String> = events
        .0
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Step { text } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        steps,
        [
            "Install Neovim",
            "Write its system-wide defaults",
            "Write its config",
            "Sync its plugins",
            "Write KubaOS's defaults",
        ]
    );
    fs::write(&defaults, "vim.o.number = false\n").unwrap();
    let mut events = Events::default();
    let (configured, _) = world.with(&mut events, |e| e.configure("lazyvim"));
    assert_eq!(configured.unwrap().name, "LazyVim");
    assert_eq!(fs::read_to_string(&defaults).unwrap(), "vim.o.number = true\n");
    let ran: Vec<(usize, usize)> = events
        .0
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Action(a) => Some((a.step, a.of)),
            _ => None,
        })
        .collect();
    assert_eq!(ran, [(3, 3)]);
}

#[test]
fn removing_an_app_undoes_its_configure_steps_with_the_rest() {
    let world = world("\"neovim\"");
    let before = world.host.snapshot();
    world.add("lazyvim", &["lazyvim"]).0.unwrap();
    let mut events = Events::default();
    world.with(&mut events, |e| e.configure("lazyvim")).0.unwrap();

    world.remove_with("lazyvim", &[]).0.unwrap();

    assert!(!world.home_file(".config/kubaos/lazyvim").exists());
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn configuring_an_app_not_installed_or_without_a_configure_stage_is_refused() {
    let world = world("\"neovim\"");
    let mut events = Events::default();

    let (missing, _) = world.with(&mut events, |e| e.configure("lazyvim"));
    world.add("neovim", &["neovim"]).0.unwrap();
    let (none, _) = world.with(&mut events, |e| e.configure("neovim"));

    assert_eq!(missing.unwrap_err().to_string(), "LazyVim isn't installed");
    assert_eq!(none.unwrap_err().to_string(), "Neovim has nothing to configure");
}

#[test]
fn a_removal_says_what_it_keeps_on_purpose() {
    let mut text = neovim();
    text = text.replacen(
        "[[action]]",
        "keeps = \"Your swap files stay in ~/.local/state/nvim.\"\n\n[[action]]",
        1,
    );
    let world = World::new(&[("neovim", &text)]);
    world.add("neovim", &["neovim"]).0.unwrap();

    let (_, events) = world.remove_with("neovim", &[]);

    assert!(
        events.0.contains(&UpdateEvent::Note {
            text: "Your swap files stay in ~/.local/state/nvim.".to_owned()
        }),
        "{:?}",
        events.0
    );
}

#[test]
fn a_required_app_left_by_a_journal_that_isnt_installed_goes_with_the_next_removal_of_its_bringer() {
    let world = world("\"neovim\"");
    world.add("lazyvim", &["lazyvim"]).0.unwrap();
    let mut journal = Journal::read(&world.user_journals, "neovim").unwrap().unwrap();
    journal.brought_by = vec!["lazyvim".to_owned(), "gone".to_owned()];
    journal.save(&world.user_journals).unwrap();

    let (result, _) = world.remove_with("lazyvim", &[]);

    assert_eq!(names(&result.unwrap()), ["LazyVim", "Neovim"]);
}
