use kubaos_core::apps::AppState;
use kubaos_core::apps::Status;
use kubaos_core::apps::plan::AppStatus;

use super::table;

fn app(id: &str, name: &str, picked: bool, state: AppState, version: Option<&str>) -> AppStatus {
    AppStatus {
        id: id.into(),
        name: name.into(),
        summary: String::new(),
        source: "apt",
        picked,
        state,
        version: version.map(str::to_owned),
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
