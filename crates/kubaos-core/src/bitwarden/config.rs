//! `~/.config/dot-bw/config`: `key=value` lines, nothing secret. `server=` and `email=` are
//! what the first login asked; `folders=` narrows the SSH keys (`folder=` is its older name,
//! still read).

use super::Error;
use super::Host;
use super::Paths;

/// The config's lines, in their order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    lines: Vec<(String, String)>,
}

impl Config {
    /// The config on the machine; an empty one when it isn't there.
    pub fn read(host: &mut dyn Host, paths: &Paths) -> Config {
        Config::parse(&host.read(&paths.config).unwrap_or_default())
    }

    pub fn parse(text: &str) -> Config {
        let lines = text
            .lines()
            .filter_map(|l| l.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Config { lines }
    }

    /// The first value of `key`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.is_empty())
    }

    /// Replaces every `key=` line with one, at the end.
    pub fn set(&mut self, key: &str, value: &str) {
        self.lines.retain(|(k, _)| k != key);
        self.lines.push((key.to_string(), value.to_string()));
    }

    /// The folders SSH keys come from: `folders=`, else `folder=`, split on commas, without the
    /// spaces around each name. Empty for the whole vault.
    pub fn folders(&self) -> Vec<String> {
        let raw = self.get("folders").or_else(|| self.get("folder")).unwrap_or("");
        raw.split(',')
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(str::to_string)
            .collect()
    }

    /// Sets `folders=`, dropping the older `folder=`; none at all for the whole vault.
    pub fn set_folders(&mut self, folders: &[String]) {
        self.lines.retain(|(k, _)| k != "folder" && k != "folders");
        if !folders.is_empty() {
            self.set("folders", &folders.join(","));
        }
    }

    pub fn render(&self) -> String {
        self.lines.iter().map(|(k, v)| format!("{k}={v}\n")).collect()
    }

    /// Writes it back, mode 0600.
    ///
    /// # Errors
    ///
    /// The file can't be written.
    pub fn write(&self, host: &mut dyn Host, paths: &Paths) -> Result<(), Error> {
        host.write(&paths.config, &self.render(), 0o600)
            .map_err(|e| Error::write(&paths.config, e))
    }
}
