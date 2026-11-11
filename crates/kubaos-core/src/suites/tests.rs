use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use kubaos_user_layer::App;

use super::Cache;
use super::CachePolicy;
use super::Extra;
use super::OptionsChange;
use super::Request;
use super::ScaffoldError;
use super::Scaffolder;
use super::Seed;
use super::Suite;
use super::SuiteConfig;
use super::SuiteError;
use super::Suites;
use super::Tool;
use super::ToolState;
use super::Toolchain;
use super::catalog;
use super::default_config;
use crate::mise::ToolRequest;
use crate::mise::fake::FakeMise;
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
    mise: &[],
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

/// [`DEMO`] with mise tools: its main toolchain on the Suite's version, a language server with
/// the `lsp` extra and a formatter with `fmt`.
pub(super) const DEMO_MISE: Suite = Suite {
    mise: &[
        Tool {
            name: "demo-lang",
            request: Request::Suite,
            extra: None,
        },
        Tool {
            name: "demo-ls",
            request: Request::Fixed("0.5"),
            extra: Some("lsp"),
        },
        Tool {
            name: "demo-fmt",
            request: Request::Fixed("3.1"),
            extra: Some("fmt"),
        },
    ],
    toolchain: Toolchain::Mise,
    ..DEMO
};

pub(super) struct Home {
    _tmp: tempfile::TempDir,
    pub(super) paths: Paths,
    pub(super) suites: Suites,
    pub(super) mise: Arc<FakeMise>,
}

pub(super) fn home() -> Home {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::from_vars(Some(tmp.path().as_os_str().to_owned()), None, None).unwrap();
    fs::create_dir_all(paths.flake()).unwrap();
    let mise = Arc::new(FakeMise::new(paths.mise_config().join("conf.d")));
    let suites = Suites::with_mise(&paths, Arc::clone(&mise) as Arc<dyn crate::mise::Mise>);
    Home {
        _tmp: tmp,
        paths,
        suites,
        mise,
    }
}

pub(super) fn fragment(h: &Home) -> PathBuf {
    h.paths.mise_fragment("demo")
}

pub(super) fn requests(pairs: &[(&str, &str)]) -> Vec<ToolRequest> {
    pairs
        .iter()
        .map(|(tool, request)| ToolRequest {
            tool: (*tool).to_owned(),
            request: (*request).to_owned(),
        })
        .collect()
}

pub(super) fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
    fs::write(path, contents).expect("write the file");
}

fn extras(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

pub(super) fn suite_file(h: &Home) -> PathBuf {
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
    write(
        &layer.join("terminal/zsh/zshrc"),
        "export PATH=$HOME/demo-ls/bin:$PATH\n",
    );
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

/// A Suite's Nix module, `home/suites/<name>.nix`, as text.
fn module(suite: &Suite) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../home/suites")
        .join(format!("{}.nix", suite.name));
    fs::read_to_string(&path).expect("read the Suite's module")
}

/// The strings in a module's `versions = [ … ];` list, or none without one.
fn module_versions(text: &str) -> Vec<String> {
    let Some((_, after)) = text.split_once("versions = [") else {
        return Vec::new();
    };
    let (list, _) = after.split_once("];").expect("the versions list ends");
    list.split('"').skip(1).step_by(2).map(str::to_owned).collect()
}

#[test]
fn every_suite_s_module_declares_the_versions_of_its_catalog_entry() {
    for suite in catalog::all() {
        assert_eq!(
            module_versions(&module(suite)),
            suite.versions,
            "{}: the module's versions and the catalog's differ",
            suite.name
        );
    }
}

#[test]
fn every_scaffolder_runs_a_command_its_module_installs() {
    for suite in catalog::all() {
        let text = module(suite);
        for scaffolder in suite.scaffolders {
            let command = scaffolder.argv[0];
            assert!(
                text.contains(command),
                "{}: {} runs {command}, which its module never names",
                suite.name,
                scaffolder.id
            );
        }
    }
}

#[test]
fn no_cache_folder_holds_another_suite_s() {
    let caches: Vec<(&str, &str)> = catalog::all()
        .flat_map(|suite| suite.caches.iter().map(|cache| (suite.name, cache.path)))
        .collect();

    for (i, (suite, path)) in caches.iter().enumerate() {
        for (other, other_path) in &caches[i + 1..] {
            let a = Path::new(path);
            let b = Path::new(other_path);
            assert!(
                !a.starts_with(b) && !b.starts_with(a),
                "{suite}'s {path} and {other}'s {other_path} overlap: purging one deletes the other"
            );
        }
    }
}

#[test]
fn every_seed_says_which_suite_it_is_for() {
    for suite in catalog::all() {
        for seed in suite.seeds {
            let first = seed.contents.lines().next().unwrap_or_default().to_lowercase();
            let wanted = format!("-- your settings for the {} suite.", suite.title.to_lowercase());
            assert!(
                first.starts_with(&wanted),
                "{}: {} starts {first:?}",
                suite.name,
                seed.path
            );
        }
    }
}

#[test]
fn the_node_python_php_and_proto_suites_are_shipped() {
    let shipped: Vec<&str> = catalog::SHIPPED.iter().map(|s| s.name).collect();

    for name in ["node", "python", "php", "proto"] {
        assert!(shipped.contains(&name), "{name} is not in SHIPPED");
    }
}

#[test]
fn a_suite_s_default_version_is_its_first() {
    let node = catalog::find("node").expect("the Node Suite");
    let php = catalog::find("php").expect("the PHP Suite");
    let python = catalog::find("python").expect("the Python Suite");

    assert_eq!(node.versions.first(), Some(&"26"));
    assert_eq!(php.versions.first(), Some(&"8.5"));
    assert!(python.versions.is_empty(), "uv picks a project's Python");
}

#[test]
fn the_proto_suite_leaves_versions_to_proto() {
    let proto = catalog::find("proto").expect("the proto + moon Suite");

    assert!(proto.versions.is_empty(), "a project's .prototools pins its tools");
    assert!(matches!(proto.toolchain, Toolchain::Upstream { manager: "proto" }));
    assert_eq!(
        proto.caches.iter().map(|c| c.path).collect::<Vec<_>>(),
        [".local/share/proto"]
    );
}

#[test]
fn the_node_suite_s_first_scaffolder_needs_no_extra() {
    let node = catalog::find("node").expect("the Node Suite");

    assert_eq!(node.scaffolders.first().map(|s| s.id), Some("node"));
    assert!(node.extras.iter().any(|e| e.name == "bun" && e.default));
}

#[test]
fn the_wallpaper_depth_cache_is_the_folder_its_queue_writes_into() {
    let suite = catalog::find("wallpaper-depth").expect("the wallpaper-depth Suite");
    let queue = include_str!("../../../../home/suites/wallpaper-depth-queue.sh");

    let [cache] = suite.caches else {
        panic!("one cache: {:?}", suite.caches);
    };

    assert!(
        queue.contains(&format!("RESULTS=\"$HOME/{}\"", cache.path)),
        "{}",
        cache.path
    );
}

#[test]
fn a_suite_is_found_by_name() {
    assert_eq!(catalog::find("test").map(|s| s.name), Some("test"));
    assert!(catalog::find("nope").is_none());
}

#[test]
fn kubaos_ships_the_rust_and_go_suites() {
    let shipped: Vec<&str> = catalog::SHIPPED.iter().map(|s| s.name).collect();

    for name in ["rust", "go"] {
        assert!(shipped.contains(&name), "{name} is not in SHIPPED");
    }
    assert_eq!(
        catalog::find("rust").map(|s| s.toolchain),
        Some(Toolchain::Upstream { manager: "rustup" })
    );
    assert_eq!(catalog::find("go").map(|s| s.toolchain), Some(Toolchain::Nix));
}

#[test]
fn rust_has_no_versions_since_rust_toolchain_toml_pins_a_project() {
    let rust = catalog::find("rust").unwrap();

    assert!(rust.versions.is_empty());
    assert!(rust.description.contains("rust-toolchain.toml"), "{}", rust.description);
    assert_eq!(default_config(rust).version, None);
}

#[test]
fn rust_links_with_mold_unless_the_wild_extra_is_on() {
    let rust = catalog::find("rust").unwrap();

    assert_eq!(
        rust.extras.iter().map(|e| (e.name, e.default)).collect::<Vec<_>>(),
        [("wild", false)]
    );
    assert!(default_config(rust).extras.is_empty());
}

#[test]
fn rust_scaffolds_a_binary_a_library_and_a_workspace() {
    let rust = catalog::find("rust").unwrap();

    let ids: Vec<&str> = rust.scaffolders.iter().map(|s| s.id).collect();

    assert_eq!(ids, ["bin", "lib", "workspace"]);
}

#[test]
fn go_installs_the_version_nixpkgs_builds_its_go_tools_with_by_default() {
    let go = catalog::find("go").unwrap();

    assert_eq!(default_config(go).version.as_deref(), Some("1.26"));
}

#[test]
fn every_seed_is_a_commented_example_that_mentions_its_suite() {
    for suite in catalog::all() {
        for seed in suite.seeds {
            let code: Vec<&str> = seed
                .contents
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with("--"))
                .collect();
            let text = seed.contents.to_ascii_lowercase();

            assert_eq!(code, ["return {", "}"], "{}: {}", suite.name, seed.path);
            assert!(
                suite
                    .mentions
                    .iter()
                    .any(|word| super::has_word(&text, &word.to_ascii_lowercase())),
                "{}: removal wouldn't list {}",
                suite.name,
                seed.path
            );
        }
    }
}

#[test]
fn a_suite_without_mise_tools_never_runs_mise() {
    let h = home();

    h.suites.add(&DEMO, &OptionsChange::default()).unwrap();
    h.suites
        .configure(
            &DEMO,
            &OptionsChange {
                version: Some("2".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap();
    let tools = h.suites.tools([&DEMO]).unwrap();
    h.suites.remove(&DEMO, CachePolicy::Keep).unwrap();

    assert!(h.mise.calls().is_empty(), "{:?}", h.mise.calls());
    assert!(tools.is_empty(), "{tools:?}");
    assert!(!fragment(&h).exists());
}

#[test]
fn adding_installs_the_tools_first_then_writes_the_fragment_and_the_record() {
    let h = home();

    let added = h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    assert_eq!(added.tools, requests(&[("demo-lang", "1.0"), ("demo-ls", "0.5")]));
    assert_eq!(
        h.mise.calls(),
        [
            "mise latest demo-lang@1",
            "mise ls --json",
            "mise install demo-lang@1.0 demo-ls@0.5",
            "mise lock --global"
        ]
    );
    assert_eq!(h.mise.installed(), ["demo-lang@1.0.3", "demo-ls@0.5.2"]);
    assert!(suite_file(&h).exists());
    let text = fs::read_to_string(fragment(&h)).unwrap();
    assert!(text.starts_with('#'), "the fragment says who writes it: {text}");
    assert!(text.contains("demo-lang = \"1.0\"\ndemo-ls = \"0.5\""), "{text}");
    assert_eq!(added.before.config, None);
}

#[test]
fn the_fragment_holds_tools_and_no_setting() {
    let h = home();

    h.suites
        .add(
            &DEMO_MISE,
            &OptionsChange {
                enable: vec!["fmt".into()],
                ..OptionsChange::default()
            },
        )
        .unwrap();

    let table: toml::Table = toml::from_str(&fs::read_to_string(fragment(&h)).unwrap()).unwrap();
    assert_eq!(table.keys().collect::<Vec<_>>(), ["tools"]);
    assert_eq!(fragment(&h).file_name().unwrap(), "kubaos-demo.toml");
    assert_eq!(
        fragment(&h).parent().unwrap(),
        h.paths.config_home().join("mise/conf.d")
    );
}

#[test]
fn a_failed_install_leaves_no_fragment_no_record_and_no_leftover_versions() {
    let h = home();
    h.mise.preinstall("demo-lang", "1.0.0");
    h.mise.fail_on("demo-ls");

    let err = h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap_err();

    assert!(matches!(err, SuiteError::Install { .. }), "{err:?}");
    assert!(!suite_file(&h).exists());
    assert!(!fragment(&h).exists());
    assert_eq!(
        h.mise.installed(),
        ["demo-lang@1.0.0"],
        "only what was there before stays"
    );
    assert!(
        h.mise.calls().contains(&"mise uninstall demo-lang@1.0.3".to_owned()),
        "{:?}",
        h.mise.calls()
    );
    assert!(!h.mise.calls().iter().any(|c| c.starts_with("mise lock")));
}

#[test]
fn removing_deletes_the_fragment_and_prunes_at_once() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    h.suites.remove(&DEMO_MISE, CachePolicy::Keep).unwrap();

    assert!(!fragment(&h).exists());
    assert!(!suite_file(&h).exists());
    assert!(h.mise.installed().is_empty(), "{:?}", h.mise.installed());
    assert_eq!(
        h.mise.calls()[4..],
        ["mise prune --tools", "mise lock --global"],
        "{:?}",
        h.mise.calls()
    );
}

#[test]
fn removing_leaves_the_user_s_own_mise_config_alone() {
    let h = home();
    let own = h.paths.mise_config().join("config.toml");
    write(&own, "[tools]\ndemo-lang = \"2\"\n\n[settings]\nexperimental = true\n");
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    h.suites.remove(&DEMO_MISE, CachePolicy::Keep).unwrap();

    assert_eq!(
        fs::read_to_string(&own).unwrap(),
        "[tools]\ndemo-lang = \"2\"\n\n[settings]\nexperimental = true\n"
    );
}

#[test]
fn configuring_a_new_version_installs_it_and_prunes_the_old_one() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    let configured = h
        .suites
        .configure(
            &DEMO_MISE,
            &OptionsChange {
                version: Some("2".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap();

    assert!(configured.changed);
    assert_eq!(configured.tools, requests(&[("demo-lang", "2.1"), ("demo-ls", "0.5")]));
    assert_eq!(
        configured.config.version.as_deref(),
        Some("2"),
        "the record keeps the line"
    );
    assert_eq!(h.mise.installed(), ["demo-lang@2.1.0", "demo-ls@0.5.2"]);
    assert_eq!(
        configured.before.tools,
        requests(&[("demo-lang", "1.0"), ("demo-ls", "0.5")])
    );
}

#[test]
fn configuring_keeps_the_requests_an_upgrade_moved() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();
    write(&fragment(&h), "[tools]\ndemo-lang = \"1.0\"\ndemo-ls = \"0.4\"\n");

    let configured = h
        .suites
        .configure(
            &DEMO_MISE,
            &OptionsChange {
                enable: vec!["fmt".into()],
                ..OptionsChange::default()
            },
        )
        .unwrap();

    assert_eq!(
        configured.tools,
        requests(&[("demo-lang", "1.0"), ("demo-ls", "0.4"), ("demo-fmt", "3.1")])
    );
}

#[test]
fn configuring_to_the_same_options_runs_no_install() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();
    let calls = h.mise.calls().len();

    let configured = h.suites.configure(&DEMO_MISE, &OptionsChange::default()).unwrap();

    assert!(!configured.changed);
    assert_eq!(h.mise.calls().len(), calls, "{:?}", h.mise.calls());
}

#[test]
fn a_failed_configure_changes_nothing() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();
    let record = fs::read_to_string(suite_file(&h)).unwrap();
    let before = fs::read_to_string(fragment(&h)).unwrap();
    h.mise.fail_on("demo-fmt");

    let err = h
        .suites
        .configure(
            &DEMO_MISE,
            &OptionsChange {
                version: Some("2".into()),
                enable: vec!["fmt".into()],
                ..OptionsChange::default()
            },
        )
        .unwrap_err();

    assert!(matches!(err, SuiteError::Install { .. }), "{err:?}");
    assert_eq!(fs::read_to_string(suite_file(&h)).unwrap(), record);
    assert_eq!(fs::read_to_string(fragment(&h)).unwrap(), before);
    assert_eq!(h.mise.installed(), ["demo-lang@1.0.3", "demo-ls@0.5.2"]);
}

#[test]
fn restoring_after_a_failed_switch_takes_an_add_back_out() {
    let h = home();
    let added = h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    h.suites.restore(&DEMO_MISE, &added.before).unwrap();

    assert!(!suite_file(&h).exists());
    assert!(!fragment(&h).exists());
    assert!(h.mise.installed().is_empty(), "{:?}", h.mise.installed());
}

#[test]
fn restoring_after_a_failed_switch_puts_a_configure_back() {
    let h = home();
    let added = h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();
    let record = fs::read_to_string(suite_file(&h)).unwrap();
    let configured = h
        .suites
        .configure(
            &DEMO_MISE,
            &OptionsChange {
                version: Some("2".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap();

    h.suites.restore(&DEMO_MISE, &configured.before).unwrap();

    assert_eq!(fs::read_to_string(suite_file(&h)).unwrap(), record);
    assert_eq!(h.suites.tools([&DEMO_MISE]).unwrap()["demo"].len(), added.tools.len());
    assert_eq!(h.mise.installed(), ["demo-lang@1.0.3", "demo-ls@0.5.2"]);
}

#[test]
fn tools_reads_each_suite_s_versions_from_mise() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    let tools = h.suites.tools([&DEMO_MISE, &DEMO]).unwrap();

    assert_eq!(
        tools["demo"],
        [
            ToolState {
                tool: "demo-lang".into(),
                request: "1.0".into(),
                version: Some("1.0.3".into()),
                installed: true,
            },
            ToolState {
                tool: "demo-ls".into(),
                request: "0.5".into(),
                version: Some("0.5.2".into()),
                installed: true,
            },
        ]
    );
    assert_eq!(h.mise.calls().last().map(String::as_str), Some("mise ls --json"));
}

#[test]
fn the_test_suite_installs_through_mise() {
    let h = home();

    let added = h.suites.add(&catalog::TEST, &OptionsChange::default()).unwrap();
    h.suites.remove(&catalog::TEST, CachePolicy::Keep).unwrap();

    assert_eq!(catalog::TEST.toolchain, Toolchain::Mise);
    assert_eq!(added.tools, requests(&[("kubaos-test-tool", "1.0")]));
    assert_eq!(h.mise.installed(), Vec::<String>::new());
    assert!(!h.paths.mise_fragment("test").exists());
}

#[test]
fn every_mise_tool_in_the_catalog_is_well_formed() {
    for suite in catalog::all() {
        let name = suite.name;
        let mut tools = BTreeSet::new();
        for tool in suite.mise {
            assert!(tools.insert(tool.name), "{name}: {} twice", tool.name);
            if let Some(extra) = tool.extra {
                assert!(suite.extras.iter().any(|e| e.name == extra), "{name}: no extra {extra}");
            }
            match tool.request {
                Request::Suite => assert!(!suite.versions.is_empty(), "{name}: {} takes no version", tool.name),
                Request::Fixed(request) => {
                    let parts: Vec<&str> = request.split('.').collect();
                    assert!(
                        parts.len() == 2 && parts.iter().all(|p| p.parse::<u32>().is_ok()),
                        "{name}: {} asks for {request:?}, not major.minor",
                        tool.name
                    );
                }
            }
        }
        assert_eq!(
            suite.toolchain == Toolchain::Mise,
            suite.mise.iter().any(|t| t.request == Request::Suite),
            "{name}: a Suite's toolchain is mise's when its main tool is"
        );
    }
}
