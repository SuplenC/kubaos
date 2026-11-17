//! A fake machine for the tests: a folder for its files, the rest (dpkg, the groups, the units,
//! logind, the User flake) in memory, shared by the user's side and root's, and a snapshot of it
//! all to compare. What a `run` starts runs in the machine's [`Sandbox`], never on the real one.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use super::AptPlan;
use super::AptSimulation;
use super::Compression;
use super::Downgrade;
use super::Host;
use super::HostError;
use super::UnitState;
use super::untar_args;
use crate::apps::action::Scope;
use crate::apps::action::Side;
use crate::apps::engine::Keeper;
use crate::apps::engine::Undone;
use crate::apps::root::Reply;
use crate::apps::root::Request;
use crate::apps::root::RootError;
use crate::apps::root::RootSide;
use crate::apps::root::Server;
use crate::apps::version::compare;
use crate::apt::manifest::Repo;
use crate::sandbox;
use crate::sandbox::Sandbox;
use crate::unfree::Unfree;

/// Root's side of the fake machine, its requests sent as JSON lines through the real server.
#[derive(Debug)]
pub struct FakeRoot {
    pub host: FakeHost,
    pub catalog: PathBuf,
    pub journals: PathBuf,
    pub owner: u32,
    pub declined: bool,
    pub started: bool,
    pub requests: Vec<String>,
    /// `start` and each request's line, shared with whoever made it, which outlives it.
    pub calls: Rc<RefCell<Vec<String>>>,
    /// What root's keeper undid in the run, kept between requests as the helper's one keeper
    /// keeps it.
    pub undone: Vec<Undone>,
}

impl RootSide for FakeRoot {
    fn start(&mut self) -> Result<(), RootError> {
        self.calls.borrow_mut().push("start".to_owned());
        if self.declined {
            return Err(RootError::Declined);
        }
        self.started = true;
        Ok(())
    }

    fn call(&mut self, request: &Request) -> Result<Reply, RootError> {
        assert!(self.started, "a request before the root helper started");
        let line = serde_json::to_string(request).unwrap();
        self.requests.push(line.clone());
        self.calls.borrow_mut().push(line.clone());
        Ok(self.serve(&line))
    }
}

impl FakeRoot {
    pub fn serve(&mut self, line: &str) -> Reply {
        let keeper = Keeper::new(&mut self.host, Side::Root, &self.catalog, &self.journals)
            .with_undone(std::mem::take(&mut self.undone));
        let mut server = Server::new(keeper, &self.catalog, self.owner);
        let reply = server.handle(line);
        self.undone = server.into_keeper().take_undone();
        reply
    }
}

/// What the fake machine has besides its files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// Package to version.
    pub dpkg: BTreeMap<String, String>,
    /// Package to the version apt would install.
    pub candidates: BTreeMap<String, String>,
    pub repos: BTreeSet<String>,
    /// Repositories set up as files, as the real machine sets them up: to its manifest entry and
    /// its sources file. Setting one up writes the file and its keyrings, and it is ready by
    /// [`Repo::in_place`] on the machine's files; the rest are ready once in `repos`.
    pub repo_files: BTreeMap<String, (Repo, String)>,
    /// A download's URL to its sha256 and its bytes. A `.deb`'s bytes are `deb <package>
    /// <version>`.
    pub downloads: BTreeMap<String, (String, Vec<u8>)>,
    pub nix: BTreeSet<String>,
    /// Whether the user's nixpkgs takes unfree packages.
    pub unfree: Unfree,
    /// Group to its members.
    pub groups: BTreeMap<String, BTreeSet<String>>,
    /// `<scope>:<unit>` to its state.
    pub units: BTreeMap<String, Unit>,
    pub linger: bool,
    /// The architectures dpkg takes packages of beside the machine's.
    pub foreign: BTreeSet<String>,
    /// Package to the installed packages installing it removes, as Docker's `docker-ce` removes
    /// Ubuntu's `docker.io`.
    pub conflicts: BTreeMap<String, BTreeSet<String>>,
    /// Package to the packages it depends on: installing it installs those that aren't there, and
    /// removing one of those removes it.
    pub depends: BTreeMap<String, BTreeSet<String>>,
    /// Packages apt marks installed by hand, as `apt install` on one makes it.
    pub manual: BTreeSet<String>,
    /// Package to the group its preinst makes on install, empties and puts one user in, a guess of
    /// its own rather than the machine's user, as the `nordvpn` package does: the group and the
    /// user.
    pub preinst_groups: BTreeMap<String, (String, String)>,
    /// Package to its units as its postinst leaves them on a first install (`<scope>:<unit>` to
    /// the unit), as openssh-server's enables and starts `ssh.socket`; removing the package takes
    /// them, as its unit files go with it.
    pub package_units: BTreeMap<String, BTreeMap<String, Unit>>,
    /// Operations that fail: `enable:<unit>`, `apt:<package>`, `fetch:<url>`, `groupdel:<group>`.
    pub fail: BTreeSet<String>,
    /// The machine is arm64's, not amd64's.
    pub arm64: bool,
    /// Vendor repositories whose key nears its expiry, until a setup of the repository fetches
    /// it again.
    pub keys_due: BTreeSet<String>,
    /// Lines the keys' check prints besides the repositories it was asked about.
    pub keys_stray: BTreeSet<String>,
    /// Every change made, in order.
    pub log: Vec<String>,
}

impl State {
    /// Refuses, as apt does, a package of an architecture dpkg doesn't take: apt has no list of
    /// it, so it can't locate the package.
    fn knows(&self, packages: &[String]) -> Result<(), HostError> {
        match packages
            .iter()
            .find(|p| p.split_once(':').is_some_and(|(_, arch)| !self.foreign.contains(arch)))
        {
            Some(p) => Err(HostError::Failed {
                command: "apt-get install".to_owned(),
                code: Some(100),
                stderr: format!("E: Unable to locate package {p}"),
            }),
            None => Ok(()),
        }
    }

    /// The installed packages among `packages` and what they depend on, through what that
    /// depends on, whose candidate sorts below the version dpkg has: what apt would move down.
    fn downgraded(&self, packages: &[String]) -> BTreeSet<String> {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut next: Vec<String> = packages.to_vec();
        while let Some(package) = next.pop() {
            if !seen.insert(package.clone()) {
                continue;
            }
            next.extend(self.depends.get(&package).into_iter().flatten().cloned());
        }
        seen.into_iter()
            .filter(|p| {
                let (Some(installed), Some(candidate)) = (self.dpkg.get(p), self.candidates.get(p)) else {
                    return false;
                };
                compare(candidate, installed) == Ordering::Less
            })
            .collect()
    }

    /// `packages` and what they depend on, through what that depends on, that dpkg hasn't.
    fn missing(&self, packages: &[String]) -> BTreeSet<String> {
        let mut needed: BTreeSet<String> = BTreeSet::new();
        let mut next: Vec<String> = packages.to_vec();
        while let Some(package) = next.pop() {
            if self.dpkg.contains_key(&package) || !needed.insert(package.clone()) {
                continue;
            }
            next.extend(self.depends.get(&package).into_iter().flatten().cloned());
        }
        needed
    }
}

/// A unit as systemd keeps it, which the fake shows as `systemctl show` does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Each is a property systemd shows on its own."
)]
pub struct Unit {
    pub exists: bool,
    pub enabled: bool,
    pub active: bool,
    /// `Type=oneshot`: started, it runs and exits, never staying active, unless it remains after
    /// exit.
    pub oneshot: bool,
    pub remain_after_exit: bool,
    /// Its main process exited cleanly since the machine started: a oneshot that ran, or any
    /// service that was stopped.
    pub exited: bool,
}

impl Unit {
    /// What `systemctl show` says of it.
    fn show(self) -> String {
        let yes = |b: bool| if b { "yes" } else { "no" };
        format!(
            "LoadState={}\nUnitFileState={}\nActiveState={}\nResult=success\n\
             ExecMainExitTimestampMonotonic={}\nType={}\nRemainAfterExit={}\n",
            if self.exists { "loaded" } else { "not-found" },
            if self.enabled { "enabled" } else { "disabled" },
            if self.active { "active" } else { "inactive" },
            if self.exited { 1_000_000 } else { 0 },
            if self.oneshot { "oneshot" } else { "simple" },
            yes(self.remain_after_exit),
        )
    }
}

/// One machine, as the user or as root; clones share it.
#[derive(Clone, Debug)]
pub struct FakeHost {
    pub state: Rc<RefCell<State>>,
    home: PathBuf,
    system: PathBuf,
    sandbox: Sandbox,
}

impl FakeHost {
    /// A machine whose home is `<dir>/home/kuba`, whose system is under `<dir>/system`, and whose
    /// programs run in a sandbox in `<dir>/sandbox`.
    ///
    /// # Panics
    ///
    /// When the folders can't be made.
    pub fn new(dir: &Path) -> Self {
        let home = dir.join("home/kuba");
        let system = dir.join("system");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(system.join("etc")).unwrap();
        let sandbox = Sandbox::new(&dir.join("sandbox"), &home, "kuba");
        Self {
            state: Rc::new(RefCell::new(State::default())),
            home,
            system,
            sandbox,
        }
    }

    /// Where what it runs runs: its stand-ins for the machine's commands, and their log.
    pub fn sandbox(&self) -> &Sandbox {
        &self.sandbox
    }

    /// Everything on the machine as text: its files, then its state but the log, the vendor
    /// repositories, which an `apt` action sets up and leaves, as they are the package manifest's,
    /// and whether a unit's main process exited since boot, which systemd keeps and no stop undoes.
    ///
    /// # Panics
    ///
    /// When a file can't be read.
    pub fn snapshot(&self) -> String {
        let mut out = String::new();
        for (label, root) in [("home", &self.home), ("system", &self.system)] {
            let mut paths = Vec::new();
            walk(root, &mut paths);
            paths.sort();
            for path in paths {
                let rel = path.strip_prefix(root).unwrap().display();
                let meta = fs::symlink_metadata(&path).unwrap();
                if meta.file_type().is_symlink() {
                    let _ = writeln!(out, "{label} link {rel} -> {}", fs::read_link(&path).unwrap().display());
                } else if meta.is_dir() {
                    let _ = writeln!(out, "{label} dir {rel}");
                } else {
                    let mode = meta.permissions().mode() & 0o777;
                    let text = String::from_utf8_lossy(&fs::read(&path).unwrap()).into_owned();
                    let _ = writeln!(out, "{label} file {rel} {mode:o} {text:?}");
                }
            }
        }
        let units = self
            .state
            .borrow()
            .units
            .iter()
            .map(|(key, unit)| {
                let unit = Unit { exited: false, ..*unit };
                (key.clone(), unit)
            })
            // A unit nothing has, neither loaded, enabled nor running, is no unit.
            .filter(|(_, unit)| *unit != Unit::default())
            .collect();
        let state = State {
            units,
            log: Vec::new(),
            repos: BTreeSet::new(),
            repo_files: BTreeMap::new(),
            fail: BTreeSet::new(),
            conflicts: BTreeMap::new(),
            depends: BTreeMap::new(),
            preinst_groups: BTreeMap::new(),
            package_units: BTreeMap::new(),
            ..self.state.borrow().clone()
        };
        let _ = write!(out, "{state:#?}");
        out
    }

    /// The machine's units where a test's `systemctl` stand-in can read them, `units` beside the
    /// sandbox's log: a line a unit, `<scope>:<unit> enabled|disabled active|inactive`, as a
    /// script a `run` starts sees the same machine the engine does.
    fn show_units(&self) {
        let mut text = String::new();
        for (key, unit) in &self.state.borrow().units {
            let enabled = if unit.enabled { "enabled" } else { "disabled" };
            let active = if unit.active { "active" } else { "inactive" };
            let _ = writeln!(text, "{key} {enabled} {active}");
        }
        fs::write(self.sandbox.log().with_file_name("units"), text).unwrap();
    }

    fn fails(&self, op: &str) -> Result<(), HostError> {
        if self.state.borrow().fail.contains(op) {
            return Err(HostError::Failed {
                command: op.to_owned(),
                code: Some(1),
                stderr: "the fake machine fails it".to_owned(),
            });
        }
        Ok(())
    }

    fn log(&self, line: impl Into<String>) {
        self.state.borrow_mut().log.push(line.into());
    }

    fn unit(scope: Scope, unit: &str) -> String {
        format!("{}:{unit}", scope.word())
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        out.push(path.clone());
        if fs::symlink_metadata(&path).unwrap().is_dir() {
            walk(&path, out);
        }
    }
}

impl Host for FakeHost {
    fn user(&self) -> &'static str {
        "kuba"
    }

    fn home(&self) -> &Path {
        &self.home
    }

    fn arch(&self) -> &'static str {
        if self.state.borrow().arm64 { "arm64" } else { "amd64" }
    }

    fn place(&self, path: &Path) -> PathBuf {
        if path.starts_with(&self.home) {
            path.to_path_buf()
        } else {
            self.system.join(path.strip_prefix("/").unwrap_or(path))
        }
    }

    fn installed(&mut self, packages: &[String]) -> Result<BTreeSet<String>, HostError> {
        let state = self.state.borrow();
        Ok(packages
            .iter()
            .filter(|p| state.dpkg.contains_key(*p))
            .cloned()
            .collect())
    }

    fn manual(&mut self, packages: &[String]) -> Result<BTreeSet<String>, HostError> {
        let state = self.state.borrow();
        Ok(packages
            .iter()
            .filter(|p| state.dpkg.contains_key(*p) && state.manual.contains(*p))
            .cloned()
            .collect())
    }

    fn version(&mut self, package: &str) -> Result<Option<String>, HostError> {
        Ok(self.state.borrow().dpkg.get(package).cloned())
    }

    fn candidate(&mut self, package: &str) -> Result<Option<String>, HostError> {
        Ok(self.state.borrow().candidates.get(package).cloned())
    }

    fn apt_repo(&mut self, repo: &str) -> Result<(), HostError> {
        self.fails(&format!("apt-repo:{repo}"))?;
        self.state.borrow_mut().repos.insert(repo.to_owned());
        self.state.borrow_mut().keys_due.remove(repo);
        if let Some((manifest, text)) = self.state.borrow().repo_files.get(repo) {
            let sources = self.place(Path::new(&format!("/etc/apt/sources.list.d/{repo}.sources")));
            for (path, text) in std::iter::once((sources, text.as_str()))
                .chain(manifest.keyrings().map(|k| (self.place(Path::new(k)), "key")))
            {
                fs::create_dir_all(path.parent().unwrap()).map_err(HostError::io(&path))?;
                fs::write(&path, text).map_err(HostError::io(&path))?;
            }
        }
        Ok(())
    }

    fn apt_repo_ready(&mut self, repo: &str) -> Result<bool, HostError> {
        let state = self.state.borrow();
        Ok(match state.repo_files.get(repo) {
            Some((manifest, _)) => manifest.in_place(repo, &self.system),
            None => state.repos.contains(repo),
        })
    }

    fn apt_keys_due(&mut self, repos: &[String]) -> Result<BTreeSet<String>, HostError> {
        let state = self.state.borrow();
        let due = repos.iter().filter(|r| state.keys_due.contains(*r));
        Ok(due.chain(&state.keys_stray).cloned().collect())
    }

    fn apt_simulate(&mut self, plan: AptPlan<'_>) -> Result<AptSimulation, HostError> {
        let state = self.state.borrow();
        let installing: Vec<String> = match plan {
            AptPlan::Install(packages) => packages.to_vec(),
            AptPlan::Deb(deb) => {
                let text = fs::read_to_string(deb).map_err(HostError::io(deb))?;
                text.split_whitespace().nth(1).map(str::to_owned).into_iter().collect()
            }
            AptPlan::Remove(packages) => {
                let mut gone: BTreeSet<String> = packages.iter().cloned().collect();
                loop {
                    let more: Vec<String> = state
                        .dpkg
                        .keys()
                        .filter(|p| !gone.contains(*p))
                        .filter(|p| {
                            state
                                .depends
                                .get(*p)
                                .is_some_and(|d| d.iter().any(|d| gone.contains(d)))
                        })
                        .cloned()
                        .collect();
                    if more.is_empty() {
                        break;
                    }
                    gone.extend(more);
                }
                return Ok(AptSimulation {
                    removes: gone.into_iter().filter(|p| state.dpkg.contains_key(p)).collect(),
                    ..AptSimulation::default()
                });
            }
        };
        state.knows(&installing)?;
        Ok(AptSimulation {
            removes: installing
                .iter()
                .filter_map(|p| state.conflicts.get(p))
                .flatten()
                .filter(|p| state.dpkg.contains_key(*p))
                .cloned()
                .collect(),
            installs: state.missing(&installing),
            downgrades: state.downgraded(&installing),
        })
    }

    fn apt_install(&mut self, packages: &[String], downgrade: Downgrade) -> Result<(), HostError> {
        for p in packages {
            self.fails(&format!("apt:{p}"))?;
        }
        self.state.borrow().knows(packages)?;
        let mut state = self.state.borrow_mut();
        // apt moves an installed package asked for by name to its candidate, as it does a
        // distribution's build of a name a pinned repository also has; `-y` refuses to move one
        // down unless allowed, before it changes anything.
        let moving: Vec<(String, String)> = packages
            .iter()
            .filter_map(|p| Some((p.clone(), state.dpkg.get(p)?.clone(), state.candidates.get(p)?.clone())))
            .map(|(p, installed, candidate)| {
                if downgrade == Downgrade::Refuse && compare(&candidate, &installed) == Ordering::Less {
                    Err(HostError::Failed {
                        command: "apt-get install".to_owned(),
                        code: Some(100),
                        stderr: "E: Packages were downgraded and -y was used without --allow-downgrades.".to_owned(),
                    })
                } else {
                    Ok((p, candidate))
                }
            })
            .collect::<Result<_, _>>()?;
        for p in packages {
            let conflicts = state.conflicts.get(p).cloned().unwrap_or_default();
            for gone in conflicts {
                state.dpkg.remove(&gone);
            }
        }
        for p in state.missing(packages) {
            if let Some((group, user)) = state.preinst_groups.get(&p).cloned() {
                state.groups.insert(group, BTreeSet::from([user]));
            }
            let units = state.package_units.get(&p).cloned().unwrap_or_default();
            state.units.extend(units);
            let version = state.candidates.get(&p).cloned().unwrap_or_else(|| "1.0".to_owned());
            state.dpkg.insert(p, version);
        }
        state.dpkg.extend(moving);
        state.log.push(format!("apt install {}", packages.join(" ")));
        Ok(())
    }

    fn untar(&mut self, archive: &Path, into: &Path, compression: Compression, strip: u32) -> Result<(), HostError> {
        let output = self
            .sandbox
            .command("tar")
            .args(untar_args(archive, into, compression, strip))
            .output()
            .map_err(|source| HostError::Spawn {
                program: "tar".to_owned(),
                source,
            })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(HostError::Failed {
                command: format!("tar -x {}", archive.display()),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }

    fn deb_install(&mut self, deb: &Path) -> Result<(), HostError> {
        let text = fs::read_to_string(deb).map_err(HostError::io(deb))?;
        let mut words = text.split_whitespace().skip(1);
        let (Some(package), Some(version)) = (words.next(), words.next()) else {
            return Err(HostError::Failed {
                command: "apt-get install".to_owned(),
                code: Some(100),
                stderr: "not a .deb".to_owned(),
            });
        };
        let mut state = self.state.borrow_mut();
        for dependency in state.missing(&[package.to_owned()]) {
            state.dpkg.insert(dependency, "1.0".to_owned());
        }
        state.dpkg.insert(package.to_owned(), version.to_owned());
        state.log.push(format!("deb install {package} {version}"));
        Ok(())
    }

    fn apt_remove(&mut self, packages: &[String]) -> Result<(), HostError> {
        // As apt does with dpkg's lock held: it fails before it changes anything.
        for p in packages {
            self.fails(&format!("apt-remove:{p}"))?;
        }
        let gone = self.apt_simulate(AptPlan::Remove(packages))?.removes;
        let mut state = self.state.borrow_mut();
        for p in &gone {
            state.dpkg.remove(p);
            let units = state.package_units.get(p).cloned().unwrap_or_default();
            for unit in units.keys() {
                state.units.remove(unit);
            }
        }
        state.log.push(format!("apt remove {}", packages.join(" ")));
        Ok(())
    }

    fn fetch(&mut self, url: &str, sha256: &str, to: &Path) -> Result<(), HostError> {
        self.fails(&format!("fetch:{url}"))?;
        let (sha, bytes) = self
            .state
            .borrow()
            .downloads
            .get(url)
            .cloned()
            .ok_or_else(|| HostError::Failed {
                command: format!("curl {url}"),
                code: Some(22),
                stderr: "404".to_owned(),
            })?;
        if sha != sha256 {
            return Err(HostError::Sha256 {
                url: url.to_owned(),
                sha256: sha256.to_owned(),
            });
        }
        fs::write(to, bytes).map_err(HostError::io(to))?;
        self.log(format!("fetch {url}"));
        Ok(())
    }

    fn nix_list(&mut self) -> Result<BTreeSet<String>, HostError> {
        Ok(self.state.borrow().nix.clone())
    }

    fn unfree(&mut self) -> Result<Unfree, HostError> {
        Ok(self.state.borrow().unfree)
    }

    fn nix_add(&mut self, names: &[String]) -> Result<(), HostError> {
        self.state.borrow_mut().nix.extend(names.iter().cloned());
        self.log(format!("nix add {}", names.join(" ")));
        Ok(())
    }

    fn nix_remove(&mut self, names: &[String]) -> Result<(), HostError> {
        let mut state = self.state.borrow_mut();
        for name in names {
            state.nix.remove(name);
        }
        Ok(())
    }

    fn group_exists(&mut self, group: &str) -> Result<bool, HostError> {
        Ok(self.state.borrow().groups.contains_key(group))
    }

    fn member(&mut self, group: &str) -> Result<bool, HostError> {
        Ok(self
            .state
            .borrow()
            .groups
            .get(group)
            .is_some_and(|m| m.contains("kuba")))
    }

    fn group_create(&mut self, group: &str) -> Result<(), HostError> {
        self.state.borrow_mut().groups.insert(group.to_owned(), BTreeSet::new());
        Ok(())
    }

    fn group_delete(&mut self, group: &str) -> Result<(), HostError> {
        self.fails(&format!("groupdel:{group}"))?;
        self.state.borrow_mut().groups.remove(group);
        Ok(())
    }

    fn set_member(&mut self, group: &str, member: bool) -> Result<(), HostError> {
        let mut state = self.state.borrow_mut();
        let members = state.groups.entry(group.to_owned()).or_default();
        if member {
            members.insert("kuba".to_owned());
        } else {
            members.remove("kuba");
        }
        Ok(())
    }

    fn unit_state(&mut self, scope: Scope, name: &str) -> Result<UnitState, HostError> {
        let key = Self::unit(scope, name);
        let mut unit = self.state.borrow().units.get(&key).copied().unwrap_or_default();
        // systemd loads a unit whose file is in its unit folder.
        let folder = match scope {
            Scope::System => self.place(Path::new("/etc/systemd/system")),
            Scope::User => self.home.join(".config/systemd/user"),
        };
        unit.exists = unit.exists || folder.join(name).is_file();
        Ok(UnitState::from_show(&unit.show()))
    }

    fn daemon_reload(&mut self, _scope: Scope) -> Result<(), HostError> {
        Ok(())
    }

    fn set_enabled(&mut self, scope: Scope, unit: &str, enabled: bool) -> Result<(), HostError> {
        if enabled {
            self.fails(&format!("enable:{unit}"))?;
        }
        let key = Self::unit(scope, unit);
        self.state.borrow_mut().units.entry(key).or_default().enabled = enabled;
        self.log(format!("enable {unit} {enabled}"));
        Ok(())
    }

    fn set_active(&mut self, scope: Scope, unit: &str, active: bool) -> Result<(), HostError> {
        let key = Self::unit(scope, unit);
        let mut state = self.state.borrow_mut();
        let unit = state.units.entry(key).or_default();
        if active && unit.oneshot && !unit.remain_after_exit {
            unit.exited = true;
        } else {
            unit.exited = unit.exited || unit.active;
            unit.active = active;
        }
        Ok(())
    }

    fn restart(&mut self, scope: Scope, unit: &str) -> Result<(), HostError> {
        let key = Self::unit(scope, unit);
        let mut state = self.state.borrow_mut();
        if let Some(unit) = state.units.get_mut(&key) {
            unit.exited = true;
        }
        state.log.push(format!("restart {unit}"));
        Ok(())
    }

    fn lingers(&mut self) -> Result<bool, HostError> {
        Ok(self.state.borrow().linger)
    }

    fn set_linger(&mut self, on: bool) -> Result<(), HostError> {
        self.state.borrow_mut().linger = on;
        self.log(format!("linger {on}"));
        Ok(())
    }

    fn foreign_arches(&mut self) -> Result<BTreeSet<String>, HostError> {
        Ok(self.state.borrow().foreign.clone())
    }

    fn set_foreign_arch(&mut self, arch: &str, on: bool) -> Result<(), HostError> {
        self.fails(&format!("arch:{arch}"))?;
        if !on && !self.arch_packages(arch)?.is_empty() {
            return Err(HostError::Failed {
                command: format!("dpkg --remove-architecture {arch}"),
                code: Some(2),
                stderr: format!("dpkg: error: cannot remove architecture '{arch}' currently in use by the database"),
            });
        }
        let mut state = self.state.borrow_mut();
        if on {
            state.foreign.insert(arch.to_owned());
        } else {
            state.foreign.remove(arch);
        }
        state.log.push(format!("arch {arch} {on}"));
        Ok(())
    }

    fn arch_packages(&mut self, arch: &str) -> Result<BTreeSet<String>, HostError> {
        let suffix = format!(":{arch}");
        Ok(self
            .state
            .borrow()
            .dpkg
            .keys()
            .filter(|p| p.ends_with(&suffix))
            .cloned()
            .collect())
    }

    fn apt_update(&mut self) -> Result<(), HostError> {
        self.fails("apt-update")?;
        self.log("apt update");
        Ok(())
    }

    fn run(&mut self, argv: &[String], dir: &Path) -> Result<(), HostError> {
        self.show_units();
        let (program, rest) = argv.split_first().unwrap();
        let program = match program.strip_prefix("./") {
            Some(script) => {
                // The engine writes a run's scripts on this test's thread, while other tests'
                // threads start processes.
                let script = dir.join(script);
                sandbox::settle(&script);
                script
            }
            None => PathBuf::from(program),
        };
        let output = self
            .sandbox
            .command(&program)
            .args(rest)
            .current_dir(dir)
            .output()
            .map_err(|source| HostError::Spawn {
                program: program.display().to_string(),
                source,
            })?;
        self.log(format!("run {}", argv.join(" ")));
        if output.status.success() {
            Ok(())
        } else {
            Err(HostError::Failed {
                command: argv.join(" "),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }
}

#[cfg(test)]
mod tests;
