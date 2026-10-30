use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::time::Duration;

use kotodama_protocol::ActionRole;
use kotodama_protocol::Table;

use super::*;
use crate::states::Reading;
use crate::states::States;
use crate::states::Values;

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

    let items: Vec<Item> = registry.items(&States::default()).collect();

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

    let command = registry
        .command("theme-next", RUN, &BTreeMap::new())
        .expect("a command");

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

    let ids: Vec<String> = registry.items(&States::default()).map(|item| item.id).collect();

    assert_eq!(ids, ["theme-previous"]);
    assert!(registry.hides("theme-next"));
    assert!(registry.command("theme-next", RUN, &BTreeMap::new()).is_none());
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

    let ids: Vec<String> = registry.items(&States::default()).map(|item| item.id).collect();

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

    assert_eq!(registry.items(&States::default()).count(), 0);
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

    assert_eq!(registry.items(&States::default()).count(), 0);
    assert!(problems[0].contains("`keywords`"), "{problems:?}");
}

#[test]
fn an_unknown_setting_is_a_problem() {
    let (registry, problems) = Registry::from_config(
        &config("[actions.typo]\ntitle = \"Typo\"\nrun = [\"true\"]\ntermnal = true"),
        &[],
    );

    assert_eq!(registry.items(&States::default()).count(), 0);
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

    assert_eq!(registry.items(&States::default()).count(), 0);
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

    let command = registry.command("u2f", RUN, &BTreeMap::new()).expect("a command");

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

    let command = registry.command("u2f", RUN, &BTreeMap::new()).expect("a command");

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

    let item = registry.items(&States::default()).next().expect("an item");
    let command = registry
        .command("ssh-key", RUN, &arguments(&[("name", "server")]))
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

    let command = registry.command("record", RUN, &BTreeMap::new()).expect("a command");

    assert_eq!(command.argv, ["dot-hypr-record", "output"]);
}

#[test]
fn a_path_argument_asks_for_a_file_of_the_kinds_it_accepts_or_a_folder() {
    let registry = loaded(
        r#"
        [actions.import]
        title = "Import"
        run = ["import", "{config}", "{into}"]
        arguments = [
          { name = "config", kind = "file", accept = ["conf", "application/x-openvpn-profile"] },
          { name = "into", kind = "folder" },
        ]
        "#,
    );

    let item = registry.items(&States::default()).next().expect("an item");
    let [config, into] = &item.actions[0].arguments[..] else {
        panic!("two arguments: {:?}", item.actions[0].arguments);
    };

    assert_eq!(config.kind, ArgumentKind::File);
    assert_eq!(config.accept, ["conf", "application/x-openvpn-profile"]);
    assert_eq!(into.kind, ArgumentKind::Folder);
    assert!(into.accept.is_empty());
}

#[test]
fn an_argument_without_a_kind_is_text() {
    let registry =
        loaded("[actions.say]\ntitle = \"Say\"\nrun = [\"say\", \"{what}\"]\narguments = [{ name = \"what\" }]");

    let item = registry.items(&States::default()).next().expect("an item");

    assert_eq!(item.actions[0].arguments[0].kind, ArgumentKind::Text);
}

#[test]
fn an_unknown_argument_kind_is_a_problem() {
    let (registry, problems) = Registry::from_config(
        &config("[actions.odd]\ntitle = \"Odd\"\nrun = [\"true\"]\narguments = [{ name = \"x\", kind = \"path\" }]"),
        &[],
    );

    assert_eq!(registry.items(&States::default()).count(), 0);
    assert!(problems[0].contains("`path`"), "{problems:?}");
}

#[test]
fn accept_on_an_argument_that_isnt_a_file_is_a_problem() {
    let (registry, problems) = Registry::from_config(
        &config(
            "[actions.odd]\ntitle = \"Odd\"\nrun = [\"true\"]\narguments = [{ name = \"x\", kind = \"folder\", accept = [\"conf\"] }]",
        ),
        &[],
    );

    assert_eq!(registry.items(&States::default()).count(), 0);
    assert!(problems[0].contains("`accept`"), "{problems:?}");
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

    let command = registry.command("shot", RUN, &BTreeMap::new()).expect("a command");

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

    let item = registry.items(&States::default()).next().expect("an item");

    assert!(item.actions[0].confirm);
}

#[test]
fn no_actions_table_is_an_empty_registry() {
    let registry = loaded("");

    assert_eq!(registry.items(&States::default()).count(), 0);
}

#[test]
fn kubaos_toml_loads_without_a_problem() {
    let (registry, problems) = Registry::from_config(&shipped(), &crate::toggles::ids());

    assert!(problems.is_empty(), "{problems:?}");
    assert!(
        registry.items(&States::default()).count() >= 20,
        "kubaos.toml should hold KubaOS's actions"
    );
}

#[test]
fn every_dot_script_kubaos_toml_runs_is_in_scripts() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let scripts = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts");

    for argv in every_command(&registry) {
        for word in argv.iter().filter(|word| word.starts_with("dot-")) {
            assert!(
                std::path::Path::new(scripts).join(word).is_file(),
                "{argv:?} runs {word}, which isn't in scripts/"
            );
        }
    }
}

#[test]
fn the_settings_folder_and_every_group_of_it_open_from_the_menu() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let opens = every_command(&registry);

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
    let runs = every_command(&registry);

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

    let pin = registry.command("pin", RUN, &given).expect("a pin action");
    let unpin = registry.command("unpin", RUN, &given).expect("an unpin action");
    let pins = registry.command("pins", RUN, &given).expect("a pins action");

    assert_eq!(tail(&pin.argv, 5), ["kubaos", "pin", "node", "18", "~/Projects/acme"]);
    assert_eq!(tail(&unpin.argv, 4), ["kubaos", "unpin", "node", "~/Projects/acme"]);
    assert_eq!(tail(&pins.argv, 3), ["kubaos", "pins", "--all"]);
}

/// The kind of the argument `name` of the shipped action `id`, and what it accepts.
fn shipped_argument(registry: &Registry, id: &str, name: &str) -> (ArgumentKind, Vec<String>) {
    let item = registry
        .items(&States::default())
        .find(|item| item.id == id)
        .unwrap_or_else(|| panic!("no action `{id}`"));
    let argument = item
        .actions
        .iter()
        .flat_map(|action| &action.arguments)
        .find(|argument| argument.name == name)
        .unwrap_or_else(|| panic!("`{id}` has no argument `{name}`"));
    (argument.kind, argument.accept.clone())
}

#[test]
fn every_path_kubaos_toml_asks_for_is_a_file_or_a_folder_field() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let files = |kinds: &[&str]| (ArgumentKind::File, kinds.iter().map(|&kind| kind.to_owned()).collect());
    let folder = (ArgumentKind::Folder, Vec::new());

    assert_eq!(
        shipped_argument(&registry, "proxy-import", "path"),
        files(&["conf", "ovpn"])
    );
    assert_eq!(
        shipped_argument(&registry, "theme-set", "file"),
        files(&["jpg", "jpeg", "png", "webp", "mp4", "webm", "mkv", "mov"])
    );
    assert_eq!(shipped_argument(&registry, "phone-file", "path"), files(&[]));
    assert_eq!(shipped_argument(&registry, "files-open", "path"), folder);
    assert_eq!(shipped_argument(&registry, "pin", "folder"), folder);
    assert_eq!(shipped_argument(&registry, "unpin", "folder"), folder);
}

#[test]
fn the_proxy_is_switched_imported_and_set_from_the_menu() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let runs = every_command(&registry);

    for wanted in [
        &["kubaos-proxy", "on"][..],
        &["kubaos-proxy", "off"],
        &["kubaos-proxy", "import", "x"],
    ] {
        assert!(
            runs.iter().any(|argv| tail(argv, wanted.len()) == wanted),
            "no Item runs {wanted:?}"
        );
    }
    assert!(
        runs.iter().any(|argv| argv
            .last()
            .is_some_and(|script| script.contains("dot-config path network"))),
        "no Item opens proxy.toml"
    );
}

const PAIR: &str = r#"
    [actions.chrome-on]
    title = "Install Google Chrome"
    run = ["kubaos", "packages", "enable", "chrome"]
    shown_when = { state = "packages.chrome", is = false }

    [actions.chrome-off]
    title = "Remove Google Chrome"
    run = ["kubaos", "packages", "disable", "chrome"]
    shown_when = { state = "packages.chrome", is = true }
    "#;

fn shown(registry: &Registry, states: &States) -> Vec<String> {
    registry.items(states).map(|item| item.id).collect()
}

#[test]
fn an_action_shows_only_in_the_state_it_declares() {
    let registry = loaded(PAIR);
    let installed = States::default().with("packages.chrome", Reading::Bool(true));
    let missing = States::default().with("packages.chrome", Reading::Bool(false));

    assert_eq!(shown(&registry, &installed), ["chrome-off"]);
    assert_eq!(shown(&registry, &missing), ["chrome-on"]);
}

#[test]
fn both_halves_show_while_their_state_is_unknown() {
    let registry = loaded(PAIR);
    let other = States::default().with("packages.docker", Reading::Bool(true));

    assert_eq!(shown(&registry, &States::default()), ["chrome-off", "chrome-on"]);
    assert_eq!(shown(&registry, &other), ["chrome-off", "chrome-on"]);
}

#[test]
fn an_action_hidden_by_its_state_still_runs() {
    let registry = loaded(PAIR);

    let command = registry
        .command("chrome-off", RUN, &BTreeMap::new())
        .expect("its command");

    assert_eq!(command.argv, ["kubaos", "packages", "disable", "chrome"]);
}

#[test]
fn is_not_shows_an_action_in_every_other_state() {
    let registry = loaded(
        r#"
        [actions.low-battery-off]
        title = "Low battery: don't drop"
        run = ["kubaos-power", "low-battery", "off"]
        shown_when = { state = "power.low-battery", is_not = "off" }
        "#,
    );
    let off = States::default().with("power.low-battery", Reading::Word("off".to_owned()));
    let saver = States::default().with("power.low-battery", Reading::Word("power-saver".to_owned()));

    assert_eq!(shown(&registry, &off), Vec::<String>::new());
    assert_eq!(shown(&registry, &saver), ["low-battery-off"]);
}

#[test]
fn a_shown_when_of_the_wrong_shape_is_a_problem() {
    for (shown_when, problem) in [
        ("true", "`shown_when` should be a table of `state` and `is` or `is_not`"),
        ("{ is = true }", "`shown_when` needs a `state`"),
        (
            r#"{ state = "packages.chrome" }"#,
            "`shown_when` needs one of `is` and `is_not`",
        ),
        (
            r#"{ state = "packages.chrome", is = true, is_not = false }"#,
            "`shown_when` needs one of `is` and `is_not`",
        ),
        (
            r#"{ state = "packages.chrome", is = 1 }"#,
            "`shown_when`'s `is` should be true, false or a string",
        ),
        (
            r#"{ state = "chrome", is = true }"#,
            "`shown_when` names `chrome`, which is no state",
        ),
        (
            r#"{ state = "packages.chrome", is = true, when = 1 }"#,
            "`when` isn't a setting of `shown_when`",
        ),
        (
            r#"{ state = "proxy", is = "on" }"#,
            "`shown_when`: `proxy` is true or false",
        ),
        (
            r#"{ state = "power.scheduler", is = true }"#,
            "`shown_when`: `power.scheduler` is one of `kernel`, `lavd`",
        ),
        (
            r#"{ state = "power.scheduler", is_not = "fifo" }"#,
            "`shown_when`: `power.scheduler` is one of `kernel`, `lavd`",
        ),
    ] {
        let text = format!("[actions.a]\ntitle = \"A\"\nrun = [\"true\"]\nshown_when = {shown_when}\n");

        let (registry, problems) = Registry::from_config(&config(&text), &[]);

        assert_eq!(problems, [format!("`actions.a`: {problem}")], "{shown_when}");
        assert_eq!(registry.items(&States::default()).count(), 0);
    }
}

/// The words of `text`, lowercase, as a search sees them.
fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

#[test]
fn no_destructive_item_of_one_action_in_kubaos_toml_has_words_of_its_title_as_a_keyword() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());

    for item in registry.items(&States::default()).filter(|item| {
        let [action] = &item.actions[..] else {
            return false;
        };
        action.confirm || action.role == ActionRole::Destructive
    }) {
        let title = words(&item.title);
        for keyword in &item.keywords {
            assert!(
                !words(keyword).iter().all(|word| title.contains(word)),
                "`{}` has `{keyword}`, words of its title, as a keyword",
                item.id
            );
        }
    }
}

/// Whether the query word `word` names the verb `keyword`, as Kotodama reads a verb: the whole
/// keyword, or its first three letters or more.
fn names(word: &str, keyword: &str) -> bool {
    word == keyword || (word.chars().count() >= 3 && keyword.starts_with(word))
}

#[test]
fn no_word_of_a_title_in_kubaos_toml_is_a_verb_of_an_item_with_several_actions() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let items: Vec<Item> = registry.items(&States::default()).collect();
    let verbs: BTreeSet<&str> = items
        .iter()
        .filter(|item| item.actions.len() > 1)
        .flat_map(|item| &item.actions)
        .flat_map(|action| &action.keywords)
        .map(String::as_str)
        .collect();

    for item in items.iter().filter(|item| item.actions.len() > 1) {
        for word in words(&item.title) {
            let named: Vec<&&str> = verbs.iter().filter(|verb| names(&word, verb)).collect();
            assert!(
                named.is_empty(),
                "`{}`'s title has `{word}`, which names the verbs {named:?}: typing the title would choose one",
                item.id
            );
        }
    }
}

const PROXY: &str = r#"
    [actions.proxy]
    title = "Proxy"
    icon = "network-vpn"
    keywords = ["vpn"]
    state = "proxy"
    subtitle = { true = "On", false = "Off" }

    [actions.proxy.actions.on]
    title = "Turn on"
    keywords = ["on", "connect"]
    run = ["kubaos-proxy", "on"]
    primary_when = { is = false }

    [actions.proxy.actions.off]
    title = "Turn off"
    keywords = ["off", "disconnect"]
    run = ["kubaos-proxy", "off"]
    terminal = true
    delay_ms = 50
    primary_when = { is = true }
    "#;

fn proxy(on: Option<bool>) -> States {
    match on {
        Some(on) => States::default().with("proxy", Reading::Bool(on)),
        None => States::default(),
    }
}

fn action_ids(item: &Item) -> Vec<&str> {
    item.actions.iter().map(|action| action.id.as_str()).collect()
}

#[test]
fn an_item_with_actions_is_one_item_whose_actions_carry_their_verbs() {
    let registry = loaded(PROXY);

    let items: Vec<Item> = registry.items(&proxy(Some(false))).collect();

    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.id, "proxy");
    assert_eq!(item.kind.name(), KIND);
    assert_eq!(item.title, "Proxy");
    assert_eq!(item.keywords, ["vpn"]);
    assert_eq!(item.actions[0].title, "Turn on");
    assert_eq!(item.actions[0].keywords, ["on", "connect"]);
    assert_eq!(item.actions[1].keywords, ["off", "disconnect"]);
    assert!(item.actions.iter().all(|action| action.role == ActionRole::Normal));
    assert!(!item.yields);
}

#[test]
fn the_action_that_applies_next_comes_first() {
    let registry = loaded(PROXY);

    let off = registry.items(&proxy(Some(false))).next().expect("the item");
    let on = registry.items(&proxy(Some(true))).next().expect("the item");

    assert_eq!(action_ids(&off), ["on", "off"]);
    assert_eq!(action_ids(&on), ["off", "on"]);
}

#[test]
fn while_the_state_is_unknown_the_actions_keep_their_id_order() {
    let registry = loaded(PROXY);

    let item = registry.items(&proxy(None)).next().expect("the item");

    assert_eq!(action_ids(&item), ["off", "on"]);
}

#[test]
fn the_subtitle_says_the_state() {
    let registry = loaded(PROXY);
    let subtitle = |on| registry.items(&proxy(on)).next().expect("the item").subtitle;

    assert_eq!(subtitle(Some(true)).as_deref(), Some("On"));
    assert_eq!(subtitle(Some(false)).as_deref(), Some("Off"));
    assert_eq!(subtitle(None), None);
}

#[test]
fn each_action_of_an_item_runs_its_own_command() {
    let registry = loaded(PROXY);

    let on = registry.command("proxy", "on", &BTreeMap::new()).expect("on");
    let off = registry.command("proxy", "off", &BTreeMap::new()).expect("off");

    assert_eq!(on.argv, ["kubaos-proxy", "on"]);
    assert_eq!(on.delay, Duration::ZERO);
    assert_eq!(off.argv[..DEFAULT_TERMINAL.len()], *DEFAULT_TERMINAL);
    assert_eq!(tail(&off.argv, 2), ["kubaos-proxy", "off"]);
    assert_eq!(off.delay, Duration::from_millis(50));
    assert!(registry.command("proxy", RUN, &BTreeMap::new()).is_none());
    assert!(registry.command("proxy", "toggle", &BTreeMap::new()).is_none());
}

#[test]
fn an_action_with_one_command_answers_only_to_run() {
    let registry = loaded("[actions.a]\ntitle = \"A\"\nrun = [\"true\"]");

    assert!(registry.command("a", RUN, &BTreeMap::new()).is_some());
    assert!(registry.command("a", "on", &BTreeMap::new()).is_none());
}

#[test]
fn a_destructive_action_says_so() {
    let registry = loaded(
        r#"
        [actions.unpair]
        title = "Unpair the phone"
        run = ["kubaos-phone", "unpair"]
        role = "destructive"
        confirm = true

        [actions.docker]
        title = "Docker"
        state = "packages.docker"
        [actions.docker.actions.install]
        title = "Install"
        run = ["kubaos", "packages", "enable", "docker"]
        [actions.docker.actions.remove]
        title = "Remove"
        keywords = ["remove"]
        run = ["kubaos", "packages", "disable", "docker"]
        role = "destructive"
        confirm = true
        "#,
    );

    let items: Vec<Item> = registry.items(&States::default()).collect();

    assert_eq!(items[0].id, "docker");
    assert_eq!(items[0].actions[0].role, ActionRole::Normal);
    assert_eq!(items[0].actions[1].role, ActionRole::Destructive);
    assert!(items[0].actions[1].confirm);
    assert_eq!(items[1].actions[0].role, ActionRole::Destructive);
}

#[test]
fn an_item_with_an_identity_is_a_placeholder_that_yields_to_the_real_thing() {
    let registry = loaded(CHROME);

    let item = registry.items(&States::default()).next().expect("the placeholder");

    assert_eq!(item.identity.as_deref(), Some("app:google-chrome.desktop"));
    assert!(item.yields);
    assert_eq!(action_ids(&item), ["install"]);
}

const CHROME: &str = r#"
    [actions.chrome]
    title = "Google Chrome"
    subtitle = "Not installed"
    identity = "app:google-chrome.desktop"
    state = "packages.chrome"
    remove = { run = ["kubaos", "packages", "disable", "chrome"], terminal = true }

    [actions.chrome.actions.install]
    title = "Install and open"
    keywords = ["install"]
    run = ["kubaos", "packages", "enable", "chrome"]
    "#;

#[test]
fn remove_is_aimed_at_the_identity_while_the_state_says_it_is_installed() {
    let registry = loaded(CHROME);
    let chrome = |on: bool| States::default().with("packages.chrome", Reading::Bool(on));

    assert_eq!(registry.targets(&chrome(true)), ["app:google-chrome.desktop"]);
    assert_eq!(registry.targets(&chrome(false)), Vec::<String>::new());
    assert_eq!(registry.targets(&States::default()), ["app:google-chrome.desktop"]);
}

#[test]
fn removing_the_real_thing_runs_the_placeholders_remove() {
    let registry = loaded(CHROME);

    let command = registry.removal("app:google-chrome.desktop").expect("a command");

    assert_eq!(command.argv[..DEFAULT_TERMINAL.len()], *DEFAULT_TERMINAL);
    assert_eq!(tail(&command.argv, 4), ["kubaos", "packages", "disable", "chrome"]);
    assert!(registry.removal("app:firefox.desktop").is_none());
}

#[test]
fn a_hidden_placeholder_offers_no_remove() {
    let registry = loaded(&format!("hide = [\"chrome\"]\n{CHROME}"));

    assert_eq!(registry.targets(&States::default()), Vec::<String>::new());
    assert!(registry.removal("app:google-chrome.desktop").is_none());
}

#[test]
fn an_item_of_actions_of_the_wrong_shape_is_a_problem() {
    let one = "[actions.a.actions.on]\ntitle = \"On\"\nrun = [\"true\"]\n";
    for (text, problem) in [
        (
            format!("[actions.a]\ntitle = \"A\"\nrun = [\"true\"]\n{one}"),
            "takes `run` or `actions`, not both",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\nterminal = true\n{one}"),
            "`terminal` is a setting of each of its `actions`",
        ),
        (
            "[actions.a]\ntitle = \"A\"\nactions = {}\n".to_owned(),
            "`actions` should be a table of at least one action, by id",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\n{one}primary_when = {{ is = true }}\n"),
            "`actions.on`: `primary_when` needs the item's `state`",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\nstate = \"proxy\"\n{one}primary_when = {{ is = \"on\" }}\n"),
            "`actions.on`: `primary_when`: `proxy` is true or false",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\nstate = \"proxy\"\n{one}primary_when = {{ state = \"proxy\", is = true }}\n"),
            "`actions.on`: `state` isn't a setting of `primary_when`",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\n{one}shown_when = 1\n"),
            "`actions.on`: `shown_when` isn't a setting of an action",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\nstate = \"nope\"\n{one}"),
            "`state` names `nope`, which is no state",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\nsubtitle = {{ true = \"On\" }}\n{one}"),
            "a `subtitle` by state needs a `state`",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\nstate = \"proxy\"\nsubtitle = {{ on = \"On\" }}\n{one}"),
            "`subtitle`: `proxy` is true or false",
        ),
        (
            format!("[actions.a]\ntitle = \"A\"\nsubtitle = 1\n{one}"),
            "`subtitle` should be a string, or a table of strings by the `state`'s value",
        ),
        (
            "[actions.a]\ntitle = \"A\"\nrun = [\"true\"]\nrole = \"form\"\n".to_owned(),
            "`role` should be `normal` or `destructive`",
        ),
        (
            "[actions.a]\ntitle = \"A\"\nrun = [\"true\"]\nremove = { run = [\"true\"] }\n".to_owned(),
            "`remove` is for an item with an `identity`",
        ),
        (
            "[actions.a]\ntitle = \"A\"\nrun = [\"true\"]\nidentity = \"app:a.desktop\"\nstate = \"power.scheduler\"\nremove = { run = [\"true\"] }\n"
                .to_owned(),
            "`remove` needs a `state` that is true or false",
        ),
        (
            "[actions.a]\ntitle = \"A\"\nrun = [\"true\"]\nidentity = \"app:a.desktop\"\nremove = { run = [\"true\"], confirm = true }\n"
                .to_owned(),
            "`remove`: `confirm` isn't a setting of `remove`",
        ),
        (
            "[actions.a]\ntitle = \"A\"\nrun = [\"true\"]\nidentity = \"\"\n".to_owned(),
            "`identity` should be a string that isn't empty",
        ),
    ] {
        let (registry, problems) = Registry::from_config(&config(&text), &[]);

        assert_eq!(problems, [format!("`actions.a`: {problem}")], "{text}");
        assert_eq!(registry.items(&States::default()).count(), 0, "{text}");
    }
}

/// The ids of the Items of `table`'s actions, by the state each follows, through `state` or
/// `shown_when`.
fn items_by_state(table: &Table) -> BTreeMap<String, Vec<String>> {
    let Some(Value::Table(actions)) = table.get("actions") else {
        panic!("kubaos.toml has no actions");
    };
    let mut by_state: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, action) in actions {
        let Value::Table(action) = action else {
            panic!("`{id}` isn't a table");
        };
        let state = match (action.get("state"), action.get("shown_when")) {
            (Some(Value::String(state)), _) => state.clone(),
            (None, Some(Value::Table(shown_when))) => match shown_when.get("state") {
                Some(Value::String(state)) => state.clone(),
                _ => panic!("`{id}`'s `shown_when` names no state"),
            },
            _ => continue,
        };
        by_state.entry(state).or_default().push(id.clone());
    }
    by_state
}

/// Every reading the state `name` may have.
fn readings(name: &str) -> Vec<Reading> {
    match crate::states::values(name) {
        Some(Values::Bool) => vec![Reading::Bool(true), Reading::Bool(false)],
        Some(Values::Words(words)) => words.iter().map(|&word| Reading::Word(word.to_owned())).collect(),
        None => panic!("`{name}` is no state"),
    }
}

#[test]
fn every_thing_with_a_state_in_kubaos_toml_is_one_item() {
    let by_state = items_by_state(&shipped());

    assert!(by_state.len() >= 18, "{by_state:?}");
    for (state, ids) in by_state {
        assert_eq!(ids.len(), 1, "{state} has more than one Item: {ids:?}");
    }
}

#[test]
fn every_item_of_kubaos_toml_with_a_state_says_it_and_offers_what_applies_next_first() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let stateful: Vec<&Entry> = registry.actions.iter().filter(|entry| entry.state.is_some()).collect();

    assert!(stateful.len() >= 18, "{}", stateful.len());
    for entry in stateful {
        let state = entry.state.clone().expect("a state");
        let mut firsts = BTreeSet::new();
        for reading in readings(&state) {
            let states = States::default().with(state.clone(), reading.clone());
            let item = entry.item(&states);
            let first = entry
                .verbs
                .iter()
                .find(|verb| verb.id == item.actions[0].id)
                .expect("the first Action");

            assert!(
                item.subtitle.is_some(),
                "`{}` says nothing with {state} {reading:?}",
                entry.id
            );
            assert!(
                entry.verbs.len() == 1 || first.applies(Some(&reading)),
                "`{}` offers `{}` first with {state} {reading:?}",
                entry.id,
                first.id
            );
            firsts.insert(first.id.clone());
        }
        assert!(
            entry.verbs.len() == 1 || firsts.len() > 1,
            "`{}` offers `{firsts:?}` first in every state",
            entry.id
        );
    }
}

#[test]
fn every_action_of_an_item_of_kubaos_toml_with_several_has_its_own_verbs() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());

    for item in registry.items(&States::default()).filter(|item| item.actions.len() > 1) {
        let mut verbs = BTreeSet::new();
        for action in &item.actions {
            assert!(
                !action.keywords.is_empty(),
                "`{}`'s `{}` has no verb",
                item.id,
                action.id
            );
            for keyword in &action.keywords {
                assert!(
                    verbs.insert(keyword.clone()),
                    "`{}` has `{keyword}` on two Actions",
                    item.id
                );
            }
        }
    }
}

#[test]
fn every_package_removal_of_kubaos_toml_is_destructive_and_asks_first() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let mut removals = 0;

    for item in registry.items(&States::default()) {
        for action in &item.actions {
            let argv = registry
                .command(&item.id, &action.id, &BTreeMap::new())
                .expect("a command")
                .argv;
            if tail(&argv, 4).starts_with(&["kubaos".to_owned(), "packages".to_owned(), "disable".to_owned()]) {
                removals += 1;
                assert_eq!(action.role, ActionRole::Destructive, "`{}`'s `{}`", item.id, action.id);
                assert!(action.confirm, "`{}`'s `{}`", item.id, action.id);
            }
        }
    }
    assert!(removals >= 4, "{removals}");
}

#[test]
fn google_chrome_waits_as_a_placeholder_and_is_removed_from_the_installed_app() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let installed = States::default().with("packages.chrome", Reading::Bool(true));

    let placeholder = registry
        .items(&States::default())
        .find(|item| item.identity.as_deref() == Some("app:google-chrome.desktop"))
        .expect("Chrome's placeholder");
    let remove = registry.removal("app:google-chrome.desktop").expect("Chrome's removal");

    assert!(placeholder.yields);
    assert_eq!(placeholder.title, "Google Chrome");
    assert!(
        registry.items(&installed).all(|item| item.id != placeholder.id),
        "the placeholder shows while the group is on"
    );
    assert!(placeholder.actions[0].keywords.contains(&"install".to_owned()));
    assert_eq!(
        placeholder.actions[0].arguments.len(),
        0,
        "installing Chrome asks nothing"
    );
    assert!(
        registry
            .targets(&installed)
            .contains(&"app:google-chrome.desktop".to_owned())
    );
    assert_eq!(tail(&remove.argv, 4), ["kubaos", "packages", "disable", "chrome"]);
}

#[test]
fn no_word_of_the_chrome_app_is_a_verb_of_its_placeholder() {
    let (registry, _) = Registry::from_config(&shipped(), &crate::toggles::ids());
    let app = ["google", "chrome", "web", "browser", "google-chrome"];

    let placeholder = registry
        .items(&States::default())
        .find(|item| item.identity.as_deref() == Some("app:google-chrome.desktop"))
        .expect("Chrome's placeholder");

    for keyword in placeholder.actions.iter().flat_map(|action| &action.keywords) {
        assert!(
            !app.contains(&keyword.as_str()),
            "`{keyword}` is a word of the Chrome app"
        );
    }
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

/// What every Action of every Item runs, each Argument given as `x`.
fn every_command(registry: &Registry) -> Vec<Vec<String>> {
    let mut argvs = Vec::new();
    for item in registry.items(&States::default()) {
        for action in &item.actions {
            let given = action
                .arguments
                .iter()
                .map(|argument| (argument.name.clone(), "x".to_owned()))
                .collect();
            let command = registry.command(&item.id, &action.id, &given).expect("a command");
            argvs.push(command.argv);
        }
    }
    argvs
}
