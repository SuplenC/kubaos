//! What the installed machine records for `kubaos update`: the channel it follows, the release
//! it runs, and a marker for every migration of that release, which the install's payload already
//! is past (`update::record`).

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use super::Install;
use super::PAYLOAD_KUBAOS;
use super::R;
use crate::engine::files;
use crate::update::migrations;
use crate::update::record;
use crate::update::record::Channel;
use crate::update::record::Installed;
use crate::update::version::Version;

impl Install<'_> {
    /// `/etc/kubaos/channel`, `/var/lib/kubaos/release`, and the migration markers, system ones in
    /// `/var/lib/kubaos/migrations/` and the user's in their state folder.
    pub(super) fn write_release_records(&mut self, home: &str, owner: &str) -> R<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let date = files::utc_datetime(now);
        let commit = self.release_commit();
        let channel = self.install_channel();
        self.write(&format!("/{}", record::CHANNEL), &format!("{channel}\n"), 0o644)?;
        self.info(format!("updates follow the {channel} channel"));
        let checkout_version = self.r.read_file(&self.t(&format!("{PAYLOAD_KUBAOS}/VERSION")));
        let version = Version::parse(&self.manifest.version)
            .ok()
            .or_else(|| checkout_version.and_then(|v| Version::parse(&v).ok()));
        match version {
            Some(version) => {
                let installed = Installed {
                    version,
                    codename: self.manifest.codename.as_deref().map(str::to_lowercase),
                    commit,
                    channel,
                    date: date.clone(),
                };
                self.write(&format!("/{}", record::RELEASE), &installed.to_text(), 0o644)?;
            }
            // kubaos update reads the release from the checkout instead.
            None => self.warn(format!(
                "the payload's version {:?} is not a release version: no /{} written",
                self.manifest.version,
                record::RELEASE
            )),
        }

        let marker = format!("{date}\n");
        let system = format!("/{}", record::MARKERS);
        self.mkdir(&self.t(&system))?;
        for name in self.migrations("system") {
            self.write(&format!("{system}/{name}"), &marker, 0o644)?;
        }
        let user = format!("{home}/.local/state/kubaos/migrations");
        self.mkdir(&self.t(&user))?;
        for name in self.migrations("user") {
            self.write(&format!("{user}/{name}"), &marker, 0o644)?;
        }
        self.sh(&crate::engine::runner::Cmd::new("chown").args(["-R", owner, &self.ts(&format!("{home}/.local"))]))?;
        Ok(())
    }

    /// The commit of the checkout's `release` branch, read from git's files: the manifest's commit
    /// may carry `-dirty`.
    fn release_commit(&mut self) -> String {
        let git = format!("{PAYLOAD_KUBAOS}/.git");
        let branch = format!("refs/heads/{}", record::RELEASE_BRANCH);
        let loose = self.r.read_file(&self.t(&format!("{git}/{branch}")));
        let packed = || {
            self.r
                .read_file(&self.t(&format!("{git}/packed-refs")))
                .and_then(|text| {
                    text.lines()
                        .filter_map(|line| line.split_once(' '))
                        .find(|(_, name)| *name == branch)
                        .map(|(commit, _)| commit.to_owned())
                })
        };
        loose
            .map(|text| text.trim().to_owned())
            .filter(|commit| !commit.is_empty())
            .or_else(packed)
            .unwrap_or_else(|| self.manifest.commit.trim_end_matches("-dirty").to_owned())
    }

    /// Stable when a release tag at the installed commit verifies against the machine's
    /// `allowed_signers`; else edge, as for a payload built at no verified tag.
    fn install_channel(&mut self) -> Channel {
        let signers = format!("/{}", record::ALLOWED_SIGNERS);
        if self.dry() || !self.r.exists(&self.t(&signers)) {
            return Channel::Edge;
        }
        let config = format!("gpg.ssh.allowedSignersFile={signers}");
        let list = self.chroot(&[
            "git",
            "-C",
            PAYLOAD_KUBAOS,
            "tag",
            "--points-at",
            record::RELEASE_BRANCH,
            "--list",
            "v*",
        ]);
        let Ok(out) = self.sh(&list) else {
            return Channel::Edge;
        };
        let tags: Vec<String> = out.stdout.lines().map(|t| t.trim().to_owned()).collect();
        for tag in tags.iter().filter(|t| Version::parse(t).is_ok()) {
            let verify = self.chroot(&["git", "-C", PAYLOAD_KUBAOS, "-c", &config, "verify-tag", tag]);
            if self.sh(&verify).is_ok() {
                return Channel::Stable;
            }
        }
        Channel::Edge
    }

    /// The migration file names of the payload's release, in `migrations/<kind>/`; none when the
    /// folder isn't there.
    fn migrations(&mut self, kind: &str) -> Vec<String> {
        let dir = format!("{PAYLOAD_KUBAOS}/migrations/{kind}");
        let list = self.chroot(&["find", &dir, "-mindepth", "1", "-maxdepth", "1", "-printf", "%f\n"]);
        match self.sh(&list) {
            Ok(out) => out
                .stdout
                .lines()
                .map(str::trim)
                .filter(|name| migrations::stamp_of(name).is_some())
                .map(str::to_owned)
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}
