use kotodama_protocol::NamespacedKind;

use super::*;
use crate::fake::FakeRunner;

fn apps(apps: &[&str], starting: &[&str]) -> StartupApps {
    let set = |ids: &[&str]| ids.iter().map(|&id| id.to_owned()).collect();
    StartupApps {
        apps: set(apps),
        starting: set(starting),
    }
}

fn app(desktop_id: &str) -> ItemKind {
    ItemKind::App {
        desktop_id: desktop_id.to_owned(),
    }
}

#[test]
fn start_is_offered_on_the_apps_that_do_not_start_and_stop_on_every_startup_app() {
    let startup = apps(
        &["chrome.desktop", "solaar.desktop"],
        &["solaar.desktop", "user-dirs.desktop"],
    );

    let [(start, to_start), (stop, to_stop)] = startup.targets();

    assert_eq!((start, to_start), (START, vec!["app:chrome.desktop".to_owned()]));
    assert_eq!(
        (stop, to_stop),
        (
            STOP,
            vec!["app:solaar.desktop".to_owned(), "app:user-dirs.desktop".to_owned()]
        )
    );
}

#[test]
fn no_apps_withdraws_both_actions() {
    let [(_, to_start), (_, to_stop)] = StartupApps::default().targets();

    assert!(to_start.is_empty());
    assert!(to_stop.is_empty());
}

#[test]
fn each_action_runs_kubaos_autostart_on_the_apps_desktop_id() {
    let runner = FakeRunner::default();

    run(START, &app("solaar.desktop"), &runner).unwrap();
    run(STOP, &app("discord.desktop"), &runner).unwrap();

    assert_eq!(
        runner.ran(),
        [
            "kubaos autostart on solaar.desktop",
            "kubaos autostart off discord.desktop"
        ]
    );
}

#[test]
fn a_failed_change_is_what_the_user_reads() {
    let runner = FakeRunner {
        failing: Some("kubaos: writing ~/.config/autostart/x.desktop: No space left".to_owned()),
        ..FakeRunner::default()
    };

    let err = run(START, &app("x.desktop"), &runner).unwrap_err();

    assert!(err.contains("No space left"), "{err}");
}

#[test]
fn an_item_that_is_not_an_app_is_refused() {
    let runner = FakeRunner::default();
    let kind = ItemKind::Namespaced(NamespacedKind::new("kubaos.drive").unwrap());

    assert!(run(START, &kind, &runner).is_err());
    assert!(run("open", &app("x.desktop"), &runner).is_err());
    assert!(runner.ran().is_empty());
}

#[test]
fn the_manifest_declares_both_actions_targeted() {
    let manifest = kotodama_protocol::Manifest::from_toml(include_str!("../../manifest.toml")).unwrap();

    for id in [START, STOP] {
        let declared = manifest
            .contributed_actions
            .iter()
            .find(|contributed| contributed.action.id == id)
            .unwrap_or_else(|| panic!("{id} isn't in the manifest"));
        assert_eq!(declared.to, kotodama_protocol::ContributedTo::Identities, "{id}");
    }
}
