//! A `setting`'s file, edited in its own format: TOML through `toml_edit`, so its layout, comments
//! and types (dates included) stay as they were, and JSON as a tree that keeps its keys' order.
//!
//! A file that isn't settings of its format, or a key whose path goes through a value that isn't
//! a table, is refused: a `setting` changes one key and never writes over what it can't read.
//!
//! # Owns
//!
//! - [`Settings`]: reading, querying and changing one settings file's text.
//!
//! # Not here
//!
//! - A `setting`'s do, check and undo: `perform.rs`.

use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;

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
    Json(Vec<(String, Json)>),
}

/// A JSON value whose objects keep their keys in the file's order.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

/// Whether `path` is a TOML file; else it is JSON.
pub fn is_toml(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "toml")
}

impl Settings {
    /// No settings, in the format of `path`.
    pub fn empty(path: &Path) -> Self {
        if is_toml(path) {
            Self::Toml(toml_edit::DocumentMut::new())
        } else {
            Self::Json(Vec::new())
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
        match serde_json::from_str(text).map_err(|e| unreadable(Box::new(e)))? {
            Json::Object(pairs) => Ok(Self::Json(pairs)),
            _ => Err(unreadable("its top level isn't an object".into())),
        }
    }

    /// The settings as the file's text.
    pub fn render(&self) -> String {
        match self {
            Self::Toml(doc) => doc.to_string(),
            Self::Json(pairs) => {
                let object = Json::Object(pairs.clone());
                let mut text = serde_json::to_string_pretty(&object).unwrap_or_default();
                text.push('\n');
                text
            }
        }
    }

    /// Whether it has no settings.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Toml(doc) => doc.is_empty(),
            Self::Json(pairs) => pairs.is_empty(),
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
        let through = |part: &str| SettingsError::Through {
            path: path.to_path_buf(),
            key: key.to_owned(),
            part: part.to_owned(),
        };
        let mut found = 0;
        match self {
            Self::Toml(doc) => {
                let mut here: &dyn toml_edit::TableLike = doc.as_table();
                for part in above {
                    let Some(item) = here.get(part) else {
                        return Ok(found);
                    };
                    here = item.as_table_like().ok_or_else(|| through(part))?;
                    found += 1;
                }
            }
            Self::Json(pairs) => {
                let mut here = pairs;
                for part in above {
                    let Some(value) = get(here, part) else {
                        return Ok(found);
                    };
                    let Json::Object(next) = value else {
                        return Err(through(part));
                    };
                    here = next;
                    found += 1;
                }
            }
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
            Self::Json(pairs) => match json_table(pairs, &above).and_then(|t| get(t, last)) {
                None => Ok(None),
                Some(Json::Object(_)) => Err(table()),
                Some(value) => Ok(Some(serde_json::to_string(value).unwrap_or_default())),
            },
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
            Self::Json(pairs) => {
                let value = value
                    .map(|v| serde_json::from_str::<Json>(v).map_err(|e| unreadable(Box::new(e))))
                    .transpose()?;
                set_json(pairs, &above, last, value, keep);
            }
        }
        Ok(())
    }
}

/// `value` as `format` writes it: TOML's or JSON's text of it.
pub fn written(settings: &Settings, value: &toml::Value) -> String {
    match settings {
        Settings::Toml(_) => value.to_string(),
        Settings::Json(_) => serde_json::to_string(value).unwrap_or_default(),
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

fn get<'a>(pairs: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn json_table<'a>(root: &'a [(String, Json)], above: &[&str]) -> Option<&'a [(String, Json)]> {
    let mut here = root;
    for part in above {
        let Json::Object(next) = get(here, part)? else {
            return None;
        };
        here = next;
    }
    Some(here)
}

fn set_json(here: &mut Vec<(String, Json)>, above: &[&str], last: &str, value: Option<Json>, keep: usize) {
    let Some((first, rest)) = above.split_first() else {
        let at = here.iter().position(|(k, _)| k == last);
        match (value, at) {
            (Some(value), Some(at)) => here[at].1 = value,
            (Some(value), None) => here.push((last.to_owned(), value)),
            (None, Some(at)) => {
                here.remove(at);
            }
            (None, None) => {}
        }
        return;
    };
    let at = match here.iter().position(|(k, _)| k == first) {
        Some(at) => at,
        None if value.is_none() => return,
        None => {
            here.push(((*first).to_owned(), Json::Object(Vec::new())));
            here.len() - 1
        }
    };
    let Json::Object(next) = &mut here[at].1 else {
        return;
    };
    set_json(next, rest, last, value, keep.saturating_sub(1));
    if keep == 0 && next.is_empty() {
        here.remove(at);
    }
}

impl Serialize for Json {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Number(n) => n.serialize(serializer),
            Self::String(s) => serializer.serialize_str(s),
            Self::Array(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Self::Object(pairs) => {
                let mut map = serializer.serialize_map(Some(pairs.len()))?;
                for (key, value) in pairs {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = Json;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_unit<E>(self) -> Result<Json, E> {
        Ok(Json::Null)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Json, E> {
        Ok(Json::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Json, E> {
        Ok(Json::Number(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Json, E> {
        Ok(Json::Number(v.into()))
    }

    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Json, E> {
        serde_json::Number::from_f64(v)
            .map(Json::Number)
            .ok_or_else(|| E::custom("a number JSON can't hold"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Json, E> {
        Ok(Json::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Json, E> {
        Ok(Json::String(v))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Json, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Json::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Json, A::Error> {
        let mut pairs: Vec<(String, Json)> = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Json>()? {
            match pairs.iter_mut().find(|(k, _)| *k == key) {
                Some(pair) => pair.1 = value,
                None => pairs.push((key, value)),
            }
        }
        Ok(Json::Object(pairs))
    }
}
