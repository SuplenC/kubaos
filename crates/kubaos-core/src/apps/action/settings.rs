//! A `setting`'s file, edited in its own format: TOML through `toml_edit`, so its layout, comments
//! and types (dates included) stay as they were, and JSON and a flat `key = value` `.conf` in
//! place in their text, so nothing but the key changes.
//!
//! A file that isn't settings of its format, or a key whose path goes through a value that isn't
//! a table, is refused: a `setting` changes one key and never writes over what it can't read.
//!
//! # Owns
//!
//! - [`Settings`]: reading, querying and changing one settings file's text.
//! - The JSON file's text: `settings/json.rs`.
//! - The `.conf` file's text: `settings/conf.rs`.
//!
//! # Not here
//!
//! - A `setting`'s do, check and undo: `perform.rs`.

mod conf;
mod json;

use std::path::Path;
use std::path::PathBuf;

use self::conf::ConfText;
use self::json::Found;
use self::json::JsonText;
use self::json::Through;

/// A settings file this action can't change.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SettingsError {
    #[error("{} isn't settings this action can read", path.display())]
    Unreadable {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("{key} in {}: {part} isn't a table, so the key can't be under it", path.display())]
    Through { path: PathBuf, key: String, part: String },
    #[error("{key} in {} is a table of settings, not one setting", path.display())]
    Table { path: PathBuf, key: String },
}

/// One settings file's settings, in its format.
#[derive(Debug)]
pub enum Settings {
    Toml(toml_edit::DocumentMut),
    Json(JsonText),
    /// Flat `key = value` lines, whose keys hold no tables.
    Conf(ConfText),
}

/// Whether `path` is a TOML file.
pub fn is_toml(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "toml")
}

/// Whether `path` is a flat `key = value` file; else it is JSON or TOML.
pub fn is_conf(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "conf")
}

impl Settings {
    /// No settings, in the format of `path`.
    pub fn empty(path: &Path) -> Self {
        if is_toml(path) {
            Self::Toml(toml_edit::DocumentMut::new())
        } else if is_conf(path) {
            Self::Conf(ConfText::empty())
        } else {
            Self::Json(JsonText::empty())
        }
    }

    /// `text`, the settings of the file `path`.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Unreadable`] when it isn't settings of the file's format.
    pub fn parse(path: &Path, text: &str) -> Result<Self, SettingsError> {
        let unreadable = |source: Box<dyn std::error::Error + Send + Sync>| SettingsError::Unreadable {
            path: path.to_path_buf(),
            source,
        };
        if is_toml(path) {
            return text.parse().map(Self::Toml).map_err(|e| unreadable(Box::new(e)));
        }
        if is_conf(path) {
            return ConfText::parse(text).map(Self::Conf).map_err(unreadable);
        }
        JsonText::parse(text).map(Self::Json).map_err(unreadable)
    }

    /// The settings as the file's text.
    pub fn render(&self) -> String {
        match self {
            Self::Toml(doc) => doc.to_string(),
            Self::Json(json) => json.text().to_owned(),
            Self::Conf(conf) => conf.text().to_owned(),
        }
    }

    /// Whether it has no settings.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Toml(doc) => doc.is_empty(),
            Self::Json(json) => json.is_empty(),
            Self::Conf(conf) => conf.is_empty(),
        }
    }

    /// How many of the tables above the dotted `key` are there.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Through`] when one of them is a value that isn't a table.
    pub fn tables(&self, path: &Path, key: &str) -> Result<usize, SettingsError> {
        let parts: Vec<&str> = key.split('.').collect();
        let above = &parts[..parts.len() - 1];
        let through = through(path, key);
        let mut found = 0;
        match self {
            Self::Toml(doc) => {
                let mut here: &dyn toml_edit::TableLike = doc.as_table();
                for part in above {
                    let Some(item) = here.get(part) else {
                        return Ok(found);
                    };
                    here = item
                        .as_table_like()
                        .ok_or_else(|| through(Through((*part).to_owned())))?;
                    found += 1;
                }
            }
            Self::Json(json) => found = json.tables(above).map_err(through)?,
            Self::Conf(_) => {}
        }
        Ok(found)
    }

    /// The dotted `key`'s value as its format writes it, when it is there.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Through`] when its path goes through a value that isn't a table,
    /// [`SettingsError::Table`] when the key is a table.
    pub fn get(&self, path: &Path, key: &str) -> Result<Option<String>, SettingsError> {
        if self.tables(path, key)? < key.split('.').count() - 1 {
            return Ok(None);
        }
        let table = || SettingsError::Table {
            path: path.to_path_buf(),
            key: key.to_owned(),
        };
        let (above, last) = split(key);
        match self {
            Self::Toml(doc) => {
                let Some(item) = toml_table(doc.as_table(), &above).and_then(|t| t.get(last)) else {
                    return Ok(None);
                };
                match item.as_value() {
                    Some(value) if !value.is_inline_table() => {
                        let mut value = value.clone();
                        value.decor_mut().clear();
                        Ok(Some(value.to_string()))
                    }
                    _ => Err(table()),
                }
            }
            Self::Json(json) => match json.get(&above, last).map_err(through(path, key))? {
                None => Ok(None),
                Some(Found::Object) => Err(table()),
                Some(Found::Value(value)) => Ok(Some(value)),
            },
            Self::Conf(conf) => Ok(conf.get(key)),
        }
    }

    /// Whether the dotted `key` has `value`.
    ///
    /// # Errors
    ///
    /// As [`Settings::get`].
    pub fn has(&self, path: &Path, key: &str, value: &toml::Value) -> Result<bool, SettingsError> {
        let Some(text) = self.get(path, key)? else {
            return Ok(false);
        };
        Ok(match self {
            Self::Toml(_) => text.parse::<toml_edit::Value>().is_ok_and(|v| same(&v, value)),
            Self::Json(_) => serde_json::from_str::<serde_json::Value>(&text)
                .is_ok_and(|v| serde_json::to_value(value).is_ok_and(|want| v == want)),
            Self::Conf(_) => text == written(self, value),
        })
    }

    /// Sets the dotted `key` to `value`, a value its format writes, making the tables above it;
    /// `None` takes the key out, then the tables above it past the first `keep`, once empty.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Through`] when its path goes through a value that isn't a table,
    /// [`SettingsError::Unreadable`] when `value` isn't a value of its format.
    pub fn set(&mut self, path: &Path, key: &str, value: Option<&str>, keep: usize) -> Result<(), SettingsError> {
        self.tables(path, key)?;
        let unreadable = |source: Box<dyn std::error::Error + Send + Sync>| SettingsError::Unreadable {
            path: path.to_path_buf(),
            source,
        };
        let (above, last) = split(key);
        match self {
            Self::Toml(doc) => {
                let value = value
                    .map(|v| v.parse::<toml_edit::Value>().map_err(|e| unreadable(Box::new(e))))
                    .transpose()?;
                set_toml(doc.as_table_mut(), &above, last, value, keep);
            }
            Self::Json(json) => {
                let through = through(path, key);
                match value {
                    Some(value) => {
                        serde_json::from_str::<serde_json::Value>(value).map_err(|e| unreadable(Box::new(e)))?;
                        json.set(&above, last, value).map_err(through)?;
                    }
                    None => json.remove(&above, last, keep).map_err(through)?,
                }
            }
            Self::Conf(conf) => {
                if let Some(value) = value
                    && value.contains(['\n', '\r', '#'])
                {
                    return Err(unreadable(
                        "a value is one line with no #, which starts a comment".into(),
                    ));
                }
                conf.set(key, value);
            }
        }
        Ok(())
    }
}

/// The refusal of `key` in `path`, whose path goes through a part that isn't a table.
fn through<'a>(path: &'a Path, key: &'a str) -> impl Fn(Through) -> SettingsError + 'a {
    move |Through(part)| SettingsError::Through {
        path: path.to_path_buf(),
        key: key.to_owned(),
        part,
    }
}

/// `value` as `format` writes it: TOML's or JSON's text of it, or a `.conf`'s bare text, a string
/// unquoted.
pub fn written(settings: &Settings, value: &toml::Value) -> String {
    match settings {
        Settings::Toml(_) => value.to_string(),
        Settings::Json(_) => serde_json::to_string(value).unwrap_or_default(),
        Settings::Conf(_) => match value {
            toml::Value::String(text) => text.clone(),
            other => other.to_string(),
        },
    }
}

fn split(key: &str) -> (Vec<&str>, &str) {
    let mut parts: Vec<&str> = key.split('.').collect();
    let last = parts.pop().unwrap_or_default();
    (parts, last)
}

fn same(value: &toml_edit::Value, want: &toml::Value) -> bool {
    match (value, want) {
        (toml_edit::Value::String(v), toml::Value::String(w)) => v.value() == w,
        (toml_edit::Value::Integer(v), toml::Value::Integer(w)) => v.value() == w,
        (toml_edit::Value::Float(v), toml::Value::Float(w)) => v.value().to_bits() == w.to_bits(),
        (toml_edit::Value::Boolean(v), toml::Value::Boolean(w)) => v.value() == w,
        _ => false,
    }
}

fn toml_table<'a>(root: &'a toml_edit::Table, above: &[&str]) -> Option<&'a dyn toml_edit::TableLike> {
    let mut here: &dyn toml_edit::TableLike = root;
    for part in above {
        here = here.get(part)?.as_table_like()?;
    }
    Some(here)
}

fn set_toml(
    here: &mut dyn toml_edit::TableLike,
    above: &[&str],
    last: &str,
    value: Option<toml_edit::Value>,
    keep: usize,
) {
    let Some((first, rest)) = above.split_first() else {
        match value {
            Some(mut value) => {
                if let Some(old) = here.get(last).and_then(toml_edit::Item::as_value) {
                    *value.decor_mut() = old.decor().clone();
                }
                here.insert(last, toml_edit::Item::Value(value));
            }
            None => {
                here.remove(last);
            }
        }
        return;
    };
    if value.is_none() && here.get(first).is_none() {
        return;
    }
    if here.get(first).is_none() {
        let mut table = toml_edit::Table::new();
        table.set_implicit(true);
        here.insert(first, toml_edit::Item::Table(table));
    }
    let Some(next) = here.get_mut(first).and_then(toml_edit::Item::as_table_like_mut) else {
        return;
    };
    set_toml(next, rest, last, value, keep.saturating_sub(1));
    if keep == 0 && next.is_empty() {
        here.remove(first);
    }
}
