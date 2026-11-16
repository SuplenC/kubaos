//! The Apps catalog in the menu: an "Install <Name>" Item for each app of the catalog that isn't on
//! the machine, a "Remove <Name>" Item for each one that is, never both, so a manifest added to
//! the catalog brings its Items with it. Running one starts `kubaos app add|remove <id> --json` in
//! the background, as a Job the menu shows with each step and how far it got; the Daemon announces
//! its end on the desktop, named for the app, with why it failed and where its log is.
//!
//! Which apps are on the machine is the engine's status (`kubaos app status`): the catalog, the
//! picks, one `dpkg-query` and the User flake's `packages.toml`, read on a thread of its own at
//! the start, at each config and when one of the files it comes from changes, never as the menu
//! searches.
//!
//! # Owns
//!
//! - Reading the status, one read at a time off the protocol loop, the app Items, starting a Job
//!   and turning its events into the Job's progress.
//!
//! # Not here
//!
//! - The catalog, the picks and the status: `kubaos_core::apps`. Installing and removing: `kubaos
//!   app` (`crates/kubaos-cli/src/app.rs`).
//! - Watching the files and sending the Job: `main.rs`.

use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Fraction;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::Job;
use kotodama_protocol::JobState;
use kotodama_protocol::NamespacedKind;
use kubaos_core::apps;
use kubaos_core::apps::App;
use kubaos_core::apps::AppState;
use kubaos_core::apps::Catalog;
use kubaos_core::apps::CatalogError;
use kubaos_core::apps::Picks;
use kubaos_core::apps::PicksError;
use kubaos_core::apps::Status;
use kubaos_core::apps::plan::Facts;
use kubaos_core::apt::Dpkg;
use kubaos_core::apt::dpkg;
use kubaos_core::packages::Packages;
use kubaos_core::state::Paths;
use kubaos_core::state::ReadError as PackagesError;
use kubaos_core::update::event::UpdateEvent;
use kubaos_core::update::record::SystemPaths;

use crate::run::RunError;
use crate::run::Runner;

/// The Item kind of every app Item.
pub const KIND: &str = "kubaos.app";
/// Installs an app that isn't on the machine.
pub const INSTALL: &str = "install";
/// Removes an app that is.
pub const REMOVE: &str = "remove";
/// dpkg's database, which every install and removal of a package rewrites.
pub const DPKG_STATUS: &str = "/var/lib/dpkg/status";

/// An app's Install Item is this and its id, `app-install:google-chrome`.
const INSTALL_PREFIX: &str = "app-install:";
/// An app's Remove Item is this and its id, `app-remove:google-chrome`.
const REMOVE_PREFIX: &str = "app-remove:";
/// A Job is this and a count, `app-1`.
const JOB_PREFIX: &str = "app-";
/// The icon of an app whose manifest names none.
const ICON: &str = "system-software-install";

/// Where the apps' status is read from.
#[derive(Clone, Debug)]
pub struct AppFiles {
    catalog: PathBuf,
    picks: PathBuf,
    packages: Packages,
    packages_file: PathBuf,
}

impl AppFiles {
    /// The installed release's catalog, and the picks and `packages.toml` of `paths`.
    #[must_use]
    pub fn new(catalog: PathBuf, paths: &Paths) -> Self {
        let state = paths.state_dir();
        AppFiles {
            catalog,
            picks: paths.app_picks(),
            packages: Packages::new(&state, paths.edit_lock()),
            packages_file: state.packages_file(),
        }
    }

    /// The files of this machine and user; none without a home.
    pub fn from_env() -> Option<Self> {
        let paths = Paths::from_env().ok()?;
        Some(Self::new(SystemPaths::machine().catalog(), &paths))
    }

    /// The files whose change can change which apps are on the machine or picked: dpkg's
    /// database, the picks and the User flake's `packages.toml`. The catalog moves only with the
    /// release, whose update rewrites dpkg's database too.
    pub fn watched(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from(DPKG_STATUS),
            self.picks.clone(),
            self.packages_file.clone(),
        ]
    }
}

/// The apps' status could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("reading the Apps catalog")]
    Catalog(#[source] CatalogError),
    #[error("reading the apps you picked")]
    Picks(#[source] PicksError),
    #[error("asking dpkg which packages are installed")]
    Dpkg(#[source] RunError),
    #[error("reading your Nix packages")]
    Nix(#[source] PackagesError),
}

/// The apps of the catalog, each with where it stands on the machine.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Apps {
    /// Sorted by id, as the catalog is.
    apps: Vec<(App, AppState, Option<String>)>,
}

impl Apps {
    /// The apps of `catalog` as `status` has them; an app the status lacks is left out.
    #[must_use]
    pub fn new(catalog: &Catalog, status: &Status) -> Self {
        let apps = catalog
            .apps()
            .iter()
            .filter_map(|app| {
                let stands = status.apps.iter().find(|s| s.id == app.id)?;
                Some((app.clone(), stands.state, stands.version.clone()))
            })
            .collect();
        Apps { apps }
    }

    /// Reads the catalog, the picks, dpkg's packages with one `dpkg-query`, and `packages.toml`;
    /// only the catalog when it has no app.
    ///
    /// # Errors
    ///
    /// [`ReadError`] for the first of them that can't be read.
    pub fn read(files: &AppFiles, runner: &impl Runner) -> Result<Self, ReadError> {
        let catalog = Catalog::load(&files.catalog).map_err(ReadError::Catalog)?;
        if catalog.apps().is_empty() {
            return Ok(Apps::default());
        }
        let picks = Picks::read(&files.picks).map_err(ReadError::Picks)?;
        let query: Vec<String> = std::iter::once("dpkg-query")
            .chain(dpkg::QUERY)
            .map(str::to_owned)
            .collect();
        let dpkg = Dpkg::parse(&runner.output(&query).map_err(ReadError::Dpkg)?);
        let nix: BTreeSet<String> = files.packages.list().map_err(ReadError::Nix)?.into_iter().collect();
        let facts = Facts {
            catalog: &catalog,
            picks: &picks,
            arch: apps::arch(),
            dpkg: &dpkg,
            nix: &nix,
        };
        Ok(Self::new(&catalog, &Status::new(&facts)))
    }

    /// One Item an app: Remove while it is on the machine, Install while it isn't. An app with
    /// nothing for this machine's architecture has none.
    pub fn items(&self) -> Vec<Item> {
        self.apps
            .iter()
            .filter_map(|(app, state, version)| match state {
                AppState::Unavailable => None,
                state if state.is_installed() => Some(remove_item(app, version.as_deref())),
                _ => Some(install_item(app)),
            })
            .collect()
    }

    /// The id of every Item an app of the catalog may have, either half, which `hide` may name.
    pub fn ids(&self) -> Vec<String> {
        self.apps
            .iter()
            .flat_map(|(app, ..)| {
                [
                    format!("{INSTALL_PREFIX}{}", app.id),
                    format!("{REMOVE_PREFIX}{}", app.id),
                ]
            })
            .collect()
    }

    /// The app of the Item `id`, and whether the Item installs it, while that is the half that
    /// applies.
    fn offered(&self, id: &str) -> Option<(&App, Change)> {
        let (change, app_id) = match (id.strip_prefix(INSTALL_PREFIX), id.strip_prefix(REMOVE_PREFIX)) {
            (Some(app_id), _) => (Change::Add, app_id),
            (_, Some(app_id)) => (Change::Remove, app_id),
            _ => return None,
        };
        let (app, state, _) = self.apps.iter().find(|(app, ..)| app.id == app_id)?;
        let applies = match change {
            Change::Add => !state.is_installed() && *state != AppState::Unavailable,
            Change::Remove => state.is_installed(),
        };
        applies.then_some((app, change))
    }
}

/// Reads the apps on a thread of its own, one read at a time: `dpkg-query` never holds the
/// protocol loop, and a read never lands after a later one. Requests made while a read runs are
/// served by one read after it.
#[derive(Clone, Debug)]
pub struct AppsReader {
    requests: mpsc::Sender<()>,
}

impl AppsReader {
    /// Starts the thread, which reads `files` with `runner` and gives `read` each reading. A
    /// reading that fails is logged and given to no one, so what was read last stands.
    ///
    /// # Errors
    ///
    /// When the thread can't start.
    pub fn start(
        files: AppFiles,
        runner: impl Runner + Send + 'static,
        mut read: impl FnMut(Apps) + Send + 'static,
    ) -> io::Result<Self> {
        let (requests, requested) = mpsc::channel::<()>();
        thread::Builder::new().name("apps".into()).spawn(move || {
            while requested.recv().is_ok() {
                while requested.try_recv().is_ok() {}
                match Apps::read(&files, &runner) {
                    Ok(apps) => read(apps),
                    Err(err) => kotodama_extension::log!("not reading the apps: {}", Chain(&err)),
                }
            }
        })?;
        Ok(AppsReader { requests })
    }

    /// Asks for a reading.
    pub fn read(&self) {
        let _ = self.requests.send(());
    }
}

/// Installing or removing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    Add,
    Remove,
}

impl Change {
    /// `kubaos app`'s word for it.
    fn word(self) -> &'static str {
        match self {
            Change::Add => "add",
            Change::Remove => "remove",
        }
    }

    fn action(self) -> &'static str {
        match self {
            Change::Add => INSTALL,
            Change::Remove => REMOVE,
        }
    }
}

/// What a running Job's program did: a line it wrote, or its end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heard {
    job: String,
    said: Said,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Said {
    Line(String),
    Ended { succeeded: bool },
}

impl Heard {
    /// The Job `job`'s program wrote `line`, on stdout or stderr.
    #[must_use]
    pub fn line(job: &str, line: impl Into<String>) -> Self {
        Heard {
            job: job.to_owned(),
            said: Said::Line(line.into()),
        }
    }

    /// The Job `job`'s program exited, with success or not.
    #[must_use]
    pub fn ended(job: &str, succeeded: bool) -> Self {
        Heard {
            job: job.to_owned(),
            said: Said::Ended { succeeded },
        }
    }
}

/// Where a Job's program sends what it did, from the threads that follow it: the protocol loop.
#[derive(Clone)]
pub struct Reporter(Arc<dyn Fn(Heard) + Send + Sync>);

impl Reporter {
    #[must_use]
    pub fn new(report: impl Fn(Heard) + Send + Sync + 'static) -> Self {
        Reporter(Arc::new(report))
    }

    fn report(&self, heard: Heard) {
        (self.0)(heard);
    }
}

impl Default for Reporter {
    /// Drops what it hears, for an Extension with no loop to send to.
    fn default() -> Self {
        Reporter::new(|_| {})
    }
}

impl fmt::Debug for Reporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Reporter")
    }
}

/// The install or removal the menu started, while it runs: one at a time, as `kubaos app` holds
/// the update lock while it works.
#[derive(Debug, Default)]
pub struct AppJobs {
    /// How many Jobs have started, which names the next.
    started: u64,
    running: Option<Running>,
    reporter: Reporter,
}

impl AppJobs {
    /// Jobs whose programs report to `reporter`.
    #[must_use]
    pub fn new(reporter: Reporter) -> Self {
        AppJobs {
            reporter,
            ..AppJobs::default()
        }
    }

    /// Starts the Action `action` of the app Item `id` among `apps` in the background, and returns
    /// its Job; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When the Item isn't the half that applies, another app is changing, or `kubaos` can't
    /// start.
    pub fn start(&mut self, apps: &Apps, id: &str, action: &str, runner: &impl Runner) -> Result<Job, String> {
        let (app, change) = apps
            .offered(id)
            .filter(|(_, change)| change.action() == action)
            .ok_or_else(|| format!("`{id}` has no Action `{action}`"))?;
        if let Some(running) = &self.running {
            return Err(format!("{} is still going: one app at a time", running.title));
        }
        let command = format!("kubaos app {} {}", change.word(), app.id);
        let argv: Vec<String> = command.split(' ').chain(["--json"]).map(str::to_owned).collect();
        let job = format!("{JOB_PREFIX}{}", self.started + 1);
        let (lines, ended) = (self.reporter.clone(), self.reporter.clone());
        let (line_job, ended_job) = (job.clone(), job.clone());
        runner
            .background(
                &argv,
                Box::new(move |line| lines.report(Heard::line(&line_job, line))),
                Box::new(move |succeeded| ended.report(Heard::ended(&ended_job, succeeded))),
            )
            .map_err(|err| Chain(&err).to_string())?;
        self.started += 1;
        let running = Running::new(job, app, change, command);
        let shown = running.job();
        self.running = Some(running);
        Ok(shown)
    }

    /// The running Job as `heard` leaves it, when that changes what it shows: always at its end,
    /// after which it is gone. What another Job's program did changes nothing.
    pub fn heard(&mut self, heard: Heard) -> Option<Job> {
        let running = self.running.as_mut().filter(|running| running.id == heard.job)?;
        match heard.said {
            Said::Line(line) => running.hear(&line).then(|| running.job()),
            Said::Ended { succeeded } => self.running.take().map(|running| running.end(succeeded)),
        }
    }
}

/// One Job, as its program's events leave it.
#[derive(Debug)]
struct Running {
    id: String,
    /// "Installing Google Chrome".
    title: String,
    name: String,
    change: Change,
    /// `kubaos app add google-chrome`, which a failure names when the program said nothing.
    command: String,
    step: Option<String>,
    progress: Option<Fraction>,
    log: Option<PathBuf>,
    /// The run's warnings, each said at its end; one alone fails nothing.
    warnings: Vec<String>,
    /// Why the run failed, from its `failed` or `declined` event.
    failure: Option<String>,
    /// The last line that wasn't an event: what the program said before its events started, or
    /// instead of them.
    said: Option<String>,
    /// Whether the summary has the app among those installed or removed, as asked; `None`
    /// before the summary.
    arrived: Option<bool>,
}

impl Running {
    fn new(id: String, app: &App, change: Change, command: String) -> Self {
        let verb = match change {
            Change::Add => "Installing",
            Change::Remove => "Removing",
        };
        Running {
            id,
            title: format!("{verb} {}", app.name),
            name: app.name.clone(),
            change,
            command,
            step: None,
            progress: None,
            log: None,
            warnings: Vec::new(),
            failure: None,
            said: None,
            arrived: None,
        }
    }

    /// Takes in one line of the program's; whether the Job shows anything new.
    fn hear(&mut self, line: &str) -> bool {
        let Ok(event) = UpdateEvent::from_json_line(line) else {
            let line = line.trim();
            if !line.is_empty() {
                self.said = Some(line.to_owned());
            }
            return false;
        };
        let before = (self.step.clone(), self.progress);
        match event {
            UpdateEvent::Started { log, .. } => self.log = log,
            UpdateEvent::PhaseStarted { .. } => self.progress = None,
            UpdateEvent::Step { text } => self.step = Some(text),
            UpdateEvent::Progress { done, total } if total > 0 => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "A count of packages is far below where f32 loses whole numbers."
                )]
                let fraction = done.min(total) as f32 / total as f32;
                self.progress = Fraction::new(fraction).ok();
            }
            UpdateEvent::Warning { text } => self.warnings.push(text),
            UpdateEvent::Summary(summary) => {
                let done = match self.change {
                    Change::Add => &summary.apps.installed,
                    Change::Remove => &summary.apps.removed,
                };
                self.arrived = Some(done.contains(&self.name));
                self.log = summary.log.or(self.log.take());
            }
            UpdateEvent::Failed { error, log, .. } => {
                self.failure = Some(error);
                self.log = log.or(self.log.take());
            }
            UpdateEvent::Declined => self.failure = Some("You said no: nothing changed".to_owned()),
            _ => {}
        }
        (self.step.clone(), self.progress) != before
    }

    /// The Job while it runs.
    fn job(&self) -> Job {
        Job {
            id: self.id.clone(),
            title: self.title.clone(),
            state: JobState::Running,
            progress: self.progress,
            message: self.step.clone(),
        }
    }

    /// The Job at its end, named for what it came to, with why it failed, the warnings and the
    /// log. It failed when the program did, or its summary lacks the app: a warning alone, such as
    /// a log that couldn't be kept, leaves the app installed or removed.
    fn end(self, succeeded: bool) -> Job {
        let log = self.log.as_ref().map(|log| format!("The log: {}", log.display()));
        let failed = !succeeded || self.failure.is_some() || self.arrived != Some(true);
        let (title, state, why) = if failed {
            let not = match self.change {
                Change::Add => "wasn't installed",
                Change::Remove => "wasn't removed",
            };
            let why = self.failure.or_else(|| {
                self.warnings
                    .is_empty()
                    .then(|| self.said.unwrap_or_else(|| format!("{} stopped", self.command)))
            });
            (format!("{} {not}", self.name), JobState::Failed, why)
        } else {
            let done = match self.change {
                Change::Add => "installed",
                Change::Remove => "removed",
            };
            (format!("{} {done}", self.name), JobState::Succeeded, None)
        };
        let message: Vec<String> = why.into_iter().chain(self.warnings).chain(log).collect();
        Job {
            id: self.id,
            title,
            state,
            progress: None,
            message: (!message.is_empty()).then(|| message.join("\n")),
        }
    }
}

/// "Install Google Chrome", for an app that isn't on the machine.
fn install_item(app: &App) -> Item {
    let title = format!("Install {}", app.name);
    item(
        format!("{INSTALL_PREFIX}{}", app.id),
        title.clone(),
        app.summary.clone(),
        app,
        &["install", "add", "get"],
        Action::new(INSTALL, title),
    )
}

/// "Remove Google Chrome", for an app that is, at `version` when dpkg has one.
fn remove_item(app: &App, version: Option<&str>) -> Item {
    let title = format!("Remove {}", app.name);
    let subtitle = version.map_or_else(
        || "Installed".to_owned(),
        |version| format!("Installed, version {version}"),
    );
    item(
        format!("{REMOVE_PREFIX}{}", app.id),
        title.clone(),
        subtitle,
        app,
        &["remove", "uninstall", "delete"],
        Action::new(REMOVE, title).confirm(),
    )
}

/// An app's Item, found by its id and each word of it as well as its name, "app" and `verbs`.
fn item(id: String, title: String, subtitle: String, app: &App, verbs: &[&str], action: Action) -> Item {
    let keywords = std::iter::once(app.id.as_str())
        .chain(app.id.split('-'))
        .chain(["app", "apps"])
        .chain(verbs.iter().copied())
        .map(str::to_owned)
        .collect();
    Item {
        id,
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title,
        subtitle: Some(subtitle),
        icon: Some(app.icon.clone().unwrap_or_else(|| ICON.to_owned())),
        names: Vec::new(),
        keywords,
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        yields: false,
        actions: vec![action],
    }
}

#[cfg(test)]
mod tests;
