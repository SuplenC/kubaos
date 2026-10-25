use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;

use super::Cache;
use super::CachePolicy;
use super::Extra;
use super::OptionsChange;
use super::ScaffoldError;
use super::Scaffolder;
use super::Seed;
use super::Suite;
use super::SuiteConfig;
use super::SuiteError;
use super::Suites;
use super::Toolchain;
use super::catalog;
use crate::state::Paths;
use crate::state::StateDir;

const SEED: &str = "-- Your Demo settings.\nreturn {}\n";

/// A Suite with one of everything, whose scaffolder needs only `sh`.
const DEMO: Suite = Suite {
    name: "demo",
    title: "Demo",
    description: "A Suite for these tests.",
    versions: &["1", "2"],
    extras: &[
        Extra {
            name: "lsp",
            description: "A language server.",
            default: true,
        },
        Extra {
            name: "fmt",
            description: "A formatter.",
            default: false,
        },
    ],
    toolchain: Toolchain::Nix,
    caches: &[
        Cache {
            path: ".cache/demo",
            description: "Build cache.",
        },
        Cache {
            path: "demo-deps",
            description: "Downloaded dependencies.",
        },
    ],
    scaffolders: &[Scaffolder {
        id: "app",
        label: "New Demo project",
        argv: &["sh", "-c", "mkdir \"$1\" && echo '{name}' >\"$1/NAME\"", "sh", "{path}"],
    }],
    seeds: &[Seed {
        app: App::Nvim,
        path: "plugins/demo.lua",
        contents: SEED,
    }],
    mentions: &["demo", "demo-ls"],
};

struct Home {
    _tmp: tempfile::TempDir,
    paths: Paths,
    suites: Suites,
}

fn home() -> Home {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::from_vars(Some(tmp.path().as_os_str().to_owned()), None, None).unwrap();
    fs::create_dir_all(paths.flake()).unwrap();
    let suites = Suites::new(&paths);
    Home {
        _tmp: tmp,
        paths,
        suites,
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
    fs::write(path, contents).expect("write the file");
}

fn extras(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

fn suite_file(h: &Home) -> PathBuf {
    h.paths.state_dir().suite_file("demo")
}

#[test]
fn adding_a_suite_writes_its_file_with_the_default_options() {
    let h = home();

    let added = h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    assert_eq!(
        added.config,
        SuiteConfig {
            version: Some("1".into()),
            extras: extras(&["lsp"]),
        }
    );
    assert_eq!(h.suites.config(&DEMO).unwrap(), Some(added.config));
    let text = fs::read_to_string(suite_file(&h)).unwrap();
    assert!(text.starts_with('#'), "the file says who writes it: {text}");
}

#[test]
fn adding_with_options_writes_them() {
    let h = home();
    let change = OptionsChange {
        version: Some("2".into()),
        enable: vec!["fmt".into()],
        disable: vec!["lsp".into()],
    };

    let added = h.suites.add(&DEMO, &change).unwrap();

    assert_eq!(added.config.version.as_deref(), Some("2"));
    assert_eq!(added.config.extras, extras(&["fmt"]));
}

#[test]
fn the_suite_file_is_what_the_state_directory_reads() {
    let h = home();

    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    let declared = StateDir::new(h.paths.state_dir().root()).declared().unwrap();
    assert!(declared.is_empty(), "a Suite's packages come from its Nix module");
    let installed = h.suites.installed().unwrap();
    assert_eq!(installed.keys().collect::<Vec<_>>(), ["demo"]);
}

#[test]
fn adding_an_installed_suite_is_an_error_and_changes_nothing() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    let before = fs::read_to_string(suite_file(&h)).unwrap();

    let err = h
        .suites
        .add(
            &DEMO,
            &OptionsChange {
                version: Some("2".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap_err();

    assert!(matches!(err, SuiteError::AlreadyInstalled { .. }), "{err:?}");
    assert_eq!(fs::read_to_string(suite_file(&h)).unwrap(), before);
}

#[test]
fn an_unknown_version_or_extra_writes_nothing() {
    let h = home();

    let version = h
        .suites
        .add(
            &DEMO,
            &OptionsChange {
                version: Some("9".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap_err();
    let extra = h
        .suites
        .add(
            &DEMO,
            &OptionsChange {
                enable: vec!["nope".into()],
                ..OptionsChange::default()
            },
        )
        .unwrap_err();

    assert!(matches!(version, SuiteError::UnknownVersion { .. }), "{version:?}");
    assert!(version.to_string().contains("1, 2"), "{version}");
    assert!(matches!(extra, SuiteError::UnknownExtra { .. }), "{extra:?}");
    assert!(!suite_file(&h).exists());
}

#[test]
fn configuring_changes_only_what_it_names() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    let configured = h
        .suites
        .configure(
            &DEMO,
            &OptionsChange {
                enable: vec!["fmt".into()],
                ..OptionsChange::default()
            },
        )
        .unwrap();

    assert!(configured.changed);
    assert_eq!(configured.config.version.as_deref(), Some("1"));
    assert_eq!(configured.config.extras, extras(&["fmt", "lsp"]));
    assert_eq!(h.suites.config(&DEMO).unwrap(), Some(configured.config));
}

#[test]
fn configuring_to_the_same_options_changes_nothing() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    let configured = h
        .suites
        .configure(
            &DEMO,
            &OptionsChange {
                version: Some("1".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap();

    assert!(!configured.changed);
}

#[test]
fn configuring_a_suite_that_is_not_installed_is_an_error() {
    let h = home();

    let err = h.suites.configure(&DEMO, &OptionsChange::default()).unwrap_err();

    assert!(matches!(err, SuiteError::NotInstalled { .. }), "{err:?}");
    assert!(!suite_file(&h).exists());
}

#[test]
fn adding_seeds_a_missing_user_layer_file() {
    let h = home();

    let added = h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    let seeded = h.paths.user_layer().join("editor/nvim/plugins/demo.lua");
    assert_eq!(added.seeds.created, std::slice::from_ref(&seeded));
    assert_eq!(fs::read_to_string(seeded).unwrap(), SEED);
}

#[test]
fn a_user_layer_file_that_exists_is_never_written() {
    let h = home();
    let mine = h.paths.user_layer().join("editor/nvim/plugins/demo.lua");
    write(&mine, "return { { \"demo-ls\", opts = { mine = true } } }\n");

    let added = h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    h.suites
        .configure(
            &DEMO,
            &OptionsChange {
                version: Some("2".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap();

    assert!(added.seeds.created.is_empty());
    assert_eq!(added.seeds.kept, std::slice::from_ref(&mine));
    assert_eq!(
        fs::read_to_string(&mine).unwrap(),
        "return { { \"demo-ls\", opts = { mine = true } } }\n"
    );
}

#[cfg(unix)]
#[test]
fn a_dangling_link_in_the_user_layer_counts_as_existing() {
    let h = home();
    let link = h.paths.user_layer().join("editor/nvim/plugins/demo.lua");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink("/nowhere/demo.lua", &link).unwrap();

    let added = h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    assert_eq!(added.seeds.kept, std::slice::from_ref(&link));
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
}

#[test]
fn removing_deletes_the_suite_file_and_keeps_its_caches() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    write(&h.paths.home().join(".cache/demo/a"), "12345");
    write(&h.paths.home().join(".cache/demo/sub/b"), "123");

    let removed = h.suites.remove(&DEMO, CachePolicy::Keep).unwrap();

    assert!(!suite_file(&h).exists());
    assert!(h.suites.config(&DEMO).unwrap().is_none());
    assert!(h.paths.home().join(".cache/demo/a").exists());
    assert!(removed.purged.is_empty());
    let sizes: Vec<_> = removed.caches.iter().map(|c| (c.path.clone(), c.bytes)).collect();
    assert_eq!(
        sizes,
        [
            (h.paths.home().join(".cache/demo"), Some(8)),
            (h.paths.home().join("demo-deps"), None),
        ]
    );
}

#[test]
fn removing_with_purge_deletes_the_caches_that_exist() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    write(&h.paths.home().join(".cache/demo/a"), "12345");

    let removed = h.suites.remove(&DEMO, CachePolicy::Purge).unwrap();

    assert!(!h.paths.home().join(".cache/demo").exists());
    assert_eq!(removed.purged, [h.paths.home().join(".cache/demo")]);
}

#[test]
fn removing_never_touches_projects_or_the_user_layer() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    let project = h.paths.projects_root().join("my-demo/main.demo");
    write(&project, "demo");
    let mine = h.paths.user_layer().join("editor/nvim/plugins/demo.lua");
    write(&mine, "-- mine, demo\n");

    h.suites.remove(&DEMO, CachePolicy::Purge).unwrap();

    assert_eq!(fs::read_to_string(project).unwrap(), "demo");
    assert_eq!(fs::read_to_string(mine).unwrap(), "-- mine, demo\n");
}

#[test]
fn a_cache_inside_the_projects_root_is_never_purged() {
    const RISKY: Suite = Suite {
        caches: &[Cache {
            path: "Projects/.demo-cache",
            description: "A cache someone put in the wrong place.",
        }],
        ..DEMO
    };
    let h = home();
    h.suites.add(&RISKY, &OptionsChange::default()).unwrap();
    let cache = h.paths.projects_root().join(".demo-cache/x");
    write(&cache, "x");

    let removed = h.suites.remove(&RISKY, CachePolicy::Purge).unwrap();

    assert!(cache.exists());
    assert!(removed.caches.is_empty() && removed.purged.is_empty(), "{removed:?}");
}

#[test]
fn removing_a_suite_that_is_not_installed_is_an_error() {
    let h = home();

    let err = h.suites.remove(&DEMO, CachePolicy::Purge).unwrap_err();

    assert!(matches!(err, SuiteError::NotInstalled { .. }), "{err:?}");
}

#[test]
fn removing_lists_the_user_layer_files_that_mention_the_suite() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    let layer = h.paths.user_layer();
    write(&layer.join("terminal/zsh/zshrc"), "export PATH=$HOME/demo-ls/bin:$PATH\n");
    write(&layer.join("tools/git/config"), "[user]\n  name = Demonstration\n");
    write(
        &layer.join("home/home.nix"),
        "{ pkgs, ... }: { home.packages = [ pkgs.demo ]; }\n",
    );
    write(&layer.join("home/.git/config"), "demo\n");

    let removed = h.suites.remove(&DEMO, CachePolicy::Keep).unwrap();

    assert_eq!(
        removed.mentions,
        [
            layer.join("editor/nvim/plugins/demo.lua"),
            layer.join("home/home.nix"),
            layer.join("terminal/zsh/zshrc"),
        ]
    );
}

#[test]
fn the_state_directory_is_not_a_mention() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    fs::remove_file(h.paths.user_layer().join("editor/nvim/plugins/demo.lua")).unwrap();

    let mentions = h.suites.mentions(&DEMO);

    assert!(mentions.is_empty(), "{mentions:?}");
}

#[test]
fn a_suite_file_is_visible_to_git_when_the_flake_is_a_repo() {
    let h = home();
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(h.paths.flake())
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap()
    };
    assert!(git(&["init", "-q"]).status.success());

    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    let after_add = String::from_utf8(git(&["ls-files"]).stdout).unwrap();
    h.suites.remove(&DEMO, CachePolicy::Keep).unwrap();
    let after_remove = String::from_utf8(git(&["ls-files"]).stdout).unwrap();

    assert_eq!(after_add, "kubaos.d/suites/demo.toml\n");
    assert_eq!(after_remove, "");
}

#[test]
fn scaffolding_creates_the_project_in_the_projects_root() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    let path = h.suites.scaffold(&DEMO, &DEMO.scaffolders[0], "hello-demo").unwrap();

    assert_eq!(path, h.paths.projects_root().join("hello-demo"));
    assert_eq!(fs::read_to_string(path.join("NAME")).unwrap(), "hello-demo\n");
}

#[test]
fn scaffolding_needs_the_suite_installed() {
    let h = home();

    let err = h.suites.scaffold(&DEMO, &DEMO.scaffolders[0], "x").unwrap_err();

    assert!(matches!(err, ScaffoldError::NotInstalled { .. }), "{err:?}");
}

#[test]
fn scaffolding_never_replaces_a_project_that_exists() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    write(&h.paths.projects_root().join("taken/keep"), "mine");

    let err = h.suites.scaffold(&DEMO, &DEMO.scaffolders[0], "taken").unwrap_err();

    assert!(matches!(err, ScaffoldError::Exists { .. }), "{err:?}");
    assert_eq!(
        fs::read_to_string(h.paths.projects_root().join("taken/keep")).unwrap(),
        "mine"
    );
}

#[test]
fn a_project_name_is_one_plain_folder_name() {
    let h = home();
    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();

    for bad in ["", ".", "..", "a/b", "../up", "-rf", ".hidden", "a b"] {
        let err = h.suites.scaffold(&DEMO, &DEMO.scaffolders[0], bad).unwrap_err();
        assert!(matches!(err, ScaffoldError::InvalidName(_)), "{bad:?}: {err:?}");
    }
}

#[test]
fn a_scaffolder_that_fails_is_an_error() {
    const FAILING: Suite = Suite {
        scaffolders: &[Scaffolder {
            id: "app",
            label: "New Demo project",
            argv: &["sh", "-c", "exit 3"],
        }],
        ..DEMO
    };
    let h = home();
    h.suites.add(&FAILING, &OptionsChange::default()).unwrap();

    let err = h.suites.scaffold(&FAILING, &FAILING.scaffolders[0], "x").unwrap_err();

    assert!(matches!(err, ScaffoldError::Failed { .. }), "{err:?}");
}

#[test]
fn every_suite_in_the_catalog_is_well_formed() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suites: Vec<&Suite> = catalog::all().collect();
    assert!(suites.iter().any(|s| s.name == "test"), "tests see the test Suite");

    let mut names = BTreeSet::new();
    for suite in suites {
        let name = suite.name;
        assert!(names.insert(name), "{name} is in the catalog twice");
        assert!(super::is_suite_name(name), "{name:?}");
        assert!(
            repo.join(format!("home/suites/{name}.nix")).is_file(),
            "{name} has no Nix module"
        );
        assert!(!suite.versions.contains(&""), "{name}: an empty version");
        assert_eq!(
            suite.extras.iter().map(|e| e.name).collect::<BTreeSet<_>>().len(),
            suite.extras.len(),
            "{name}: an extra twice"
        );
        for path in suite
            .caches
            .iter()
            .map(|c| c.path)
            .chain(suite.seeds.iter().map(|s| s.path))
        {
            assert!(super::is_plain_relative(path), "{name}: {path:?}");
        }
        for cache in suite.caches {
            assert!(!cache.path.starts_with("Projects"), "{name}: {:?}", cache.path);
        }
        for scaffolder in suite.scaffolders {
            assert!(scaffolder.argv.contains(&"{path}"), "{name}: {}", scaffolder.id);
        }
        assert!(!suite.mentions.is_empty(), "{name} names no words to find mentions by");
    }
}

#[test]
fn a_suite_is_found_by_name() {
    assert_eq!(catalog::find("test").map(|s| s.name), Some("test"));
    assert!(catalog::find("nope").is_none());
}
