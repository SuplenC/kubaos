use std::path::Path;

use super::Category;
use super::Folder;
use super::FolderError;

const ABOUT: &str = r#"name = "Sketch"
category = "graphics"
tags = ["design", "vector graphics"]
summary = "Draw boxes, then argue about them"
description = "Sketch is a design tool for the web."
description_from = "https://sketch.example/about"
"#;

const ACTION: &str = "[[action]]\nname = \"Keep it running\"\nkind = \"linger\"\n";

fn parse(about: &str) -> Result<Folder, FolderError> {
    Folder::parse("sketch", Path::new("/catalog/sketch"), &format!("{about}{ACTION}"))
}

#[test]
fn an_app_says_what_it_is() {
    let folder = parse(ABOUT).unwrap();

    assert_eq!(folder.category, Category::Graphics);
    assert_eq!(folder.tags, ["design", "vector graphics"]);
    assert_eq!(folder.description_from, "https://sketch.example/about");
}

#[test]
fn an_app_missing_any_of_what_it_is_is_refused() {
    for key in ["category", "tags", "summary", "description", "description_from"] {
        let mut about: Vec<&str> = ABOUT
            .lines()
            .filter(|line| !line.starts_with(&format!("{key} =")))
            .collect();
        about.push("");
        let about = about.join("\n");

        assert!(parse(&about).is_err(), "{key} wasn't required");
    }
}

#[test]
fn a_category_off_the_list_is_refused() {
    let result = parse(&ABOUT.replace("\"graphics\"", "\"drawing\""));

    assert!(matches!(result, Err(FolderError::Parse(_))), "{result:?}");
}

#[test]
fn tags_are_lowercase_words_each_once() {
    for tags in [
        "[]",
        r#"["Design"]"#,
        r#"["design", "design"]"#,
        r#"[" design"]"#,
        r#"["a,b"]"#,
    ] {
        let result = parse(&ABOUT.replace(r#"["design", "vector graphics"]"#, tags));

        assert!(
            matches!(
                result,
                Err(FolderError::InvalidTag(_) | FolderError::Empty { key: "tags" })
            ),
            "{tags}: {result:?}"
        );
    }
}

#[test]
fn a_description_comes_from_an_https_url_and_says_something() {
    let insecure = parse(&ABOUT.replace("https://sketch", "http://sketch"));
    let empty = parse(&ABOUT.replace("Sketch is a design tool for the web.", " "));

    assert!(matches!(insecure, Err(FolderError::InvalidSource(_))), "{insecure:?}");
    assert!(
        matches!(empty, Err(FolderError::Empty { key: "description" })),
        "{empty:?}"
    );
}

#[test]
fn a_folder_never_sets_what_only_a_journal_records() {
    for action in [
        "[[action]]\nname = \"X\"\nkind = \"run\"\ndo = [\"true\"]\ncheck = [\"true\"]\nundo = [\"true\"]\nscripts = { x = \"rm -rf /\" }\n",
        "[[action]]\nname = \"X\"\nkind = \"web_app\"\nid = \"x\"\ntitle = \"X\"\nurl = \"https://x.example/\"\nicon = \"x.svg\"\nsvg = \"<svg/>\"\n",
    ] {
        let result = Folder::parse("sketch", Path::new("/catalog/sketch"), &format!("{ABOUT}{action}"));

        assert!(matches!(result, Err(FolderError::Action { .. })), "{result:?}");
    }
}
