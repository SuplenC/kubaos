use std::collections::BTreeMap;
use std::time::Duration;

use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;

use super::*;
use crate::fake::FakeBus;
use crate::fake::FakeRunner;
use crate::states::Reading;
use crate::states::States;

fn configured(text: &str) -> Extension<FakeBus, FakeRunner> {
    let mut extension = Extension::new(FakeBus::default(), FakeRunner::default());
    let problems = extension.configure(&toml::from_str(text).expect("the test's config"));
    assert!(problems.is_empty(), "{problems:?}");
    extension
}

fn item(kind: &str, id: &str) -> ItemRef {
    ItemRef {
        id: id.to_owned(),
        kind: ItemKind::Namespaced(NamespacedKind::new(kind).unwrap()),
    }
}

fn ids(extension: &Extension<FakeBus, FakeRunner>) -> Vec<String> {
    extension.items().map(|item| item.id).collect()
}

/// The Items the Extension always sends, in order: the toggles, the idle settings, then the
/// power stop's status and the Proxy's.
fn always() -> Vec<&'static str> {
    toggles::ids()
        .into_iter()
        .chain(crate::idle::ids())
        .chain(crate::shell::ids())
        .chain(crate::power::ids())
        .chain(crate::proxy::ids())
        .collect()
}

#[test]
fn without_a_config_there_are_the_toggles_the_idle_settings_and_the_power_status() {
    let extension = Extension::new(FakeBus::default(), FakeRunner::default());

    assert_eq!(ids(&extension), always());
}

#[test]
fn the_actions_come_before_the_toggles() {
    let extension = configured("[actions.mine]\ntitle = \"Mine\"\nrun = [\"true\"]");

    assert_eq!(ids(&extension).first().map(String::as_str), Some("mine"));
    assert_eq!(ids(&extension).len(), always().len() + 1);
}

#[test]
fn hide_takes_out_the_power_status_too() {
    let extension = configured(r#"hide = ["power-status"]"#);

    assert!(!ids(&extension).contains(&"power-status".to_owned()));
}

#[test]
fn hide_takes_out_the_proxy_status_too() {
    let extension = configured(r#"hide = ["proxy-status"]"#);

    assert!(!ids(&extension).contains(&"proxy-status".to_owned()));
    assert!(ids(&extension).contains(&"power-status".to_owned()));
}

#[test]
fn hide_takes_out_the_cpu_scheduler_status_alone() {
    let extension = configured(r#"hide = ["cpu-scheduler-status"]"#);

    assert!(!ids(&extension).contains(&"cpu-scheduler-status".to_owned()));
    assert!(ids(&extension).contains(&"power-status".to_owned()));
}

#[test]
fn hide_takes_out_a_toggle_too() {
    let extension = configured(r#"hide = ["bluetooth"]"#);

    assert!(!ids(&extension).contains(&"bluetooth".to_owned()));
    assert!(ids(&extension).contains(&"wifi".to_owned()));
}

#[test]
fn a_hidden_toggle_does_not_run() {
    let mut extension = configured(r#"hide = ["mute"]"#);

    let ran = extension.run(toggles::RUN, &item(toggles::KIND, "mute"), &BTreeMap::new());

    assert!(ran.is_err());
    assert_eq!(extension.runner.ran(), [] as [std::string::String; 0]);
}

#[test]
fn an_action_launches_its_command_and_closes_the_menu() {
    let mut extension = configured(
        r#"
        [actions.shot]
        title = "Screenshot"
        run = ["dot-hypr-screenshot", "region"]
        delay_ms = 400
        "#,
    );

    let ran = extension.run(registry::RUN, &item(registry::KIND, "shot"), &BTreeMap::new());

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(
        *extension.runner.launched.borrow(),
        [(
            vec!["dot-hypr-screenshot".to_owned(), "region".to_owned()],
            Duration::from_millis(400)
        )]
    );
}

#[test]
fn an_action_that_cannot_start_says_why() {
    let mut extension = configured("[actions.mine]\ntitle = \"Mine\"\nrun = [\"nothing-here\"]");
    extension.runner.failing = Some("no".into());

    let ran = extension.run(registry::RUN, &item(registry::KIND, "mine"), &BTreeMap::new());

    assert_eq!(ran, Err("couldn't start nothing-here".to_owned()));
}

#[test]
fn a_new_config_replaces_the_actions() {
    let mut extension = configured("[actions.old]\ntitle = \"Old\"\nrun = [\"true\"]");

    extension.configure(&toml::from_str("[actions.new]\ntitle = \"New\"\nrun = [\"true\"]").unwrap());

    assert!(ids(&extension).contains(&"new".to_owned()));
    assert!(!ids(&extension).contains(&"old".to_owned()));
}

fn with_layouts(text: &str) -> Extension<FakeBus, FakeRunner> {
    let mut extension = configured(text);
    extension.set_keyboards(Keyboards::parse(
        "XKBLAYOUT=\"us,it\"\nXKBVARIANT=\",\"\n",
        "! layout\n  us  English (US)\n  it  Italian\n  de  German\n",
    ));
    extension
}

#[test]
fn the_keyboard_items_come_after_the_toggles() {
    let extension = with_layouts("");

    let ids = ids(&extension);

    assert_eq!(ids[..always().len()], always());
    assert_eq!(ids[always().len()..], ["keyboard:us", "keyboard:it", "keyboard-add"]);
}

#[test]
fn a_keyboard_item_runs_through_the_extension() {
    let mut extension = with_layouts("");

    let ran = extension.run(keyboard::RUN, &item(keyboard::KIND, "keyboard:it"), &BTreeMap::new());

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(extension.runner.ran(), ["hyprctl switchxkblayout all 1"]);
}

#[test]
fn hide_takes_out_a_keyboard_item_and_its_run() {
    let mut extension = with_layouts(r#"hide = ["keyboard-add"]"#);
    let typed = BTreeMap::from([("layout".to_owned(), "de".to_owned())]);

    let ran = extension.run(keyboard::RUN, &item(keyboard::KIND, keyboard::ADD), &typed);

    assert!(!ids(&extension).contains(&"keyboard-add".to_owned()));
    assert!(ids(&extension).contains(&"keyboard:us".to_owned()));
    assert!(ran.is_err());
    assert!(extension.runner.launched.borrow().is_empty());
}

#[test]
fn an_action_may_not_take_a_keyboard_items_id() {
    let mut extension = Extension::new(FakeBus::default(), FakeRunner::default());

    let problems = extension.configure(
        &toml::from_str("[actions.keyboard-add]\ntitle = \"Mine\"\nrun = [\"true\"]").expect("the test's config"),
    );

    assert_eq!(problems.len(), 1, "{problems:?}");
}

#[test]
fn an_unknown_item_or_action_says_so() {
    let mut extension = configured("");

    let unknown_item = extension.run(registry::RUN, &item(registry::KIND, "nope"), &BTreeMap::new());
    let unknown_action = extension.run("explode", &item(toggles::KIND, "wifi"), &BTreeMap::new());

    assert_eq!(unknown_item, Err("`nope` has no Action `run`".to_owned()));
    assert!(unknown_action.is_err());
}

fn with_sets(text: &str) -> Extension<FakeBus, FakeRunner> {
    let mut extension = configured(text);
    extension.set_wallpaper_sets(WallpaperSets::parse(
        ["japanese", "shinjuku"].map(str::to_owned),
        Some("shinjuku\n"),
    ));
    extension
}

#[test]
fn the_wallpaper_set_items_come_after_the_keyboard_items() {
    let mut extension = with_sets("");
    extension.set_keyboards(Keyboards::parse("XKBLAYOUT=\"us\"\n", ""));

    let ids = ids(&extension);

    assert_eq!(
        ids[always().len()..],
        [
            "keyboard:us",
            "keyboard-add",
            "wallpaper-set:japanese",
            "wallpaper-set:shinjuku"
        ]
    );
}

#[test]
fn a_wallpaper_set_item_runs_through_the_extension() {
    let mut extension = with_sets("");

    let ran = extension.run(
        wallpapers::SHOW,
        &item(wallpapers::KIND, "wallpaper-set:japanese"),
        &BTreeMap::new(),
    );

    assert_eq!(ran, Ok(Ran::KeepOpen));
    assert_eq!(extension.runner.ran(), ["dot-theme show-set japanese"]);
}

const STICK: &str = "/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234";

fn with_stick(text: &str) -> Extension<FakeBus, FakeRunner> {
    let mut extension = configured(text);
    extension.set_drives(Drives::parse(&format!("{STICK}\tSTICK\t/media/kuba/STICK\n")));
    extension
}

#[test]
fn a_drive_item_comes_last_and_ejects_through_the_extension() {
    let mut extension = with_stick("");

    let ran = extension.run(
        drives::EJECT,
        &item(drives::KIND, "drive:SanDisk_Ultra_1234"),
        &BTreeMap::new(),
    );

    assert_eq!(
        ids(&extension).last().map(String::as_str),
        Some("drive:SanDisk_Ultra_1234")
    );
    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(
        extension.runner.launched.borrow()[0].0,
        ["kubaos-automount", "eject", STICK]
    );
}

#[test]
fn a_drive_hidden_by_id_is_neither_listed_nor_ejected() {
    let mut extension = Extension::new(FakeBus::default(), FakeRunner::default());
    // A drive comes and goes, so its id is never a known one; `hide` warns and still takes it
    // out, as it does a user folder's.
    let problems = extension.configure(&toml::from_str("hide = [\"drive:SanDisk_Ultra_1234\"]").unwrap());
    extension.set_drives(Drives::parse(&format!("{STICK}\tSTICK\t/media/kuba/STICK\n")));

    let ran = extension.run(
        drives::EJECT,
        &item(drives::KIND, "drive:SanDisk_Ultra_1234"),
        &BTreeMap::new(),
    );

    assert!(!ids(&extension).contains(&"drive:SanDisk_Ultra_1234".to_owned()));
    assert!(ran.is_err());
    assert_eq!(problems.len(), 1, "{problems:?}");
}

#[test]
fn a_desk_item_comes_before_the_drives_and_goes_to_its_group_through_the_extension() {
    let mut extension = with_stick("");
    extension.set_desk(Desk::parse("1\tDP-1\t1\tLG ULTRAGEAR\n"));

    let ran = extension.run(desk::SHOW, &item(desk::KIND, "desk:1"), &BTreeMap::new());

    let ids = ids(&extension);
    assert_eq!(&ids[ids.len() - 2..], ["desk:1", "drive:SanDisk_Ultra_1234"]);
    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(
        extension.runner.launched.borrow()[0].0,
        ["hyprctl", "dispatch", "dot.workspaces.go_to_position(\"1\")"]
    );
}

#[test]
fn the_states_choose_which_half_of_a_pair_is_sent() {
    let mut extension = configured(
        r#"
        [actions.chrome-on]
        title = "Install Google Chrome"
        run = ["kubaos", "packages", "enable", "chrome"]
        shown_when = { state = "packages.chrome", is = false }
        [actions.chrome-off]
        title = "Remove Google Chrome"
        run = ["kubaos", "packages", "disable", "chrome"]
        shown_when = { state = "packages.chrome", is = true }
        "#,
    );

    let unknown = ids(&extension);
    extension.set_states(States::default().with("packages.chrome", Reading::Bool(true)));
    let installed = ids(&extension);

    assert_eq!(&unknown[..2], ["chrome-off", "chrome-on"]);
    assert_eq!(installed.first().map(String::as_str), Some("chrome-off"));
    assert!(!installed.contains(&"chrome-on".to_owned()));
}

const CHROME: &str = r#"
    terminal = ["ghostty", "-e"]
    [actions.chrome]
    title = "Google Chrome"
    identity = "app:google-chrome.desktop"
    state = "packages.chrome"
    remove = { run = ["kubaos", "packages", "disable", "chrome"], terminal = true }
    [actions.chrome.actions.install]
    title = "Install and open"
    keywords = ["install"]
    run = ["kubaos", "packages", "enable", "chrome"]
    "#;

#[test]
fn each_action_of_an_item_with_several_runs_through_the_extension() {
    let mut extension = configured(CHROME);

    let ran = extension.run("install", &item(registry::KIND, "chrome"), &BTreeMap::new());
    let unknown = extension.run(registry::RUN, &item(registry::KIND, "chrome"), &BTreeMap::new());

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(
        extension.runner.launched.borrow()[0].0,
        ["kubaos", "packages", "enable", "chrome"]
    );
    assert_eq!(unknown, Err("`chrome` has no Action `run`".to_owned()));
}

#[test]
fn remove_on_an_installed_app_runs_its_placeholders_remove() {
    let mut extension = configured(CHROME);
    let app = ItemRef {
        id: "google-chrome".to_owned(),
        kind: ItemKind::App {
            desktop_id: "google-chrome.desktop".to_owned(),
        },
    };

    let ran = extension.run(registry::REMOVE, &app, &BTreeMap::new());

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(
        extension.runner.launched.borrow()[0].0,
        ["ghostty", "-e", "kubaos", "packages", "disable", "chrome"]
    );
}

#[test]
fn remove_on_a_desktop_file_runs_the_remove_of_its_app() {
    let mut extension = configured(CHROME);
    let file = ItemRef {
        id: "/usr/share/applications/google-chrome.desktop".to_owned(),
        kind: ItemKind::File {
            path: "/usr/share/applications/google-chrome.desktop".into(),
        },
    };

    let ran = extension.run(registry::REMOVE, &file, &BTreeMap::new());

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(extension.runner.launched.borrow().len(), 1);
}

#[test]
fn remove_on_an_app_no_placeholder_stands_for_says_so() {
    let mut extension = configured(CHROME);
    let app = ItemRef {
        id: "firefox".to_owned(),
        kind: ItemKind::App {
            desktop_id: "firefox.desktop".to_owned(),
        },
    };

    let ran = extension.run(registry::REMOVE, &app, &BTreeMap::new());

    assert_eq!(
        ran,
        Err("KubaOS didn't install `firefox`, so it can't remove it".to_owned())
    );
    assert!(extension.runner.launched.borrow().is_empty());
}

#[test]
fn remove_is_aimed_at_what_the_states_say_is_installed() {
    let mut extension = configured(CHROME);

    extension.set_states(States::default().with("packages.chrome", Reading::Bool(false)));
    let missing = extension.targets();
    extension.set_states(States::default().with("packages.chrome", Reading::Bool(true)));
    let installed = extension.targets();

    assert_eq!(missing, Vec::<String>::new());
    assert_eq!(installed, ["app:google-chrome.desktop"]);
}
