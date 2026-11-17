use std::path::Path;

use super::Action;
use super::ActionError;
use super::Kind;
use super::Side;
use super::Stage;
use super::Words;
use super::version_matches;

fn action(text: &str) -> Result<Action, ActionError> {
    let table: toml::Table = toml::from_str(text).unwrap();
    Action::from_folder(table)
}

#[test]
fn a_journaled_run_of_a_program_under_the_home_reads_back() {
    // Regression: T3 Code's service journal, with `t3`'s path filled in, couldn't be read.
    let words = Words {
        user: "kuba".to_owned(),
        home: "/home/kuba".into(),
        arch: "amd64".to_owned(),
        version: None,
    };
    let run = action(
        "name = \"Start\"\nkind = \"run\"\ndo = [\"@HOME@/.local/bin/t3\", \"service\", \"install\"]\ncheck = [\"true\"]\nundo = [\"true\"]\n",
    )
    .unwrap()
    .resolve(&words, Path::new("/nowhere"))
    .unwrap();
    let text = toml::to_string(&run).unwrap();

    let read: Action = toml::from_str(&text).unwrap();

    assert_eq!(read, run);
}

#[test]
fn a_self_updating_download_goes_to_a_folder_named_for_its_version_or_one_its_updater_rewrites() {
    let download = |to: &str| {
        action(&format!(
            "name = \"Get\"\nkind = \"download\"\nto = \"{to}\"\nversion = \"1.0\"\nunpack = \"tar-gz\"\nself_updates = true\n\
             amd64 = {{ url = \"https://example.com/t\", sha256 = \"{sha}\" }}\n",
            sha = "a".repeat(64)
        ))
    };

    assert!(download("~/.vendor/versions/@VERSION@").is_ok());
    assert!(download("~/.local/share/vendor/bin").is_ok());
}

#[test]
fn a_download_its_updater_rewrites_in_place_names_no_version_but_through_version() {
    let download = |to: &str, self_updates: bool| {
        action(&format!(
            "name = \"Get\"\nkind = \"download\"\nto = \"{to}\"\nversion = \"1.0\"\nunpack = \"tar-gz\"\n\
             self_updates = {self_updates}\namd64 = {{ url = \"https://example.com/t\", sha256 = \"{sha}\" }}\n",
            sha = "a".repeat(64)
        ))
    };

    assert!(download("~/.vendor/tool-@VERSION@", true).is_ok());
    assert!(download("~/.vendor/tool-1.0", false).is_ok());
    assert!(matches!(
        download("~/.vendor/tool-1.0", true),
        Err(ActionError::SelfUpdatesNamesVersion { .. })
    ));
}

#[test]
fn a_user_naming_the_version_leaves_a_download_rewritten_in_place() {
    let words = Words {
        user: "dev1.0".to_owned(),
        home: "/home/dev1.0".into(),
        arch: "amd64".to_owned(),
        version: None,
    };
    let resolve = |to: &str| {
        action(&format!(
            "name = \"Get\"\nkind = \"download\"\nto = \"{to}\"\nversion = \"1.0\"\nunpack = \"tar-gz\"\n\
             self_updates = true\namd64 = {{ url = \"https://example.com/t\", sha256 = \"{sha}\" }}\n",
            sha = "a".repeat(64)
        ))
        .unwrap()
        .resolve(&words, Path::new("/nowhere"))
    };
    let in_place = |action: &Action| matches!(action.kind, Kind::Download { in_place: true, .. });

    let under_home = resolve("~/.vendor/@USER@/tool").unwrap();
    let user = resolve("/opt/@USER@/tool").unwrap();
    let versioned = resolve("/opt/@USER@/tool-@VERSION@").unwrap();

    assert!(in_place(&under_home), "{under_home:?}");
    assert!(in_place(&user), "{user:?}");
    assert!(!in_place(&versioned), "{versioned:?}");
}

#[test]
fn a_folder_setting_what_only_a_journal_records_is_refused() {
    let result = action(&format!(
        "name = \"Get\"\nkind = \"download\"\nto = \"~/.vendor/tool-@VERSION@\"\nversion = \"1.0\"\n\
         unpack = \"tar-gz\"\nin_place = true\namd64 = {{ url = \"https://example.com/t\", sha256 = \"{sha}\" }}\n",
        sha = "a".repeat(64)
    ));

    assert!(
        matches!(result, Err(ActionError::JournalOnly { key: "in_place" })),
        "{result:?}"
    );
}

#[test]
fn an_apt_actions_packages_of_one_architecture_join_its_own_on_that_architecture_alone() {
    let text = "name = \"Install\"\nkind = \"apt\"\npackages = [\"qemu-utils\"]\n\
                amd64 = { packages = [\"ovmf\"] }\narm64 = { packages = [\"qemu-efi-aarch64\"] }\n";
    let on = |arch: &str| {
        let words = Words {
            user: "kuba".to_owned(),
            home: "/home/kuba".into(),
            arch: arch.to_owned(),
            version: None,
        };
        let resolved = action(text).unwrap().resolve(&words, Path::new("/nowhere")).unwrap();
        let Kind::Apt {
            packages, amd64, arm64, ..
        } = resolved.kind
        else {
            panic!("not apt");
        };
        assert_eq!((amd64, arm64), (None, None), "a journal records one list");
        packages
    };

    assert_eq!(on("amd64"), ["qemu-utils", "ovmf"]);
    assert_eq!(on("arm64"), ["qemu-utils", "qemu-efi-aarch64"]);
}

#[test]
fn an_apt_action_for_one_architecture_lists_no_packages_of_another() {
    let result = action(
        "name = \"Install\"\nkind = \"apt\"\narch = \"amd64\"\npackages = [\"tool\"]\n\
         arm64 = { packages = [\"other\"] }\n",
    );
    let bad =
        action("name = \"Install\"\nkind = \"apt\"\npackages = [\"tool\"]\narm64 = { packages = [\"Not A Name\"] }\n");

    assert!(
        matches!(result, Err(ActionError::ArchPackagesWithArch { .. })),
        "{result:?}"
    );
    assert!(matches!(bad, Err(ActionError::InvalidPackage { .. })), "{bad:?}");
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
        version: None,
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

#[test]
fn root_never_runs_a_program_from_the_home() {
    let result = action(
        "name = \"X\"\nkind = \"run\"\ndo = [\"@HOME@/.local/bin/t3\"]\ncheck = [\"true\"]\nundo = [\"true\"]\nroot = true\n",
    );

    assert!(
        matches!(result, Err(ActionError::InvalidArgv { key: "do" })),
        "{result:?}"
    );
}

#[test]
fn an_action_is_in_the_install_stage_unless_it_says_configure() {
    let install = action("name = \"X\"\nkind = \"linger\"\n").unwrap();
    let configure = action("name = \"X\"\nkind = \"linger\"\nstage = \"configure\"\n").unwrap();
    let other = action("name = \"X\"\nkind = \"linger\"\nstage = \"later\"\n");

    assert_eq!(install.stage, Stage::Install);
    assert_eq!(configure.stage, Stage::Configure);
    assert!(matches!(other, Err(ActionError::InvalidStage { .. })), "{other:?}");
}

#[test]
fn a_web_app_is_the_users_and_takes_an_https_url_and_an_svg_of_the_folder_or_a_theme_icon() {
    let web_app = |url: &str, icon: &str| {
        action(&format!(
            "name = \"X\"\nkind = \"web_app\"\nid = \"sketch\"\ntitle = \"Sketch\"\nurl = \"{url}\"\nicon = \"{icon}\"\n"
        ))
    };

    assert_eq!(
        web_app("https://sketch.example/", "sketch.svg").unwrap().side(),
        Side::User
    );
    assert!(matches!(
        web_app("http://sketch.example/", "sketch.svg"),
        Err(ActionError::InvalidUrl { .. })
    ));
    assert!(web_app("https://sketch.example/", "whatsapp").is_ok());
    assert!(web_app("https://sketch.example/", "org.gnome.Nautilus").is_ok());
    for icon in [
        "sketch.png",
        "../sketch.svg",
        "/etc/sketch.svg",
        "",
        "-whatsapp",
        "what sapp",
        "whats/app",
        "../whatsapp",
    ] {
        assert!(
            matches!(
                web_app("https://sketch.example/", icon),
                Err(ActionError::InvalidIcon { .. })
            ),
            "{icon}"
        );
    }
}

#[test]
fn resolving_a_run_records_the_scripts_it_names() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("pair"), "#!/bin/sh\nt3 pair\n").unwrap();
    let words = Words {
        user: "kuba".to_owned(),
        home: Path::new("/home/kuba").to_path_buf(),
        arch: "amd64".to_owned(),
        version: None,
    };
    let run =
        action("name = \"X\"\nkind = \"run\"\ndo = [\"./pair\"]\ncheck = [\"true\"]\nundo = [\"true\"]\n").unwrap();

    let resolved = run.resolve(&words, dir.path()).unwrap();

    let super::Kind::Run { scripts, .. } = resolved.kind else {
        panic!("not a run");
    };
    assert_eq!(scripts.get("pair").map(String::as_str), Some("#!/bin/sh\nt3 pair\n"));
}

fn web_app(title: &str, url: &str) -> Result<Action, ActionError> {
    let mut table = toml::Table::new();
    for (key, value) in [
        ("name", "Open it"),
        ("kind", "web_app"),
        ("id", "it"),
        ("title", title),
        ("url", url),
        ("icon", "it.svg"),
    ] {
        table.insert(key.to_owned(), toml::Value::String(value.to_owned()));
    }
    Action::try_from(table)
}

#[test]
fn a_web_apps_title_is_one_line() {
    let result = web_app("Penpot\nExec=rm -rf ~", "https://penpot.app/");

    assert!(matches!(result, Err(ActionError::InvalidTitle)), "{result:?}");
}

#[test]
fn a_web_app_opens_https_or_http_on_this_machine_with_its_port() {
    for url in [
        "https://penpot.app/",
        "http://localhost:9001",
        "http://127.0.0.1:9001/#/dashboard",
        "http://[::1]:9001/?a=1",
    ] {
        assert!(web_app("Penpot", url).is_ok(), "{url}");
    }
    for url in [
        "http://penpot.app/",
        "http://localhost/",
        "http://localhost:/",
        "http://localhost:99999/",
        "http://localhost.evil.example:9001/",
        "http://user@localhost:9001@evil.example/",
        "http://127.0.0.2:9001/",
        "ftp://localhost:9001/",
    ] {
        let result = web_app("Penpot", url);
        assert!(
            matches!(result, Err(ActionError::InvalidUrl { .. })),
            "{url}: {result:?}"
        );
    }
}

#[test]
fn a_pattern_of_versions_takes_a_run_of_anything_for_each_star() {
    for (pattern, version) in [
        ("*~build*", "160.0~build1"),
        ("*~build*", "~build"),
        ("1.*", "1.2.3"),
        ("1.2", "1.2"),
        ("*", ""),
        ("a*b*c", "abbc"),
    ] {
        assert!(version_matches(pattern, version), "{pattern} {version}");
    }
    for (pattern, version) in [
        ("*~build*", "1:1snap1-0ubuntu9.1"),
        ("1.*", "11.0"),
        ("1.2", "1.2.3"),
        ("a*b*c", "abcd"),
        ("ab*ba", "aba"),
    ] {
        assert!(!version_matches(pattern, version), "{pattern} {version}");
    }
}

#[test]
fn an_apts_pattern_of_versions_is_a_version_with_stars() {
    let apt = |versions: &str| {
        action(&format!(
            "name = \"Install\"\nkind = \"apt\"\npackages = [\"firefox\"]\nversions = \"{versions}\"\n"
        ))
    };

    assert!(apt("*~build*").is_ok());
    assert!(matches!(apt(""), Err(ActionError::InvalidVersions { .. })));
    assert!(matches!(apt("1 2"), Err(ActionError::InvalidVersions { .. })));
}

#[test]
fn a_conf_setting_takes_one_bare_key_and_other_files_dotted_ones() {
    let setting = |file: &str, key: &str| {
        action(&format!(
            "name = \"Set\"\nkind = \"setting\"\nfile = \"{file}\"\nkey = \"{key}\"\nvalue = \"pc\"\n"
        ))
    };

    assert!(setting("~/.config/sunshine/sunshine.conf", "origin_web_ui_allowed").is_ok());
    assert!(setting("~/.config/tool/config.toml", "ui.theme").is_ok());
    for key in ["ui.theme", "a=b", "# note", " padded"] {
        assert!(
            matches!(
                setting("~/.config/sunshine/sunshine.conf", key),
                Err(ActionError::InvalidKey { .. })
            ),
            "{key:?}"
        );
    }
    assert!(matches!(
        setting("~/.config/tool/config.ini", "theme"),
        Err(ActionError::InvalidSettingFile { .. })
    ));
}

#[test]
fn a_section_makes_a_conf_setting_an_ini_ones_whose_key_may_have_dots() {
    let setting = |file: &str, section: &str, key: &str| {
        action(&format!(
            "name = \"Set\"\nkind = \"setting\"\nfile = \"{file}\"\nsection = \"{section}\"\nkey = \"{key}\"\n\
             value = false\n"
        ))
    };

    assert!(setting("~/.config/zoomus.conf", "General", "xwayland").is_ok());
    assert!(setting("~/.config/zoomus.conf", "chat.recent", "conf.webserver").is_ok());
    for section in ["", "a]b", "[General", " General"] {
        assert!(
            matches!(
                setting("~/.config/zoomus.conf", section, "xwayland"),
                Err(ActionError::InvalidSection { .. })
            ),
            "{section:?}"
        );
    }
    assert!(matches!(
        setting("~/.config/tool/config.toml", "General", "xwayland"),
        Err(ActionError::InvalidSection { .. })
    ));
    for key in ["", "a=b", "; note", "[x", " padded"] {
        assert!(
            matches!(
                setting("~/.config/zoomus.conf", "General", key),
                Err(ActionError::InvalidKey { .. })
            ),
            "{key:?}"
        );
    }
}

#[test]
fn an_architecture_is_roots_and_one_dpkg_installs_beside_the_machines() {
    let i386 = action("name = \"X\"\nkind = \"architecture\"\narchitecture = \"i386\"\n").unwrap();
    let bogus = action("name = \"X\"\nkind = \"architecture\"\narchitecture = \"x86\"\n");

    assert_eq!(i386.side(), Side::Root);
    assert!(
        matches!(bogus, Err(ActionError::InvalidForeignArch { .. })),
        "{bogus:?}"
    );
}

#[test]
fn an_apt_package_of_another_architecture_names_one_dpkg_installs() {
    let i386 = action("name = \"X\"\nkind = \"apt\"\npackages = [\"libgl1:i386\", \"libgl1\"]\n");
    let bogus = action("name = \"X\"\nkind = \"apt\"\npackages = [\"libgl1:x86\"]\n");

    assert!(i386.is_ok(), "{i386:?}");
    assert!(matches!(bogus, Err(ActionError::InvalidPackage { .. })), "{bogus:?}");
}

#[test]
fn a_dkms_module_is_roots_from_a_pinned_source_and_names_what_dkms_takes() {
    let dkms = |module: &str, version: &str, folder: &str| {
        action(&format!(
            "name = \"X\"\nkind = \"dkms\"\nmodule = \"{module}\"\nversion = \"{version}\"\n\
             url = \"https://example.com/m.tar.gz\"\nsha256 = \"{}\"\nfolder = \"{folder}\"\nconf = \"dkms.conf\"\n",
            "a".repeat(64)
        ))
    };

    assert_eq!(dkms("hid-xpadneo", "0.10.5", "hid-xpadneo").unwrap().side(), Side::Root);
    assert!(matches!(
        dkms("Hid", "0.10.5", "src"),
        Err(ActionError::InvalidModule { .. })
    ));
    assert!(matches!(
        dkms("hid", "v0.10.5", "src"),
        Err(ActionError::InvalidModuleVersion { .. })
    ));
    assert!(matches!(
        dkms("hid", "0.10.5", "../etc"),
        Err(ActionError::InvalidSourceFolder { .. })
    ));
    let journal_only = action(&format!(
        "name = \"X\"\nkind = \"dkms\"\nmodule = \"m\"\nversion = \"1\"\nurl = \"https://example.com/m.tar.gz\"\n\
         sha256 = \"{}\"\nconf = \"dkms.conf\"\nconf_text = \"x\"\n",
        "a".repeat(64)
    ));
    assert!(
        matches!(journal_only, Err(ActionError::JournalOnly { key: "conf_text" })),
        "{journal_only:?}"
    );
}

#[test]
fn resolving_a_dkms_module_reads_its_conf_with_its_own_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("dkms.conf"), "PACKAGE_VERSION=\"@VERSION@\"\n").unwrap();
    let words = Words {
        user: "kuba".to_owned(),
        home: "/home/kuba".into(),
        arch: "amd64".to_owned(),
        version: Some("9".to_owned()),
    };
    let text = format!(
        "name = \"X\"\nkind = \"dkms\"\nmodule = \"m\"\nversion = \"1.2\"\nurl = \"https://example.com/m.tar.gz\"\n\
         sha256 = \"{}\"\nconf = \"dkms.conf\"\n",
        "a".repeat(64)
    );

    let resolved = action(&text).unwrap().resolve(&words, dir.path()).unwrap();

    let Kind::Dkms { conf_text, .. } = resolved.kind else {
        panic!("not dkms");
    };
    assert_eq!(conf_text.as_deref(), Some("PACKAGE_VERSION=\"1.2\"\n"));
}

#[test]
fn a_template_that_is_not_text_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("icon.in"), [0x89, b'P', b'N', b'G']).unwrap();
    let words = Words {
        user: "kuba".to_owned(),
        home: Path::new("/home/kuba").to_path_buf(),
        arch: "amd64".to_owned(),
        version: None,
    };
    let file =
        action("name = \"X\"\nkind = \"file\"\nto = \"~/.local/share/x.png\"\nfrom = \"icon.in\"\ntemplate = true\n")
            .unwrap();

    let resolved = file.resolve(&words, dir.path());

    assert!(
        matches!(resolved, Err(ActionError::BinaryTemplate { .. })),
        "{resolved:?}"
    );
}

#[test]
fn a_folder_marking_a_file_binary_is_refused() {
    let result = action("name = \"X\"\nkind = \"file\"\nto = \"~/.local/share/x\"\ntext = \"x\"\nbinary = true\n");

    assert!(
        matches!(result, Err(ActionError::JournalOnly { key: "binary" })),
        "{result:?}"
    );
}
