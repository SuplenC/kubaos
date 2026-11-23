//! `--bump`: moves an app's pinned downloads on to their vendors' newest releases, writing the new
//! version, URLs and sha256s into its `app.toml` in place, its comments and layout kept.
//!
//! An app is written only when every pin of it either moved or is current: a `.deb` and a download
//! of the same release (T3 Code's) move together or not at all, and so do an action's
//! architectures.
//!
//! # Owns
//!
//! - Finding the pins in `app.toml` (every `deb` and `download` action, in every setup), agreeing
//!   their architectures on one version, and rewriting them ([`app`]).
//!
//! # Not here
//!
//! - Where a newer release is found: `bump/source.rs`. The table: `main.rs`.

mod source;

use std::fs;
use std::path::Path;

use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Table;
use toml_edit::Value;

use self::source::Found;
use self::source::Pin;
use crate::check::Outcome;
use crate::check::Row;

/// The architectures a pin may name, in the order `app.toml` writes them.
const ARCHES: [&str; 2] = ["amd64", "arm64"];

/// One action's pins, as read before anything is fetched.
#[derive(Debug)]
struct Pins {
    name: String,
    version: String,
    /// Each architecture's URL and sha256.
    arches: Vec<(&'static str, String, String)>,
}

/// What became of one action.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    Current(String),
    Moved {
        from: String,
        to: String,
        /// Each architecture's new URL and sha256.
        arches: Vec<(&'static str, String, String)>,
    },
    NoSource(String),
    Failed(String),
}

/// Moves the app `id`'s pins in `catalog` on to their newest releases, one row an action.
pub fn app(catalog: &Path, id: &str) -> Vec<Row> {
    let path = catalog.join(id).join("app.toml");
    let row = |subject: &str, outcome, detail: String| Row {
        app: id.to_owned(),
        check: "bump",
        subject: subject.to_owned(),
        outcome,
        detail,
    };
    let mut doc = match fs::read_to_string(&path)
        .map_err(|e| format!("reading {}: {e}", path.display()))
        .and_then(|text| {
            text.parse::<DocumentMut>()
                .map_err(|e| format!("{}: {e}", path.display()))
        }) {
        Ok(doc) => doc,
        Err(e) => return vec![row("app.toml", Outcome::Error, e)],
    };
    let pins: Vec<Option<Pins>> = actions(&mut doc).into_iter().map(|table| pins(table)).collect();
    let steps: Vec<Option<(String, Step)>> = pins
        .iter()
        .map(|pins| pins.as_ref().map(|pins| (pins.name.clone(), step(pins))))
        .collect();
    let failed = steps.iter().flatten().any(|(_, step)| matches!(step, Step::Failed(_)));
    let mut rows = Vec::new();
    let mut moved = false;
    for (table, step) in actions(&mut doc).into_iter().zip(&steps) {
        let Some((name, step)) = step else { continue };
        rows.push(match step {
            Step::Current(version) => row(name, Outcome::Ok, format!("{version} is the newest")),
            Step::NoSource(why) => row(name, Outcome::Note, why.clone()),
            Step::Failed(why) => row(name, Outcome::Error, why.clone()),
            Step::Moved { from, to, .. } if failed => row(
                name,
                Outcome::Note,
                format!("{from} → {to} waits: another pin of the app failed"),
            ),
            Step::Moved { from, to, arches } => {
                write(table, to, arches);
                moved = true;
                row(name, Outcome::Moved, format!("{from} → {to}"))
            }
        });
    }
    if moved && let Err(e) = fs::write(&path, doc.to_string()) {
        rows.push(row(
            "app.toml",
            Outcome::Error,
            format!("writing {}: {e}", path.display()),
        ));
    }
    rows
}

/// The app's actions, its own and every setup's, in the file's order.
fn actions(doc: &mut DocumentMut) -> Vec<&mut Table> {
    let mut tables = Vec::new();
    for (key, item) in doc.as_table_mut().iter_mut() {
        match key.get() {
            "action" => {
                if let Some(actions) = item.as_array_of_tables_mut() {
                    tables.extend(actions.iter_mut());
                }
            }
            "setup" => {
                let setups = item.as_table_mut().into_iter().flat_map(|setups| setups.iter_mut());
                for (_, setup) in setups {
                    if let Some(actions) = setup
                        .as_table_mut()
                        .and_then(|setup| setup.get_mut("action"))
                        .and_then(Item::as_array_of_tables_mut)
                    {
                        tables.extend(actions.iter_mut());
                    }
                }
            }
            _ => {}
        }
    }
    tables
}

/// A `deb` or `download` action's pins; none for another kind.
fn pins(table: &Table) -> Option<Pins> {
    let kind = table.get("kind").and_then(Item::as_str)?;
    if !matches!(kind, "deb" | "download") {
        return None;
    }
    let text = |item: &Item, key: &str| item.as_table_like()?.get(key)?.as_str().map(str::to_owned);
    let arches = ARCHES
        .into_iter()
        .filter_map(|arch| {
            let pin = table.get(arch)?;
            Some((arch, text(pin, "url")?, text(pin, "sha256")?))
        })
        .collect();
    Some(Pins {
        name: table.get("name").and_then(Item::as_str).unwrap_or(kind).to_owned(),
        version: table.get("version").and_then(Item::as_str)?.to_owned(),
        arches,
    })
}

/// Each architecture's newest release, agreed on one version.
fn step(pins: &Pins) -> Step {
    let found: Result<Vec<_>, String> = pins
        .arches
        .iter()
        .map(|(arch, url, sha256)| {
            let pin = Pin {
                version: &pins.version,
                arch,
                url,
                sha256,
            };
            source::newest(&pin).map(|found| (*arch, found))
        })
        .collect();
    match found {
        Ok(found) => agree(&pins.version, &found),
        Err(e) => Step::Failed(e),
    }
}

/// One step from each architecture's find: all current, all moved to one version, or a failure
/// naming how they differ.
fn agree(version: &str, found: &[(&'static str, Found)]) -> Step {
    if let Some((_, Found::NoSource(why))) = found.iter().find(|(_, found)| matches!(found, Found::NoSource(_))) {
        return Step::NoSource(why.clone());
    }
    if found.iter().all(|(_, found)| *found == Found::Current) {
        return Step::Current(version.to_owned());
    }
    let mut to: Option<&str> = None;
    let mut arches = Vec::new();
    for (arch, found) in found {
        let Found::Newer {
            version: newer,
            url,
            sha256,
        } = found
        else {
            return Step::Failed(format!("{arch} has no newer release than {version}, unlike the others"));
        };
        if to.is_some_and(|to| to != newer) {
            return Step::Failed(format!(
                "the architectures move to different versions, {} and {newer}",
                to.unwrap_or_default()
            ));
        }
        to = Some(newer);
        arches.push((*arch, url.clone(), sha256.clone()));
    }
    Step::Moved {
        from: version.to_owned(),
        to: to.unwrap_or(version).to_owned(),
        arches,
    }
}

/// Writes the new version and each architecture's URL and sha256 into the action, each value
/// keeping the spacing around it.
fn write(table: &mut Table, version: &str, arches: &[(&'static str, String, String)]) {
    if let Some(value) = table.get_mut("version").and_then(Item::as_value_mut) {
        replace(value, version);
    }
    for (arch, url, sha256) in arches {
        let Some(pin) = table.get_mut(arch).and_then(Item::as_table_like_mut) else {
            continue;
        };
        for (key, new) in [("url", url), ("sha256", sha256)] {
            if let Some(value) = pin.get_mut(key).and_then(Item::as_value_mut) {
                replace(value, new);
            }
        }
    }
}

fn replace(value: &mut Value, text: &str) {
    let decor = value.decor().clone();
    *value = Value::from(text);
    *value.decor_mut() = decor;
}

#[cfg(test)]
mod tests;
