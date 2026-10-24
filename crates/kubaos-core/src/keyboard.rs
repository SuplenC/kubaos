//! Keyboard layouts: the list a machine types with, the first of them the default, as
//! `/etc/default/keyboard` holds it (`XKBLAYOUT="us,it"`, `XKBVARIANT=",intl"`).
//!
//! # Owns
//!
//! - A layout and its `layout(variant)` spelling, the rules a list of them follows, and the
//!   changes the Command menu and `kubaos keyboard` make to one.
//! - Reading and writing the layout fields of `/etc/default/keyboard`, and the layout names in
//!   xkb's rules `.lst` files, with the catalogue of every layout and variant by name.
//! - The Mac-style accents on Left Alt: their table ([`accents`]) and their setting
//!   ([`settings`]).
//!
//! # Not here
//!
//! - Writing the file at install: `engine::install`. Changing it on a running system goes
//!   through `kubaos keyboard` (in the CLI) and its root helper, `system/files/keyboard`.
//! - Hyprland reading it: `config/hypr/input.lua`.

use std::collections::HashMap;
use std::fmt;

/// The most layouts a keymap holds: XKB has four groups.
pub const MAX_LAYOUTS: usize = 4;

/// Where the system's keyboard settings live, for the console, the installer and Hyprland.
pub const DEFAULT_KEYBOARD: &str = "/etc/default/keyboard";

/// xkb's rules list, with a name for every layout and variant.
pub const RULES_LST: &str = "/usr/share/X11/xkb/rules/evdev.lst";

/// One XKB layout, with its variant (empty for the layout's own).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Layout {
    pub layout: String,
    pub variant: String,
}

impl Layout {
    #[must_use]
    pub fn new(layout: impl Into<String>, variant: impl Into<String>) -> Self {
        Layout {
            layout: layout.into(),
            variant: variant.into(),
        }
    }

    /// Reads `it` or `us(intl)`, as Kōbō's list and `kubaos keyboard` spell a layout.
    #[must_use]
    pub fn parse(spelled: &str) -> Self {
        let spelled = spelled.trim();
        match spelled.split_once('(') {
            Some((layout, variant)) => {
                Layout::new(layout.trim(), variant.trim_end_matches(')').trim())
            }
            None => Layout::new(spelled, ""),
        }
    }
}

/// `it`, or `us(intl)` with a variant.
impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.variant.is_empty() {
            f.write_str(&self.layout)
        } else {
            write!(f, "{}({})", self.layout, self.variant)
        }
    }
}

/// What is wrong with a list of layouts.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LayoutsError {
    #[error("pick at least one keyboard layout")]
    Empty,
    #[error("at most {MAX_LAYOUTS} keyboard layouts, XKB holds no more")]
    TooMany,
    #[error("keyboard layout \"{0}\" is listed twice")]
    Twice(String),
    #[error("odd keyboard layout \"{0}\"")]
    Odd(String),
    #[error("unknown keyboard layout \"{0}\"")]
    Unknown(String),
    #[error("keyboard layout \"{0}\" isn't one of the layouts")]
    NotThere(String),
    #[error("the last keyboard layout stays")]
    Last,
    /// A move to a place past the last, with the number of layouts.
    #[error("keyboard layouts go from place 1 to {0}")]
    Place(usize),
}

/// Every problem with `layouts`, in order. `known` holds the layout names the machine has; empty
/// when they couldn't be read, and then any name passes.
pub fn problems(layouts: &[Layout], known: &[String]) -> Vec<LayoutsError> {
    let mut out = Vec::new();
    if layouts.is_empty() {
        out.push(LayoutsError::Empty);
    }
    if layouts.len() > MAX_LAYOUTS {
        out.push(LayoutsError::TooMany);
    }
    for (i, l) in layouts.iter().enumerate() {
        if !plain(&l.layout) || l.layout.is_empty() || !plain(&l.variant) {
            out.push(LayoutsError::Odd(l.to_string()));
        } else if !known.is_empty() && !known.contains(&l.layout) {
            out.push(LayoutsError::Unknown(l.layout.clone()));
        }
        if layouts[..i].contains(l) {
            out.push(LayoutsError::Twice(l.to_string()));
        }
    }
    out
}

/// The characters XKB names are made of. A comma or a quote would break the file's fields.
fn plain(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
}

/// Reads a comma-separated list, as `kubaos keyboard set us,it(intl)` takes it.
pub fn parse_list(spelled: &str) -> Vec<Layout> {
    spelled
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(Layout::parse)
        .collect()
}

/// The comma-separated list, as `parse_list` reads it.
pub fn spell_list(layouts: &[Layout]) -> String {
    layouts
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Pairs XKB's two comma-separated fields, `us,it` and `,intl`. A variant list shorter than the
/// layouts leaves the rest at their own variant.
pub fn from_fields(layouts: &str, variants: &str) -> Vec<Layout> {
    let mut variants = variants.split(',').map(str::trim);
    layouts
        .split(',')
        .map(str::trim)
        .map(|layout| (layout, variants.next().unwrap_or("")))
        .filter(|(layout, _)| !layout.is_empty())
        .map(|(layout, variant)| Layout::new(layout, variant))
        .collect()
}

/// XKB's two fields for `layouts`: `("us,it", ",intl")`.
pub fn to_fields(layouts: &[Layout]) -> (String, String) {
    let join = |part: fn(&Layout) -> &str| layouts.iter().map(part).collect::<Vec<_>>().join(",");
    (join(|l| &l.layout), join(|l| &l.variant))
}

/// The keyboard fields of `/etc/default/keyboard`, which is shell syntax: `KEY="value"`,
/// `KEY='value'` or `KEY=value`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyboardFile {
    pub model: String,
    pub layouts: Vec<Layout>,
    pub options: String,
}

impl KeyboardFile {
    /// Reads the file's text; fields it lacks are empty.
    pub fn parse(text: &str) -> Self {
        let mut fields: HashMap<&str, &str> = HashMap::new();
        for line in text.lines() {
            let Some((key, value)) = line.trim().split_once('=') else {
                continue;
            };
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            fields.insert(key.trim(), value);
        }
        let field = |key| fields.get(key).copied().unwrap_or("");
        KeyboardFile {
            model: field("XKBMODEL").to_string(),
            layouts: from_fields(field("XKBLAYOUT"), field("XKBVARIANT")),
            options: field("XKBOPTIONS").to_string(),
        }
    }
}

/// The text of `/etc/default/keyboard` for `layouts`, the first the default. console-setup reads
/// it for the console, which types with the first layout, and Hyprland (`config/hypr/input.lua`).
pub fn default_keyboard(layouts: &[Layout]) -> String {
    let (layout, variant) = to_fields(layouts);
    format!(
        "# KEYBOARD CONFIGURATION FILE (written by the KubaOS install)\n\
         XKBMODEL=\"pc105\"\n\
         XKBLAYOUT=\"{layout}\"\n\
         XKBVARIANT=\"{variant}\"\n\
         XKBOPTIONS=\"\"\n\
         \n\
         BACKSPACE=\"guess\"\n"
    )
}

/// A change to the layouts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Adds one at the end.
    Add(Layout),
    /// Takes one out; the last one stays.
    Remove(Layout),
    /// Moves one to the front, where it is the default.
    MakeDefault(Layout),
    /// Moves one to the place at this index, the front 0.
    Move(Layout, usize),
    /// Replaces them all.
    Set(Vec<Layout>),
}

impl Change {
    /// `layouts` after the change, checked against `known` as `problems` checks.
    ///
    /// # Errors
    ///
    /// The first problem with the result, a removal, a move or a new default that names a
    /// layout that isn't there, the removal of the last one, or a move past the last place.
    pub fn apply(self, layouts: &[Layout], known: &[String]) -> Result<Vec<Layout>, LayoutsError> {
        let not_there = |l: &Layout| LayoutsError::NotThere(l.to_string());
        let next = match self {
            Change::Add(l) => {
                let mut next = layouts.to_vec();
                next.push(l);
                next
            }
            Change::Remove(l) => {
                if !layouts.contains(&l) {
                    return Err(not_there(&l));
                }
                if layouts.len() == 1 {
                    return Err(LayoutsError::Last);
                }
                layouts.iter().filter(|x| **x != l).cloned().collect()
            }
            Change::MakeDefault(l) => {
                if !layouts.contains(&l) {
                    return Err(not_there(&l));
                }
                std::iter::once(l.clone())
                    .chain(layouts.iter().filter(|x| **x != l).cloned())
                    .collect()
            }
            Change::Move(l, to) => {
                let Some(from) = layouts.iter().position(|x| *x == l) else {
                    return Err(not_there(&l));
                };
                if to >= layouts.len() {
                    return Err(LayoutsError::Place(layouts.len()));
                }
                let mut next = layouts.to_vec();
                let moved = next.remove(from);
                next.insert(to, moved);
                next
            }
            Change::Set(next) => next,
        };
        match problems(&next, known).into_iter().next() {
            Some(problem) => Err(problem),
            None => Ok(next),
        }
    }
}

/// The display names in an xkb rules `.lst` file: `it` is "Italian", and `us(intl)` is
/// "English (US, intl., with dead keys)".
pub fn names(lst: &str) -> HashMap<String, String> {
    let mut section = "";
    let mut out = HashMap::new();
    for line in lst.lines() {
        let t = line.trim();
        if let Some(name) = t.strip_prefix('!') {
            section = match name.trim() {
                "layout" => "layout",
                "variant" => "variant",
                _ => "",
            };
            continue;
        }
        let Some((code, rest)) = t.split_once(char::is_whitespace) else {
            continue;
        };
        let rest = rest.trim();
        match section {
            "layout" => {
                out.insert(code.to_string(), rest.to_string());
            }
            // A variant's line names its layout first: `intl  us: English (US, intl.)`.
            "variant" => {
                if let Some((layout, name)) = rest.split_once(": ") {
                    out.insert(format!("{layout}({code})"), name.trim().to_string());
                }
            }
            _ => {}
        }
    }
    out
}

/// Every layout and variant in an xkb rules `.lst` file with its name, sorted by name, as a
/// list to pick a layout from shows them.
pub fn catalogue(lst: &str) -> Vec<(Layout, String)> {
    let mut out: Vec<(Layout, String)> = names(lst)
        .into_iter()
        .map(|(spelled, name)| (Layout::parse(&spelled), name))
        .collect();
    out.sort_by_cached_key(|(layout, name)| (name.to_lowercase(), layout.to_string()));
    out
}

/// What a person calls `layout`: its name in `names`, else its spelling in capitals.
pub fn name(layout: &Layout, names: &HashMap<String, String>) -> String {
    names
        .get(&layout.to_string())
        .cloned()
        .unwrap_or_else(|| layout.to_string().to_uppercase())
}

pub mod accents;
pub mod settings;

#[cfg(test)]
mod tests;
