//! `kubaos suite`: installing, configuring and removing Suites, and scaffolding projects with
//! them.
//!
//!   kubaos suite list [--json]
//!   kubaos suite show NAME [--json]
//!   kubaos suite add NAME [--version V] [--extra E]... [--no-switch] [--detach | --foreground]
//!   kubaos suite configure NAME [--version V] [--extra E]... [--no-extra E]... [--no-switch] [...]
//!   kubaos suite remove NAME [--purge-caches] [--no-switch] [--detach | --foreground]
//!   kubaos suite new NAME PROJECT [--with SCAFFOLDER]
//!
//! `add`, `configure` and `remove` write or delete `kubaos.d/suites/<name>.toml`, then switch.
//! They never write to a user-layer file that exists. `remove` shows the size of each cache and
//! deletes them only with `--purge-caches`, and lists the user-layer files that mention the
//! Suite. `new` creates a project in the Projects root with an installed Suite. Exit codes: 0 ok,
//! 1 the work failed, 2 usage.
//!
//! # Owns
//!
//! - The command line of `suite`, and what it prints.
//!
//! # Not here
//!
//! - The Suites, their files, caches and scaffolders: `kubaos_core::suites`.
//! - Wiring `home.nix`: `package.rs`. The switch: `job.rs`.

use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use clap::Subcommand;
use kubaos_core::state::Paths;
use kubaos_core::suites::CachePolicy;
use kubaos_core::suites::CacheUse;
use kubaos_core::suites::OptionsChange;
use kubaos_core::suites::Seeded;
use kubaos_core::suites::Suite;
use kubaos_core::suites::SuiteConfig;
use kubaos_core::suites::Suites;
use kubaos_core::suites::Toolchain;
use kubaos_core::suites::catalog;
use serde_json::Value;
use serde_json::json;

use crate::job;
use crate::job::SwitchMode;
use crate::job::report;
use crate::out::out;
use crate::package;
use crate::usage_error;

#[derive(Subcommand)]
pub enum SuiteCmd {
    /// Every Suite, and which are installed.
    List {
        #[arg(long)]
        json: bool,
    },
    /// A Suite's options, caches, the user-layer files that mention it, and its scaffolders.
    Show {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Install a Suite, then switch.
    Add {
        name: String,
        #[command(flatten)]
        options: OptionArgs,
        /// Only record it; the next switch installs it.
        #[arg(long)]
        no_switch: bool,
        #[command(flatten)]
        mode: SwitchMode,
    },
    /// Change an installed Suite's options, then switch.
    Configure {
        name: String,
        #[command(flatten)]
        options: OptionArgs,
        /// Turn an extra off.
        #[arg(long = "no-extra", value_name = "EXTRA")]
        disable: Vec<String>,
        /// Only record it; the next switch applies it.
        #[arg(long)]
        no_switch: bool,
        #[command(flatten)]
        mode: SwitchMode,
    },
    /// Uninstall a Suite, then switch. Your projects and your own files stay.
    Remove {
        name: String,
        /// Also delete the Suite's caches and downloads.
        #[arg(long)]
        purge_caches: bool,
        /// Only record it; the next switch uninstalls it.
        #[arg(long)]
        no_switch: bool,
        #[command(flatten)]
        mode: SwitchMode,
    },
    /// Create a project in the Projects root (~/Projects, or $KUBAOS_PROJECTS) with an installed
    /// Suite.
    #[expect(
        clippy::doc_markdown,
        reason = "clap prints the doc as the help text, where backticks would show."
    )]
    New {
        name: String,
        project: String,
        /// Which of the Suite's scaffolders; the first by default.
        #[arg(long = "with", value_name = "SCAFFOLDER")]
        scaffolder: Option<String>,
    },
}

/// The options `add` and `configure` share.
#[derive(Args)]
pub struct OptionArgs {
    /// The version to use.
    #[arg(long)]
    version: Option<String>,
    /// Turn an extra on.
    #[arg(long = "extra", value_name = "EXTRA")]
    enable: Vec<String>,
}

pub fn main(cmd: SuiteCmd) -> ExitCode {
    if rustix::process::geteuid().is_root() {
        return usage_error("run kubaos suite as yourself, not root: it changes your home");
    }
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    let suites = Suites::new(&paths);
    match cmd {
        SuiteCmd::List { json } => list(&suites, json),
        SuiteCmd::Show { name, json } => with_suite(&name, |suite| show(&paths, &suites, suite, json)),
        SuiteCmd::Add {
            name,
            options,
            no_switch,
            mode,
        } => with_suite(&name, |suite| {
            let change = OptionsChange {
                version: options.version,
                enable: options.enable,
                disable: Vec::new(),
            };
            add(&paths, &suites, suite, &change, no_switch, mode)
        }),
        SuiteCmd::Configure {
            name,
            options,
            disable,
            no_switch,
            mode,
        } => with_suite(&name, |suite| {
            let change = OptionsChange {
                version: options.version,
                enable: options.enable,
                disable,
            };
            configure(&paths, &suites, suite, &change, no_switch, mode)
        }),
        SuiteCmd::Remove {
            name,
            purge_caches,
            no_switch,
            mode,
        } => with_suite(&name, |suite| {
            let caches = if purge_caches {
                CachePolicy::Purge
            } else {
                CachePolicy::Keep
            };
            remove(&paths, &suites, suite, caches, no_switch, mode)
        }),
        SuiteCmd::New {
            name,
            project,
            scaffolder,
        } => with_suite(&name, |suite| {
            new(&paths, &suites, suite, &project, scaffolder.as_deref())
        }),
    }
}

/// Runs `then` with the Suite called `name`, or reports that there is none.
pub(crate) fn with_suite(name: &str, then: impl FnOnce(&'static Suite) -> ExitCode) -> ExitCode {
    if let Some(suite) = catalog::find(name) {
        return then(suite);
    }
    let known: Vec<&str> = catalog::all().map(|s| s.name).collect();
    let known = if known.is_empty() {
        "none yet".to_owned()
    } else {
        known.join(", ")
    };
    usage_error(format!("no Suite called {name:?} (Suites: {known})"))
}

fn add(
    paths: &Paths,
    suites: &Suites,
    suite: &Suite,
    change: &OptionsChange,
    no_switch: bool,
    mode: SwitchMode,
) -> ExitCode {
    if let Err(code) = package::wire(paths) {
        return code;
    }
    let added = match suites.add(suite, change) {
        Ok(added) => added,
        Err(e) => return report(e),
    };
    out!("added the {} Suite ({})", suite.title, describe(&added.config));
    print_seeds(paths, &added.seeds);
    if no_switch {
        return ExitCode::SUCCESS;
    }
    job::switch(paths, mode)
}

fn configure(
    paths: &Paths,
    suites: &Suites,
    suite: &Suite,
    change: &OptionsChange,
    no_switch: bool,
    mode: SwitchMode,
) -> ExitCode {
    let wired = match package::wire(paths) {
        Ok(wired) => wired,
        Err(code) => return code,
    };
    let configured = match suites.configure(suite, change) {
        Ok(configured) => configured,
        Err(e) => return report(e),
    };
    if configured.changed {
        out!(
            "configured the {} Suite ({})",
            suite.title,
            describe(&configured.config)
        );
    } else {
        out!("the {} Suite is already {}", suite.title, describe(&configured.config));
    }
    print_seeds(paths, &configured.seeds);
    if no_switch || !(configured.changed || wired) {
        return ExitCode::SUCCESS;
    }
    job::switch(paths, mode)
}

fn remove(
    paths: &Paths,
    suites: &Suites,
    suite: &Suite,
    caches: CachePolicy,
    no_switch: bool,
    mode: SwitchMode,
) -> ExitCode {
    if let Err(code) = package::wire(paths) {
        return code;
    }
    let removed = match suites.remove(suite, caches) {
        Ok(removed) => removed,
        Err(e) => return report(e),
    };
    out!("removed the {} Suite", suite.title);
    for cache in removed.caches.iter().filter(|c| c.bytes.is_some()) {
        let what = if removed.purged.contains(&cache.path) {
            "deleted"
        } else {
            "kept (--purge-caches deletes it)"
        };
        out!("cache {} ({}): {what}", tilde(paths, &cache.path), size(cache));
    }
    if !removed.mentions.is_empty() {
        out!("your files that mention it, left as they are:");
        for path in &removed.mentions {
            out!("  {}", tilde(paths, path));
        }
    }
    if no_switch {
        return ExitCode::SUCCESS;
    }
    job::switch(paths, mode)
}

fn new(paths: &Paths, suites: &Suites, suite: &Suite, project: &str, id: Option<&str>) -> ExitCode {
    let scaffolder = match id {
        Some(id) => suite.scaffolders.iter().find(|s| s.id == id),
        None => suite.scaffolders.first(),
    };
    let Some(scaffolder) = scaffolder else {
        let ids: Vec<&str> = suite.scaffolders.iter().map(|s| s.id).collect();
        return usage_error(format!(
            "the {} Suite has no scaffolder {:?} (it has: {})",
            suite.title,
            id.unwrap_or_default(),
            ids.join(", ")
        ));
    };
    match suites.scaffold(suite, scaffolder, project) {
        Ok(path) => {
            out!("created {}", tilde(paths, &path));
            ExitCode::SUCCESS
        }
        Err(e) => report(e),
    }
}

fn list(suites: &Suites, json: bool) -> ExitCode {
    let installed = match suites.installed() {
        Ok(installed) => installed,
        Err(e) => return report(e),
    };
    let mut rows: Vec<Value> = catalog::all()
        .map(|suite| {
            let config = installed.get(suite.name);
            json!({
                "name": suite.name,
                "title": suite.title,
                "description": suite.description,
                "installed": config.is_some(),
                "version": config.and_then(|c| c.version.clone()),
                "extras": config.map(|c| c.extras.iter().cloned().collect::<Vec<_>>()).unwrap_or_default(),
                "scaffolders": if config.is_some() { scaffolders(suite) } else { json!([]) },
            })
        })
        .collect();
    for name in installed.keys().filter(|name| catalog::find(name).is_none()) {
        rows.push(json!({ "name": name, "installed": true, "known": false }));
    }
    if json {
        out!("{}", serde_json::to_string_pretty(&rows).expect("the list serializes"));
        return ExitCode::SUCCESS;
    }
    for suite in catalog::all() {
        let state = installed
            .get(suite.name)
            .map_or_else(|| "not installed".to_owned(), describe);
        out!("{}  {}  ({state})", suite.name, suite.description);
    }
    for name in installed.keys().filter(|name| catalog::find(name).is_none()) {
        out!("{name}  (suites/{name}.toml: not a KubaOS Suite, its packages still install)");
    }
    ExitCode::SUCCESS
}

fn show(paths: &Paths, suites: &Suites, suite: &Suite, json: bool) -> ExitCode {
    let config = match suites.config(suite) {
        Ok(config) => config,
        Err(e) => return report(e),
    };
    let caches = suites.caches(suite);
    let mentions = suites.mentions(suite);
    if json {
        let value = json!({
            "name": suite.name,
            "title": suite.title,
            "description": suite.description,
            "toolchain": match suite.toolchain {
                Toolchain::Nix => "nix",
                Toolchain::Upstream { manager } => manager,
                _ => "other",
            },
            "versions": suite.versions,
            "extras": suite.extras.iter().map(|e| json!({
                "name": e.name,
                "description": e.description,
                "default": e.default,
            })).collect::<Vec<_>>(),
            "installed": config.is_some(),
            "config": config,
            "caches": caches.iter().map(|c| json!({
                "path": c.path,
                "description": c.description,
                "bytes": c.bytes,
            })).collect::<Vec<_>>(),
            "mentions": mentions,
            "scaffolders": scaffolders(suite),
        });
        out!(
            "{}",
            serde_json::to_string_pretty(&value).expect("the Suite serializes")
        );
        return ExitCode::SUCCESS;
    }
    out!("{} ({}): {}", suite.title, suite.name, suite.description);
    match &config {
        Some(config) => out!("installed: {}", describe(config)),
        None => out!("not installed"),
    }
    if !suite.versions.is_empty() {
        out!("versions: {}", suite.versions.join(", "));
    }
    for extra in suite.extras {
        out!("extra {}: {}", extra.name, extra.description);
    }
    for cache in &caches {
        out!("cache {} ({})", tilde(paths, &cache.path), size(cache));
    }
    for path in &mentions {
        out!("mentioned in {}", tilde(paths, path));
    }
    for scaffolder in suite.scaffolders {
        out!(
            "{}: kubaos suite new {} NAME --with {}",
            scaffolder.label,
            suite.name,
            scaffolder.id
        );
    }
    ExitCode::SUCCESS
}

fn scaffolders(suite: &Suite) -> Value {
    suite
        .scaffolders
        .iter()
        .map(|s| json!({ "id": s.id, "label": s.label }))
        .collect()
}

fn print_seeds(paths: &Paths, seeds: &Seeded) {
    for path in &seeds.created {
        out!("created {} (yours to edit)", tilde(paths, path));
    }
    for path in &seeds.kept {
        out!("kept {}: it is yours", tilde(paths, path));
    }
}

/// "version 2, extras: greeting".
fn describe(config: &SuiteConfig) -> String {
    let mut parts = Vec::new();
    if let Some(version) = &config.version {
        parts.push(format!("version {version}"));
    }
    if config.extras.is_empty() {
        parts.push("no extras".to_owned());
    } else {
        parts.push(format!(
            "extras: {}",
            config.extras.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    parts.join(", ")
}

fn size(cache: &CacheUse) -> String {
    cache.bytes.map_or_else(|| "missing".to_owned(), human_bytes)
}

/// `bytes` in the largest binary unit that keeps it at 1 or more: "512 B", "1.5 MiB".
fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / 1024.0;
    let mut unit = UNITS[0];
    for next in &UNITS[1..] {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next;
    }
    format!("{value:.1} {unit}")
}

/// `path` with the home shown as `~`.
pub(crate) fn tilde(paths: &Paths, path: &Path) -> String {
    match path.strip_prefix(paths.home()) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}
