use std::collections::BTreeMap;
use std::fs;
use std::time::Duration;

use kubaos_core::suites::SuiteConfig;

use super::*;
use crate::fake::FakeRunner;

const TERMINAL: &[&str] = &["ghostty", "-e"];

fn installed(version: &str) -> Installed {
    let config = SuiteConfig {
        version: Some(version.to_owned()),
        ..SuiteConfig::default()
    };
    Installed::default().with("test", Some(config))
}

fn item(suites: &Installed, id: &str) -> Item {
    suites.items().into_iter().find(|item| item.id == id).expect("the Item")
}

fn ids(suites: &Installed) -> Vec<String> {
    suites.items().into_iter().map(|item| item.id).collect()
}

fn actions(suites: &Installed, id: &str) -> Vec<String> {
    item(suites, id).actions.into_iter().map(|action| action.id).collect()
}

/// The Item of every Suite in the catalog, none installed.
const NOT_INSTALLED: [&str; 7] = [
    "suite:rust",
    "suite:go",
    "suite:node",
    "suite:python",
    "suite:php",
    "suite:proto",
    "suite:test",
];

fn arguments(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

/// Runs `action` on `id` and gives the command line launched, after the terminal.
fn launched(suites: &Installed, id: &str, action: &str, given: &[(&str, &str)]) -> Result<String, String> {
    let runner = FakeRunner::default();
    let terminal: Vec<String> = TERMINAL.iter().map(|&word| word.to_owned()).collect();

    let ran = suites.run(id, action, &arguments(given), &terminal, &runner)?;

    assert_eq!(ran, Ran::Close);
    let launched = runner.launched.borrow();
    let [(argv, delay)] = launched.as_slice() else {
        panic!("one launch, not {launched:?}");
    };
    assert_eq!(*delay, Duration::ZERO);
    assert_eq!(&argv[..TERMINAL.len()], TERMINAL);
    Ok(argv[TERMINAL.len()..].join(" "))
}

#[test]
fn a_suite_that_is_not_installed_is_one_item_that_installs_it() {
    let suites = Installed::default();

    let install = item(&suites, "suite:test");

    assert_eq!(ids(&suites), NOT_INSTALLED);
    assert_eq!(install.kind.name(), KIND);
    assert_eq!(install.title, "Install the Test Suite");
    assert_eq!(install.subtitle.as_deref(), Some(catalog::TEST.description));
    assert_eq!(actions(&suites, "suite:test"), [INSTALL]);
    assert_eq!(install.actions[0].title, "Install the Test Suite");
}

#[test]
fn installing_runs_suite_add_in_the_terminal() {
    let suites = Installed::default();

    let command = launched(&suites, "suite:test", INSTALL, &[]);

    assert_eq!(command.as_deref(), Ok("kubaos suite add test"));
}

#[test]
fn an_installed_suite_configures_first_then_removes_and_says_its_version() {
    let suites = installed("2");

    let suite = item(&suites, "suite:test");

    assert_eq!(suite.title, "Test Suite");
    assert_eq!(suite.subtitle.as_deref(), Some("Installed, version 2"));
    assert_eq!(actions(&suites, "suite:test"), [CONFIGURE, REMOVE]);
    assert!(!suite.actions[0].confirm);
    assert!(suite.actions[1].confirm);
}

#[test]
fn configure_takes_the_version_and_each_extra_all_optional() {
    let suites = installed("1");

    let configure = &item(&suites, "suite:test").actions[0];

    let names: Vec<&str> = configure
        .arguments
        .iter()
        .map(|argument| argument.name.as_str())
        .collect();
    assert_eq!(names, ["version", "greeting"]);
    assert!(configure.arguments.iter().all(|argument| argument.optional));
    assert_eq!(configure.arguments[0].placeholder.as_deref(), Some("1, 2, 3"));
    assert_eq!(configure.arguments[1].placeholder.as_deref(), Some("on or off"));
}

#[test]
fn configure_passes_on_what_is_typed_and_leaves_out_what_is_not() {
    let suites = installed("1");

    let nothing = launched(&suites, "suite:test", CONFIGURE, &[("version", " "), ("greeting", "")]);
    let version = launched(&suites, "suite:test", CONFIGURE, &[("version", "3")]);
    let on = launched(&suites, "suite:test", CONFIGURE, &[("greeting", "On")]);
    let off = launched(
        &suites,
        "suite:test",
        CONFIGURE,
        &[("version", "2"), ("greeting", "off")],
    );

    assert_eq!(nothing.as_deref(), Ok("kubaos suite configure test"));
    assert_eq!(version.as_deref(), Ok("kubaos suite configure test --version 3"));
    assert_eq!(on.as_deref(), Ok("kubaos suite configure test --extra greeting"));
    assert_eq!(
        off.as_deref(),
        Ok("kubaos suite configure test --version 2 --no-extra greeting")
    );
}

#[test]
fn configure_refuses_a_version_or_an_extra_word_the_suite_does_not_take() {
    let suites = installed("1");

    let version = launched(&suites, "suite:test", CONFIGURE, &[("version", "9")]);
    let extra = launched(&suites, "suite:test", CONFIGURE, &[("greeting", "maybe")]);

    assert_eq!(
        version,
        Err("The Test Suite has no version 9. It has 1, 2, 3".to_owned())
    );
    assert_eq!(extra, Err("`maybe` isn't on or off, for greeting".to_owned()));
}

#[test]
fn removing_runs_suite_remove_in_the_terminal() {
    let suites = installed("1");

    let command = launched(&suites, "suite:test", REMOVE, &[]);

    assert_eq!(command.as_deref(), Ok("kubaos suite remove test"));
}

#[test]
fn only_the_actions_of_the_suites_state_run() {
    let missing = Installed::default();
    let present = installed("1");

    let configure = launched(&missing, "suite:test", CONFIGURE, &[]);
    let remove = launched(&missing, "suite:test", REMOVE, &[]);
    let install = launched(&present, "suite:test", INSTALL, &[]);
    let unknown = launched(&present, "suite:nope", INSTALL, &[]);

    assert_eq!(configure, Err("`suite:test` has no Action `configure`".to_owned()));
    assert_eq!(remove, Err("`suite:test` has no Action `remove`".to_owned()));
    assert_eq!(install, Err("`suite:test` has no Action `install`".to_owned()));
    assert_eq!(unknown, Err("`suite:nope` has no Action `install`".to_owned()));
}

#[test]
fn an_installed_suite_has_a_new_project_item_a_scaffolder_after_it() {
    let suites = installed("1");

    let new = item(&suites, "suite-new:test:new");

    assert_eq!(ids(&suites), [&NOT_INSTALLED[..], &["suite-new:test:new"]].concat());
    assert_eq!(new.kind.name(), KIND);
    assert_eq!(new.title, "New Test project…");
    assert_eq!(new.actions.len(), 1);
    assert_eq!(new.actions[0].id, NEW);
    let [argument] = new.actions[0].arguments.as_slice() else {
        panic!("one Argument");
    };
    assert_eq!(argument.name, "name");
    assert!(!argument.optional);
}

#[test]
fn a_new_project_runs_suite_new_with_its_scaffolder_and_the_trimmed_name() {
    let suites = installed("1");

    let command = launched(&suites, "suite-new:test:new", NEW, &[("name", " hello ")]);
    let unnamed = launched(&suites, "suite-new:test:new", NEW, &[]);

    assert_eq!(command.as_deref(), Ok("kubaos suite new test hello --with new"));
    assert_eq!(unnamed, Err("Name the new project".to_owned()));
}

#[test]
fn a_suite_that_is_not_installed_scaffolds_nothing() {
    let suites = Installed::default();

    let command = launched(&suites, "suite-new:test:new", NEW, &[("name", "hello")]);

    assert_eq!(command, Err("`suite-new:test:new` has no Action `new`".to_owned()));
}

#[test]
fn every_suite_item_is_found_by_its_name_title_suite_and_its_languages_words() {
    let missing = Installed::default();
    let present = installed("1");

    let test_items = missing.items().into_iter().chain(present.items());
    for item in test_items.filter(|item| item.id.ends_with(":test") || item.id.contains(":test:")) {
        for word in ["test", "Test", "suite", "kubaos-test-suite"] {
            assert!(
                item.keywords.iter().any(|keyword| keyword == word),
                "{} lacks {word}",
                item.id
            );
        }
    }
    for word in ["node", "Node", "suite", "typescript", "javascript", "bun", "npm"] {
        assert!(
            item(&missing, "suite:node")
                .keywords
                .iter()
                .any(|keyword| keyword == word),
            "suite:node lacks {word}"
        );
    }
    assert!(item(&missing, "suite:test").keywords.contains(&"install".to_owned()));
    assert!(item(&present, "suite:test").keywords.contains(&"remove".to_owned()));
    assert!(
        item(&present, "suite-new:test:new")
            .keywords
            .contains(&"new project".to_owned())
    );
}

#[test]
fn a_suite_file_that_does_not_read_is_installed_with_its_options_unknown() {
    let dir = tempfile::tempdir().unwrap();
    let state = StateDir::new(dir.path());
    fs::create_dir_all(state.suites_dir()).unwrap();
    fs::write(state.suite_file("test"), "version = [").unwrap();

    let suites = Installed::read(&state);

    assert_eq!(suites.is_installed("test"), None);
    assert_eq!(
        item(&suites, "suite:test").subtitle.as_deref(),
        Some("Installed, but kubaos.d/suites/test.toml doesn't read")
    );
    assert_eq!(actions(&suites, "suite:test"), [CONFIGURE, REMOVE]);
}

#[test]
fn reading_takes_each_catalog_suites_file_and_its_options() {
    let dir = tempfile::tempdir().unwrap();
    let state = StateDir::new(dir.path());

    let missing = Installed::read(&state);
    fs::create_dir_all(state.suites_dir()).unwrap();
    fs::write(state.suite_file("test"), "version = \"2\"\nextras = [\"greeting\"]\n").unwrap();
    fs::write(state.suite_file("unknown"), "packages = [\"hello\"]\n").unwrap();
    let present = Installed::read(&state);

    assert_eq!(missing, Installed::default());
    assert_eq!(missing.is_installed("test"), Some(false));
    let config = SuiteConfig {
        version: Some("2".to_owned()),
        extras: ["greeting".to_owned()].into(),
    };
    assert_eq!(present, Installed::default().with("test", Some(config)));
    assert_eq!(present.is_installed("test"), Some(true));
}

#[test]
fn ids_name_every_item_a_suite_may_have() {
    assert_eq!(
        super::ids(),
        [
            "suite:rust",
            "suite-new:rust:bin",
            "suite-new:rust:lib",
            "suite-new:rust:workspace",
            "suite:go",
            "suite-new:go:module",
            "suite:node",
            "suite-new:node:node",
            "suite-new:node:bun",
            "suite:python",
            "suite-new:python:uv",
            "suite:php",
            "suite-new:php:composer",
            "suite-new:php:laravel",
            "suite:proto",
            "suite-new:proto:moon",
            "suite:test",
            "suite-new:test:new",
        ]
    );
}
