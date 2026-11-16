//! `kubaos-apps`: the Apps catalog's root helper, for apps that are folders of actions.
//!
//!   kubaos-apps serve
//!   kubaos-apps --help
//!
//! `serve` says `{"reply":"ready"}` on stdout, then answers one JSON request a line on stdin with
//! one reply a line, until stdin ends. A request names an app id, a setup id and a step number,
//! nothing else: the helper reads the step's action from the root-owned catalog,
//! `/usr/share/kubaos/apps/<id>/app.toml`, refuses one that isn't root's, and undoes only what
//! root's journal (`/var/lib/kubaos/apps/`) recorded (`kubaos_core::apps::root`). It acts for the
//! user who asked, whom pkexec (`PKEXEC_UID`) or sudo (`SUDO_UID`) names, and holds
//! `/run/kubaos/apps.lock` (a folder only root may write) while it serves, so one run at a time
//! changes the machine's apps.
//!
//! Run as anyone but root, it serves only the checkout `KUBAOS_APPS_CHECKOUT` names, owned by
//! that user, with its journals and lock under `KUBAOS_APPS_STATE`: the tests' way in. Root never
//! reads either: pkexec clears the environment and sudo resets it.
//!
//! Exit codes: 0 done, 1 it couldn't serve, 2 a usage error.
//!
//! # Owns
//!
//! - The helper's command line, who it acts for and where it reads and writes.
//!
//! # Not here
//!
//! - The requests, the refusals and the steps: `kubaos_core::apps::root` and `engine`.

use std::fs;
use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use kubaos_core::apps::action::Side;
use kubaos_core::apps::engine::Keeper;
use kubaos_core::apps::host::SystemHost;
use kubaos_core::apps::journal;
use kubaos_core::apps::root::Server;
use kubaos_core::update::event::chain;

/// The release's checkout on a machine.
const CHECKOUT: &str = "/usr/share/kubaos";

/// The folder of root's lock, which only root may write, so no one else can put a file or a link
/// where the lock is.
const LOCK_DIR: &str = "/run/kubaos";

/// The lock one run holds while it changes the machine's apps, in [`LOCK_DIR`].
const LOCK: &str = "apps.lock";

/// Where the helper reads and writes, and for whom.
struct Place {
    checkout: PathBuf,
    journals: PathBuf,
    lock: PathBuf,
    /// Who must own the catalog.
    owner: u32,
    /// The user it acts for, by uid.
    for_uid: u32,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["serve"] => {}
        ["--help" | "-h"] => {
            say_usage(&mut io::stdout());
            return ExitCode::SUCCESS;
        }
        _ => {
            say_usage(&mut io::stderr());
            return ExitCode::from(2);
        }
    }
    match serve() {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            fail(&why);
            ExitCode::from(1)
        }
    }
}

#[expect(clippy::print_stderr, reason = "The helper's one error is its output.")]
fn fail(why: &str) {
    eprintln!("kubaos-apps: {why}");
}

fn say_usage(out: &mut dyn io::Write) {
    let _ = writeln!(out, "kubaos-apps serve\nkubaos-apps --help");
}

fn serve() -> Result<(), String> {
    let place = place()?;
    rustix::process::umask(rustix::fs::Mode::from_raw_mode(0o022));
    let (user, home) = account(place.for_uid)?;
    if place.owner == 0 {
        root_only_dir(Path::new(LOCK_DIR))?;
    }
    let _lock = journal::lock(&place.lock).map_err(|e| chain(&e))?;
    let catalog = place.checkout.join("apps");
    let mut host = SystemHost::new(user, home, &place.checkout).as_root();
    let keeper = Keeper::new(&mut host, Side::Root, &catalog, &place.journals);
    let mut server = Server::new(keeper, &catalog, place.owner);
    server
        .serve(io::stdin().lock(), io::stdout().lock())
        .map_err(|e| format!("serving: {e}"))
}

/// Makes `dir` root's alone (`0700`), or checks that it is: a folder that is a link, someone
/// else's, or writable by anyone else is refused.
fn root_only_dir(dir: &Path) -> Result<(), String> {
    match fs::DirBuilder::new().mode(0o700).create(dir) {
        Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(format!("making {}: {e}", dir.display())),
        _ => {}
    }
    let meta = fs::symlink_metadata(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    if !meta.is_dir() || meta.uid() != 0 || meta.mode() & 0o022 != 0 {
        return Err(format!("{} isn't a folder only root may write", dir.display()));
    }
    Ok(())
}

fn place() -> Result<Place, String> {
    let euid = rustix::process::geteuid().as_raw();
    if euid == 0 {
        let asked = ["PKEXEC_UID", "SUDO_UID"]
            .iter()
            .find_map(|name| std::env::var(name).ok()?.parse::<u32>().ok())
            .ok_or("run it through pkexec or sudo, which name the user it acts for")?;
        return Ok(Place {
            checkout: PathBuf::from(CHECKOUT),
            journals: PathBuf::from(journal::ROOT_JOURNALS),
            lock: Path::new(LOCK_DIR).join(LOCK),
            owner: 0,
            for_uid: asked,
        });
    }
    let (Some(checkout), Some(state)) = (
        std::env::var_os("KUBAOS_APPS_CHECKOUT"),
        std::env::var_os("KUBAOS_APPS_STATE"),
    ) else {
        return Err("this runs as root, through pkexec or sudo".to_owned());
    };
    let state = PathBuf::from(state);
    Ok(Place {
        checkout: PathBuf::from(checkout),
        journals: state.join("journals"),
        lock: state.join("lock"),
        owner: euid,
        for_uid: euid,
    })
}

/// The name and home of the user `uid`, from the password database.
fn account(uid: u32) -> Result<(String, PathBuf), String> {
    let passwd = fs::read_to_string("/etc/passwd").map_err(|e| format!("reading /etc/passwd: {e}"))?;
    let found = passwd.lines().find_map(|line| {
        let fields: Vec<&str> = line.split(':').collect();
        (fields.len() >= 6 && fields[2].parse::<u32>().ok() == Some(uid))
            .then(|| (fields[0].to_owned(), PathBuf::from(fields[5])))
    });
    if let Some(found) = found {
        return Ok(found);
    }
    // A user the file doesn't list (a directory service's) is named by the environment only
    // when it is the user running the helper: never as root.
    if uid == rustix::process::geteuid().as_raw()
        && uid != 0
        && let (Ok(user), Some(home)) = (std::env::var("USER"), std::env::var_os("HOME"))
    {
        return Ok((user, PathBuf::from(home)));
    }
    Err(format!("no user {uid} in /etc/passwd"))
}
