//! A mise for the tests: it knows a few versions of each tool, installs and uninstalls them in
//! memory, reads the fragments in `conf.d/` and, in a folder, the `mise.toml` files of the folder
//! and those above it (`.mise.toml` and the `.local` ones too) as the real one reads its configs,
//! and keeps the global lockfile beside them in the shape mise writes (`lockfile_version = 4`).

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::mise::ListedVersion;
use crate::mise::Mise;
use crate::mise::MiseError;
use crate::mise::OutdatedTool;
use crate::mise::ToolRequest;

#[derive(Debug)]
pub(crate) struct FakeMise {
    conf_d: PathBuf,
    lockfile: PathBuf,
    /// The releases of each tool, oldest first.
    releases: BTreeMap<&'static str, Vec<&'static str>>,
    state: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    installed: BTreeSet<(String, String)>,
    /// A tool whose install fails, after the tools before it in the call installed.
    failing: Option<String>,
    /// Configs outside `conf.d/` that mise tracks, such as a project's `mise.toml`: prune keeps
    /// what they ask for, and nothing else here reads them, as nothing run from `/` does.
    tracked: Vec<PathBuf>,
    /// Every call, as `mise <args>`.
    calls: Vec<String>,
    /// Whether [`Mise::current`] fails.
    current_fails: bool,
}

impl FakeMise {
    /// A mise whose fragments are in `conf_d` and whose lockfile is `mise.lock` beside it.
    pub(crate) fn new(conf_d: PathBuf) -> Self {
        let lockfile = conf_d
            .parent()
            .map_or_else(|| conf_d.join(".."), Path::to_path_buf)
            .join("mise.lock");
        Self {
            conf_d,
            lockfile,
            releases: BTreeMap::from([
                ("demo-lang", vec!["1.0.0", "1.0.3", "2.0.1", "2.1.0", "3.0.2", "4.0.0"]),
                ("demo-ls", vec!["0.4.1", "0.5.0", "0.5.2", "0.6.1"]),
                ("demo-fmt", vec!["3.1.0"]),
                ("demo-nightly", vec!["nightly"]),
                ("kubaos-test-tool", vec!["1.0.0", "1.0.1", "2.0.0", "3.0.0"]),
                ("go", vec!["1.25.9", "1.26.10", "1.27.3"]),
                ("go:golang.org/x/tools/gopls", vec!["0.24.0"]),
                ("go:golang.org/x/tools/cmd/goimports", vec!["0.52.0"]),
                ("aqua:mvdan/gofumpt", vec!["0.12.0"]),
                ("aqua:golangci/golangci-lint", vec!["2.15.0"]),
                ("aqua:go-delve/delve", vec!["1.27.2"]),
                ("node", vec!["22.23.4", "24.22.2", "26.13.0"]),
                ("npm:@vtsls/language-server", vec!["0.3.0"]),
                ("bun", vec!["1.4.4"]),
            ]),
            state: Mutex::default(),
        }
    }

    pub(crate) fn fail_on(&self, tool: &str) {
        self.state().failing = Some(tool.to_owned());
    }

    /// Installs a version as if the user had installed it themselves.
    pub(crate) fn preinstall(&self, tool: &str, version: &str) {
        self.state().installed.insert((tool.to_owned(), version.to_owned()));
    }

    /// Tracks a config outside `conf.d/`, as mise tracks a project's once it has read it there.
    pub(crate) fn track(&self, config: PathBuf) {
        self.state().tracked.push(config);
    }

    pub(crate) fn tracked(&self) -> Vec<PathBuf> {
        self.state().tracked.clone()
    }

    pub(crate) fn fail_current(&self) {
        self.state().current_fails = true;
    }

    pub(crate) fn installed(&self) -> Vec<String> {
        self.state().installed.iter().map(|(t, v)| format!("{t}@{v}")).collect()
    }

    pub(crate) fn calls(&self) -> Vec<String> {
        self.state().calls.clone()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("the fake mise's state")
    }

    /// The newest release of `tool` that `request` selects.
    fn resolve(&self, tool: &str, request: &str) -> Option<String> {
        self.releases
            .get(tool)?
            .iter()
            .rev()
            .find(|v| on(v, request))
            .map(|v| (*v).to_owned())
    }

    /// The version mise selects for `tool` at `request`: the lockfile's when it is on the
    /// request, else the newest.
    fn select(&self, tool: &str, request: &str) -> Option<String> {
        self.locked(tool)
            .filter(|v| on(v, request) && self.resolve(tool, v).is_some())
            .or_else(|| self.resolve(tool, request))
    }

    /// The version the lockfile names for `tool`.
    fn locked(&self, tool: &str) -> Option<String> {
        let doc = self.lock_doc()?;
        let entries = doc.get("tools")?.get(tool)?.as_array_of_tables()?;
        entries
            .iter()
            .find_map(|e| e.get("version")?.as_str().map(str::to_owned))
    }

    fn lock_doc(&self) -> Option<toml_edit::DocumentMut> {
        fs::read_to_string(&self.lockfile).ok()?.parse().ok()
    }

    /// Writes `tool`'s entry in the lockfile as mise does, keeping every other entry.
    fn write_lock_entry(doc: &mut toml_edit::DocumentMut, tool: &str, version: &str, request: &str) {
        let mut entry = toml_edit::Table::new();
        entry.insert("version", toml_edit::value(version));
        entry.insert("backend", toml_edit::value(format!("asdf:{tool}")));
        let mut specifiers = toml_edit::Array::new();
        specifiers.push(request);
        entry.insert("specifiers", toml_edit::value(specifiers));
        let mut entries = toml_edit::ArrayOfTables::new();
        entries.push(entry);
        let tools = doc
            .entry("tools")
            .or_insert_with(|| {
                let mut t = toml_edit::Table::new();
                t.set_implicit(true);
                toml_edit::Item::Table(t)
            })
            .as_table_mut()
            .expect("the lockfile's tools are a table");
        tools.insert(tool, toml_edit::Item::ArrayOfTables(entries));
    }

    /// Updates `tool`'s entry in the lockfile, when there is a lockfile: `mise upgrade` keeps it
    /// current.
    fn relock(&self, tool: &str, version: &str, request: &str) {
        if let Some(mut doc) = self.lock_doc() {
            Self::write_lock_entry(&mut doc, tool, version, request);
            fs::write(&self.lockfile, doc.to_string()).expect("write the lockfile");
        }
    }

    /// Every tool the fragments ask for, with the fragment that asks.
    fn requested(&self) -> Vec<(String, String, PathBuf)> {
        let Ok(entries) = fs::read_dir(&self.conf_d) else {
            return Vec::new();
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        paths.iter().flat_map(|path| tools_of(path)).collect()
    }

    fn record(&self, call: String) {
        self.state().calls.push(call);
    }

    /// Installs `version` of `tool`, or fails as a failing tool does.
    fn put(&self, command: &str, tool: &str, version: Option<String>) -> Result<String, MiseError> {
        let failing = self.state().failing.as_deref() == Some(tool);
        let (Some(version), false) = (version, failing) else {
            return Err(MiseError::Failed {
                command: format!("{command} {tool}"),
                status: "exit status: 1".into(),
                message: format!("no version of {tool}"),
            });
        };
        self.state().installed.insert((tool.to_owned(), version.clone()));
        Ok(version)
    }
}

impl Mise for FakeMise {
    fn install(&self, tools: &[ToolRequest]) -> Result<(), MiseError> {
        self.record(format!("mise install {}", joined(tools)));
        for tool in tools {
            self.put("install", &tool.tool, self.resolve(&tool.tool, &tool.request))?;
        }
        Ok(())
    }

    fn uninstall(&self, versions: &[ToolRequest]) -> Result<(), MiseError> {
        self.record(format!("mise uninstall {}", joined(versions)));
        let mut state = self.state();
        for v in versions {
            state.installed.remove(&(v.tool.clone(), v.request.clone()));
        }
        Ok(())
    }

    fn list(&self) -> Result<Vec<ListedVersion>, MiseError> {
        self.record("mise ls --json".into());
        let installed = self.state().installed.clone();
        let mut listed: Vec<ListedVersion> = Vec::new();
        for (tool, request, path) in self.requested() {
            let version = self.select(&tool, &request).unwrap_or_else(|| request.clone());
            listed.push(ListedVersion {
                installed: installed.contains(&(tool.clone(), version.clone())),
                tool,
                version,
                requested: Some(request),
                source: Some(path),
            });
        }
        for (tool, version) in installed {
            if !listed.iter().any(|l| l.tool == tool && l.version == version) {
                listed.push(ListedVersion {
                    tool,
                    version,
                    requested: None,
                    source: None,
                    installed: true,
                });
            }
        }
        Ok(listed)
    }

    fn outdated(&self, bump: bool) -> Result<Vec<OutdatedTool>, MiseError> {
        self.record(
            if bump {
                "mise outdated --bump --json"
            } else {
                "mise outdated --json"
            }
            .into(),
        );
        let installed = self.state().installed.clone();
        // A config read later wins, as `conf.d/` loads in name order: one entry a tool.
        let winners: BTreeMap<String, (String, PathBuf)> = self
            .requested()
            .into_iter()
            .map(|(tool, request, path)| (tool, (request, path)))
            .collect();
        let mut outdated = Vec::new();
        for (tool, (request, path)) in winners {
            let current = self
                .select(&tool, &request)
                .filter(|v| installed.contains(&(tool.clone(), v.clone())));
            let within = self.resolve(&tool, &request);
            let newest = self
                .releases
                .get(tool.as_str())
                .and_then(|r| r.last())
                .map(|v| (*v).to_owned());
            let (latest, bump_to) = if bump {
                let Some(newest) = newest else { continue };
                let precision = request.split('.').count();
                let to =
                    (!on(&newest, &request)).then(|| newest.split('.').take(precision).collect::<Vec<_>>().join("."));
                (newest, to)
            } else {
                let Some(within) = within else { continue };
                (within, None)
            };
            if current.as_deref() == Some(latest.as_str()) {
                continue;
            }
            outdated.push(OutdatedTool {
                tool,
                requested: request,
                current,
                latest,
                bump: bump_to,
                source: Some(path),
            });
        }
        Ok(outdated)
    }

    fn latest(&self, tool: &str, prefix: &str) -> Result<Option<String>, MiseError> {
        self.record(format!("mise latest {tool}@{prefix}"));
        Ok(self.resolve(tool, prefix))
    }

    fn upgrade(&self, tools: &[String]) -> Result<(), MiseError> {
        self.record(format!("mise upgrade {}", tools.join(" ")));
        let requested = self.requested();
        for tool in tools {
            let Some((_, request, _)) = requested.iter().find(|(t, _, _)| t == tool) else {
                continue;
            };
            let version = self.put("upgrade", tool, self.resolve(tool, request))?;
            self.relock(tool, &version, request);
        }
        Ok(())
    }

    fn bump(&self, targets: &[ToolRequest]) -> Result<(), MiseError> {
        self.record(format!("mise upgrade --bump {}", joined(targets)));
        let requested = self.requested();
        for target in targets {
            let Some((_, _, path)) = requested.iter().find(|(t, _, _)| *t == target.tool) else {
                continue;
            };
            let version = self.put(
                "upgrade --bump",
                &target.tool,
                self.resolve(&target.tool, &target.request),
            )?;
            let mut doc: toml_edit::DocumentMut = fs::read_to_string(path).unwrap().parse().unwrap();
            doc["tools"][target.tool.as_str()] = toml_edit::value(target.request.as_str());
            fs::write(path, doc.to_string()).unwrap();
            self.relock(&target.tool, &version, &target.request);
        }
        Ok(())
    }

    fn install_locked(&self) -> Result<(), MiseError> {
        self.record("mise install --locked".into());
        if !self.lockfile.exists() {
            return Err(MiseError::Failed {
                command: "install --locked".into(),
                status: "exit status: 1".into(),
                message: "no lockfile".into(),
            });
        }
        for (tool, request, _) in self.requested() {
            self.put("install --locked", &tool, self.select(&tool, &request))?;
        }
        Ok(())
    }

    fn current(&self, folder: &Path) -> Result<Vec<ListedVersion>, MiseError> {
        self.record(format!("mise ls --current --json in {}", folder.display()));
        if self.state().current_fails {
            return Err(MiseError::Failed {
                command: "ls --current --json".into(),
                status: "exit status: 1".into(),
                message: "the fake fails".into(),
            });
        }
        let mut configs = Vec::new();
        for dir in folder.ancestors() {
            for name in ["mise.local.toml", ".mise.local.toml", "mise.toml", ".mise.toml"] {
                let config = dir.join(name);
                if config.is_file() {
                    configs.push(config);
                }
            }
        }
        {
            let mut state = self.state();
            for config in &configs {
                if !state.tracked.contains(config) {
                    state.tracked.push(config.clone());
                }
            }
        }
        let installed = self.state().installed.clone();
        let mut listed: Vec<ListedVersion> = Vec::new();
        let project = configs.iter().flat_map(|path| tools_of(path));
        for (tool, request, path) in project.chain(self.requested()) {
            if listed.iter().any(|l| l.tool == tool) {
                continue;
            }
            let version = self.select(&tool, &request).unwrap_or_else(|| request.clone());
            listed.push(ListedVersion {
                installed: installed.contains(&(tool.clone(), version.clone())),
                tool,
                version,
                requested: Some(request),
                source: Some(path),
            });
        }
        listed.sort_by(|a, b| a.tool.cmp(&b.tool));
        Ok(listed)
    }

    fn trust(&self, config: &Path) -> Result<(), MiseError> {
        self.record(format!("mise trust {}", config.display()));
        Ok(())
    }

    fn prune(&self) -> Result<(), MiseError> {
        self.record("mise prune --tools".into());
        let tracked: Vec<(String, String, PathBuf)> = self
            .state()
            .tracked
            .clone()
            .iter()
            .filter(|path| path.is_file())
            .flat_map(|path| tools_of(path))
            .collect();
        let selected: BTreeSet<(String, String)> = self
            .requested()
            .into_iter()
            .chain(tracked)
            .filter_map(|(tool, request, _)| self.select(&tool, &request).map(|v| (tool, v)))
            .collect();
        self.state().installed.retain(|v| selected.contains(v));
        Ok(())
    }

    fn lock(&self) -> Result<(), MiseError> {
        self.record("mise lock --global".into());
        let mut doc = self.lock_doc().unwrap_or_else(|| {
            "# @generated - this file is auto-generated by `mise lock`\n\nlockfile_version = 4\n"
                .parse()
                .unwrap()
        });
        for (tool, request, _) in self.requested() {
            if let Some(version) = self.select(&tool, &request) {
                Self::write_lock_entry(&mut doc, &tool, &version, &request);
            }
        }
        fs::write(&self.lockfile, doc.to_string()).expect("write the lockfile");
        Ok(())
    }
}

/// Whether `version` is `request` or within it: `1.0.3` is within `1` and `1.0`.
fn on(version: &str, request: &str) -> bool {
    version == request || version.starts_with(&format!("{request}."))
}

/// The tools the config at `path` asks for, with the config. A request that isn't a string is
/// left out, as KubaOS leaves it to the user.
fn tools_of(path: &Path) -> Vec<(String, String, PathBuf)> {
    let text = fs::read_to_string(path).expect("read a config");
    let table: toml::Table = toml::from_str(&text).expect("a config is TOML");
    table
        .get("tools")
        .and_then(|t| t.as_table())
        .into_iter()
        .flatten()
        .filter_map(|(tool, request)| Some((tool.clone(), request.as_str()?.to_owned(), path.to_path_buf())))
        .collect()
}

fn joined(tools: &[ToolRequest]) -> String {
    tools.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")
}
