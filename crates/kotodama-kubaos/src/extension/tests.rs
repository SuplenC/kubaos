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

/// The Items the Extension always sends, in order: the toggles, the idle settings, the shell's,
/// then the cursor.
fn always() -> Vec<&'static str> {
    toggles::ids()
        .into_iter()
        .chain(crate::idle::ids())
        .chain(crate::shell::ids())
        .chain(crate::cursor::ids())
        .collect()
}

#[test]
fn without_a_config_there_are_the_toggles_the_idle_settings_and_the_cursor() {
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
fn hide_takes_out_the_cursor_too() {
    let extension = configured(r#"hide = ["cursor"]"#);

    assert!(!ids(&extension).contains(&"cursor".to_owned()));
}

#[test]
fn the_cursor_follows_its_settings_and_runs_its_actions() {
    let mut extension = configured("");
    extension.set_states(States::default().with_cursor(crate::states::CursorSettings {
        style: kubaos_cursor::settings::Style::Classic,
        size: 48,
    }));

    let cursor = extension.items().find(|item| item.id == "cursor").expect("the cursor");
    let ran = extension.run("accent", &item(crate::cursor::KIND, "cursor"), &BTreeMap::new());

    assert!(
        cursor
            .subtitle
            .as_deref()
            .is_some_and(|subtitle| subtitle.starts_with("Classic, 48 px"))
    );
    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(
        extension.runner.launched.borrow()[0].0,
        ["kubaos", "cursor", "style", "accent"]
    );
}

#[test]
fn a_registry_action_with_a_view_pushes_it() {
    let mut extension = configured(
        r#"
        [actions.lavd]
        title = "CPU scheduler: LAVD"

        [actions.lavd.actions.status]
        title = "Status"
        keywords = ["status"]
        view = "cpu-scheduler"
        "#,
    );

    let ran = extension.run("status", &item(registry::KIND, "lavd"), &BTreeMap::new());

    let Ok(Ran::Push(view)) = ran else {
        panic!("no View: {ran:?}");
    };
    assert_eq!(view.id, crate::power::SCHEDULER_VIEW_ID);
    assert!(extension.runner.launched.borrow().is_empty());
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
fn the_states_choose_which_actions_are_sent() {
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

/// The Extension under the Defaults' `[extensions.kubaos]`.
fn shipped() -> Extension<FakeBus, FakeRunner> {
    let defaults: toml::Table =
        toml::from_str(include_str!("../../../../config/kotodama/kubaos.toml")).expect("kubaos.toml");
    let kubaos = &defaults["extensions"]["kubaos"];
    let mut extension = Extension::new(FakeBus::default(), FakeRunner::default());
    let problems = extension.configure(&toml::from_str(&toml::to_string(kubaos).unwrap()).unwrap());
    assert!(problems.is_empty(), "{problems:?}");
    extension
}

/// The keywords the query word `word` names, as Kotodama's verbs table has them: the whole
/// keyword, or every one it starts with three letters or more.
fn named<'a>(verbs: &'a [String], word: &str) -> Vec<&'a str> {
    verbs
        .iter()
        .filter(|verb| **verb == word || (word.chars().count() >= 3 && verb.starts_with(word)))
        .map(String::as_str)
        .collect()
}

/// Whether every word of `words` is in `text` as Kotodama finds a word of a query: a single letter
/// at a word's start, a longer one anywhere, its letters in order.
fn found_in(text: &str, words: &[String]) -> bool {
    let text = text.to_lowercase();
    words.iter().all(|word| {
        let mut letters = word.chars();
        if let (Some(letter), None) = (letters.clone().next(), letters.clone().nth(1)) {
            return text.char_indices().any(|(at, c)| {
                c == letter && text[..at].chars().last().is_none_or(|before| !before.is_alphanumeric())
            });
        }
        let mut rest = text.chars();
        letters.all(|c| rest.any(|t| t == c))
    })
}

#[test]
fn a_former_items_title_finds_its_subject_with_that_action_chosen() {
    let extension = shipped();
    let items: Vec<Item> = extension.items().collect();
    let verbs: Vec<String> = items
        .iter()
        .flat_map(|item| &item.actions)
        .flat_map(|action| &action.keywords)
        .filter(|keyword| !keyword.contains(char::is_whitespace))
        .map(|keyword| keyword.to_lowercase())
        .chain(["remove", "uninstall", "delete"].map(str::to_owned))
        .collect();

    // A title is typed as it reads, but for the "…" no one types.
    for (title, subject, action) in [
        ("Ring the phone", "phone-link", "ring"),
        ("Send the clipboard to the phone", "phone-link", "clipboard"),
        ("Phone link status", "phone-link", "status"),
        ("Unpair the phone", "phone-link", "unpair"),
        ("Send a file to the phone", "phone-link", "file"),
        ("Proxy status", "proxy", "status"),
        ("Open proxy settings", "proxy", "settings"),
        ("Import VPN config", "proxy", "import"),
        ("Screenshot region", "screenshot", "region"),
        ("Screenshot monitor", "screenshot", "monitor"),
        ("Screenshot window", "screenshot", "window"),
        ("Record region", "record", "region"),
        ("Record monitor", "record", "monitor"),
        ("Stop recording", "record", "stop"),
        ("Save the last 30 seconds", "replay", "save"),
        ("Keep awake for 1 hour", "idle", "1h"),
        ("Keep awake for 2 hours", "idle", "2h"),
        ("Super Saver", "power", "super-saver"),
        ("Power Saver", "power", "power-saver"),
        ("Max Performance", "power", "max-performance"),
        ("Undo the low-battery drop", "low-battery", "undo"),
        ("Low battery drop at", "low-battery", "level"),
        ("Dark cursor", "cursor", "dark"),
        ("Accent cursor", "cursor", "accent"),
        ("Classic cursor", "cursor", "classic"),
        ("Cursor size 24", "cursor", "size-24"),
        ("Cursor size 32", "cursor", "size-32"),
        ("Cursor size 48", "cursor", "size-48"),
        ("Set cursor size", "cursor", "size"),
        ("Charge limit: 80 %", "battery", "80"),
        ("Charge limit: 90 %", "battery", "90"),
        ("Charge limit: full", "battery", "full"),
        ("Measure power draw", "battery", "measure"),
        ("CPU scheduler: status", "cpu-scheduler-lavd", "status"),
        ("Next wallpaper", "wallpaper", "next"),
        ("Previous wallpaper", "wallpaper", "previous"),
        ("Set wallpaper", "wallpaper", "set"),
        ("Switch wallpaper folder", "wallpaper", "switch"),
        ("Wallpaper picker", "wallpaper", "pick"),
        ("Show every wallpaper set", "wallpaper", "all-sets"),
        ("Animate wallpaper always", "wallpaper", "always"),
        ("Animate wallpaper on AC", "wallpaper", "ac"),
        ("Never animate wallpaper", "wallpaper", "never"),
    ] {
        let words: Vec<String> = title.split_whitespace().map(str::to_lowercase).collect();
        let (at, verb) = words
            .iter()
            .enumerate()
            .find_map(|(at, word)| Some((at, named(&verbs, word))).filter(|(_, verb)| !verb.is_empty()))
            .filter(|_| words.len() > 1)
            .unwrap_or_else(|| panic!("no word of {title:?} is a verb with a subject beside it"));
        let rest: Vec<String> = words
            .iter()
            .enumerate()
            .filter(|&(n, _)| n != at)
            .map(|(_, word)| word.clone())
            .collect();
        let item = items
            .iter()
            .find(|item| item.id == subject)
            .unwrap_or_else(|| panic!("no Item `{subject}`"));
        let chosen = item
            .actions
            .iter()
            .find(|each| each.keywords.iter().any(|keyword| verb.contains(&keyword.as_str())));

        assert!(
            found_in(&item.title, &rest) || found_in(&item.keywords.join("\u{1f}"), &rest),
            "{title:?}: `{subject}` isn't found by {rest:?}"
        );
        assert_eq!(
            chosen.map(|each| each.id.as_str()),
            Some(action),
            "{title:?} names {verb:?}"
        );
    }
}
