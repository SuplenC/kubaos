//! A mise for the tests: it knows a few versions of each tool, installs and uninstalls them in
//! memory, and reads the fragments in `conf.d/` as the real one reads its configs.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
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
    /// The releases of each tool, oldest first.
    releases: BTreeMap<&'static str, Vec<&'static str>>,
    state: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    installed: BTreeSet<(String, String)>,
    /// A tool whose install fails, after the tools before it in the call installed.
    failing: Option<String>,
    /// Every call, as `mise <args>`.
    calls: Vec<String>,
}

impl FakeMise {
    pub(crate) fn new(conf_d: PathBuf) -> Self {
        Self {
            conf_d,
            releases: BTreeMap::from([
                ("demo-lang", vec!["1.0.0", "1.0.3", "2.0.1", "2.1.0"]),
                ("demo-ls", vec!["0.4.1", "0.5.0", "0.5.2"]),
                ("demo-fmt", vec!["3.1.0"]),
                ("kubaos-test-tool", vec!["1.0.0", "1.0.1", "2.0.0", "3.0.0"]),
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
            .find(|v| **v == request || v.starts_with(&format!("{request}.")))
            .map(|v| (*v).to_owned())
    }

    /// Every tool the fragments ask for, with the fragment that asks.
    fn requested(&self) -> Vec<(String, String, PathBuf)> {
        let mut requested = Vec::new();
        let Ok(entries) = fs::read_dir(&self.conf_d) else {
            return requested;
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            let text = fs::read_to_string(&path).expect("read a fragment");
            let table: toml::Table = toml::from_str(&text).expect("a fragment is TOML");
            for (tool, request) in table.get("tools").and_then(|t| t.as_table()).into_iter().flatten() {
                requested.push((
                    tool.clone(),
                    request.as_str().expect("a string").to_owned(),
                    path.clone(),
                ));
            }
        }
        requested
    }

    fn record(&self, call: String) {
        self.state().calls.push(call);
    }
}

impl Mise for FakeMise {
    fn install(&self, tools: &[ToolRequest]) -> Result<(), MiseError> {
        self.record(format!(
            "mise install {}",
            tools.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")
        ));
        for tool in tools {
            let failing = self.state().failing.clone();
            let version = self.resolve(&tool.tool, &tool.request);
            let (Some(version), false) = (version, failing.as_deref() == Some(tool.tool.as_str())) else {
                return Err(MiseError::Failed {
                    command: format!("install {tool}"),
                    status: "exit status: 1".into(),
                    message: format!("no version of {tool}"),
                });
            };
            self.state().installed.insert((tool.tool.clone(), version));
        }
        Ok(())
    }

    fn uninstall(&self, versions: &[ToolRequest]) -> Result<(), MiseError> {
        self.record(format!(
            "mise uninstall {}",
            versions.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")
        ));
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
            let version = self.resolve(&tool, &request).unwrap_or_else(|| request.clone());
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

    fn outdated(&self, _bump: bool) -> Result<Vec<OutdatedTool>, MiseError> {
        Ok(Vec::new())
    }

    fn prune(&self) -> Result<(), MiseError> {
        self.record("mise prune --tools".into());
        let selected: BTreeSet<(String, String)> = self
            .requested()
            .into_iter()
            .filter_map(|(tool, request, _)| self.resolve(&tool, &request).map(|v| (tool, v)))
            .collect();
        self.state().installed.retain(|v| selected.contains(v));
        Ok(())
    }

    fn lock(&self) -> Result<(), MiseError> {
        self.record("mise lock --global".into());
        Ok(())
    }
}
