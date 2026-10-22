//! The Extension's binary over its stdio, as the Daemon runs it.

use std::collections::BTreeMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

use kotodama_extension::testing::Spawned;
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
