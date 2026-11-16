use std::path::Path;

use super::Action;
use super::ActionError;
use super::Side;
use super::Words;

fn action(text: &str) -> Result<Action, ActionError> {
    let table: toml::Table = toml::from_str(text).unwrap();
    Action::try_from(table)
}

#[test]
fn a_run_without_its_undo_is_refused() {
    let result = action("name = \"Install\"\nkind = \"run\"\ndo = [\"t3\", \"install\"]\ncheck = [\"true\"]\n");

    assert!(matches!(result, Err(ActionError::Parse(_))), "{result:?}");
}

#[test]
fn a_run_with_a_program_by_path_is_refused() {
    let result =
        action("name = \"Install\"\nkind = \"run\"\ndo = [\"/bin/rm\"]\ncheck = [\"true\"]\nundo = [\"true\"]\n");

    assert!(
        matches!(result, Err(ActionError::InvalidArgv { key: "do" })),
        "{result:?}"
    );
}

#[test]
fn an_unknown_kind_or_key_is_refused() {
    assert!(action("name = \"X\"\nkind = \"shell\"\nscript = \"rm -rf /\"\n").is_err());
    assert!(action("name = \"X\"\nkind = \"linger\"\nuser = \"root\"\n").is_err());
}

#[test]
fn a_path_outside_the_home_and_the_systems_own_folders_is_refused() {
    for to in ["/usr/bin/x", "~/../x", "/etc/../usr/bin/x", "relative", "~/a//b"] {
        let result = action(&format!("name = \"X\"\nkind = \"file\"\nto = \"{to}\"\ntext = \"\"\n"));

        assert!(
            matches!(result, Err(ActionError::InvalidPath { .. })),
            "{to}: {result:?}"
        );
    }
}

#[test]
fn who_runs_an_action_is_its_kinds_and_paths() {
    let side = |text: &str| action(text).unwrap().side();

    assert_eq!(side("name = \"X\"\nkind = \"apt\"\npackages = [\"ab\"]\n"), Side::Root);
    assert_eq!(
        side("name = \"X\"\nkind = \"file\"\nto = \"~/a\"\ntext = \"\"\n"),
        Side::User
    );
    assert_eq!(
        side("name = \"X\"\nkind = \"file\"\nto = \"/etc/a\"\ntext = \"\"\n"),
        Side::Root
    );
    assert_eq!(
        side("name = \"X\"\nkind = \"unit\"\nunit = \"a.service\"\nscope = \"user\"\nenable = true\n"),
        Side::User
    );
    assert_eq!(
        side("name = \"X\"\nkind = \"run\"\ndo = [\"a\"]\ncheck = [\"a\"]\nundo = [\"a\"]\nroot = true\n"),
        Side::Root
    );
}

#[test]
fn resolving_fills_in_the_words_and_reads_a_template() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("unit.in"), "User=@USER@\nHome=@HOME@\n").unwrap();
    let words = Words {
        user: "kuba".to_owned(),
        home: Path::new("/home/kuba").to_path_buf(),
        arch: "amd64".to_owned(),
    };
    let file = action(
        "name = \"X\"\nkind = \"file\"\nto = \"/etc/systemd/system/x.service\"\nfrom = \"unit.in\"\ntemplate = true\n",
    )
    .unwrap();

    let resolved = file.resolve(&words, dir.path()).unwrap();

    let super::Kind::File { text, from, .. } = resolved.kind else {
        panic!("not a file");
    };
    assert_eq!(text.as_deref(), Some("User=kuba\nHome=/home/kuba\n"));
    assert_eq!(from, None);
}
