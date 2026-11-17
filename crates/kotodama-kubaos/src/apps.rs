//! The Apps catalog in the menu: an "Install <Name>" Item for each app of the catalog that isn't on
//! the machine, a "Remove <Name>" Item for each one KubaOS put there, never both, so a folder added
//! to the catalog brings its Items with it. An app found there, put there by other means, has no
//! Remove Item: KubaOS owns nothing of it, and a removal would leave it as it is. An Install Item's
//! subtitle is the app's summary, and its tags are keywords of it alone: a word that finds an app
//! to install never ranks its removal. An app with setups has an Install Item for each setup
//! instead, named for it, as none is the default: all of them while the app isn't on the machine,
//! and those of the setups it isn't in while it is, which switch. Running one starts `kubaos app
//! add|remove <id> --json` (`add <id> --setup <setup>`) in the background, as a Job the menu shows
//! with each step and how far it got; the Daemon announces its end on the desktop, named for the
//! app, with why it failed and where its log is. Its message is the step it is on, "Start the
//! service, 2 of 4", after the app it is on for a run of several ("Neovim (1 of 2): …"), as an
//! install brings the apps it requires and a removal takes those it brought. An installed app with
//! a configure stage has a "Configure <Name>" Item, which runs `kubaos app configure <id>` the
//! same way.
//!
//! Which apps are on the machine is the engine's status (`kubaos app status`): the catalog, the
//! picks, each app's journals and one `dpkg-query` for the versions and the apps on the machine
//! with no journal, read on a thread of its own at the start, at each config, when one of the
//! files it comes from changes and at a Job's end, never as the menu searches.
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

use std::fmt;
use std::io;
use std::path::Path;
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
use kubaos_core::apps::Catalog;
use kubaos_core::apps::CatalogError;
use kubaos_core::apps::Picks;
use kubaos_core::apps::PicksError;
use kubaos_core::apps::Platform;
use kubaos_core::apps::Status;
use kubaos_core::apps::journal::JournalError;
use kubaos_core::apps::status::AppStatus;
use kubaos_core::apps::status::SetupInfo;
use kubaos_core::apt::Dpkg;
use kubaos_core::apt::dpkg;
use kubaos_core::state::NIXPKGS_FILE;
use kubaos_core::state::Paths;
use kubaos_core::update::event::AppEvent;
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
/// Runs an installed app's configure stage again.
pub const CONFIGURE: &str = "configure";
/// dpkg's database, which every install and removal of a package rewrites.
pub const DPKG_STATUS: &str = "/var/lib/dpkg/status";

/// An app's Install Item is this and its id, `app-install:google-chrome`; a setup's adds `:` and
/// the setup's id, `app-install:t3-code:service`.
const INSTALL_PREFIX: &str = "app-install:";
/// An app's Remove Item is this and its id, `app-remove:google-chrome`.
const REMOVE_PREFIX: &str = "app-remove:";
/// An app's Configure Item is this and its id, `app-configure:lazyvim`.
const CONFIGURE_PREFIX: &str = "app-configure:";
/// A Job is this and a count, `app-1`.
const JOB_PREFIX: &str = "app-";
/// The icon of an app whose folder names none.
const ICON: &str = "system-software-install";

/// What starts an Item's icon that is a KubaOS Symbol, not an icon theme's name or a path: the
/// menu's row draws the rest as its tile's symbol.
const SYMBOL_PREFIX: &str = "symbol:";

/// Where the apps' status is read from.
#[derive(Clone, Debug)]
pub struct AppFiles {
    catalog: PathBuf,
    picks: PathBuf,
    /// The apps' user journals.
    journals: PathBuf,
    /// Root's journals of the apps, which say whether an app KubaOS took in owns anything.
    root_journals: PathBuf,
    /// The User flake's state directory, whose `nixpkgs.toml` says whether an app with unfree
    /// nixpkgs packages can be installed.
    state_dir: PathBuf,
}

impl AppFiles {
    /// The installed release's catalog `system` has, its root journals, and the picks and the
    /// apps' journals of `paths`.
    #[must_use]
    pub fn new(system: &SystemPaths, paths: &Paths) -> Self {
        AppFiles {
            catalog: system.catalog(),
            picks: paths.app_picks(),
            journals: paths.app_journals(),
            root_journals: system.app_journals(),
            state_dir: paths.state_dir().root().to_path_buf(),
        }
    }

    /// The files of this machine and user; none without a home.
    pub fn from_env() -> Option<Self> {
        let paths = Paths::from_env().ok()?;
        Some(Self::new(&SystemPaths::from_env(), &paths))
    }

    /// The apps' user journals' folder, which the watch needs to exist: an app's journal folder
    /// appears in it with the app's first install, after the crossing update too.
    pub fn journals(&self) -> &Path {
        &self.journals
    }

    /// These files with their folders' symlinks followed, as the watch's events name them.
    #[must_use]
    pub fn canonical(&self) -> Self {
        let real = |path: &Path| match (path.parent(), path.file_name()) {
            (Some(dir), Some(name)) => dir
                .canonicalize()
                .map_or_else(|_| path.to_path_buf(), |dir| dir.join(name)),
            _ => path.to_path_buf(),
        };
        AppFiles {
            catalog: self.catalog.clone(),
            picks: real(&self.picks),
            journals: self.journals.canonicalize().unwrap_or_else(|_| self.journals.clone()),
            root_journals: self.root_journals.clone(),
            state_dir: self.state_dir.canonicalize().unwrap_or_else(|_| self.state_dir.clone()),
        }
    }

    /// The folders whose change can change which apps are on the machine, picked or available:
    /// dpkg's database's, which has the versions, the picks', the User flake's state directory,
    /// whose `nixpkgs.toml` lets unfree packages in or not, the journals' folder, where an app's
    /// folder comes and goes with its install and removal, and each app's folder in it, whose
    /// journal its install and removal rename into place. Read again after each change, so an
    /// app's folder made since is watched.
    pub fn watched_dirs(&self) -> Vec<PathBuf> {
        let parent = |path: &Path| path.parent().map(Path::to_path_buf);
        let mut dirs: Vec<PathBuf> = [
            parent(Path::new(DPKG_STATUS)),
            parent(&self.picks),
            Some(self.state_dir.clone()),
            Some(self.journals.clone()),
        ]
        .into_iter()
        .flatten()
        .collect();
        if let Ok(entries) = std::fs::read_dir(&self.journals) {
            let mut apps: Vec<PathBuf> = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect();
            apps.sort();
            dirs.extend(apps);
        }
        dirs.dedup();
        dirs
    }

    /// Whether a change to `path` can change which apps are on the machine, picked or available:
    /// dpkg's database, the picks, `nixpkgs.toml`, or anything in the journals' folder but its
    /// lock.
    pub fn changes(&self, path: &Path) -> bool {
        path == Path::new(DPKG_STATUS)
            || path == self.picks
            || path == self.state_dir.join(NIXPKGS_FILE)
            || (path.starts_with(&self.journals) && path != self.journals.join(LOCK))
    }
}

/// The user side's lock in the journals' folder, which every `kubaos app` run takes.
const LOCK: &str = ".lock";

/// The apps' status could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("reading the Apps catalog")]
    Catalog(#[source] CatalogError),
    #[error("reading the apps you picked")]
    Picks(#[source] PicksError),
    #[error("asking dpkg which packages are installed")]
    Dpkg(#[source] RunError),
    #[error("reading the apps' journals")]
    Journal(#[source] JournalError),
}

/// The apps of the catalog, each with where it stands on the machine.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Apps {
    /// Sorted by id, as the catalog is.
    apps: Vec<Standing>,
    /// The machine's architecture, as dpkg names it.
    arch: String,
}

/// Where one app stands.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Standing {
    app: AppStatus,
}

impl Standing {
    /// The setup it is installed in.
    fn installed_setup(&self) -> Option<&SetupInfo> {
        self.app.setup_info(self.app.setup.as_deref()?)
    }

    /// Whether the Install Item of `setup`, or of the app without setups, applies: the app isn't
    /// on the machine and has something for it, or it is there in another setup.
    fn offers_install(&self, setup: Option<&SetupInfo>) -> bool {
        let id = setup.map(|s| s.id.as_str());
        if self.app.state.is_installed() {
            return setup
                .is_some_and(|setup| self.app.setup.as_deref() != Some(setup.id.as_str()) && self.app.offers(id));
        }
        self.app.offers(id)
    }

    fn has_setups(&self) -> bool {
        !self.app.setups.is_empty()
    }
}

impl Apps {
    /// The apps as `status` has them.
    #[must_use]
    pub fn new(status: &Status) -> Self {
        Apps {
            apps: status.apps.iter().map(|app| Standing { app: app.clone() }).collect(),
            arch: status.arch.clone(),
        }
    }

    /// Reads the catalog, the picks, each app's user journal and dpkg's packages with one
    /// `dpkg-query`; only the catalog when it has no app.
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
        let status = Status::read(
            &catalog,
            &picks,
            &files.journals,
            &files.root_journals,
            Platform::this_machine(&files.state_dir),
            Some(&dpkg),
        )
        .map_err(ReadError::Journal)?;
        Ok(Self::new(&status))
    }

    /// The Items of each app: Remove while it is on the machine, Install while it isn't, or for
    /// an app with setups one Install a setup it isn't in. An app with nothing for this machine's
    /// architecture has none, and a setup with nothing for it neither.
    pub fn items(&self) -> Vec<Item> {
        let mut items = Vec::new();
        for standing in &self.apps {
            if standing.app.state.is_removable() {
                items.push(remove_item(standing));
            }
            if standing.app.configurable {
                items.push(configure_item(&standing.app));
            }
            if standing.has_setups() {
                for setup in &standing.app.setups {
                    if standing.offers_install(Some(setup)) {
                        items.push(setup_item(standing, setup));
                    }
                }
            } else if standing.offers_install(None) {
                items.push(install_item(&standing.app));
            }
        }
        items
    }

    /// The id of every Item an app of the catalog may have, its Install, Remove and Configure,
    /// which `hide` may name.
    pub fn ids(&self) -> Vec<String> {
        self.apps
            .iter()
            .flat_map(|standing| {
                let app = &standing.app;
                let installs: Vec<String> = if standing.has_setups() {
                    app.setups
                        .iter()
                        .map(|setup| format!("{INSTALL_PREFIX}{}:{}", app.id, setup.id))
                        .collect()
                } else {
                    vec![format!("{INSTALL_PREFIX}{}", app.id)]
                };
                installs.into_iter().chain([
                    format!("{REMOVE_PREFIX}{}", app.id),
                    format!("{CONFIGURE_PREFIX}{}", app.id),
                ])
            })
            .collect()
    }

    /// The app of the Item `id`, what the Item does, and the setup it installs, while the Item
    /// applies.
    fn offered(&self, id: &str) -> Option<(&AppStatus, Change, Option<&SetupInfo>)> {
        let (change, rest) = if let Some(rest) = id.strip_prefix(INSTALL_PREFIX) {
            (Change::Add, rest)
        } else if let Some(rest) = id.strip_prefix(REMOVE_PREFIX) {
            (Change::Remove, rest)
        } else if let Some(rest) = id.strip_prefix(CONFIGURE_PREFIX) {
            (Change::Configure, rest)
        } else {
            return None;
        };
        let (app_id, setup_id) = match (change, rest.split_once(':')) {
            (Change::Add, Some((app_id, setup_id))) => (app_id, Some(setup_id)),
            (_, None) => (rest, None),
            (Change::Remove | Change::Configure, Some(_)) => return None,
        };
        let standing = self.apps.iter().find(|standing| standing.app.id == app_id)?;
        let app = &standing.app;
        if change == Change::Add && standing.has_setups() != setup_id.is_some() {
            return None;
        }
        let setup = match setup_id {
            Some(setup_id) => Some(app.setup_info(setup_id)?),
            None => None,
        };
        let applies = match change {
            Change::Add => standing.offers_install(setup),
            Change::Remove => app.state.is_removable(),
            Change::Configure => app.configurable,
        };
        applies.then_some((app, change, setup))
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

/// Installing, removing or configuring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    Add,
    Remove,
    Configure,
}

impl Change {
    /// `kubaos app`'s word for it.
    fn word(self) -> &'static str {
        match self {
            Change::Add => "add",
            Change::Remove => "remove",
            Change::Configure => "configure",
        }
    }

    fn action(self) -> &'static str {
        match self {
            Change::Add => INSTALL,
            Change::Remove => REMOVE,
            Change::Configure => CONFIGURE,
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
        let (app, change, setup) = apps
            .offered(id)
            .filter(|(_, change, _)| change.action() == action)
            .ok_or_else(|| format!("`{id}` has no Action `{action}`"))?;
        if let Some(running) = &self.running {
            return Err(format!("{} is still going: one app at a time", running.title));
        }
        let command = match setup {
            Some(setup) => format!("kubaos app {} {} --setup {}", change.word(), app.id, setup.id),
            None => format!("kubaos app {} {}", change.word(), app.id),
        };
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
        let name = setup.map_or(&app.name, |setup| &setup.name);
        let running = Running::new(job, name, change, command);
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
    /// The app's name, or its setup's for a setup's install, as the summary lists it.
    name: String,
    change: Change,
    /// `kubaos app add google-chrome`, which a failure names when the program said nothing.
    command: String,
    /// The app of the run it is on, for a run of more than one.
    app: Option<AppEvent>,
    /// The step it is on.
    step: Option<String>,
    /// The step's place among the app's, from 1, and how many there are.
    count: Option<(usize, usize)>,
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
    /// Whether the summary has the app among those a removal left, as KubaOS didn't install it.
    left: bool,
}

impl Running {
    fn new(id: String, name: &str, change: Change, command: String) -> Self {
        let title = match change {
            Change::Add => format!("Installing {name}"),
            Change::Remove => format!("Removing {name}"),
            Change::Configure => format!("Configuring {name}"),
        };
        Running {
            id,
            title,
            name: name.to_owned(),
            change,
            command,
            app: None,
            step: None,
            count: None,
            progress: None,
            log: None,
            warnings: Vec::new(),
            failure: None,
            said: None,
            arrived: None,
            left: false,
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
        let before = (self.message(), self.progress);
        match event {
            UpdateEvent::Started { log, .. } => self.log = log,
            UpdateEvent::PhaseStarted { .. } => self.progress = None,
            UpdateEvent::App(app) => {
                // The bar stays indeterminate until the first step's progress, the password
                // prompt included; from then on, a new app starts at its share of the run.
                if self.progress.is_some() {
                    self.progress = Fraction::new(share(app.n.saturating_sub(1), app.of)).ok();
                }
                (self.app, self.step, self.count) = (Some(app), None, None);
            }
            UpdateEvent::Step { text } => (self.step, self.count) = (Some(text), None),
            UpdateEvent::Progress { done, total } if total > 0 => {
                let done = done.min(total);
                self.count = Some(((done + 1).min(total), total));
                let fraction = match &self.app {
                    Some(app) => share(app.n.saturating_sub(1), app.of) + share(done, total * app.of),
                    _ => share(done, total),
                };
                self.progress = Fraction::new(fraction).ok();
            }
            UpdateEvent::Warning { text } => self.warnings.push(text),
            UpdateEvent::Summary(summary) => {
                let done = match self.change {
                    Change::Add => &summary.apps.installed,
                    Change::Remove => &summary.apps.removed,
                    Change::Configure => &summary.apps.configured,
                };
                self.arrived = Some(done.contains(&self.name));
                self.left = self.change == Change::Remove && summary.apps.left.contains(&self.name);
                self.log = summary.log.or(self.log.take());
            }
            UpdateEvent::Failed { error, log, .. } => {
                self.failure = Some(error);
                self.log = log.or(self.log.take());
            }
            UpdateEvent::Declined => self.failure = Some("You said no: nothing changed".to_owned()),
            _ => {}
        }
        (self.message(), self.progress) != before
    }

    /// What the Job says while it runs: the step and its place among the app's, "Start the
    /// service, 2 of 4", after the app and its place in the run when the run has more than one,
    /// "Neovim (1 of 2): Install Neovim, 1 of 3".
    fn message(&self) -> Option<String> {
        let step = match (&self.step, self.count) {
            (Some(step), Some((n, of))) => Some(format!("{step}, {n} of {of}")),
            (Some(step), None) => Some(step.clone()),
            (None, _) => None,
        };
        match (self.app.as_ref().filter(|app| app.of > 1), step) {
            (Some(app), Some(step)) => Some(format!("{} ({} of {}): {step}", app.name, app.n, app.of)),
            (Some(app), None) => Some(format!("{} ({} of {})", app.name, app.n, app.of)),
            (None, step) => step,
        }
    }

    /// The Job while it runs.
    fn job(&self) -> Job {
        Job {
            id: self.id.clone(),
            title: self.title.clone(),
            state: JobState::Running,
            progress: self.progress,
            message: self.message(),
        }
    }

    /// The Job at its end, named for what it came to, with why it failed, the warnings and the
    /// log. It failed when the program did, or its summary lacks the app: a warning alone, such as
    /// a log that couldn't be kept, leaves the app installed or removed.
    fn end(self, succeeded: bool) -> Job {
        let log = self.log.as_ref().map(|log| format!("The log: {}", log.display()));
        let left = succeeded && self.failure.is_none() && self.left;
        let failed = !succeeded || self.failure.is_some() || (self.arrived != Some(true) && !left);
        let (title, state, why) = if left {
            (
                format!("{} left as it is", self.name),
                JobState::Succeeded,
                Some(kubaos_core::update::event::left_line(&self.name)),
            )
        } else if failed {
            let not = match self.change {
                Change::Add => "wasn't installed",
                Change::Remove => "wasn't removed",
                Change::Configure => "wasn't configured",
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
                Change::Configure => "configured",
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

/// "Install Google Chrome", for an app that isn't on the machine: its summary, and its tags among
/// its keywords.
fn install_item(app: &AppStatus) -> Item {
    let title = format!("Install {}", app.name);
    item(
        format!("{INSTALL_PREFIX}{}", app.id),
        title.clone(),
        app.summary.clone(),
        app,
        &install_words(app),
        Action::new(INSTALL, title),
    )
}

/// The words an Install Item is found by beyond the app's id: the verbs, then its tags.
fn install_words(app: &AppStatus) -> Vec<&str> {
    ["install", "add", "get"]
        .into_iter()
        .chain(app.tags.iter().map(String::as_str))
        .collect()
}

/// "Install T3 Code as a service", for a setup of an app the machine doesn't have in it: the
/// app's summary, as the title names the setup, and which setup it replaces while the app is there
/// in another.
fn setup_item(standing: &Standing, setup: &SetupInfo) -> Item {
    let app = &standing.app;
    let title = format!("Install {}", setup.name);
    let subtitle = match standing.installed_setup() {
        Some(installed) => format!("Replaces {}: {}", installed.name, app.summary),
        None => app.summary.clone(),
    };
    item(
        format!("{INSTALL_PREFIX}{}:{}", app.id, setup.id),
        title.clone(),
        subtitle,
        app,
        &install_words(app),
        Action::new(INSTALL, title),
    )
}

/// "Remove Google Chrome", for an app that is, at its version when dpkg has one, in its setup
/// when it has setups.
fn remove_item(standing: &Standing) -> Item {
    let app = &standing.app;
    let title = format!("Remove {}", app.name);
    let version = app.version.as_deref();
    let subtitle = match (standing.installed_setup(), version) {
        (Some(setup), Some(version)) => format!("Installed: {}, version {version}", setup.name),
        (Some(setup), None) => format!("Installed: {}", setup.name),
        (None, Some(version)) => format!("Installed, version {version}"),
        (None, None) => "Installed".to_owned(),
    };
    item(
        format!("{REMOVE_PREFIX}{}", app.id),
        title.clone(),
        subtitle,
        app,
        &["remove", "uninstall", "delete"],
        Action::new(REMOVE, title).confirm(),
    )
}

/// "Configure Penpot", for an installed app with a configure stage: KubaOS's defaults for it
/// again.
fn configure_item(app: &AppStatus) -> Item {
    let title = format!("Configure {}", app.name);
    item(
        format!("{CONFIGURE_PREFIX}{}", app.id),
        title.clone(),
        format!("Put KubaOS's defaults for {} back", app.name),
        app,
        &["configure", "settings", "defaults", "reset"],
        Action::new(CONFIGURE, title),
    )
}

/// `n` over `of`, as a fraction of a Job's bar.
#[expect(
    clippy::cast_precision_loss,
    reason = "A count of steps or apps is far below where f32 loses whole numbers."
)]
fn share(n: usize, of: usize) -> f32 {
    if of == 0 { 0.0 } else { n as f32 / of as f32 }
}

/// An app's Item's icon: its picture, else its symbol as `symbol:<name>`, which the menu's row
/// draws as a KubaOS Symbol, else the Extension's own.
fn item_icon(app: &AppStatus) -> String {
    match (&app.icon, &app.symbol) {
        (Some(icon), _) => icon.clone(),
        (None, Some(symbol)) => format!("{SYMBOL_PREFIX}{symbol}"),
        (None, None) => ICON.to_owned(),
    }
}

/// An app's Item, found by its id and each word of it as well as its name, "app" and `words`.
fn item(id: String, title: String, subtitle: String, app: &AppStatus, words: &[&str], action: Action) -> Item {
    let keywords = std::iter::once(app.id.as_str())
        .chain(app.id.split('-'))
        .chain(["app", "apps"])
        .chain(words.iter().copied())
        .map(str::to_owned)
        .collect();
    Item {
        id,
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title,
        subtitle: Some(subtitle),
        icon: Some(item_icon(app)),
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
