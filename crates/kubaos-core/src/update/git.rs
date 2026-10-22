//! Reading KubaOS's checkout and `origin` with git, as the user: the refs a [`Refs`] offers, the
//! commits between two releases, and a file at a commit. Nothing here writes: moving `release`
//! and fetching are root's, through `machine.rs`.
//!
//! The checkout is root-owned, and `/etc/gitconfig` lists it in `safe.directory`, so git reads it
//! as the user.
//!
//! # Owns
//!
//! - [`GitRefs`], the refs of the checkout after a fetch, signatures checked.
//! - [`RemoteRefs`], the refs `git ls-remote` lists, for the daily check.
//!
//! # Not here
//!
//! - Which ref an update takes: `policy.rs`.

use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use super::policy::Refs;
use super::policy::RefsError;
use super::policy::Tag;
use super::policy::Verified;

/// The refs of KubaOS's checkout as its last fetch left them.
#[derive(Clone, Debug)]
pub struct GitRefs {
    dir: PathBuf,
    signers: Option<PathBuf>,
}

impl GitRefs {
    /// The refs of the checkout `dir`, checking signatures against `signers` when it exists.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>, signers: &Path) -> Self {
        Self {
            dir: dir.into(),
            signers: signers.is_file().then(|| signers.to_path_buf()),
        }
    }

    /// The commit `rev` names, or `None` when it names none.
    pub fn commit_of(&self, rev: &str) -> Option<String> {
        let out = git(
            &self.dir,
            ["rev-parse", "--verify", "--quiet", &format!("{rev}^{{commit}}")],
        )
        .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
    }

    /// `git log --oneline from..to`, at most `limit` lines, newest first.
    pub fn log(&self, from: &str, to: &str, limit: usize) -> Vec<String> {
        let range = format!("{from}..{to}");
        let max = format!("--max-count={limit}");
        match git(&self.dir, ["log", "--oneline", "--no-decorate", &max, &range]) {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_owned)
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The names of the files in the folder `dir` at `commit`; none when it has no such folder.
    pub fn file_list(&self, commit: &str, dir: &str) -> Vec<String> {
        match git(&self.dir, ["ls-tree", "--name-only", &format!("{commit}:{dir}")]) {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_owned)
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The URL of `origin`.
    pub fn origin_url(&self) -> Option<String> {
        let out = git(&self.dir, ["remote", "get-url", "origin"]).ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
    }

    fn verify(&self, what: &str, rev: &str) -> Result<Verified, RefsError> {
        let Some(signers) = &self.signers else {
            return Ok(Verified::NoKeys);
        };
        let mut config = std::ffi::OsString::from("gpg.ssh.allowedSignersFile=");
        config.push(signers);
        let out = git(
            &self.dir,
            [OsStr::new("-c"), &config, OsStr::new(what), OsStr::new(rev)],
        )
        .map_err(|e| RefsError::new(format!("git {what} {rev}"), e))?;
        Ok(if out.status.success() {
            Verified::Good
        } else {
            Verified::Bad
        })
    }
}

impl Refs for GitRefs {
    fn main_head(&self) -> Result<Option<String>, RefsError> {
        Ok(self.commit_of("refs/remotes/origin/main"))
    }

    fn tags(&self) -> Result<Vec<Tag>, RefsError> {
        let out = git(
            &self.dir,
            [
                "for-each-ref",
                "--format=%(refname:short) %(objectname) %(*objectname)",
                "refs/tags/v*",
            ],
        )
        .map_err(|e| RefsError::new("listing the release tags", e))?;
        if !out.status.success() {
            return Err(RefsError::new("listing the release tags", stderr(&out)));
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| {
                let mut words = line.split_whitespace();
                let name = words.next()?.to_owned();
                let object = words.next()?;
                // An annotated tag's own object is the tag; the commit is the peeled one.
                let commit = words.next().unwrap_or(object).to_owned();
                Some(Tag { name, commit })
            })
            .collect())
    }

    fn verify_commit(&self, commit: &str) -> Result<Verified, RefsError> {
        self.verify("verify-commit", commit)
    }

    fn verify_tag(&self, name: &str) -> Result<Verified, RefsError> {
        self.verify("verify-tag", name)
    }

    fn file_at(&self, commit: &str, path: &str) -> Option<String> {
        let out = git(&self.dir, ["show", &format!("{commit}:{path}")]).ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// The refs `git ls-remote` lists for `origin`, without its objects. Files and signatures come
/// from the checkout, for the commits it already has.
#[derive(Clone, Debug)]
pub struct RemoteRefs {
    main: Option<String>,
    tags: Vec<Tag>,
    local: GitRefs,
}

impl RemoteRefs {
    /// Lists `url`'s `main` and `v*` tags with `git ls-remote`.
    ///
    /// # Errors
    ///
    /// [`RefsError`] when git can't reach `url`.
    pub fn fetch(url: &str, local: GitRefs) -> Result<Self, RefsError> {
        let out = Command::new("git")
            .args(["ls-remote", "--", url, "refs/heads/main", "refs/tags/v*"])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| RefsError::new(format!("git ls-remote {url}"), e))?;
        if !out.status.success() {
            return Err(RefsError::new(format!("git ls-remote {url}"), stderr(&out)));
        }
        Ok(Self::parse(&String::from_utf8_lossy(&out.stdout), local))
    }

    /// The refs in `git ls-remote`'s output.
    #[must_use]
    pub fn parse(listing: &str, local: GitRefs) -> Self {
        let mut main = None;
        let mut tags: Vec<Tag> = Vec::new();
        for line in listing.lines() {
            let Some((commit, name)) = line.split_once('\t') else {
                continue;
            };
            if name == "refs/heads/main" {
                main = Some(commit.to_owned());
                continue;
            }
            let Some(tag) = name.strip_prefix("refs/tags/") else {
                continue;
            };
            // `v1^{}` is the commit an annotated tag `v1` points at, listed after the tag.
            let (tag, peeled) = match tag.strip_suffix("^{}") {
                Some(tag) => (tag, true),
                None => (tag, false),
            };
            match tags.iter_mut().find(|t| t.name == tag) {
                Some(known) if peeled => commit.clone_into(&mut known.commit),
                Some(_) => {}
                None => tags.push(Tag {
                    name: tag.to_owned(),
                    commit: commit.to_owned(),
                }),
            }
        }
        Self { main, tags, local }
    }
}

impl Refs for RemoteRefs {
    fn main_head(&self) -> Result<Option<String>, RefsError> {
        Ok(self.main.clone())
    }

    fn tags(&self) -> Result<Vec<Tag>, RefsError> {
        Ok(self.tags.clone())
    }

    fn verify_commit(&self, _commit: &str) -> Result<Verified, RefsError> {
        Ok(Verified::Unchecked)
    }

    fn verify_tag(&self, _name: &str) -> Result<Verified, RefsError> {
        Ok(Verified::Unchecked)
    }

    fn file_at(&self, commit: &str, path: &str) -> Option<String> {
        self.local.file_at(commit, path)
    }
}

fn git<I, S>(dir: &Path, args: I) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stdin(Stdio::null())
        .output()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).trim().to_owned()
}

#[cfg(test)]
mod tests;
