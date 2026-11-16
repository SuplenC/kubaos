use std::fs;

use kubaos_core::apps::AppState;
use kubaos_core::apps::Catalog;
use kubaos_core::apps::Picks;
use kubaos_core::apps::Status;
use kubaos_core::apps::folder::Category;
use kubaos_core::apps::status::AppStatus;
use kubaos_core::apps::status::SetupInfo;
use kubaos_core::apps::status::listing;

use super::list_markdown;
use super::list_text;
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
            requires: Vec::new(),
            sources: Vec::new(),
        },
        SetupInfo {
            id: "service".into(),
            name: "T3 Code as a service".into(),
            summary: String::new(),
            requires: Vec::new(),
            sources: Vec::new(),
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

const BROWSER: &str = r#"
name = "Browser"
category = "browsers"
tags = ["web"]
summary = "Goes on the web | fast"
description = "A browser."
description_from = "kubaos"

[[action]]
name = "Install it"
kind = "apt"
repo = "vendor"
arch = "amd64"
packages = ["browser"]
"#;

const EDITOR: &str = r#"
name = "Editor"
category = "development"
tags = ["code"]
summary = "Edits text"
description = "An editor."
description_from = "kubaos"

[[action]]
name = "Install it"
kind = "apt"
packages = ["editor"]

[[action]]
name = "Its plugins"
kind = "nix"
packages = ["editor-plugins"]
"#;

const BOARD: &str = r#"
name = "Board"
category = "development"
tags = ["design"]
summary = "Draws boxes"
description = "A design board."
description_from = "kubaos"

[setup.web]
name = "Board"
summary = "board.example in a window"
requires = ["browser"]

[[setup.web.action]]
name = "Window"
kind = "web_app"
id = "board"
title = "Board"
url = "https://board.example/"
icon = "board.svg"

[setup.local]
name = "Board here"
summary = "on this machine"
requires = ["editor"]

[[setup.local.action]]
name = "Install it"
kind = "download"
to = "~/.local/share/board"
version = "1.0"
unpack = "tar-gz"
arm64 = { url = "https://board.example/board-arm64.tar.gz", sha256 = "0000000000000000000000000000000000000000000000000000000000000000" }
"#;

/// The fixture catalog, Browser picked and Board picked in its web setup.
fn fixture() -> (tempfile::TempDir, Catalog, Picks) {
    let dir = tempfile::tempdir().unwrap();
    for (id, text) in [("browser", BROWSER), ("editor", EDITOR), ("board", BOARD)] {
        fs::create_dir_all(dir.path().join(id)).unwrap();
        fs::write(dir.path().join(id).join("app.toml"), text).unwrap();
    }
    let catalog = Catalog::load(dir.path()).unwrap();
    let mut picks = Picks::default();
    picks.add("browser");
    picks.add("board");
    picks.add_setup("board", "web");
    (dir, catalog, picks)
}

#[test]
fn the_list_groups_the_apps_by_category_each_with_its_summary() {
    let (_dir, catalog, picks) = fixture();

    assert_eq!(
        list_text(&listing(&catalog, &picks)),
        "Browsers\n\
         \x20 picked  browser  Browser: Goes on the web | fast\n\
         \n\
         Development\n\
         \x20 picked  board    Board: Draws boxes\n\
         \x20                    --setup local: Board here, on this machine\n\
         \x20 picked             --setup web: Board, board.example in a window\n\
         \x20         editor   Editor: Edits text\n"
    );
}

#[test]
fn the_markdown_table_has_a_row_an_app_in_category_order() {
    let (_dir, catalog, picks) = fixture();

    assert_eq!(
        list_markdown(&listing(&catalog, &picks)),
        "| App | Category | Setups | Requires | Summary | Source | Architectures |\n\
         | --- | --- | --- | --- | --- | --- | --- |\n\
         | Browser (`browser`) | Browsers | — | — | Goes on the web \\| fast | apt repo (`vendor`) | amd64 |\n\
         | Board (`board`) | Development | `local`, `web` | local: `editor`; web: `browser` | Draws boxes \
         | local: pinned download; web: web app | amd64, arm64 |\n\
         | Editor (`editor`) | Development | — | — | Edits text | Ubuntu archive, nixpkgs | amd64, arm64 |\n"
    );
}

#[test]
fn the_json_list_keeps_its_fields_and_adds_where_each_app_comes_from() {
    let (_dir, catalog, picks) = fixture();

    let list = serde_json::to_value(listing(&catalog, &picks)).unwrap();

    let board = &list[1];
    for key in ["id", "name", "summary", "category", "tags", "picked", "setup", "setups"] {
        assert!(board.get(key).is_some(), "{key}: {board}");
    }
    assert_eq!(board["setup"], "web");
    assert_eq!(
        board["setups"][0]["sources"],
        serde_json::json!([{ "kind": "download" }])
    );
    assert_eq!(
        board["setups"][1]["sources"],
        serde_json::json!([{ "kind": "web_app" }])
    );
    assert_eq!(board["arches"], serde_json::json!(["amd64", "arm64"]));
}

#[test]
fn a_catalog_dir_is_only_for_the_markdown_table() {
    #[derive(clap::Parser)]
    struct Cli {
        #[command(subcommand)]
        cmd: super::AppCmd,
    }

    let parse = |args: &[&str]| <Cli as clap::Parser>::try_parse_from(args.iter().copied());

    assert!(parse(&["app", "list", "--catalog", "apps"]).is_err());
    assert!(parse(&["app", "list", "--json", "--catalog", "apps"]).is_err());
    assert!(parse(&["app", "list", "--markdown", "--catalog", "apps"]).is_ok());
}
