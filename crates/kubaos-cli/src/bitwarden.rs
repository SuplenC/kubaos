//! `kubaos bitwarden`: log in to Bitwarden once, then set up everything that comes from the
//! vault. Safe to run again; docs/SECRETS.md has the vault's conventions.
//!
//!   kubaos bitwarden setup                 asks for the server once, logs in, unlocks, pulls
//!   kubaos bitwarden refresh               after a vault change: unlock and pull, no questions
//!   kubaos bitwarden check                 no bw at all: the SSH agent and what is set up
//!   kubaos bitwarden env pull <project>... a project's variables, vault to keyring
//!   kubaos bitwarden env refresh           pull every project the keyring has
//!   kubaos bitwarden env list [project]    projects and variable names, never values
//!   kubaos bitwarden env clear <project>... | --all
//!   kubaos bitwarden env export <project>  NAME=VALUE records, NUL-separated, from the keyring
//!                                          alone: what `use bwenv` in an .envrc reads
//!
//! The output is the old `dot-bw-setup`'s: `==>` a part, `ok`, `!!` something that needs you
//! (on stderr), `xx` the reason it stopped. Exit codes: 0 ok, 1 it stopped, 2 usage.
//!
//! # Owns
//!
//! - The terminal: colours, the questions, the summary.
//!
//! # Not here
//!
//! - Everything else, Kōbō's card included: `kubaos_core::bitwarden`.

use std::error::Error as _;
use std::io::BufRead as _;
use std::io::IsTerminal as _;
use std::io::Write as _;
use std::process::ExitCode;

use clap::Subcommand;
use clap::ValueEnum;
use kubaos_core::bitwarden;
use kubaos_core::bitwarden::Ask;
use kubaos_core::bitwarden::Bw as _;
use kubaos_core::bitwarden::Error;
use kubaos_core::bitwarden::Host as _;
use kubaos_core::bitwarden::Line;
use kubaos_core::bitwarden::LoginRequest;
use kubaos_core::bitwarden::Parts;
use kubaos_core::bitwarden::Paths;
use kubaos_core::bitwarden::Progress;
use kubaos_core::bitwarden::Real;
use kubaos_core::bitwarden::Report;
use kubaos_core::bitwarden::Session;
use kubaos_core::bitwarden::env;

use crate::usage_error;

#[derive(Subcommand)]
pub enum BitwardenCmd {
    /// Log in (asking for the server the first time), unlock, and set up everything from the vault.
    Setup(PullArgs),
    /// After a vault change: unlock and set everything up again, asking nothing.
    Refresh {
        /// Just these parts: ssh, git, gh, env.
        #[arg(long, value_name = "LIST")]
        only: Option<String>,
    },
    /// No bw at all: is the SSH agent up, and what is set up.
    Check,
    /// Project env: vault to GNOME keyring to direnv.
    Env {
        #[command(subcommand)]
        cmd: EnvCmd,
    },
}

#[derive(clap::Args)]
pub struct PullArgs {
    /// The Bitwarden server: com, eu, or your self-hosted https:// URL. Stored.
    #[arg(long, value_name = "URL")]
    server: Option<String>,
    /// How to log in when bw is logged out (default: ask).
    #[arg(long, value_enum)]
    login: Option<LoginArg>,
    /// Just these parts: ssh, git, gh, env.
    #[arg(long, value_name = "LIST")]
    only: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum LoginArg {
    /// Email, master password and the authenticator-app code (bw can't do FIDO2 keys).
    Totp,
    /// The same as totp.
    Password,
    /// The personal API key: the fallback, with no two-step code.
    Apikey,
    /// Your organization's identity provider, in the browser.
    Sso,
}

#[derive(Subcommand)]
pub enum EnvCmd {
    /// Fetch the projects' variables from the vault and replace what the keyring holds.
    Pull {
        #[arg(required = true)]
        projects: Vec<String>,
    },
    /// Pull every project the keyring already has.
    Refresh,
    /// Projects and their variables' names, never the values.
    #[command(alias = "ls")]
    List { project: Option<String> },
    /// Forget projects.
    #[command(alias = "rm")]
    Clear {
        projects: Vec<String>,
        /// Every project.
        #[arg(long, conflicts_with = "projects")]
        all: bool,
    },
    /// A project's NAME=VALUE records, NUL-separated, from the keyring alone.
    Export { project: String },
}

/// The terminal's colours, off when stdout isn't one.
struct Colours {
    step: &'static str,
    ok: &'static str,
    warn: &'static str,
    err: &'static str,
    off: &'static str,
}

impl Colours {
    fn new() -> Self {
        if std::io::stdout().is_terminal() {
            Self {
                step: "\x1b[1;34m",
                ok: "\x1b[32m",
                warn: "\x1b[1;33m",
                err: "\x1b[1;31m",
                off: "\x1b[0m",
            }
        } else {
            Self {
                step: "",
                ok: "",
                warn: "",
                err: "",
                off: "",
            }
        }
    }

    fn print(&self, l: &Line) {
        match l {
            Line::Step(s) => println!("\n{}==> {s}{}", self.step, self.off),
            Line::Ok(s) => println!("{}  ok{}  {s}", self.ok, self.off),
            Line::Info(s) => println!("      {s}"),
            Line::Warn(s) => eprintln!("{}  !!  {s}{}", self.warn, self.off),
        }
    }

    /// The reason it stopped, and bw's own words under it. Exit code 1.
    fn die(&self, e: &Error) -> ExitCode {
        eprintln!("{}  xx  {e}{}", self.err, self.off);
        let mut cause = e.source();
        while let Some(c) = cause {
            for line in c.to_string().lines().filter(|l| !l.trim().is_empty()) {
                eprintln!("      bw: {line}");
            }
            cause = c.source();
        }
        ExitCode::from(1)
    }

    fn summary(&self, r: &Report) {
        self.print(&Line::Step("Summary".into()));
        for line in &r.summary {
            self.print(&Line::Info(line.clone()));
        }
        if r.warnings.is_empty() {
            println!("\n{}  all done, nothing needs you{}", self.ok, self.off);
        } else {
            println!("\n{}  {} thing(s) need you:{}", self.warn, r.warnings.len(), self.off);
            for w in &r.warnings {
                self.print(&Line::Info(format!("- {w}")));
            }
        }
        self.print(&Line::Info(String::new()));
        self.print(&Line::Info(
            "Re-run after vault changes: kubaos bitwarden refresh. In a project: echo 'use bwenv <project>' >> .envrc"
                .into(),
        ));
    }
}

/// The person at the terminal, asked on stderr the way `read -p` asks.
struct Terminal {
    tty: bool,
}

impl Ask for Terminal {
    fn ask(&mut self, prompt: &str) -> Option<String> {
        if !self.tty {
            return None;
        }
        eprint!("      {prompt} ");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line).ok()?;
        Some(line.trim_end_matches(['\n', '\r']).to_string())
    }

    fn tty(&self) -> bool {
        self.tty
    }
}

/// `BW_SESSION`, when the person exported one.
fn exported_session() -> Option<Session> {
    std::env::var("BW_SESSION").ok().filter(|s| !s.is_empty()).map(Session::new)
}

fn parts(only: Option<&str>) -> Result<Parts, String> {
    only.map_or(Ok(Parts::ALL), Parts::parse)
}

pub fn main(cmd: BitwardenCmd) -> ExitCode {
    let c = Colours::new();
    let paths = Paths::from_env();
    let mut m = Real::new();
    match cmd {
        BitwardenCmd::Check => {
            let mut emit = |l: &Line| c.print(l);
            let mut p = Progress::new(&mut emit);
            bitwarden::check(&mut m, &paths, &mut p);
            let report = p.finish();
            c.summary(&report);
            ExitCode::SUCCESS
        }
        BitwardenCmd::Setup(args) => {
            let method = args.login.map(|l| match l {
                LoginArg::Totp | LoginArg::Password => bitwarden::Method::Totp,
                LoginArg::Apikey => bitwarden::Method::ApiKey,
                LoginArg::Sso => bitwarden::Method::Sso,
            });
            let req = LoginRequest {
                server: args.server,
                method,
                ..LoginRequest::default()
            };
            pull(&c, &mut m, &paths, &req, args.only.as_deref())
        }
        BitwardenCmd::Refresh { only } => {
            let req = LoginRequest {
                refresh: true,
                ..LoginRequest::default()
            };
            pull(&c, &mut m, &paths, &req, only.as_deref())
        }
        BitwardenCmd::Env { cmd } => env_main(&c, &mut m, cmd),
    }
}

fn pull(c: &Colours, m: &mut Real, paths: &Paths, req: &LoginRequest, only: Option<&str>) -> ExitCode {
    let parts = match parts(only) {
        Ok(p) => p,
        Err(e) => return usage_error(format!("--only: {e}")),
    };
    let req = LoginRequest {
        session: exported_session(),
        ..req.clone()
    };
    let mut emit = |l: &Line| c.print(l);
    let mut p = Progress::new(&mut emit);
    let mut ask = Terminal {
        tty: std::io::stdin().is_terminal(),
    };
    let run = bitwarden::preflight(m, parts)
        .and_then(|()| bitwarden::login(m, paths, &req, &mut ask, &mut p))
        .and_then(|session| bitwarden::pull(m, paths, &session, parts, &mut p));
    let report = p.finish();
    if let Err(e) = run {
        return c.die(&e);
    }
    c.summary(&report);
    ExitCode::SUCCESS
}

fn env_main(c: &Colours, m: &mut Real, cmd: EnvCmd) -> ExitCode {
    if !m.has("secret-tool") {
        return c.die(&Error::Missing("secret-tool", "run system/70-bitwarden.sh"));
    }
    let result = match cmd {
        EnvCmd::Export { project } => return export(c, m, &project),
        EnvCmd::List { project } => list(m, project.as_deref()),
        EnvCmd::Clear { projects, all } => clear(m, &projects, all),
        EnvCmd::Pull { projects } => env_pull(c, m, &projects),
        EnvCmd::Refresh => match env::projects(m) {
            Ok(projects) if projects.is_empty() => {
                eprintln!("kubaos bitwarden: no projects in the keyring yet (kubaos bitwarden env pull <project>)");
                Ok(())
            }
            Ok(projects) => env_pull(c, m, &projects),
            Err(e) => Err(e),
        },
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => c.die(&e),
    }
}

fn export(c: &Colours, m: &mut Real, project: &str) -> ExitCode {
    let vars = match env::export(m, project) {
        Ok(v) => v,
        Err(e) => return c.die(&e),
    };
    if vars.is_empty() {
        eprintln!("kubaos bitwarden: nothing for '{project}' in the keyring; run: kubaos bitwarden env pull {project}");
        return ExitCode::SUCCESS;
    }
    let mut out = std::io::stdout().lock();
    for (name, value) in vars {
        let _ = write!(out, "{name}={}\0", value.0);
    }
    let _ = out.flush();
    ExitCode::SUCCESS
}

fn list(m: &mut Real, project: Option<&str>) -> Result<(), Error> {
    for (p, names) in env::list(m, project)? {
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        println!("{p}: {}", names.join(" "));
    }
    Ok(())
}

fn clear(m: &mut Real, projects: &[String], all: bool) -> Result<(), Error> {
    if all {
        env::clear(m, None);
        eprintln!("kubaos bitwarden: cleared every project");
        return Ok(());
    }
    if projects.is_empty() {
        eprintln!("kubaos bitwarden: name the projects, or --all");
        return Ok(());
    }
    for p in projects {
        env::check_project(p)?;
        env::clear(m, Some(p));
        eprintln!("kubaos bitwarden: cleared {p}");
    }
    Ok(())
}

fn env_pull(c: &Colours, m: &mut Real, projects: &[String]) -> Result<(), Error> {
    for p in projects {
        env::check_project(p)?;
    }
    if !m.installed() {
        return Err(Error::Missing("bw", "run system/70-bitwarden.sh"));
    }
    let session = match exported_session() {
        Some(s) => s,
        None => m.unlock(None).map_err(Error::NotUnlocked)?,
    };
    let mut emit = |l: &Line| c.print(l);
    let mut p = Progress::new(&mut emit);
    env::pull(m, &session, projects, &mut p)
}
