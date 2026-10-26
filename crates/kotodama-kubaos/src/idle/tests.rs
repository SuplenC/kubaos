use std::collections::BTreeMap;

use super::*;
use crate::fake::FakeRunner;

fn typed(value: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(VALUE.to_owned(), value.to_owned())])
}

fn launched(runner: &FakeRunner) -> Vec<String> {
    runner
        .launched
        .borrow()
        .iter()
        .map(|(argv, _)| argv.join(" "))
        .collect()
}

fn list_of(ran: Ran) -> View {
    match ran {
        Ran::Push(view) => view,
        other => panic!("expected a list, got {other:?}"),
    }
}

fn rows(view: &View) -> Vec<(String, String, Option<String>)> {
    let ViewContent::List { items, .. } = &view.content else {
        panic!("not a list");
    };
    items
        .iter()
        .map(|item| (item.id.clone(), item.title.clone(), item.subtitle.clone()))
        .collect()
}

#[test]
fn there_are_four_settings_each_with_its_word_and_an_optional_value() {
    let items = items();

    let titles: Vec<&str> = items.iter().map(|item| item.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "Lock after…",
            "Screens off after…",
            "Suspend after…",
            "When the lid closes…"
        ]
    );
    assert_eq!(items[0].keyword.as_deref(), Some("lock-after"));
    assert!(
        items
            .iter()
            .all(|item| item.actions.len() == 1 && item.actions[0].arguments[0].optional)
    );
    assert_eq!(ids(), ["idle-lock", "idle-screens-off", "idle-suspend", "idle-lid"]);
}

#[test]
fn enter_lists_the_choices_with_the_ones_in_use_marked() {
    let runner = FakeRunner::default();
    let mut settings = Settings::default();
    settings.ac.lock = Wait::from_secs(15 * 60);

    let view = list_of(run("idle-lock", &BTreeMap::new(), &settings, &runner).expect("lists"));

    assert_eq!(view.id, "idle-lock");
    assert_eq!(view.title, "Lock after");
    let rows = rows(&view);
    assert_eq!(rows[0], ("idle-lock:1m".into(), "1 min".into(), None));
    assert_eq!(
        rows[2],
        ("idle-lock:5m".into(), "5 min".into(), Some("Now, on battery".into()))
    );
    assert_eq!(
        rows[4],
        ("idle-lock:15m".into(), "15 min".into(), Some("Now, plugged in".into()))
    );
    assert_eq!(rows.last().map(|row| row.1.as_str()), Some("Never"));
    assert_eq!(launched(&runner), Vec::<String>::new());
}

#[test]
fn the_lid_lists_suspend_lock_and_nothing() {
    let view = list_of(
        run(
            "idle-lid",
            &BTreeMap::new(),
            &Settings::default(),
            &FakeRunner::default(),
        )
        .expect("lists"),
    );

    assert_eq!(
        rows(&view),
        [
            (
                "idle-lid:suspend".into(),
                "Suspend".into(),
                Some("Now, on battery".into())
            ),
            ("idle-lid:lock".into(), "Lock".into(), None),
            (
                "idle-lid:nothing".into(),
                "Do nothing".into(),
                Some("Now, plugged in".into())
            ),
        ]
    );
}

#[test]
fn a_choice_sets_both_sources_or_the_one_its_action_names() {
    let runner = FakeRunner::default();

    assert_eq!(choose("idle-lock:15m", BOTH, &runner), Ok(Ran::Pop("idle-lock".into())));
    choose("idle-suspend:never", AC, &runner).expect("ac");
    choose("idle-lid:lock", BATTERY, &runner).expect("battery");

    assert_eq!(
        launched(&runner),
        [
            "kubaos-idle set lock 15m both",
            "kubaos-idle set suspend never ac",
            "kubaos-idle set lid lock battery",
        ]
    );
}

#[test]
fn a_typed_value_sets_it_at_once_with_an_optional_source() {
    let runner = FakeRunner::default();

    assert_eq!(
        run("idle-lock", &typed("2m"), &Settings::default(), &runner),
        Ok(Ran::Close)
    );
    run("idle-suspend", &typed("1h 30m ac"), &Settings::default(), &runner).expect("ac");
    run(
        "idle-screens-off",
        &typed("never battery"),
        &Settings::default(),
        &runner,
    )
    .expect("battery");
    run("idle-lid", &typed("nothing"), &Settings::default(), &runner).expect("lid");

    assert_eq!(
        launched(&runner),
        [
            "kubaos-idle set lock 2m both",
            "kubaos-idle set suspend 1h 30m ac",
            "kubaos-idle set screens-off never battery",
            "kubaos-idle set lid nothing both",
        ]
    );
}

#[test]
fn a_refused_value_reads_in_the_menu_and_runs_nothing() {
    let runner = FakeRunner::default();

    let soon = run("idle-lock", &typed("10s"), &Settings::default(), &runner).expect_err("too soon");
    let nonsense = run("idle-suspend", &typed("soon"), &Settings::default(), &runner).expect_err("nonsense");
    let lid = run("idle-lid", &typed("explode"), &Settings::default(), &runner).expect_err("no such action");

    assert_eq!(soon, "The lock can't come sooner than 30 s idle, not after 10 s");
    assert!(nonsense.contains("try 90s, 2m, 1h30m or never"), "{nonsense}");
    assert!(lid.contains("suspend, lock or nothing"), "{lid}");
    assert_eq!(launched(&runner), Vec::<String>::new());
}

#[test]
fn a_choice_that_isnt_one_is_refused() {
    let runner = FakeRunner::default();

    assert!(choose("idle-lock:15m", "explode", &runner).is_err());
    assert!(choose("nothing", BOTH, &runner).is_err());
    assert!(run("idle-dim", &BTreeMap::new(), &Settings::default(), &runner).is_err());
}
