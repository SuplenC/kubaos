//! A fake machine for the tests: a folder for its files, the rest (dpkg, the groups, the units,
//! logind, the User flake) in memory, shared by the user's side and root's, and a snapshot of it
//! all to compare.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use super::Host;
use super::HostError;
use super::UnitState;
use crate::apps::action::Scope;

/// What the fake machine has besides its files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// Package to version.
    pub dpkg: BTreeMap<String, String>,
    pub repos: BTreeSet<String>,
    /// A download's URL to its sha256 and its bytes. A `.deb`'s bytes are `deb <package>
    /// <version>`.
    pub downloads: BTreeMap<String, (String, Vec<u8>)>,
    pub nix: BTreeSet<String>,
    /// Group to its members.
    pub groups: BTreeMap<String, BTreeSet<String>>,
    /// `<scope>:<unit>` to its state.
    pub units: BTreeMap<String, UnitState>,
    pub linger: bool,
    /// Operations that fail: `enable:<unit>`, `apt:<package>`, `fetch:<url>`.
    pub fail: BTreeSet<String>,
    /// Every change made, in order.
    pub log: Vec<String>,
}

/// One machine, as the user or as root; clones share it.
#[derive(Clone, Debug)]
pub struct FakeHost {
    pub state: Rc<RefCell<State>>,
    home: PathBuf,
    system: PathBuf,
}

impl FakeHost {
    /// A machine whose home is `<dir>/home/kuba` and whose system is under `<dir>/system`.
    ///
    /// # Panics
    ///
    /// When the folders can't be made.
    pub fn new(dir: &Path) -> Self {
        let home = dir.join("home/kuba");
        let system = dir.join("system");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(system.join("etc")).unwrap();
        Self {
            state: Rc::new(RefCell::new(State::default())),
            home,
            system,
        }
    }

    /// Everything on the machine as text: its files, then its state but the log and the vendor
    /// repositories, which an `apt` action sets up and leaves, as they are the package manifest's.
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
        let state = State {
            log: Vec::new(),
            repos: BTreeSet::new(),
            fail: BTreeSet::new(),
            ..self.state.borrow().clone()
        };
        let _ = write!(out, "{state:#?}");
        out
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
        "amd64"
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

    fn version(&mut self, package: &str) -> Result<Option<String>, HostError> {
        Ok(self.state.borrow().dpkg.get(package).cloned())
    }

    fn apt_install(&mut self, repo: Option<&str>, packages: &[String]) -> Result<(), HostError> {
        for p in packages {
            self.fails(&format!("apt:{p}"))?;
        }
        let mut state = self.state.borrow_mut();
        if let Some(repo) = repo {
            state.repos.insert(repo.to_owned());
        }
        for p in packages {
            state.dpkg.insert(p.clone(), "1.0".to_owned());
        }
        state.log.push(format!("apt install {}", packages.join(" ")));
        Ok(())
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
        self.state
            .borrow_mut()
            .dpkg
            .insert(package.to_owned(), version.to_owned());
        self.log(format!("deb install {package}"));
        Ok(())
    }

    fn apt_remove(&mut self, packages: &[String]) -> Result<(), HostError> {
        let mut state = self.state.borrow_mut();
        for p in packages {
            state.dpkg.remove(p);
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

    fn unit_state(&mut self, scope: Scope, unit: &str) -> Result<UnitState, HostError> {
        let key = Self::unit(scope, unit);
        Ok(self.state.borrow().units.get(&key).copied().unwrap_or_default())
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
        self.state.borrow_mut().units.entry(key).or_default().active = active;
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

    fn run(&mut self, argv: &[String], dir: &Path) -> Result<(), HostError> {
        let (program, rest) = argv.split_first().unwrap();
        let program = match program.strip_prefix("./") {
            Some(script) => dir.join(script),
            None => PathBuf::from(program),
        };
        let output = Command::new(&program)
            .args(rest)
            .current_dir(dir)
            .env("HOME", &self.home)
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
