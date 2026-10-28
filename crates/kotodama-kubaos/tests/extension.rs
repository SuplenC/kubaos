//! The Extension's binary over its stdio, as the Daemon runs it.

use std::collections::BTreeMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

use kotodama_extension::testing::Spawned;
use kotodama_protocol::Clipboard;
use kotodama_protocol::FromExtension;
use kotodama_protocol::ItemKind;
use kotodama_protocol::ItemRef;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::Outcome;
use kotodama_protocol::Table;
use kotodama_protocol::ToExtension;

const WAIT: Duration = Duration::from_secs(5);

fn start() -> Spawned {
    Spawned::start(&mut Command::new(env!("CARGO_BIN_EXE_kotodama-kubaos")), WAIT)
}

fn config(text: &str) -> ToExtension {
    ToExtension::Config(toml::from_str::<Table>(text).expect("the test's config"))
}

fn action(id: &str) -> ItemRef {
    ItemRef {
        id: id.to_owned(),
        kind: ItemKind::Namespaced(NamespacedKind::new("kubaos.action").unwrap()),
    }
}

#[test]
fn it_is_usable_from_the_start() {
    let extension = start();

    assert_eq!(extension.status(), (true, None));
}

#[test]
fn the_layouts_app_is_usable_from_the_start() {
    let config_home = tempfile::tempdir().expect("a config folder");
    let extension = Spawned::start(
        Command::new(env!("CARGO_BIN_EXE_kotodama-kubaos"))
            .arg("layouts")
            .env("XDG_CONFIG_HOME", config_home.path()),
        WAIT,
    );

    assert_eq!(extension.status(), (true, None));
}

#[test]
fn a_user_action_shows_and_a_hidden_one_does_not() {
    let mut extension = start();

    extension.send(config(
        r#"
        hide = ["theme-next", "bluetooth"]
        [actions.theme-next]
        title = "Next wallpaper"
        run = ["dot-theme", "next"]
        [actions.my-backup]
        title = "Back up my notes"
        run = ["true"]
        "#,
    ));
    let ids: Vec<String> = extension.added().into_iter().map(|item| item.id).collect();

    assert!(ids.contains(&"my-backup".to_owned()), "{ids:?}");
    assert!(ids.contains(&"wifi".to_owned()), "{ids:?}");
    assert!(!ids.contains(&"theme-next".to_owned()), "{ids:?}");
    assert!(!ids.contains(&"bluetooth".to_owned()), "{ids:?}");
}

#[test]
fn hiding_an_action_later_takes_its_item_away() {
    let mut extension = start();
    extension.send(config("[actions.mine]\ntitle = \"Mine\"\nrun = [\"true\"]"));
    extension.added();

    extension.send(config(
        "hide = [\"mine\"]\n[actions.mine]\ntitle = \"Mine\"\nrun = [\"true\"]",
    ));

    assert_eq!(extension.removed(), ["mine"]);
}

#[test]
fn the_upgrade_action_is_listed_while_the_update_check_has_one() {
    let state = tempfile::tempdir().expect("a state folder");
    let dir = state.path().join("kubaos");
    std::fs::create_dir_all(&dir).expect("make the state folder");
    std::fs::write(
        dir.join("update-available"),
        "upgrade=0.2.0\nupgrade-codename=akihabara\n",
    )
    .expect("write the check");
    let mut extension = Spawned::start(
        Command::new(env!("CARGO_BIN_EXE_kotodama-kubaos")).env("XDG_STATE_HOME", state.path()),
        WAIT,
    );
    extension.send(config(
        "[actions.upgrade]\ntitle = \"Upgrade KubaOS\"\nrun = [\"kubaos\", \"upgrade\"]\n\
         [actions.update]\ntitle = \"Update KubaOS\"\nrun = [\"kubaos\", \"update\"]",
    ));
    let added = extension.added();
    let upgrade = added
        .iter()
        .find(|item| item.id == "upgrade")
        .expect("the upgrade Item");
    assert_eq!(upgrade.title, "Upgrade to Akihabara");

    std::fs::write(dir.join("update-available.tmp"), "").expect("write the check");
    std::fs::rename(dir.join("update-available.tmp"), dir.join("update-available")).expect("replace the check");

    assert_eq!(extension.removed(), ["upgrade"]);
}

#[test]
fn turning_the_greeting_off_swaps_its_pair_in_the_menu() {
    let config_home = tempfile::tempdir().expect("a config folder");
    let runtime = tempfile::tempdir().expect("a runtime folder");
    let mut extension = Spawned::start(
        Command::new(env!("CARGO_BIN_EXE_kotodama-kubaos"))
            .env("XDG_CONFIG_HOME", config_home.path())
            .env("XDG_RUNTIME_DIR", runtime.path()),
        WAIT,
    );
    extension.send(config(
        r#"
        [actions.greeting-on]
        title = "Terminal greeting on"
        run = ["dot-greeting", "on"]
        shown_when = { state = "terminal.greeting", is = false }
        [actions.greeting-off]
        title = "Terminal greeting off"
        run = ["dot-greeting", "off"]
        shown_when = { state = "terminal.greeting", is = true }
        "#,
    ));
    let first: Vec<String> = extension.added().into_iter().map(|item| item.id).collect();

    let zsh = config_home.path().join("kubaos/terminal/zsh");
    std::fs::create_dir_all(&zsh).expect("make the zsh folder");
    std::fs::write(zsh.join("no-greeting"), "").expect("turn the greeting off");
    let removed = extension.removed();
    let added: Vec<String> = extension.added().into_iter().map(|item| item.id).collect();

    assert!(first.contains(&"greeting-off".to_owned()), "{first:?}");
    assert!(!first.contains(&"greeting-on".to_owned()), "{first:?}");
    assert_eq!(added, ["greeting-on"]);
    assert_eq!(removed, ["greeting-off"]);
}

#[test]
fn running_an_action_starts_its_command_and_closes_the_menu() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("ran");
    let mut extension = start();
    extension.send(config(&format!(
        "[actions.touch]\ntitle = \"Touch\"\nrun = [\"touch\", {:?}]",
        marker.display().to_string()
    )));
    extension.added();

    extension.start_run(1, "run", action("touch"), BTreeMap::new());

    assert_eq!(extension.outcome(1, |_| false), Outcome::Close);
    for _ in 0..100 {
        if marker.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("the action's command never ran");
}

#[test]
fn a_run_that_fails_says_why_and_keeps_the_menu_open() {
    let mut extension = start();
    extension.send(config(""));
    extension.added();
    let volume = ItemRef {
        id: "volume".to_owned(),
        kind: ItemKind::Namespaced(NamespacedKind::new("kubaos.toggle").unwrap()),
    };

    extension.start_run(
        2,
        "run",
        volume,
        BTreeMap::from([("percent".to_owned(), "loud".to_owned())]),
    );

    assert_eq!(
        extension.outcome(2, |message| matches!(message, FromExtension::PushView(_))),
        Outcome::Failed {
            message: "Type a percentage from 0 to 100, not `loud`".to_owned()
        }
    );
}

#[test]
fn a_date_query_is_answered_and_its_answer_copies() {
    let config = tempfile::tempdir().expect("a config folder");
    let mut extension = Spawned::start(
        Command::new(env!("CARGO_BIN_EXE_kotodama-kubaos")).env("XDG_CONFIG_HOME", config.path()),
        WAIT,
    );
    extension.status();

    extension.send(ToExtension::LiveQuery {
        id: 3,
        query: "days until 2099-01-01".to_owned(),
        prefix: None,
    });
    let FromExtension::LiveResults {
        live_query,
        results,
        done,
    } = extension.recv()
    else {
        panic!("no Live results");
    };
    assert_eq!((live_query, done, results.len()), (3, true, 1));
    assert!(results[0].answer);
    assert!(results[0].item.title.ends_with(" days"), "{}", results[0].item.title);

    let date = ItemRef {
        id: "days until 2099-01-01".to_owned(),
        kind: ItemKind::Namespaced(NamespacedKind::new("kubaos.date").unwrap()),
    };
    extension.start_run(1, "copy-iso", date, BTreeMap::new());
    let Outcome::Copy(Clipboard::Text(iso)) = extension.outcome(1, |_| false) else {
        panic!("the answer wasn't copied");
    };
    assert!(iso.starts_with('P') && iso.ends_with('D'), "{iso}");
}

#[test]
fn a_query_that_is_not_a_date_is_answered_with_nothing() {
    let mut extension = start();
    extension.status();

    extension.send(ToExtension::LiveQuery {
        id: 4,
        query: "next wallpaper".to_owned(),
        prefix: None,
    });

    assert_eq!(
        extension.recv(),
        FromExtension::LiveResults {
            live_query: 4,
            results: vec![],
            done: true
        }
    );
}

#[test]
fn ram_is_answered_by_the_monitor() {
    let runtime = tempfile::tempdir().expect("a runtime folder");
    let mut extension = Spawned::start(
        Command::new(env!("CARGO_BIN_EXE_kotodama-kubaos")).env("XDG_RUNTIME_DIR", runtime.path()),
        WAIT,
    );
    extension.status();

    extension.send(ToExtension::LiveQuery {
        id: 5,
        query: "ram".to_owned(),
        prefix: None,
    });
    let FromExtension::LiveResults { results, done, .. } = extension.recv() else {
        panic!("no Live results");
    };

    assert!(done);
    assert_eq!(results.len(), 1);
    assert!(results[0].answer);
    assert_eq!(results[0].item.kind.name(), "kubaos.sysmon");
    assert!(results[0].item.title.starts_with("RAM"), "{}", results[0].item.title);
}
