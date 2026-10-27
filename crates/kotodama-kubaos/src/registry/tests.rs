use std::collections::BTreeMap;
use std::time::Duration;

use kotodama_protocol::Table;

use super::*;

fn config(text: &str) -> Table {
    toml::from_str(text).expect("the test's config")
}

fn loaded(text: &str) -> Registry {
    let (registry, problems) = Registry::from_config(&config(text), &[]);
    assert_eq!(problems, Vec::<String>::new());
    registry
}

fn arguments(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|&(name, value)| (name.to_owned(), value.to_owned()))
        .collect()
}

#[test]
fn an_action_is_an_item_with_its_title_words_and_one_action() {
    let registry = loaded(
        r#"
        [actions.theme-next]
        title = "Next wallpaper"
        subtitle = "And its colors"
        icon = "go-next"
        keywords = ["theme", "next theme"]
        run = ["dot-theme", "next"]
        "#,
    );

    let items: Vec<Item> = registry.items().collect();

    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.id, "theme-next");
    assert_eq!(item.kind.name(), KIND);
    assert_eq!(item.title, "Next wallpaper");
    assert_eq!(item.subtitle.as_deref(), Some("And its colors"));
    assert_eq!(item.icon.as_deref(), Some("go-next"));
    assert_eq!(item.keywords, ["theme", "next theme"]);
    assert_eq!(item.actions.len(), 1);
    assert_eq!(item.actions[0].id, RUN);
    assert!(!item.actions[0].confirm);
}

#[test]
fn an_action_runs_its_command_at_once() {
    let registry = loaded(
        r#"
        [actions.theme-next]
        title = "Next wallpaper"
        run = ["dot-theme", "next"]
        "#,
    );

    let command = registry.command("theme-next", &BTreeMap::new()).expect("a command");

    assert_eq!(command.argv, ["dot-theme", "next"]);
    assert_eq!(command.delay, Duration::ZERO);
}

#[test]
fn a_hidden_action_neither_shows_nor_runs() {
    let registry = loaded(
        r#"
        hide = ["theme-next"]
        [actions.theme-next]
        title = "Next wallpaper"
        run = ["dot-theme", "next"]
        [actions.theme-previous]
        title = "Previous wallpaper"
        run = ["dot-theme", "prev"]
        "#,
    );

    let ids: Vec<String> = registry.items().map(|item| item.id).collect();

    assert_eq!(ids, ["theme-previous"]);
    assert!(registry.hides("theme-next"));
    assert!(registry.command("theme-next", &BTreeMap::new()).is_none());
}

#[test]
fn hide_may_name_a_reserved_id() {
    let (registry, problems) = Registry::from_config(&config(r#"hide = ["wifi"]"#), &["wifi"]);

    assert!(problems.is_empty(), "{problems:?}");
    assert!(registry.hides("wifi"));
}

#[test]
fn hiding_an_unknown_id_is_a_problem() {
    let (_, problems) = Registry::from_config(&config(r#"hide = ["no-such-action"]"#), &["wifi"]);

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("`no-such-action`"), "{problems:?}");
}

#[test]
fn an_action_without_a_command_is_left_out_as_a_problem() {
    let (registry, problems) = Registry::from_config(
        &config(
            r#"
            [actions.broken]
            title = "Broken"
            [actions.fine]
            title = "Fine"
            run = ["true"]
            "#,
        ),
        &[],
    );

    let ids: Vec<String> = registry.items().map(|item| item.id).collect();

    assert_eq!(ids, ["fine"]);
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].contains("`actions.broken`") && problems[0].contains("`run`"),
        "{problems:?}"
    );
}

#[test]
fn an_action_without_a_title_is_left_out_as_a_problem() {
    let (registry, problems) = Registry::from_config(&config("[actions.untitled]\nrun = [\"true\"]"), &[]);

    assert_eq!(registry.items().count(), 0);
    assert!(problems[0].contains("`title`"), "{problems:?}");
}

#[test]
fn a_setting_of_the_wrong_shape_is_a_problem() {
    let (registry, problems) = Registry::from_config(
        &config(
            r#"
            [actions.odd]
            title = "Odd"
            run = ["true"]
            keywords = "not a list"
            "#,
        ),
        &[],
    );

    assert_eq!(registry.items().count(), 0);
    assert!(problems[0].contains("`keywords`"), "{problems:?}");
}

#[test]
fn an_unknown_setting_is_a_problem() {
    let (registry, problems) = Registry::from_config(
        &config("[actions.typo]\ntitle = \"Typo\"\nrun = [\"true\"]\ntermnal = true"),
        &[],
    );

    assert_eq!(registry.items().count(), 0);
    assert!(problems[0].contains("`termnal`"), "{problems:?}");
}

#[test]
fn an_action_may_not_take_a_reserved_id() {
    let (registry, problems) = Registry::from_config(
        &config(
            r#"
            [actions.wifi]
            title = "My Wi-Fi"
            run = ["true"]
            "#,
        ),
        &["wifi"],
    );

    assert_eq!(registry.items().count(), 0);
    assert!(problems[0].contains("`wifi`"), "{problems:?}");
}

#[test]
fn a_terminal_action_runs_in_the_configured_terminal() {
    let registry = loaded(
        r#"
        terminal = ["ghostty", "-e"]
        [actions.u2f]
        title = "Enroll a security key"
        run = ["dot-u2f-enroll"]
        terminal = true
        "#,
    );

    let command = registry.command("u2f", &BTreeMap::new()).expect("a command");

    assert_eq!(command.argv, ["ghostty", "-e", "dot-u2f-enroll"]);
}

#[test]
fn a_terminal_action_runs_in_the_default_terminal_without_one_configured() {
    let registry = loaded(
        r#"
        [actions.u2f]
        title = "Enroll a security key"
        run = ["dot-u2f-enroll"]
        terminal = true
        "#,
    );

    let command = registry.command("u2f", &BTreeMap::new()).expect("a command");

    assert_eq!(command.argv[..DEFAULT_TERMINAL.len()], *DEFAULT_TERMINAL);
    assert_eq!(command.argv.last().map(String::as_str), Some("dot-u2f-enroll"));
}

#[test]
fn arguments_fill_their_placeholders() {
    let registry = loaded(
        r#"
        [actions.ssh-key]
        title = "New SSH key on the security key"
        keyword = "ssh-key"
        run = ["dot-ssh-sk-new", "{name}", "--comment={name}@kubaos"]
        arguments = [{ name = "name", placeholder = "github" }]
        "#,
    );

    let item = registry.items().next().expect("an item");
    let command = registry
        .command("ssh-key", &arguments(&[("name", "server")]))
        .expect("a command");

    assert_eq!(item.keyword.as_deref(), Some("ssh-key"));
    assert_eq!(item.actions[0].arguments[0].name, "name");
    assert_eq!(item.actions[0].arguments[0].placeholder.as_deref(), Some("github"));
    assert!(!item.actions[0].arguments[0].optional);
    assert_eq!(command.argv, ["dot-ssh-sk-new", "server", "--comment=server@kubaos"]);
}

#[test]
fn an_optional_argument_left_out_drops_its_own_word() {
    let registry = loaded(
        r#"
        [actions.record]
        title = "Record the screen"
        run = ["dot-hypr-record", "output", "{sound}"]
        arguments = [{ name = "sound", optional = true }]
        "#,
    );

    let command = registry.command("record", &BTreeMap::new()).expect("a command");

    assert_eq!(command.argv, ["dot-hypr-record", "output"]);
}

#[test]
fn delay_ms_waits_before_the_command_runs() {
    let registry = loaded(
        r#"
        [actions.shot]
        title = "Screenshot"
        run = ["dot-hypr-screenshot", "region"]
        delay_ms = 400
        "#,
    );

    let command = registry.command("shot", &BTreeMap::new()).expect("a command");

    assert_eq!(command.delay, Duration::from_millis(400));
}

#[test]
fn confirm_asks_before_the_action_runs() {
    let registry = loaded(
        r#"
        [actions.restart]
        title = "Restart the shell"
        run = ["true"]
        confirm = true
        "#,
    );

    let item = registry.items().next().expect("an item");

    assert!(item.actions[0].confirm);
}

#[test]
fn no_actions_table_is_an_empty_registry() {
    let registry = loaded("");

    assert_eq!(registry.items().count(), 0);
}

#[test]
fn kubaos_toml_loads_without_a_problem() {
    let (registry, problems) = Registry::from_config(&shipped(), &crate::toggles::ids());

    assert!(problems.is_empty(), "{problems:?}");
    assert!(
        registry.items().count() >= 20,
        "kubaos.toml should hold KubaOS's actions"
    );
}

#[test]
fn every_dot_script_kubaos_toml_runs_is_in_scripts() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let scripts = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts");

    for item in registry.items() {
        let command = registry.command(&item.id, &every_argument(&item)).expect("a command");
        for word in command.argv.iter().filter(|word| word.starts_with("dot-")) {
            assert!(
                std::path::Path::new(scripts).join(word).is_file(),
                "`{}` runs {word}, which isn't in scripts/",
                item.id
            );
        }
    }
}

#[test]
fn the_settings_folder_and_every_group_of_it_open_from_the_menu() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let opens: Vec<Vec<String>> = registry
        .items()
        .filter_map(|item| registry.command(&item.id, &every_argument(&item)))
        .map(|command| command.argv)
        .collect();

    let mut wanted = vec![vec!["dot-config".to_owned(), "open".to_owned()]];
    for group in kubaos_user_layer::Group::ALL {
        wanted.push(vec![
            "dot-config".to_owned(),
            "open".to_owned(),
            group.name().to_owned(),
        ]);
    }
    for argv in wanted {
        assert!(opens.contains(&argv), "no Item runs {argv:?}");
    }
}

#[test]
fn the_phone_link_is_set_up_used_and_turned_off_from_the_menu() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let runs: Vec<Vec<String>> = registry
        .items()
        .filter_map(|item| registry.command(&item.id, &every_argument(&item)))
        .map(|command| command.argv)
        .collect();

    for wanted in [
        &["kubaos-setup", "--card", "phone"][..],
        &["kubaos-phone", "off"],
        &["kubaos-phone", "send-clipboard"],
        &["kubaos-phone", "send-file", "x"],
        &["kubaos-phone", "ring"],
        &["kubaos-phone", "unpair"],
    ] {
        assert!(runs.iter().any(|argv| argv == wanted), "no Item runs {wanted:?}");
    }
}

#[test]
fn project_pins_are_set_removed_and_listed_from_the_menu() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let given = arguments(&[("suite", "node"), ("version", "18"), ("folder", "~/Projects/acme")]);

    let pin = registry.command("pin", &given).expect("a pin action");
    let unpin = registry.command("unpin", &given).expect("an unpin action");
    let pins = registry.command("pins", &given).expect("a pins action");

    assert_eq!(tail(&pin.argv, 5), ["kubaos", "pin", "node", "18", "~/Projects/acme"]);
    assert_eq!(tail(&unpin.argv, 4), ["kubaos", "unpin", "node", "~/Projects/acme"]);
    assert_eq!(tail(&pins.argv, 3), ["kubaos", "pins", "--all"]);
}

#[test]
fn the_proxy_is_switched_imported_and_set_from_the_menu() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let runs: Vec<Vec<String>> = registry
        .items()
        .filter_map(|item| registry.command(&item.id, &every_argument(&item)))
        .map(|command| command.argv)
        .collect();

    for wanted in [
        &["kubaos-proxy", "on"][..],
        &["kubaos-proxy", "off"],
        &["kubaos-proxy", "import", "x"],
    ] {
        assert!(runs.iter().any(|argv| tail(argv, wanted.len()) == wanted), "no Item runs {wanted:?}");
    }
    assert!(
        runs.iter()
            .any(|argv| argv.last().is_some_and(|script| script.contains("dot-config path network"))),
        "no Item opens proxy.toml"
    );
}

/// The `[extensions.kubaos]` table of the Defaults.
fn shipped() -> Table {
    let defaults: Table = toml::from_str(include_str!("../../../../config/kotodama/kubaos.toml")).expect("kubaos.toml");
    let Some(Value::Table(extensions)) = defaults.get("extensions") else {
        panic!("kubaos.toml has no [extensions]");
    };
    let Some(Value::Table(kubaos)) = extensions.get("kubaos") else {
        panic!("kubaos.toml has no [extensions.kubaos]");
    };
    kubaos.clone()
}

/// The last `n` words of `argv`: what an action runs, after the terminal it runs in.
fn tail(argv: &[String], n: usize) -> &[String] {
    &argv[argv.len().saturating_sub(n)..]
}

fn every_argument(item: &Item) -> BTreeMap<String, String> {
    item.actions[0]
        .arguments
        .iter()
        .map(|argument| (argument.name.clone(), "x".to_owned()))
        .collect()
}
