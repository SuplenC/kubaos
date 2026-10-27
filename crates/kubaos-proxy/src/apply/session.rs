//! The session's parts: the user manager's environment and the shells' env file, GNOME's proxy
//! settings, Docker's client config and git's include. Each keeps what the user had before it
//! first went up, and `off` puts that back rather than a default.
//!
//! # Owns
//!
//! - Putting each in place and taking it away, and the files that remember what was there.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::Value;
use serde_json::json;

use super::StepError;
use super::remove_file;
use super::run;
use crate::machine::Cmd;
use crate::machine::Machine;
use crate::paths::Paths;
use crate::plan::DockerClient;
use crate::plan::ENV_NAMES;
use crate::plan::Gnome;
use crate::profile::write_atomic;

/// GNOME's proxy keys the Proxy sets, by schema; `off` puts back what each held.
const GNOME_KEYS: [(&str, &str); 7] = [
    ("org.gnome.system.proxy", "mode"),
    ("org.gnome.system.proxy", "autoconfig-url"),
    ("org.gnome.system.proxy", "ignore-hosts"),
    ("org.gnome.system.proxy.http", "host"),
    ("org.gnome.system.proxy.http", "port"),
    ("org.gnome.system.proxy.https", "host"),
    ("org.gnome.system.proxy.https", "port"),
];

/// `value` in single quotes for a POSIX shell.
pub(super) fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// `value` as a GVariant string, which gsettings parses.
pub(super) fn gvariant_str(value: &str) -> String {
    format!("'{}'", value.replace('\\', r"\\").replace('\'', r"\'"))
}

fn read_optional(path: &Path) -> Result<Option<String>, StepError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(StepError::File(path.to_path_buf(), err)),
    }
}

/// The user manager's proxy variables now, `NAME=VALUE` lines. A value systemd shows quoted
/// (`$'…'`, for characters a shell reads specially) is left out: no proxy URL has them.
fn env_now(m: &mut dyn Machine) -> Result<String, StepError> {
    let ran = run(m, &Cmd::new("systemctl").args(["--user", "show-environment"]))?;
    Ok(ran
        .stdout
        .lines()
        .filter(|line| {
            line.split_once('=')
                .is_some_and(|(name, value)| ENV_NAMES.contains(&name) && !value.starts_with("$'"))
        })
        .flat_map(|line| [line, "\n"])
        .collect())
}

pub(super) fn env_on(m: &mut dyn Machine, paths: &Paths, vars: &[(String, String)]) -> Result<(), StepError> {
    let before = paths.env_before();
    if !before.exists() {
        let now = env_now(m)?;
        write_atomic(&before, &now, 0o600).map_err(|err| StepError::File(before, err))?;
    }
    run(
        m,
        &Cmd::new("systemctl")
            .args(["--user", "set-environment"])
            .args(vars.iter().map(|(name, value)| format!("{name}={value}"))),
    )?;
    let mut text = String::from("# The Proxy's variables for new shells (kubaos-proxy); gone when it goes off.\n");
    for (name, value) in vars {
        text.push_str(&["export ", name, "=", &sh_quote(value), "\n"].concat());
    }
    write_atomic(&paths.env(), &text, 0o644).map_err(|err| StepError::File(paths.env(), err))
}

pub(super) fn env_off(m: &mut dyn Machine, paths: &Paths) -> Result<(), StepError> {
    run(
        m,
        &Cmd::new("systemctl")
            .args(["--user", "unset-environment"])
            .args(ENV_NAMES),
    )?;
    let before = paths.env_before();
    if let Some(text) = read_optional(&before)? {
        let vars: Vec<&str> = text.lines().filter(|line| line.contains('=')).collect();
        if !vars.is_empty() {
            run(m, &Cmd::new("systemctl").args(["--user", "set-environment"]).args(vars))?;
        }
    }
    remove_file(&before)?;
    remove_file(&paths.env())
}

pub(super) fn gnome_on(m: &mut dyn Machine, paths: &Paths, gnome: &Gnome) -> Result<(), StepError> {
    let before = paths.gnome_before();
    if !before.exists() {
        let mut text = String::new();
        for (schema, key) in GNOME_KEYS {
            let ran = run(m, &Cmd::new("gsettings").args(["get", schema, key]))?;
            text.push_str(&[schema, " ", key, " ", ran.stdout.trim(), "\n"].concat());
        }
        write_atomic(&before, &text, 0o644).map_err(|err| StepError::File(before, err))?;
    }
    let set = |schema: &str, key: &str, value: String| Cmd::new("gsettings").args(["set", schema, key]).arg(value);
    let ignore = |hosts: &[String]| {
        let list: Vec<String> = hosts.iter().map(|h| gvariant_str(h)).collect();
        format!("[{}]", list.join(", "))
    };
    let cmds = match gnome {
        Gnome::Manual {
            host,
            port,
            ignore: hosts,
        } => vec![
            set("org.gnome.system.proxy.http", "host", gvariant_str(host)),
            set("org.gnome.system.proxy.http", "port", port.to_string()),
            set("org.gnome.system.proxy.https", "host", gvariant_str(host)),
            set("org.gnome.system.proxy.https", "port", port.to_string()),
            set("org.gnome.system.proxy", "ignore-hosts", ignore(hosts)),
            set("org.gnome.system.proxy", "mode", gvariant_str("manual")),
        ],
        Gnome::Auto { pac, ignore: hosts } => vec![
            set("org.gnome.system.proxy", "autoconfig-url", gvariant_str(pac)),
            set("org.gnome.system.proxy", "ignore-hosts", ignore(hosts)),
            set("org.gnome.system.proxy", "mode", gvariant_str("auto")),
        ],
    };
    for cmd in &cmds {
        run(m, cmd)?;
    }
    Ok(())
}

/// Puts back each key as it was, the mode last, so the switch back happens once; with nothing
/// remembered, resets them.
pub(super) fn gnome_off(m: &mut dyn Machine, paths: &Paths) -> Result<(), StepError> {
    let before = paths.gnome_before();
    let Some(text) = read_optional(&before)? else {
        for (schema, key) in GNOME_KEYS.iter().rev() {
            run(m, &Cmd::new("gsettings").args(["reset", schema, key]))?;
        }
        return Ok(());
    };
    let mut lines: Vec<(&str, &str, &str)> = text
        .lines()
        .filter_map(|line| {
            let (schema, rest) = line.split_once(' ')?;
            let (key, value) = rest.split_once(' ')?;
            GNOME_KEYS.contains(&(schema, key)).then_some((schema, key, value))
        })
        .collect();
    lines.sort_by_key(|(_, key, _)| *key == "mode");
    for (schema, key, value) in lines {
        run(m, &Cmd::new("gsettings").args(["set", schema, key, value]))?;
    }
    remove_file(&before)
}

/// Docker's client config as an object: none when there is no file.
fn docker_config(path: &Path) -> Result<Option<serde_json::Map<String, Value>>, StepError> {
    let Some(text) = read_optional(path)? else {
        return Ok(None);
    };
    if text.trim().is_empty() {
        return Ok(Some(serde_json::Map::new()));
    }
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => Ok(Some(map)),
        _ => Err(StepError::NotJson(path.to_path_buf())),
    }
}

/// The mode of Docker's config: its own, else 0600, as the docker CLI makes it (it can hold
/// registry logins).
fn docker_mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt as _;
    fs::metadata(path).map_or(0o600, |meta| meta.permissions().mode() & 0o777)
}

fn write_docker(path: &Path, config: serde_json::Map<String, Value>) -> Result<(), StepError> {
    let text = serde_json::to_string_pretty(&Value::Object(config)).unwrap_or_default() + "\n";
    write_atomic(path, &text, docker_mode(path)).map_err(|err| StepError::File(path.to_path_buf(), err))
}

pub(super) fn docker_on(paths: &Paths, docker: &DockerClient) -> Result<(), StepError> {
    let path = paths.docker_config();
    let mut config = docker_config(&path)?.unwrap_or_default();
    let ours = json!({
        "httpProxy": docker.proxy,
        "httpsProxy": docker.proxy,
        "noProxy": docker.no_proxy,
    });
    let record = paths.docker_record();
    let kept: Option<Value> = read_optional(&record)?.and_then(|text| serde_json::from_str(&text).ok());
    let before = match kept {
        Some(kept) => kept.get("before").cloned().unwrap_or(Value::Null),
        None => config
            .get("proxies")
            .and_then(|proxies| proxies.get("default"))
            .cloned()
            .unwrap_or(Value::Null),
    };
    let proxies = config.entry("proxies").or_insert_with(|| json!({}));
    let Some(proxies) = proxies.as_object_mut() else {
        return Err(StepError::NotJson(path));
    };
    proxies.insert("default".to_owned(), ours.clone());
    let text = json!({ "ours": ours, "before": before }).to_string();
    write_atomic(&record, &text, 0o600).map_err(|err| StepError::File(record, err))?;
    write_docker(&path, config)
}

/// Puts back the `proxies.default` the user had, while the one there is still the Proxy's.
pub(super) fn docker_off(paths: &Paths) -> Result<(), StepError> {
    let record = paths.docker_record();
    let Some(kept) = read_optional(&record)?.and_then(|text| serde_json::from_str::<Value>(&text).ok()) else {
        return remove_file(&record);
    };
    let path = paths.docker_config();
    if let Some(mut config) = docker_config(&path)? {
        let mine = config
            .get("proxies")
            .and_then(|proxies| proxies.get("default"))
            .is_some_and(|default| Some(default) == kept.get("ours"));
        if mine {
            if let Some(Value::Object(proxies)) = config.get_mut("proxies") {
                match kept.get("before") {
                    Some(before) if !before.is_null() => {
                        proxies.insert("default".to_owned(), before.clone());
                    }
                    _ => {
                        proxies.remove("default");
                    }
                }
                if proxies.is_empty() {
                    config.remove("proxies");
                }
            }
            write_docker(&path, config)?;
        }
    }
    remove_file(&record)
}

pub(super) fn git_on(paths: &Paths, proxy: &str) -> Result<(), StepError> {
    let text = format!(
        "# Written by kubaos-proxy while the Proxy is on; it goes when the Proxy goes off.\n\
         [http]\n\tproxy = {proxy}\n"
    );
    write_atomic(&paths.git(), &text, 0o644).map_err(|err| StepError::File(paths.git(), err))
}
