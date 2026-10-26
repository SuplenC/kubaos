//! The updater's side of the package manifest: the sync every update runs, `kubaos packages
//! status`, and turning an optional group on or off.
//!
//! A sync, in order: the vendor repositories of the groups with packages to install
//! (`system/apt-repo.sh`, as root); `apt-get install` of the wanted packages that have an
//! install candidate; holding the held groups; then the removals `apt::plan` allows, each one
//! checked first with `apt-get -s remove`, so a package something else still depends on stays.
//! What it installed and removed goes into the record, `/var/lib/kubaos/apt-installed`.
//!
//! # Owns
//!
//! - Running apt for the manifest, and writing the choices and the record as root.
//!
//! # Not here
//!
//! - What to install, hold and remove: `apt::plan`. The files' formats: `apt`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::DPKG_KEEP_CONFIG;
use super::UpdateError;
use super::Updater;
use super::read_or_empty;
use super::release_of;
use crate::apt::Choices;
use crate::apt::Dpkg;
use crate::apt::Manifest;
use crate::apt::Record;
use crate::apt::Status;
use crate::apt::Switch;
use crate::apt::dpkg;
use crate::apt::plan;
use crate::apt::plan::Facts;
use crate::apt::plan::Want;
use crate::update::event::Command;
use crate::update::event::GroupChange;
use crate::update::event::Phase;
use crate::update::event::Restart;
use crate::update::event::Summary;
use crate::update::event::UpdateEvent;
use crate::update::machine::Cmd;
use crate::update::machine::Machine;

/// What a sync reads before it decides.
struct AptState {
    manifest: Manifest,
    choices: Choices,
    record: Record,
    arch: String,
    dpkg: Dpkg,
}

impl AptState {
    fn facts(&self) -> Facts<'_> {
        Facts {
            manifest: &self.manifest,
            choices: &self.choices,
            record: &self.record,
            arch: &self.arch,
            dpkg: &self.dpkg,
        }
    }
}

/// What a sync changed, by group, sorted by the group.
#[derive(Default)]
struct Changes(BTreeMap<String, GroupChange>);

impl Changes {
    fn entry(&mut self, group: &str) -> &mut GroupChange {
        self.0.entry(group.to_owned()).or_insert_with(|| GroupChange {
            group: group.to_owned(),
            ..GroupChange::default()
        })
    }

    fn into_vec(self) -> Vec<GroupChange> {
        self.0.into_values().collect()
    }
}

impl<M: Machine> Updater<'_, M> {
    /// How the machine compares with the checked-out release's package manifest.
    ///
    /// # Errors
    ///
    /// [`UpdateError::NoManifest`] when the release has none, [`UpdateError::Manifest`] when it
    /// can't be read, else the dpkg or apt query that failed.
    pub fn package_status(&mut self) -> Result<Status, UpdateError> {
        let state = self.apt_state()?.ok_or_else(|| UpdateError::NoManifest {
            path: self.sys.manifest(),
        })?;
        let facts = state.facts();
        let wanted = plan::status_candidates(&facts);
        let candidates = self.candidates(&wanted);
        Ok(Status::new(&facts, &candidates))
    }

    /// Turns the optional group `name` on or off: records the choice, then installs its packages
    /// and runs its `when_on` script, or runs its `when_off` script and removes the packages
    /// KubaOS installed for it. A failed script puts the old choice back.
    ///
    /// # Errors
    ///
    /// [`UpdateError::NoGroup`] or [`UpdateError::NotOptional`] for a group that can't be turned
    /// on or off, else the step that failed.
    pub fn set_group(&mut self, name: &str, switch: Switch) -> Result<(), UpdateError> {
        self.begin(Command::Packages);
        let result = self.try_set_group(name, switch);
        self.end(result)
    }

    fn try_set_group(&mut self, name: &str, switch: Switch) -> Result<(), UpdateError> {
        let (_lock, installed) = self.phase(Phase::Checking, |u| {
            let lock = u.lock()?;
            let installed = u.checked_installed()?;
            Ok((lock, installed))
        })?;
        let path = self.sys.manifest();
        let manifest = Manifest::read(&path)
            .map_err(UpdateError::Manifest)?
            .ok_or(UpdateError::NoManifest { path })?;
        let group = manifest
            .group(name)
            .ok_or_else(|| UpdateError::NoGroup { name: name.to_owned() })?;
        if group.optional.is_none() {
            return Err(UpdateError::NotOptional { name: name.to_owned() });
        }
        let script = match switch {
            Switch::On => group.when_on.clone(),
            Switch::Off => group.when_off.clone(),
        };
        let choices_path = self.sys.apt_choices();
        let before = read_or_empty(&choices_path)?;
        let mut choices = Choices::parse(&before);
        choices.set(name, switch);
        self.write_root(&choices_path, &choices.to_text())?;
        self.j
            .note(format!("{name} is {switch} from now on: kubaos update follows it"));

        let apt = self.phase(Phase::Manifest, |u| {
            let scripted = |u: &mut Self| match u.run_group_script(&script) {
                Ok(()) => Ok(()),
                Err(e) => {
                    u.write_root(&choices_path, &before)?;
                    Err(e)
                }
            };
            match switch {
                Switch::On => {
                    let apt = u.sync_manifest(None)?;
                    scripted(u)?;
                    Ok(apt)
                }
                Switch::Off => {
                    scripted(u)?;
                    let apt = u.sync_manifest(Some(name))?;
                    u.warn_kept(name)?;
                    Ok(apt)
                }
            }
        })?;
        let summary = Summary {
            command: Command::Packages,
            from: release_of(&installed),
            to: release_of(&installed),
            channel: self.channel(&installed),
            changes: Vec::new(),
            packages: 0,
            apt,
            inputs: Vec::new(),
            migrations: Vec::new(),
            reloaded: Vec::new(),
            restart: Restart::None,
            restart_reason: None,
            undo: None,
            log: self.j.log_path(),
        };
        self.j.emit(&UpdateEvent::Summary(summary));
        Ok(())
    }

    /// Says which of the group `name`'s packages stay installed after it was turned off because
    /// KubaOS didn't install them, as on a machine installed before the record, and how to remove
    /// them.
    fn warn_kept(&mut self, name: &str) -> Result<(), UpdateError> {
        let Some(state) = self.apt_state()? else {
            return Ok(());
        };
        let facts = state.facts();
        let Some(group) = state.manifest.group(name) else {
            return Ok(());
        };
        let kept: Vec<&str> = group
            .packages
            .iter()
            .filter(|p| state.dpkg.has(p) && !facts.wanted(p))
            .map(String::as_str)
            .collect();
        if !kept.is_empty() {
            self.j.warn(format!(
                "kept {}: KubaOS didn't install them, so it leaves them to you (sudo apt remove {})",
                kept.join(", "),
                kept.join(" ")
            ));
        }
        Ok(())
    }

    /// Runs a group's `when_on` or `when_off` script from the checkout, as the user: the scripts
    /// ask for sudo themselves.
    fn run_group_script(&mut self, script: &[String]) -> Result<(), UpdateError> {
        let Some((program, args)) = script.split_first() else {
            return Ok(());
        };
        let line = script.join(" ");
        self.j.step(format!("Running {line}"));
        let cmd = Cmd::user(self.sys.checkout().join(program))
            .args(args)
            .env("KUBAOS_DIR", self.sys.checkout())
            .runs_sudo();
        self.run(&cmd).map_err(UpdateError::step(format!("running {line}")))
    }

    /// The manifest, the choices, the record, the architecture and dpkg's packages; `None` when
    /// the checked-out release has no manifest.
    fn apt_state(&mut self) -> Result<Option<AptState>, UpdateError> {
        let Some(manifest) = Manifest::read(&self.sys.manifest()).map_err(UpdateError::Manifest)? else {
            return Ok(None);
        };
        let choices = Choices::parse(&read_or_empty(&self.sys.apt_choices())?);
        let record = Record::parse(&read_or_empty(&self.sys.apt_record())?);
        let arch = self
            .quiet(&Cmd::user("dpkg").arg("--print-architecture"))
            .map_err(UpdateError::step("reading the architecture"))?
            .trim()
            .to_owned();
        let dpkg = self.dpkg()?;
        Ok(Some(AptState {
            manifest,
            choices,
            record,
            arch,
            dpkg,
        }))
    }

    fn dpkg(&mut self) -> Result<Dpkg, UpdateError> {
        let out = self
            .quiet(&Cmd::user("dpkg-query").args(dpkg::QUERY))
            .map_err(UpdateError::step("listing the installed packages"))?;
        Ok(Dpkg::parse(&out))
    }

    /// Which of `packages` apt has an install candidate for. A failed query counts as none.
    fn candidates(&mut self, packages: &[String]) -> BTreeSet<String> {
        if packages.is_empty() {
            return BTreeSet::new();
        }
        let policy = self
            .quiet(&Cmd::user("apt-cache").arg("policy").args(packages))
            .unwrap_or_default();
        dpkg::candidates(&policy)
    }

    /// Brings the apt packages in line with the checked-out release's manifest, as the running
    /// phase; `turning_off` is the group being turned off, whose packages may go too.
    pub(super) fn sync_manifest(&mut self, turning_off: Option<&str>) -> Result<Vec<GroupChange>, UpdateError> {
        let Some(mut state) = self.apt_state()? else {
            self.j.skip("this release has no package manifest");
            return Ok(Vec::new());
        };
        let mut changes = Changes::default();
        let mut record_changed = false;

        let installs = self.installable(&state)?;
        if !installs.is_empty() {
            self.install_wants(&mut state, &installs, &mut changes)?;
            record_changed = true;
        }
        self.hold(&state)?;
        let removed = self.remove_dropped(&state, turning_off)?;
        for package in &removed {
            let group = state.record.group_of(package).unwrap_or_default().to_owned();
            changes.entry(&group).removed.push(package.clone());
            state.record.remove(package);
            record_changed = true;
        }
        if record_changed {
            self.write_root(&self.sys.apt_record(), &state.record.to_text())?;
        }

        let changes = changes.into_vec();
        let installed: usize = changes.iter().map(|c| c.installed.len()).sum();
        let removed: usize = changes.iter().map(|c| c.removed.len()).sum();
        self.j.detail(match (installed, removed) {
            (0, 0) => "in line with the manifest".to_owned(),
            (i, 0) => format!("{i} installed"),
            (0, r) => format!("{r} removed"),
            (i, r) => format!("{i} installed, {r} removed"),
        });
        Ok(changes)
    }

    /// The wanted packages the machine lacks that apt can install, after setting up their
    /// groups' vendor repositories. One without a candidate is reported, unless its group takes
    /// only what apt has.
    fn installable(&mut self, state: &AptState) -> Result<Vec<Want>, UpdateError> {
        let wants = plan::missing(&state.facts());
        if wants.is_empty() {
            return Ok(Vec::new());
        }
        let repos = plan::repos_for(&state.manifest, &wants);
        if !repos.is_empty() {
            self.j
                .step(format!("Setting up apt repositories: {}", repos.join(", ")));
            self.run(&Cmd::root(self.sys.apt_repo_script()).args(&repos))
                .map_err(UpdateError::step("setting up the apt repositories"))?;
        }
        let packages: Vec<String> = wants.iter().map(|w| w.package.clone()).collect();
        let candidates = self.candidates(&packages);
        let mut installs = Vec::new();
        for want in wants {
            if candidates.contains(&want.package) {
                installs.push(want);
            } else if !plan::quietly_unavailable(&state.manifest, &want) {
                self.j.warn(format!(
                    "apt has no {} (group {}): not installed",
                    want.package, want.group
                ));
            }
        }
        Ok(installs)
    }

    /// Installs `installs`, and records each one dpkg then lists.
    fn install_wants(
        &mut self,
        state: &mut AptState,
        installs: &[Want],
        changes: &mut Changes,
    ) -> Result<(), UpdateError> {
        let held: Vec<&str> = installs
            .iter()
            .map(|w| w.package.as_str())
            .filter(|p| state.dpkg.held_missing(p))
            .collect();
        if !held.is_empty() {
            self.quiet(&Cmd::root("apt-mark").arg("unhold").args(&held))
                .map_err(UpdateError::step("unholding the packages to install"))?;
        }
        let packages: Vec<&str> = installs.iter().map(|w| w.package.as_str()).collect();
        let cmd = Cmd::root("apt-get")
            .args(["install", "-y", "--no-install-recommends"])
            .args(DPKG_KEEP_CONFIG)
            .args(&packages)
            .env("DEBIAN_FRONTEND", "noninteractive");
        self.j.step(format!("Installing {}", packages.join(", ")));
        self.j.count("Setting up ", packages.len());
        self.run(&cmd)
            .map_err(UpdateError::step("installing the package manifest's packages"))?;
        state.dpkg = self.dpkg()?;
        for want in installs {
            if state.dpkg.has(&want.package) {
                state.record.add(&want.package, &want.group);
                changes.entry(&want.group).installed.push(want.package.clone());
            } else {
                self.j
                    .warn(format!("apt-get installed {}, but dpkg doesn't list it", want.package));
            }
        }
        Ok(())
    }

    /// Holds the held groups' installed packages that aren't held yet.
    fn hold(&mut self, state: &AptState) -> Result<(), UpdateError> {
        let hold = plan::to_hold(&state.facts());
        if hold.is_empty() {
            return Ok(());
        }
        self.j.step(format!("Holding {}", hold.join(", ")));
        self.quiet(&Cmd::root("apt-mark").arg("hold").args(&hold))
            .map(drop)
            .map_err(UpdateError::step("holding the held groups' packages"))
    }

    /// Removes what `plan::removable` allows and nothing else would go with; the names removed.
    fn remove_dropped(&mut self, state: &AptState, turning_off: Option<&str>) -> Result<Vec<String>, UpdateError> {
        let removable = plan::removable(&state.facts(), turning_off);
        if removable.is_empty() {
            return Ok(Vec::new());
        }
        let allowed: BTreeSet<String> = removable.iter().cloned().collect();
        let mut remove = Vec::new();
        if self.removes_only(&removable, &allowed)? {
            remove = removable;
        } else {
            for package in removable {
                if self.removes_only(std::slice::from_ref(&package), &allowed)? {
                    remove.push(package);
                } else {
                    self.j.warn(format!(
                        "kept {package}: removing it would take packages along that are still wanted or yours"
                    ));
                }
            }
        }
        if remove.is_empty() {
            return Ok(remove);
        }
        let cmd = Cmd::root("apt-get")
            .args(["remove", "-y"])
            .args(DPKG_KEEP_CONFIG)
            .args(&remove)
            .env("DEBIAN_FRONTEND", "noninteractive");
        self.j.step(format!("Removing {}", remove.join(", ")));
        self.run(&cmd)
            .map_err(UpdateError::step("removing the packages the manifest dropped"))?;
        Ok(remove)
    }

    /// Whether removing `packages` would remove nothing outside `allowed`, by apt's simulation.
    fn removes_only(&mut self, packages: &[String], allowed: &BTreeSet<String>) -> Result<bool, UpdateError> {
        let simulated = self
            .quiet(&Cmd::user("apt-get").args(["-s", "-q", "remove"]).args(packages))
            .map_err(UpdateError::step("simulating the removal of dropped packages"))?;
        Ok(dpkg::removals(&simulated).is_subset(allowed))
    }
}
