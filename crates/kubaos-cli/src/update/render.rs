//! What `kubaos update`, `upgrade`, `rollback`, `channel` and `packages enable|disable` show: the
//! updater's events (`kubaos_core::update::event`) as a live view on a terminal, as plain lines
//! elsewhere, or as JSON lines with `--json`.
//!
//! The live view keeps one area at the bottom of the terminal: the running phase with a spinner,
//! and under it the last line its program printed. Everything else (a finished phase, a warning,
//! the preview, the summary) is printed above it and stays. The area is cleared before a prompt,
//! so sudo's password prompt and a question have the line to themselves. Colours are the
//! terminal's own 16, so its theme applies, and `NO_COLOR` turns them off.
//!
//! # Owns
//!
//! - [`Mode`], picking how to show the events, and [`Ui`], the sink that shows them.
//!
//! # Not here
//!
//! - The events and when they come: `kubaos_core::update`.

use std::fmt::Write as _;
use std::io;
use std::io::IsTerminal;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use kubaos_core::update::changelog::Change;
use kubaos_core::update::changelog::Changelog;
use kubaos_core::update::changelog::Section;
use kubaos_core::update::event::Command;
use kubaos_core::update::event::GroupChange;
use kubaos_core::update::event::Outcome;
use kubaos_core::update::event::PackageChange;
use kubaos_core::update::event::Phase;
use kubaos_core::update::event::Preview;
use kubaos_core::update::event::ProgramChange;
use kubaos_core::update::event::ProgramResult;
use kubaos_core::update::event::Prompt;
use kubaos_core::update::event::Restart;
use kubaos_core::update::event::Sink;
use kubaos_core::update::event::SuiteChanges;
use kubaos_core::update::event::SuiteTool;
use kubaos_core::update::event::Summary;
use kubaos_core::update::event::UpdateEvent;
use kubaos_core::update::firmware::Firmware;
use kubaos_core::update::firmware::FirmwareOutcome;

/// How often the spinner turns.
const TICK: Duration = Duration::from_millis(80);

/// The spinner's frames.
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// The width a phase's name is padded to: the longest, "Reviewing changes", and a space.
const TITLE_WIDTH: usize = 18;

/// The width a phase's time is right-aligned to: the longest, "59m59s", so a space always parts
/// it from the name.
const TIME_WIDTH: usize = 6;

/// The width of the summary's labels: the longest, "Nix packages", and two spaces.
const LABEL_WIDTH: usize = 14;

/// How many of the release's changes the summary names.
const CHANGES_SHOWN: usize = 5;

/// How many upgraded apt packages the summary names on its line, and the preview one a line; the
/// log has them all.
const APT_SHOWN: usize = 8;

/// The width of the preview's section labels: the longest, "Breaking", and two spaces.
const SECTION_WIDTH: usize = 10;

/// The widest a change's subject is padded to, so the pull requests after it line up; a longer
/// subject pushes its own number out rather than every line's.
const SUBJECT_WIDTH: usize = 60;

/// How many of the home's changed Nix packages the summary names, one a line.
const NIX_SHOWN: usize = 6;

/// The width the live view assumes when the terminal doesn't say.
const DEFAULT_WIDTH: usize = 80;

/// The fewest output lines between two redraws of the live area; the spinner's tick catches up.
const REDRAW_EVERY: Duration = Duration::from_millis(40);

/// How the events are shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// One JSON event per line on stdout.
    Json,
    /// The live view, on a terminal.
    Live,
    /// Plain lines, without colour or redraws: a pipe, a systemd unit, a dumb terminal.
    Plain,
}

impl Mode {
    /// JSON when asked, the live view when stdout is a terminal that can redraw, else plain.
    pub fn detect(json: bool) -> Self {
        let dumb = std::env::var_os("TERM").is_some_and(|term| term == "dumb");
        if json {
            Self::Json
        } else if io::stdout().is_terminal() && !dumb {
            Self::Live
        } else {
            Self::Plain
        }
    }
}

/// The sink that shows the events in one [`Mode`].
#[derive(Debug)]
pub enum Ui {
    Json(Json<io::Stdout>),
    Live(LiveView),
    Plain(Plain<io::Stdout, io::Stderr>),
}

impl Ui {
    /// The sink for `mode`, on stdout.
    pub fn new(mode: Mode) -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        match mode {
            Mode::Json => Self::Json(Json::new(io::stdout())),
            Mode::Live => {
                let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
                let style = if no_color { Style::PLAIN } else { Style::COLOR };
                Self::Live(LiveView::start(Text::new(style, home)))
            }
            Mode::Plain => Self::Plain(Plain::new(io::stdout(), io::stderr(), Text::new(Style::PLAIN, home))),
        }
    }
}

impl Sink for Ui {
    fn event(&mut self, event: &UpdateEvent) {
        match self {
            Self::Json(json) => json.event(event),
            Self::Live(live) => live.event(event),
            Self::Plain(plain) => plain.event(event),
        }
    }
}

/// ANSI colours, or none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    color: bool,
}

impl Style {
    pub const PLAIN: Self = Self { color: false };
    pub const COLOR: Self = Self { color: true };

    fn paint(self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_owned()
        }
    }

    fn bold(self, text: &str) -> String {
        self.paint("1", text)
    }

    fn dim(self, text: &str) -> String {
        self.paint("2", text)
    }

    fn red(self, text: &str) -> String {
        self.paint("31", text)
    }

    fn green(self, text: &str) -> String {
        self.paint("32", text)
    }

    fn yellow(self, text: &str) -> String {
        self.paint("33", text)
    }

    fn cyan(self, text: &str) -> String {
        self.paint("36", text)
    }

    fn bold_yellow(self, text: &str) -> String {
        self.paint("1;33", text)
    }

    /// `text` as an OSC 8 hyperlink to `url`, which terminals that know it open on a click and
    /// others print as `text`; only with colour, since a pipe or a log would keep the escape.
    fn link(self, url: &str, text: &str) -> String {
        if self.color {
            format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
        } else {
            text.to_owned()
        }
    }
}

/// The lines every text view prints the same way.
#[derive(Debug)]
pub struct Text {
    style: Style,
    /// The home, shown as `~` in paths.
    home: Option<PathBuf>,
}

impl Text {
    pub fn new(style: Style, home: Option<PathBuf>) -> Self {
        Self { style, home }
    }

    /// The lines `event` leaves on screen, each ending in a newline; none for the events each
    /// view shows its own way.
    fn lines(&self, event: &UpdateEvent) -> Option<String> {
        let s = self.style;
        Some(match event {
            UpdateEvent::Started { command, .. } => format!("{}\n\n", s.bold(&heading(*command))),
            UpdateEvent::PhaseFinished {
                phase,
                outcome,
                elapsed_ms,
                detail,
            } => self.finished(*phase, *outcome, *elapsed_ms, detail.as_deref()),
            UpdateEvent::Note { text } => format!("    {text}\n"),
            UpdateEvent::Warning { text } => format!("    {} {}\n", s.yellow("!"), s.dim(text)),
            UpdateEvent::Preview(preview) => self.preview(preview),
            UpdateEvent::Available {
                update,
                upgrade,
                apt_security,
            } => available(update.as_deref(), upgrade.as_deref(), *apt_security),
            UpdateEvent::Summary(summary) => self.summary(summary),
            UpdateEvent::Declined => "\n  Nothing changed.\n".to_owned(),
            UpdateEvent::Failed { error, output, log, .. } => self.failed(error, output, log.as_deref()),
            UpdateEvent::PhaseStarted { .. }
            | UpdateEvent::Step { .. }
            | UpdateEvent::Progress { .. }
            | UpdateEvent::Output { .. }
            | UpdateEvent::Prompt { .. }
            | UpdateEvent::Answered { .. } => return None,
        })
    }

    fn finished(&self, phase: Phase, outcome: Outcome, elapsed_ms: u64, detail: Option<&str>) -> String {
        let s = self.style;
        let title = format!("{:<TITLE_WIDTH$}", phase.title());
        let (glyph, title, detail) = match outcome {
            Outcome::Done => (s.green("✓"), title, detail.map(str::to_owned)),
            Outcome::Failed => (s.red("✗"), s.bold(&title), detail.map(str::to_owned)),
            Outcome::Skipped => (
                s.dim("–"),
                s.dim(&title),
                Some(format!("skipped: {}", detail.unwrap_or("nothing to do"))),
            ),
        };
        let detail = detail.map(|d| format!("  {}", s.dim(&d))).unwrap_or_default();
        format!(
            "  {glyph} {title}{:>TIME_WIDTH$}{detail}\n",
            finished_duration(elapsed_ms)
        )
    }

    fn preview(&self, preview: &Preview) -> String {
        let s = self.style;
        let log = &preview.changelog;
        let mut out = String::from("\n");
        match &preview.to {
            Some(to) => {
                let release = format!(
                    "{} {} → {}, {}",
                    s.bold("KubaOS"),
                    preview.from.describe(),
                    s.bold(&to.describe()),
                    preview.channel
                );
                match log.count() {
                    0 => {
                        let _ = writeln!(out, "  {release}");
                    }
                    n => {
                        let _ = writeln!(out, "  {release} · {}", count(n, "change"));
                    }
                }
            }
            None => {
                let _ = writeln!(
                    out,
                    "  {} stays at {}, {}: packages only",
                    s.bold("KubaOS"),
                    preview.from.describe(),
                    preview.channel
                );
            }
        }
        out.push_str(&self.changelog(log));
        out.push_str(&self.apt_upgrades(preview.packages, &preview.upgrades));
        if !preview.apt.is_empty() {
            out.push_str("  New apt packages to install:\n");
            for change in &preview.apt {
                let _ = writeln!(out, "    {}: {}", change.group, change.installed.join(", "));
            }
        }
        if preview.hyprland_pin {
            out.push_str("  The Hyprland pin changes: Hyprland is installed at the release's versions.\n");
        }
        if !preview.migrations.is_empty() {
            let _ = writeln!(out, "  System migrations to run: {}", preview.migrations.join(", "));
        }
        out.push('\n');
        out
    }

    /// The preview's changes, under the section labels: the breaking ones first, highlighted,
    /// then each listed section in order, then how many more and how many internal ones.
    fn changelog(&self, log: &Changelog) -> String {
        let s = self.style;
        let rows = listed(log);
        let scope_width = rows
            .iter()
            .filter_map(|(_, change)| change.scope.as_ref())
            .map(|scope| scope.chars().count() + 2)
            .max()
            .unwrap_or(0);
        let subject_width = rows
            .iter()
            .map(|(_, change)| change.subject.chars().count())
            .max()
            .unwrap_or(0)
            .min(SUBJECT_WIDTH);
        let mut out = String::new();
        let mut last = None;
        for (section, change) in &rows {
            let label = if last == Some(*section) {
                String::new()
            } else {
                section.map_or("Breaking", Section::title).to_owned()
            };
            last = Some(*section);
            let label = format!("{label:<SECTION_WIDTH$}");
            let scope = format!("{:<scope_width$}", change.scope.as_deref().unwrap_or(""));
            let pr = change.pr.map(|pr| self.pr(log, pr));
            let subject = match pr {
                Some(_) => format!("{:<subject_width$}", change.subject),
                None => change.subject.clone(),
            };
            let (label, scope, subject) = match section {
                None => (s.bold_yellow(&label), scope, s.bold(&subject)),
                Some(Section::New | Section::Fixed | Section::Faster) => (s.bold(&label), scope, subject),
                Some(_) => (s.dim(&label), s.dim(&scope), s.dim(&subject)),
            };
            let pr = pr.map(|pr| format!("  {pr}")).unwrap_or_default();
            let _ = writeln!(out, "    {label}{scope}{subject}{pr}");
        }
        let indent = " ".repeat(4 + SECTION_WIDTH);
        if log.more_changes > 0 {
            let _ = writeln!(out, "{indent}{}", s.dim(&format!("+{} more", log.more_changes)));
        }
        if log.internal_changes > 0 {
            let _ = writeln!(out, "{indent}{}", s.dim(&format!("+{} internal", log.internal_changes)));
        }
        out
    }

    /// `#12`, dim, linked to the pull request when the repository is known.
    fn pr(&self, log: &Changelog, pr: u32) -> String {
        let s = self.style;
        let text = s.dim(&format!("#{pr}"));
        match log.pr_url(pr) {
            Some(url) => s.link(&url, &text),
            None => text,
        }
    }

    /// The apt packages the preview's update will upgrade: their count, then one a line with
    /// their versions, [`APT_SHOWN`] at most and how many more.
    fn apt_upgrades(&self, packages: usize, upgrades: &[PackageChange]) -> String {
        let s = self.style;
        let mut out = String::new();
        if upgrades.is_empty() {
            let _ = writeln!(out, "  {} to upgrade", count(packages, "apt package"));
            return out;
        }
        let _ = writeln!(out, "  {} to upgrade:", count(packages, "apt package"));
        for package in upgrades.iter().take(APT_SHOWN) {
            let _ = writeln!(out, "    {}", package.describe());
        }
        let more = upgrades.len().saturating_sub(APT_SHOWN);
        if more > 0 {
            let _ = writeln!(out, "    {}", s.dim(&format!("and {more} more")));
        }
        out
    }

    fn summary(&self, summary: &Summary) -> String {
        let s = self.style;
        let row = |label: &str, value: &str| format!("  {}{value}\n", s.dim(&format!("{label:<LABEL_WIDTH$}")));
        let mut out = String::from("\n");
        if summary.command == Command::Packages {
            out.push_str(&self.apt_rows(&summary.apt, "nothing changed"));
            if let Some(log) = &summary.log {
                out.push_str(&row("Log", &self.path(log)));
            }
            return out;
        }
        if summary.changed_nothing() {
            return format!(
                "\n  Nothing changed: KubaOS {}, {}, and its packages are up to date.\n",
                summary.to.describe(),
                summary.channel
            );
        }
        let moved = summary.from.commit != summary.to.commit;
        let release = if moved {
            format!(
                "{} → {}, {}",
                summary.from.describe(),
                s.bold(&summary.to.describe()),
                summary.channel
            )
        } else {
            format!("unchanged at {}, {}", summary.to.describe(), summary.channel)
        };
        let label = match (summary.command, moved) {
            (_, false) => "KubaOS",
            (Command::Upgrade, true) => "Upgraded",
            (Command::Rollback, true) => "Rolled back",
            (Command::Update | Command::Check | Command::Packages, true) => "Updated",
        };
        out.push_str(&row(label, &release));
        out.push_str(&self.change_rows(&summary.changelog));
        if summary.command != Command::Rollback {
            out.push_str(&row("Apt", &upgraded(summary.packages, &summary.upgraded)));
            if !summary.apt.is_empty() {
                let indent = " ".repeat(LABEL_WIDTH + 2);
                for line in group_lines(&summary.apt) {
                    let _ = writeln!(out, "{indent}{line}");
                }
            }
        }
        if let Some(nix) = &summary.nix_packages {
            if nix.is_empty() {
                out.push_str(&row("Nix packages", "none"));
            } else {
                let lines = nix.iter().map(PackageChange::describe);
                out.push_str(&self.rows("Nix packages", lines, nix.len(), NIX_SHOWN));
            }
        }
        if !summary.inputs.is_empty() {
            out.push_str(&row("Nix inputs", &summary.inputs.join(", ")));
        }
        for program in &summary.programs {
            out.push_str(&row(program.program.title(), &self.program(program)));
        }
        out.push_str(&self.suite_rows(&summary.suites));
        if !summary.firmware.is_empty() {
            out.push_str(&self.firmware_rows(&summary.firmware));
        }
        if !summary.migrations.is_empty() {
            out.push_str(&row("Migrations", &summary.migrations.join(", ")));
        }
        let session = if summary.reloaded.is_empty() {
            "nothing to reload".to_owned()
        } else {
            summary.reloaded.join(", ")
        };
        out.push_str(&row("Session", &session));
        let reason = summary.restart_reason.as_deref().unwrap_or("");
        let next = match summary.restart {
            Restart::None => "nothing, it's all running".to_owned(),
            Restart::Relogin => s.yellow(&format!("log out and back in: {reason}")),
            Restart::Reboot => s.yellow(&format!("reboot: {reason}")),
        };
        out.push_str(&row("Next", &next));
        if let Some(undo) = &summary.undo {
            out.push_str(&row("Undo", undo));
        }
        if let Some(log) = &summary.log {
            out.push_str(&row("Log", &self.path(log)));
        }
        out
    }

    /// The summary's `Changes` rows: the first [`CHANGES_SHOWN`] changes in the preview's order,
    /// each `scope: subject` and its pull request, and how many more; none without a change.
    fn change_rows(&self, log: &Changelog) -> String {
        let s = self.style;
        if log.count() == 0 {
            return String::new();
        }
        let rows: Vec<_> = listed(log)
            .into_iter()
            .take(CHANGES_SHOWN)
            .map(|(section, change)| {
                let text = match &change.scope {
                    Some(scope) => format!("{scope}: {}", change.subject),
                    None => change.subject.clone(),
                };
                (section, change.pr, text)
            })
            .collect();
        let width = rows
            .iter()
            .map(|(_, _, text)| text.chars().count())
            .max()
            .unwrap_or(0)
            .min(SUBJECT_WIDTH);
        let lines = rows.into_iter().map(|(section, pr, text)| {
            let text = match pr {
                Some(_) => format!("{text:<width$}"),
                None => text,
            };
            let text = if section.is_none() { s.bold(&text) } else { text };
            match pr {
                Some(pr) => format!("{text}  {}", self.pr(log, pr)),
                None => text,
            }
        });
        self.rows("Changes", lines, log.count(), CHANGES_SHOWN)
    }

    /// The summary's `Apt` rows: one line a group the package manifest's phase changed, or
    /// `none` when it changed nothing.
    fn apt_rows(&self, apt: &[GroupChange], none: &str) -> String {
        let mut lines = group_lines(apt);
        let first = lines.next().unwrap_or_else(|| none.to_owned());
        let n = apt.len().max(1);
        self.rows("Apt", std::iter::once(first).chain(lines), n, n)
    }

    /// The row `label` with the first of `lines` on it and the rest under it, `shown` of the
    /// `total` at most and how many more after them.
    fn rows<'l>(
        &self,
        label: &str,
        lines: impl IntoIterator<Item = impl AsRef<str> + 'l>,
        total: usize,
        shown: usize,
    ) -> String {
        let s = self.style;
        let indent = " ".repeat(LABEL_WIDTH + 2);
        let mut out = String::new();
        for (i, line) in lines.into_iter().take(shown).enumerate() {
            if i == 0 {
                let _ = writeln!(out, "  {}{}", s.dim(&format!("{label:<LABEL_WIDTH$}")), line.as_ref());
            } else {
                let _ = writeln!(out, "{indent}{}", line.as_ref());
            }
        }
        let more = total.saturating_sub(shown);
        if more > 0 {
            let _ = writeln!(out, "{indent}{}", s.dim(&format!("and {more} more")));
        }
        out
    }

    /// The summary's `Suites` rows: the tools moved or put back, or the failure; then the
    /// `Upgrades` waiting and the command that installs them.
    fn suite_rows(&self, suites: &SuiteChanges) -> String {
        let s = self.style;
        let mut out = String::new();
        let moved = if suites.updated.is_empty() {
            &suites.restored
        } else {
            &suites.updated
        };
        if let Some(error) = &suites.error {
            out.push_str(&self.rows("Suites", [s.red(&format!("failed: {error}"))], 1, 1));
        } else if !moved.is_empty() {
            let n = moved.len();
            out.push_str(&self.rows("Suites", moved.iter().map(SuiteTool::describe), n, n));
        }
        if !suites.upgrades.is_empty() {
            let lines = suites
                .upgrades
                .iter()
                .map(SuiteTool::describe)
                .chain(std::iter::once("kubaos suite upgrade installs them".to_owned()));
            let n = suites.upgrades.len() + 1;
            out.push_str(&self.rows("Upgrades", lines, n, n));
        }
        out
    }

    /// The summary's `Firmware` rows: one line a device, with what became of its update.
    fn firmware_rows(&self, firmware: &Firmware) -> String {
        let lines = firmware.changes().iter().map(|change| {
            let outcome = match change.outcome {
                FirmwareOutcome::Applied => "applied",
                FirmwareOutcome::Staged => "staged for the reboot",
                FirmwareOutcome::Waiting => "waiting for a reboot",
                FirmwareOutcome::Failed => "failed",
            };
            format!("{}: {outcome}", change.update.describe())
        });
        let n = firmware.changes().len();
        self.rows("Firmware", lines, n, n)
    }

    /// What the phase of a program outside apt and the user flake did, on its row.
    fn program(&self, change: &ProgramChange) -> String {
        let s = self.style;
        let mut text = match &change.result {
            ProgramResult::Updated { from, to } => format!("{from} → {}", s.bold(to)),
            ProgramResult::Current { version } => format!("{version}, the newest of its series"),
            ProgramResult::Off => "updates off".to_owned(),
            ProgramResult::Failed { error } => s.red(&format!("failed: {error}")),
        };
        if let Some(waiting) = &change.waiting {
            let _ = write!(text, "; {waiting} waits for a KubaOS release");
        }
        text
    }

    fn failed(&self, error: &str, output: &[String], log: Option<&Path>) -> String {
        let s = self.style;
        let mut out = format!("\n  {} {error}\n", s.red("✗"));
        for line in output {
            let _ = writeln!(out, "    {} {}", s.dim("│"), s.dim(line));
        }
        if let Some(log) = log {
            let _ = writeln!(
                out,
                "  {}{}",
                s.dim(&format!("{:<LABEL_WIDTH$}", "Log")),
                self.path(log)
            );
        }
        out
    }

    /// `path`, with the home as `~`.
    fn path(&self, path: &Path) -> String {
        match self.home.as_deref().and_then(|home| path.strip_prefix(home).ok()) {
            Some(rest) => format!("~/{}", rest.display()),
            None => path.display().to_string(),
        }
    }
}

fn heading(command: Command) -> String {
    match command {
        Command::Check => "KubaOS update check".to_owned(),
        command => format!("KubaOS {}", command.word()),
    }
}

fn available(update: Option<&str>, upgrade: Option<&str>, apt_security: usize) -> String {
    let mut out = String::from("\n");
    if let Some(update) = update {
        let _ = writeln!(out, "  KubaOS {update} is ready: kubaos update");
    }
    if let Some(upgrade) = upgrade {
        let _ = writeln!(out, "  KubaOS {upgrade} is out: kubaos upgrade");
    }
    if apt_security > 0 {
        let _ = writeln!(
            out,
            "  {} waiting: kubaos update",
            count(apt_security, "apt security update")
        );
    }
    if update.is_none() && upgrade.is_none() && apt_security == 0 {
        out.push_str("  Nothing new: KubaOS is up to date.\n");
    }
    out
}

/// The changes `log` lists, in the order people read them: the breaking ones first (no section),
/// then each listed section's, newest first within each.
fn listed(log: &Changelog) -> Vec<(Option<Section>, &Change)> {
    let breaking = log.changes.iter().filter(|c| c.section().is_none()).map(|c| (None, c));
    let sections = Section::LISTED.into_iter().flat_map(|section| {
        log.changes
            .iter()
            .filter(move |c| c.section() == Some(section))
            .map(move |c| (Some(section), c))
    });
    breaking.chain(sections).collect()
}

/// "1 package", "12 packages".
fn count(n: usize, what: &str) -> String {
    if n == 1 {
        format!("1 {what}")
    } else {
        format!("{n} {what}s")
    }
}

/// The apt packages `apt-get upgrade` moved: their count and names, "none" without any.
fn upgraded(packages: usize, upgraded: &[PackageChange]) -> String {
    if packages == 0 {
        return "none".to_owned();
    }
    let counted = if packages == 1 {
        "1 upgraded".to_owned()
    } else {
        format!("{packages} upgraded")
    };
    if upgraded.is_empty() {
        return counted;
    }
    let mut names: Vec<&str> = upgraded.iter().take(APT_SHOWN).map(|p| p.name.as_str()).collect();
    let more = upgraded.len().saturating_sub(APT_SHOWN);
    let more = (more > 0).then(|| format!("and {more} more"));
    names.extend(more.as_deref());
    format!("{counted}: {}", names.join(", "))
}

/// One line a group the package manifest's phase changed: "files: installed loupe; removed eog".
fn group_lines(apt: &[GroupChange]) -> impl Iterator<Item = String> + '_ {
    apt.iter().map(|change| {
        let mut parts = Vec::new();
        if !change.installed.is_empty() {
            parts.push(format!("installed {}", change.installed.join(", ")));
        }
        if !change.removed.is_empty() {
            parts.push(format!("removed {}", change.removed.join(", ")));
        }
        format!("{}: {}", change.group, parts.join("; "))
    })
}

/// A finished phase's time: "0.4s", "42s", "3m05s".
fn finished_duration(ms: u64) -> String {
    if ms < 10_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        whole_duration(ms)
    }
}

/// A running phase's time, in whole seconds: "4s", "3m05s".
fn whole_duration(ms: u64) -> String {
    let secs = ms / 1000;
    if secs < 60 {
        format!("{secs}s")
    } else {
        format!("{}m{:02}s", secs / 60, secs % 60)
    }
}

/// `text` cut to `width` characters, with an ellipsis when cut.
fn fit(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(width.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// One JSON event per line.
#[derive(Debug)]
pub struct Json<W: Write> {
    out: W,
}

impl<W: Write> Json<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }

    pub fn event(&mut self, event: &UpdateEvent) {
        // A closed stdout loses the events, never the update.
        let _ = writeln!(self.out, "{}", event.to_json_line()).and_then(|()| self.out.flush());
    }
}

/// Plain lines: a phase as it starts and ends, its steps, the notes and the summary. Programs'
/// output stays in the log. The failure goes to `err`.
#[derive(Debug)]
pub struct Plain<W: Write, E: Write> {
    out: W,
    err: E,
    text: Text,
}

impl<W: Write, E: Write> Plain<W, E> {
    pub fn new(out: W, err: E, text: Text) -> Self {
        Self { out, err, text }
    }

    pub fn event(&mut self, event: &UpdateEvent) {
        let text = match event {
            UpdateEvent::PhaseStarted { phase } => format!("  > {}\n", phase.title()),
            UpdateEvent::Step { text } => format!("      {text}\n"),
            UpdateEvent::Prompt {
                prompt: Prompt::Confirm { question },
            } => format!("  {question} [Y/n] "),
            UpdateEvent::Prompt {
                prompt: Prompt::Password,
            } => "  sudo asks for your password, for the steps that run as root.\n".to_owned(),
            UpdateEvent::Answered { yes } => format!("{}\n", if *yes { "yes" } else { "no" }),
            UpdateEvent::Failed { .. } => {
                let _ = self.out.flush();
                if let Some(text) = self.text.lines(event) {
                    let _ = self.err.write_all(text.as_bytes()).and_then(|()| self.err.flush());
                }
                return;
            }
            event => match self.text.lines(event) {
                Some(text) => text,
                None => return,
            },
        };
        let _ = self.out.write_all(text.as_bytes()).and_then(|()| self.out.flush());
    }
}

/// The running phase, as the live area shows it.
#[derive(Debug)]
struct Running {
    title: &'static str,
    started: Instant,
    step: Option<String>,
    progress: Option<(usize, usize)>,
    line: Option<String>,
}

/// The live view's screen: lines printed above, and the live area at the bottom, redrawn in
/// place. Pure, so the tests drive it with their own clock and width.
pub struct Screen<W: Write> {
    out: W,
    text: Text,
    width: Box<dyn Fn() -> usize + Send>,
    running: Option<Running>,
    /// How many lines of the live area are on screen; the cursor is at the end of the last.
    drawn: usize,
    /// A prompt has the terminal: nothing is drawn until it is answered.
    paused: bool,
    frame: usize,
    last_draw: Option<Instant>,
}

impl<W: Write> std::fmt::Debug for Screen<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Screen")
            .field("running", &self.running)
            .field("drawn", &self.drawn)
            .field("paused", &self.paused)
            .finish_non_exhaustive()
    }
}

impl<W: Write> Screen<W> {
    pub fn new(out: W, text: Text, width: impl Fn() -> usize + Send + 'static) -> Self {
        Self {
            out,
            text,
            width: Box::new(width),
            running: None,
            drawn: 0,
            paused: false,
            frame: 0,
            last_draw: None,
        }
    }

    pub fn event(&mut self, event: &UpdateEvent, now: Instant) {
        match event {
            UpdateEvent::PhaseStarted { phase } => {
                self.running = Some(Running {
                    title: phase.title(),
                    started: now,
                    step: None,
                    progress: None,
                    line: None,
                });
                self.draw(now);
            }
            UpdateEvent::Step { text } => {
                if let Some(running) = &mut self.running {
                    running.step = Some(text.clone());
                    running.line = None;
                }
                self.draw(now);
            }
            UpdateEvent::Progress { done, total } => {
                if let Some(running) = &mut self.running {
                    running.progress = Some((*done, *total));
                }
                self.draw_soon(now);
            }
            UpdateEvent::Output { line } => {
                if let Some(running) = &mut self.running {
                    running.line = Some(line.clone());
                }
                self.draw_soon(now);
            }
            UpdateEvent::Prompt { prompt } => {
                self.clear();
                self.paused = true;
                let s = self.text.style;
                let text = match prompt {
                    Prompt::Confirm { question } => format!("  {} {} ", s.bold(question), s.dim("[Y/n]")),
                    Prompt::Password => format!(
                        "  {}\n",
                        s.dim("sudo asks for your password, for the steps that run as root.")
                    ),
                };
                self.write(&text);
            }
            UpdateEvent::Answered { .. } => {
                self.paused = false;
                self.draw(now);
            }
            UpdateEvent::PhaseFinished { .. } => {
                self.running = None;
                self.print(event, now);
            }
            event => self.print(event, now),
        }
    }

    /// Turns the spinner and redraws the live area, unless a prompt has the terminal.
    pub fn tick(&mut self, now: Instant) {
        if self.running.is_some() && !self.paused {
            self.frame = (self.frame + 1) % SPINNER.len();
            self.draw(now);
        }
    }

    /// Prints the event's lines above the live area.
    fn print(&mut self, event: &UpdateEvent, now: Instant) {
        let Some(text) = self.text.lines(event) else {
            return;
        };
        self.clear();
        self.write(&text);
        self.draw(now);
    }

    /// Redraws for a burst of output at most every [`REDRAW_EVERY`]; the tick draws the rest.
    fn draw_soon(&mut self, now: Instant) {
        if self
            .last_draw
            .is_none_or(|last| now.saturating_duration_since(last) >= REDRAW_EVERY)
        {
            self.draw(now);
        }
    }

    fn clear(&mut self) {
        if self.drawn == 0 {
            return;
        }
        let mut erase = String::from("\r\x1b[2K");
        for _ in 1..self.drawn {
            erase.push_str("\x1b[1A\x1b[2K");
        }
        self.drawn = 0;
        self.write(&erase);
    }

    fn draw(&mut self, now: Instant) {
        if self.paused {
            return;
        }
        self.clear();
        let Some(running) = &self.running else {
            return;
        };
        let s = self.text.style;
        let width = (self.width)().max(20) - 1;
        let elapsed = whole_duration(now.saturating_duration_since(running.started).as_millis() as u64);
        let mut head = format!(
            "  {} {:<TITLE_WIDTH$}{elapsed:>TIME_WIDTH$}",
            SPINNER[self.frame], running.title
        );
        let mut extra = String::new();
        if let Some((done, total)) = running.progress {
            let _ = write!(extra, "  {done}/{total}");
        }
        if let Some(step) = &running.step {
            let _ = write!(extra, "  {step}");
        }
        let room = width.saturating_sub(head.chars().count());
        let extra = fit(&extra, room);
        head = format!(
            "  {} {}{elapsed:>TIME_WIDTH$}{}",
            s.cyan(SPINNER[self.frame]),
            s.bold(&format!("{:<TITLE_WIDTH$}", running.title)),
            s.dim(&extra)
        );
        let mut area = head;
        self.drawn = 1;
        if let Some(line) = &running.line {
            let _ = write!(area, "\n    {}", s.dim(&fit(line, width.saturating_sub(4))));
            self.drawn = 2;
        }
        self.last_draw = Some(now);
        self.write(&area);
    }

    fn write(&mut self, text: &str) {
        // A terminal that went away loses the view, never the update.
        let _ = self.out.write_all(text.as_bytes()).and_then(|()| self.out.flush());
    }
}

/// The live view: a [`Screen`] on stdout and the thread that turns its spinner.
pub struct LiveView {
    screen: Arc<Mutex<Screen<io::Stdout>>>,
    stop: Option<mpsc::Sender<()>>,
    ticker: Option<thread::JoinHandle<()>>,
}

impl std::fmt::Debug for LiveView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveView").finish_non_exhaustive()
    }
}

impl LiveView {
    fn start(text: Text) -> Self {
        let screen = Arc::new(Mutex::new(Screen::new(io::stdout(), text, terminal_width)));
        let (stop, stopped) = mpsc::channel::<()>();
        let ticking = Arc::clone(&screen);
        let ticker = thread::spawn(move || {
            while let Err(mpsc::RecvTimeoutError::Timeout) = stopped.recv_timeout(TICK) {
                ticking
                    .lock()
                    .expect("the live view's screen mutex poisoned")
                    .tick(Instant::now());
            }
        });
        Self {
            screen,
            stop: Some(stop),
            ticker: Some(ticker),
        }
    }

    fn event(&mut self, event: &UpdateEvent) {
        self.screen
            .lock()
            .expect("the live view's screen mutex poisoned")
            .event(event, Instant::now());
    }
}

impl Drop for LiveView {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(ticker) = self.ticker.take() {
            let _ = ticker.join();
        }
    }
}

/// The terminal's width in columns, from stdout.
fn terminal_width() -> usize {
    rustix::termios::tcgetwinsize(io::stdout())
        .ok()
        .map(|size| usize::from(size.ws_col))
        .filter(|&cols| cols > 0)
        .unwrap_or(DEFAULT_WIDTH)
}

#[cfg(test)]
mod tests;
