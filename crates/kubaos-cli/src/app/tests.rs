use kubaos_core::apps::AppState;
use kubaos_core::apps::Status;
use kubaos_core::apps::folder::Category;
use kubaos_core::apps::status::AppStatus;
use kubaos_core::apps::status::SetupInfo;

use super::table;

fn app(id: &str, name: &str, picked: bool, state: AppState, version: Option<&str>) -> AppStatus {
    AppStatus {
        id: id.into(),
        name: name.into(),
        summary: String::new(),
        category: Category::Development,
        tags: vec!["code".into()],
        icon: None,
        picked,
        state,
        version: version.map(str::to_owned),
        setup: None,
        setups: Vec::new(),
        configurable: false,
        available: vec![true],
    }
}

#[test]
fn the_table_lines_up_each_app_and_says_what_the_next_update_installs() {
    let status = Status {
        arch: "amd64".into(),
        apps: vec![
            app("claude-code", "Claude Code", true, AppState::Missing, None),
            app("t3-code", "T3 Code", false, AppState::Installed, Some("0.1.9")),
        ],
        unknown: vec!["gone".into()],
    };

    assert_eq!(
        table(&status),
        "KubaOS's apps (amd64)\n\
         \n  missing     picked  claude-code  Claude Code\
         \n  installed           t3-code      T3 Code  0.1.9\
         \n  gone: picked, but this release's catalog has no such app\
         \n\
         \n1 picked app missing: kubaos update installs it.\n"
    );
}

#[test]
fn an_app_in_a_setup_is_named_for_it() {
    let mut t3 = app("t3-code", "T3 Code", true, AppState::Installed, Some("0.1.9"));
    t3.setups = vec![
        SetupInfo {
            id: "desktop".into(),
            name: "T3 Code".into(),
            summary: String::new(),
        },
        SetupInfo {
            id: "service".into(),
            name: "T3 Code as a service".into(),
            summary: String::new(),
        },
    ];
    t3.available = vec![true, true];
    t3.setup = Some("service".into());
    let status = Status {
        arch: "amd64".into(),
        apps: vec![t3],
        unknown: Vec::new(),
    };

    assert_eq!(
        table(&status),
        "KubaOS's apps (amd64)\n\n  installed   picked  t3-code  T3 Code as a service  0.1.9\n"
    );
}
