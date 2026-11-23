use std::path::Path;

use super::Category;
use super::Folder;
use super::FolderError;
use super::picture;

const ABOUT: &str = r#"name = "Sketch"
category = "graphics"
tags = ["design", "vector graphics"]
summary = "Draw boxes, then argue about them"
description = "Sketch is a design tool for the web."
description_from = "https://sketch.example/about"
icon = "sketch"
"#;

const ACTION: &str = "[[action]]\nname = \"Keep it running\"\nkind = \"linger\"\n";

/// [`ABOUT`] with `icon` as its icon.
fn about_with_icon(icon: &str) -> String {
    ABOUT.replace("icon = \"sketch\"\n", &format!("icon = \"{icon}\"\n"))
}

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
    for key in ["category", "tags", "summary", "description", "description_from", "icon"] {
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
fn a_description_is_copied_from_an_https_url_or_written_by_kubaos() {
    let ours = parse(&ABOUT.replace("\"https://sketch.example/about\"", "\"kubaos\"")).unwrap();
    let elsewhere = parse(&ABOUT.replace("\"https://sketch.example/about\"", "\"kuba\""));

    assert!(!ours.description_copied());
    assert!(matches!(elsewhere, Err(FolderError::InvalidSource(_))), "{elsewhere:?}");
}

#[test]
fn an_svg_icon_of_the_folder_is_put_in_the_icon_theme_after_the_apps_own_install_steps() {
    let folder = parse(&about_with_icon("sketch.svg")).unwrap();

    let steps = folder.steps(None).unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(folder.shared_steps(), 2);
    assert_eq!(steps[1].name, "Put Sketch's icon in place");
    assert_eq!(
        steps[1].kind,
        super::Kind::File {
            to: "~/.local/share/icons/hicolor/scalable/apps/sketch.svg".to_owned(),
            from: Some("sketch.svg".to_owned()),
            text: None,
            template: false,
            mode: Some("644".to_owned()),
            vendor_keys: false,
            binary: false,
        }
    );
    assert_eq!(folder.icon_shown(), "/catalog/sketch/sketch.svg");
}

#[test]
fn a_theme_icon_is_named_as_it_is_and_installs_nothing() {
    let folder = parse(ABOUT).unwrap();

    assert_eq!(folder.steps(None).unwrap().len(), 1);
    assert_eq!(folder.icon_shown(), "sketch");
}

#[test]
fn an_icon_that_is_neither_a_picture_of_the_folder_nor_a_theme_name_is_refused() {
    for icon in ["sketch.xpm", "../sketch.svg", "art/sketch.svg", "-sketch"] {
        let result = parse(&about_with_icon(icon));

        assert!(matches!(result, Err(FolderError::InvalidIcon(_))), "{icon}: {result:?}");
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

#[test]
fn an_app_requires_apps_by_id_or_with_a_min_and_a_setup_adds_its_own() {
    let text = format!(
        "{ABOUT}requires = [\"neovim\", {{ id = \"ripgrep\", min = \"14.0\" }}]\n{ACTION}\
         [setup.web]\nname = \"Sketch\"\nsummary = \"In a window\"\nrequires = [\"google-chrome\"]\n\
         [setup.local]\nname = \"Sketch here\"\nsummary = \"On this machine\"\nrequires = [\"docker\"]\n"
    );

    let folder = Folder::parse("sketch", Path::new("/catalog/sketch"), &text).unwrap();

    let required = |setup: &str| -> Vec<(String, Option<String>)> {
        folder
            .requires(folder.setup(setup))
            .map(|r| (r.id().to_owned(), r.min().map(str::to_owned)))
            .collect()
    };
    assert_eq!(
        required("web"),
        [
            ("neovim".to_owned(), None),
            ("ripgrep".to_owned(), Some("14.0".to_owned())),
            ("google-chrome".to_owned(), None),
        ]
    );
    assert_eq!(required("local")[2].0, "docker");
}

#[test]
fn a_requirement_that_isnt_another_app_once_or_whose_min_isnt_a_version_is_refused() {
    for requires in [
        "[\"sketch\"]",
        "[\"neovim\", \"neovim\"]",
        "[\"Neovim\"]",
        "[{ id = \"neovim\", min = \"latest\" }]",
    ] {
        let result = parse(&format!("{ABOUT}requires = {requires}\n"));

        assert!(result.is_err(), "{requires} was taken");
    }
}

#[test]
fn configure_steps_come_after_every_install_step_of_the_app_and_its_setup() {
    let text = format!(
        "{ABOUT}\
         [[action]]\nname = \"Defaults\"\nkind = \"file\"\nstage = \"configure\"\n\
         to = \"~/.config/kubaos/sketch/defaults.toml\"\ntext = \"a\"\n\
         [[action]]\nname = \"Install\"\nkind = \"linger\"\n\
         [setup.web]\nname = \"Sketch\"\nsummary = \"In a window\"\n\
         [[setup.web.action]]\nname = \"Theme\"\nkind = \"setting\"\nstage = \"configure\"\n\
         file = \"~/.config/kubaos/sketch/settings.toml\"\nkey = \"theme\"\nvalue = \"dark\"\n\
         [[setup.web.action]]\nname = \"Window\"\nkind = \"web_app\"\nid = \"sketch\"\ntitle = \"Sketch\"\n\
         url = \"https://sketch.example/\"\nicon = \"sketch.svg\"\n\
         [setup.local]\nname = \"Sketch here\"\nsummary = \"On this machine\"\n\
         [[setup.local.action]]\nname = \"Serve\"\nkind = \"linger\"\n"
    );

    let folder = Folder::parse("sketch", Path::new("/catalog/sketch"), &text).unwrap();

    let names: Vec<&str> = folder
        .steps(Some("web"))
        .unwrap()
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert_eq!(names, ["Install", "Window", "Defaults", "Theme"]);
    assert_eq!(folder.shared_steps(), 1);
    assert!(folder.configurable(folder.setup("web")));
    assert!(folder.configurable(folder.setup("local")));
}

#[test]
fn a_configure_step_writes_only_the_apps_own_folder_of_the_user_layer_as_the_user() {
    for action in [
        "kind = \"file\"\nto = \"~/.config/sketch/defaults.toml\"\ntext = \"a\"",
        "kind = \"file\"\nto = \"~/.config/kubaos/other/defaults.toml\"\ntext = \"a\"",
        "kind = \"file\"\nto = \"/etc/sketch.conf\"\ntext = \"a\"",
        "kind = \"setting\"\nfile = \"~/.config/sketch/settings.json\"\nkey = \"a\"\nvalue = 1",
        "kind = \"linger\"",
        "kind = \"run\"\ndo = [\"true\"]\ncheck = [\"true\"]\nundo = [\"true\"]\nroot = true",
    ] {
        let text = format!("{ABOUT}{ACTION}[[action]]\nname = \"Defaults\"\nstage = \"configure\"\n{action}\n");

        let result = Folder::parse("sketch", Path::new("/catalog/sketch"), &text);

        assert!(
            matches!(result, Err(FolderError::ConfigureOutside { .. })),
            "{action}: {result:?}"
        );
    }
}

/// A download of `version` to `to`, as an action.
fn download(to: &str, version: &str) -> String {
    format!(
        "[[action]]\nname = \"Download\"\nkind = \"download\"\nto = \"{to}\"\nversion = \"{version}\"\n\
         unpack = \"tar-gz\"\namd64 = {{ url = \"https://x.example/a\", sha256 = \"{}\" }}\n",
        "a".repeat(64)
    )
}

#[test]
fn a_zstd_tarball_comes_after_an_apt_step_installing_zstd_in_the_same_install() {
    let zst = download("~/.v/@VERSION@", "1.0").replace("tar-gz", "tar-zst");
    let zstd = "[[action]]\nname = \"Install zstd\"\nkind = \"apt\"\npackages = [\"zstd\"]\n";
    let setup = |id: &str, actions: &str| format!("\n[setup.{id}]\nname = \"{id}\"\nsummary = \"{id}\"\n{actions}");
    let in_setup = |text: &str, id: &str| text.replace("[[action]]", &format!("[[setup.{id}.action]]"));

    let first = format!("{ABOUT}{zstd}{zst}");
    let after = format!("{ABOUT}{zst}{zstd}");
    let without = format!("{ABOUT}{zst}");
    // One setup installs zstd, the other doesn't: each install is checked on its own.
    let one_setup = format!(
        "{ABOUT}{}{}",
        setup("a", &in_setup(&format!("{zstd}{zst}"), "a")),
        setup("b", &in_setup(&zst, "b"))
    );
    let app_wide = format!(
        "{ABOUT}{zstd}{}{}",
        setup("a", &in_setup(&zst, "a")),
        setup("b", &in_setup(&zst, "b"))
    );

    assert_eq!(folder_version(&first).as_deref(), Some("1.0"));
    assert_eq!(folder_version(&app_wide).as_deref(), Some("1.0"));
    for refused in [after, without, one_setup] {
        let result = Folder::parse("sketch", Path::new("/catalog/sketch"), &refused);
        assert!(matches!(result, Err(FolderError::ZstdFirst { .. })), "{result:?}");
    }
}

#[test]
fn an_apps_downloads_are_of_one_version_whatever_the_setup() {
    let two = format!(
        "{ABOUT}{}\n[setup.a]\nname = \"A\"\nsummary = \"a\"\n[[setup.a.action]]\n{}\n[setup.b]\nname = \"B\"\nsummary = \"b\"\n",
        download("~/.v/@VERSION@", "1.0"),
        download("~/.v/@VERSION@/gpu", "1.1").trim_start_matches("[[action]]\n"),
    );

    let result = Folder::parse("sketch", Path::new("/catalog/sketch"), &two);

    assert!(
        matches!(result, Err(FolderError::DownloadVersions { .. })),
        "{result:?}"
    );
}

#[test]
fn a_step_inside_a_downloads_folder_comes_after_the_download() {
    let before = format!(
        "{ABOUT}[[action]]\nname = \"Link\"\nkind = \"symlink\"\nto = \"~/.v/@VERSION@/bin/x\"\ntarget = \"x\"\n{}",
        download("~/.v/@VERSION@", "1.0")
    );
    let after = format!(
        "{ABOUT}{}{}",
        download("~/.v/@VERSION@", "1.0"),
        download("~/.v/@VERSION@/gpu", "1.0").replacen("name = \"Download\"", "name = \"Download its GPU part\"", 1)
    );

    let refused = Folder::parse("sketch", Path::new("/catalog/sketch"), &before);

    assert!(
        matches!(refused, Err(FolderError::InsideDownload { .. })),
        "{refused:?}"
    );
    assert_eq!(folder_version(&after).as_deref(), Some("1.0"));
}

fn folder_version(text: &str) -> Option<String> {
    Folder::parse("sketch", Path::new("/catalog/sketch"), text)
        .unwrap()
        .version()
        .map(str::to_owned)
}

#[test]
fn a_package_of_another_architecture_comes_after_the_step_that_adds_it() {
    let apt = "[[action]]\nname = \"Install\"\nkind = \"apt\"\npackages = [\"libgl1:i386\"]\n";
    let arch = "[[action]]\nname = \"Take i386\"\nkind = \"architecture\"\narchitecture = \"i386\"\n";
    let parse = |actions: &str| Folder::parse("sketch", Path::new("/catalog/sketch"), &format!("{ABOUT}{actions}"));

    let before = parse(&format!("{arch}{apt}"));
    let after = parse(&format!("{apt}{arch}"));
    let none = parse(apt);

    assert!(before.is_ok(), "{before:?}");
    for refused in [after, none] {
        assert!(
            matches!(&refused, Err(FolderError::ArchitectureFirst { arch, .. }) if arch == "i386"),
            "{refused:?}"
        );
    }
}

/// Sketch's folder in a temporary catalog, with `icon` as its picture `bytes`.
fn with_picture(icon: &str, bytes: &[u8]) -> (tempfile::TempDir, Result<Folder, FolderError>) {
    let catalog = tempfile::tempdir().unwrap();
    let dir = catalog.path().join("sketch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(icon), bytes).unwrap();
    let folder = Folder::parse("sketch", &dir, &format!("{}{ACTION}", about_with_icon(icon)));
    (catalog, folder)
}

#[test]
fn a_png_icon_is_put_in_the_icon_themes_folder_of_its_size() {
    let (_catalog, folder) = with_picture("sketch.png", &picture::png(256, 256));
    let folder = folder.unwrap();

    let steps = folder.steps(None).unwrap();
    assert_eq!(steps.len(), 2);
    assert!(
        matches!(&steps[1].kind, super::Kind::File { to, from: Some(from), .. }
            if to == "~/.local/share/icons/hicolor/256x256/apps/sketch.png" && from == "sketch.png"),
        "{:?}",
        steps[1].kind
    );
    assert_eq!(folder.icon_shown(), folder.dir.join("sketch.png").display().to_string());
}

#[test]
fn a_jpeg_icon_is_shown_from_the_folder_and_installs_nothing() {
    for icon in ["sketch.jpg", "sketch.jpeg"] {
        let (_catalog, folder) = with_picture(icon, &picture::jpeg(512, 512));
        let folder = folder.unwrap();

        assert_eq!(
            folder.steps(None).unwrap().len(),
            1,
            "{icon}: the icon theme reads no JPEG"
        );
        assert_eq!(folder.icon_shown(), folder.dir.join(icon).display().to_string());
    }
}

#[test]
fn a_picture_icon_that_isnt_square_or_a_png_of_a_size_the_theme_lacks_is_refused() {
    let (_c, tall) = with_picture("sketch.png", &picture::png(128, 256));
    let (_c2, wide) = with_picture("sketch.jpg", &picture::jpeg(512, 256));
    let (_c3, odd) = with_picture("sketch.png", &picture::png(100, 100));
    let (_c4, not_png) = with_picture("sketch.png", b"<svg/>");

    assert!(
        matches!(
            tall,
            Err(FolderError::IconNotSquare {
                width: 128,
                height: 256,
                ..
            })
        ),
        "{tall:?}"
    );
    assert!(
        matches!(
            wide,
            Err(FolderError::IconNotSquare {
                width: 512,
                height: 256,
                ..
            })
        ),
        "{wide:?}"
    );
    assert!(matches!(odd, Err(FolderError::IconSize { size: 100, .. })), "{odd:?}");
    assert!(
        matches!(not_png, Err(FolderError::UnreadableIcon { .. })),
        "{not_png:?}"
    );
}

#[test]
fn a_theme_icons_name_beside_a_picture_of_that_name_is_refused() {
    let catalog = tempfile::tempdir().unwrap();
    let dir = catalog.path().join("sketch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("sketch.svg"), b"<svg/>").unwrap();

    let named = Folder::parse("sketch", &dir, &format!("{ABOUT}{ACTION}"));

    assert!(
        matches!(&named, Err(FolderError::ThemeIconBesideFile { icon, file }) if icon == "sketch" && file == "sketch.svg"),
        "{named:?}"
    );
}

#[test]
fn an_app_with_no_icon_is_refused_naming_how_to_draw_one() {
    let without = parse(&ABOUT.replace("icon = \"sketch\"\n", ""));

    assert!(
        matches!(&without, Err(FolderError::NoIcon(id)) if id == "sketch"),
        "{without:?}"
    );
    assert!(
        without.unwrap_err().to_string().contains("just app-icon sketch"),
        "the error names the generator"
    );
}

#[test]
fn a_symbol_beside_the_icon_is_an_unknown_key() {
    let symbol = parse(&format!("{ABOUT}symbol = \"gauge\"\n"));

    assert!(
        matches!(&symbol, Err(FolderError::Parse(e)) if e.to_string().contains("unknown field `symbol`")),
        "{symbol:?}"
    );
}
