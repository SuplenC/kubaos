//! Live previews: the home switched to a KubaOS checkout (a branch's worktree) instead of the
//! release the machine is on, and back.
//!
//! A preview is a switch whose user flake has its `kubaos` input overridden to the checkout
//! ([`Target::Checkout`]). The switch job that makes one records it in the preview file,
//! `~/.local/state/kubaos/preview.json`: the checkout's path, its branch, the commit it built and
//! since when. A switch back to the release ([`Target::Release`]) empties the file. An empty or
//! missing file is no preview. The bar's indicator and the Command menu's "Back to the release"
//! read the same file.
//!
//! ```text
//!            --from A              --from B (replaces A)
//! release ───────────► preview A ───────────────────────► preview B
//!    ▲                     │ plain switch: refused             │
//!    └─────────────────────┴──── --release, update, upgrade, rollback
//! ```
//!
//! # Owns
//!
//! - The preview record and its file ([`Preview`], [`PreviewFile`]).
//! - What a `kubaos switch` asks for, given the preview in effect ([`plan`]): the guards (one
//!   preview at a time, a plain switch refused during one, a checkout that doesn't evaluate refused
//!   before anything switches).
//! - Finding a branch's worktree ([`resolve`]), and leaving a preview before an update ([`leave`]).
//!
//! # Not here
//!
//! - Running the switch, the override and writing the record when it succeeds: `switch.rs`.
//! - The `kubaos switch --from` and `--release` command line: `kubaos-cli`'s `job.rs`.
//! - The bar's indicator (`config/quickshell/modules/bar/PreviewIndicator.qml`) and the
//!   `kubaos.preview` state (`crates/kotodama-kubaos/src/states.rs`).

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::state::write_atomic;
use crate::switch::Host;
use crate::switch::HostError;
use crate::switch::Target;

/// The folder under the Projects root whose worktrees a branch given to `--from` is looked up in.
pub const MAIN_CHECKOUT: &str = "kubaos";

/// The preview in effect, as the preview file holds it. Times are milliseconds since the Unix
/// epoch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preview {
    /// The checkout the home is built from, an absolute path.
    pub path: PathBuf,
    /// Its branch, `None` on a detached head.
    pub branch: Option<String>,
    /// The commit it was at when the preview last switched.
    pub commit: String,
    /// Whether it had uncommitted changes to tracked files then, which the switch built too.
    pub dirty: bool,
    /// When the home first switched to this checkout.
    pub since: u64,
}

impl Preview {
    /// The branch, or the commit's first seven characters on a detached head: what the bar shows.
    pub fn name(&self) -> String {
        self.branch.clone().unwrap_or_else(|| self.short_commit().to_owned())
    }

    /// The commit's first seven characters.
    pub fn short_commit(&self) -> &str {
        self.commit.get(..7).unwrap_or(&self.commit)
    }
}

/// What a checkout is at: [`Host::describe`] reads it from git.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkout {
    pub branch: Option<String>,
    pub commit: String,
    pub dirty: bool,
}

/// The preview file could not be read or written.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PreviewFileError {
    #[error("using {}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("parsing {}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

/// The preview file at one path (`Paths::preview`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewFile {
    path: PathBuf,
}

impl PreviewFile {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The preview in effect, `None` when the file is empty or missing.
    ///
    /// # Errors
    ///
    /// [`PreviewFileError`] when the file can't be read or isn't a preview.
    pub fn read(&self) -> Result<Option<Preview>, PreviewFileError> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(self.io(source)),
        };
        if text.trim().is_empty() {
            return Ok(None);
        }
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|source| PreviewFileError::Parse {
                path: self.path.clone(),
                source,
            })
    }

    /// Records `preview` as the one in effect.
    ///
    /// # Errors
    ///
    /// [`PreviewFileError::Io`] when the file can't be written.
    pub fn write(&self, preview: &Preview) -> Result<(), PreviewFileError> {
        let json = serde_json::to_string_pretty(preview).map_err(|source| PreviewFileError::Parse {
            path: self.path.clone(),
            source,
        })?;
        self.put(&json)
    }

    /// Records that no preview is in effect. The file stays, empty, so a watcher of it (the bar's)
    /// keeps a file to follow.
    ///
    /// # Errors
    ///
    /// [`PreviewFileError::Io`] when the file can't be written.
    pub fn clear(&self) -> Result<(), PreviewFileError> {
        self.put("")
    }

    fn put(&self, text: &str) -> Result<(), PreviewFileError> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|e| self.io(e))?;
        }
        write_atomic(&self.path, text).map_err(|e| self.io(e))
    }

    fn io(&self, source: io::Error) -> PreviewFileError {
        PreviewFileError::Io {
            path: self.path.clone(),
            source,
        }
    }
}

/// What `kubaos switch` was asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// No flag: the home as it stands.
    Plain,
    /// `--release`: the user flake as locked.
    Release,
    /// `--from`: this checkout, already [`resolve`]d to its folder.
    From(PathBuf),
}

/// The switch to queue for a request, and what to tell the user about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub target: Target,
    pub notice: Option<Notice>,
}

/// Something the user is told before the switch is queued.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    /// The switch replaces this preview with another.
    Replaces(Preview),
    /// The switch leaves this preview for the release.
    Leaves(Preview),
    /// `--release` with no preview in effect: the switch is the release as locked all the same.
    NoPreview,
}

/// Why a switch is refused before it is queued.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlanError {
    /// A plain switch while a preview is in effect: which of the two is meant isn't known.
    #[error(
        "the home is a preview of {name} ({path}): `kubaos switch --from {path}` builds it again, \
         `kubaos switch --release` goes back to the release",
        name = preview.name(),
        path = preview.path.display()
    )]
    Active { preview: Preview },
    /// The checkout's git state could not be read: it is no git checkout.
    #[error("{} is not a git checkout", path.display())]
    NotACheckout {
        path: PathBuf,
        #[source]
        source: HostError,
    },
    /// The user flake doesn't evaluate with the checkout as KubaOS; nothing was switched.
    #[error("your home doesn't evaluate with {} as KubaOS, so nothing was switched", path.display())]
    DoesNotEvaluate {
        path: PathBuf,
        #[source]
        source: HostError,
    },
}

/// What `request` switches to while `active` is the preview in effect, checking a checkout
/// through `host` first: it must be a git checkout, and the user flake must evaluate with it.
///
/// # Errors
///
/// [`PlanError::Active`] for a plain switch during a preview, [`PlanError::NotACheckout`] and
/// [`PlanError::DoesNotEvaluate`] for a checkout that can't be previewed.
pub fn plan(request: Request, active: Option<&Preview>, host: &impl Host) -> Result<Plan, PlanError> {
    match request {
        Request::Plain => match active {
            Some(preview) => Err(PlanError::Active {
                preview: preview.clone(),
            }),
            None => Ok(Plan {
                target: Target::Current,
                notice: None,
            }),
        },
        Request::Release => Ok(Plan {
            target: Target::Release,
            notice: Some(active.map_or(Notice::NoPreview, |p| Notice::Leaves(p.clone()))),
        }),
        Request::From(path) => {
            host.describe(&path).map_err(|source| PlanError::NotACheckout {
                path: path.clone(),
                source,
            })?;
            host.evaluate(&path).map_err(|source| PlanError::DoesNotEvaluate {
                path: path.clone(),
                source,
            })?;
            let notice = active
                .filter(|preview| preview.path != path)
                .map(|preview| Notice::Replaces(preview.clone()));
            Ok(Plan {
                target: Target::Checkout { path },
                notice,
            })
        }
    }
}

/// A `--from` that names no checkout.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ResolveError {
    /// The main checkout's worktrees could not be listed.
    #[error("listing the worktrees of {}", repo.display())]
    Worktrees {
        repo: PathBuf,
        #[source]
        source: HostError,
    },
    /// Neither a folder nor a branch any worktree of the main checkout has.
    #[error(
        "{arg} is no folder, and no worktree of {} has it as its branch (`git worktree add`, or give a path)",
        repo.display()
    )]
    NoCheckout { arg: String, repo: PathBuf },
}

/// The checkout `--from <arg>` names: `arg` itself when it is a folder (relative to `cwd`), else
/// the worktree of `repo` (the main checkout, [`MAIN_CHECKOUT`] under the Projects root) whose
/// branch it is. `worktrees` lists `repo`'s worktrees as `git worktree list --porcelain` does.
///
/// # Errors
///
/// [`ResolveError`] when `arg` names neither.
pub fn resolve(
    arg: &str,
    cwd: &Path,
    repo: &Path,
    worktrees: impl FnOnce(&Path) -> Result<String, HostError>,
) -> Result<PathBuf, ResolveError> {
    let folder = cwd.join(arg);
    if folder.is_dir() {
        return Ok(folder.canonicalize().unwrap_or(folder));
    }
    let no_checkout = || ResolveError::NoCheckout {
        arg: arg.to_owned(),
        repo: repo.to_path_buf(),
    };
    if !repo.is_dir() {
        return Err(no_checkout());
    }
    let listing = worktrees(repo).map_err(|source| ResolveError::Worktrees {
        repo: repo.to_path_buf(),
        source,
    })?;
    worktree_for(&listing, arg).ok_or_else(no_checkout)
}

/// The worktree whose branch is `branch`, in `git worktree list --porcelain`'s `listing`.
pub fn worktree_for(listing: &str, branch: &str) -> Option<PathBuf> {
    let want = format!("refs/heads/{branch}");
    let mut path = None;
    for line in listing.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(p));
        } else if line.strip_prefix("branch ") == Some(want.as_str()) {
            return path;
        } else if line.is_empty() {
            path = None;
        }
    }
    None
}

/// Leaving a preview before an update failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LeaveError {
    /// The preview file could not be read.
    #[error("reading the preview")]
    Read(#[source] PreviewFileError),
    /// The switch back to the release failed; the home is still the preview.
    #[error("switching back to the release from the preview of {}: {reason}", preview.name())]
    Switch { preview: Preview, reason: String },
    /// The switch back succeeded, yet the preview file still names a preview.
    #[error("the preview of {} is still recorded after switching back to the release", preview.name())]
    StillActive { preview: Preview },
}

/// Before an update, upgrade or rollback: when a preview is in effect, switches back to the
/// release through `back` and checks it is gone, so an update never starts on a preview. Returns
/// the preview it left, `None` when there was none.
///
/// # Errors
///
/// [`LeaveError`] when the preview could not be left; the update must not run then.
pub fn leave(
    file: &PreviewFile,
    back: impl FnOnce(&Preview) -> Result<(), String>,
) -> Result<Option<Preview>, LeaveError> {
    let Some(preview) = file.read().map_err(LeaveError::Read)? else {
        return Ok(None);
    };
    back(&preview).map_err(|reason| LeaveError::Switch {
        preview: preview.clone(),
        reason,
    })?;
    match file.read().map_err(LeaveError::Read)? {
        Some(still) => Err(LeaveError::StillActive { preview: still }),
        None => Ok(Some(preview)),
    }
}

/// The flake reference `--override-input kubaos` takes for the checkout at `path`: `git+file`,
/// so the build sees the tracked files of the working tree, uncommitted changes included, and
/// never an untracked build folder (`target/`). The characters a URL gives a meaning to are
/// escaped.
pub fn flake_ref(path: &Path) -> String {
    let mut url = String::from("git+file://");
    for c in path.to_string_lossy().chars() {
        match c {
            ' ' => url.push_str("%20"),
            '#' => url.push_str("%23"),
            '%' => url.push_str("%25"),
            '?' => url.push_str("%3F"),
            _ => url.push(c),
        }
    }
    url
}

#[cfg(test)]
mod tests;
