//! The design lint: literal design values and stray components in UI code.
//!
//! A colour, length, duration or font written out in QML or in a Rust UI crate is a value the
//! tokens don't own, so changing the token leaves it behind. A button built from a toolkit's own
//! widget looks like no other button. The lint finds both, rule by rule ([`Rule`]):
//!
//! - **QML:** every `.qml` file in the repo.
//! - **Rust:** the sources of every crate that depends on iced, less their tests.
//! - **Never:** the files `kubaos-tokens` generates and the hand-written token files
//!   ([`TOKEN_SOURCES`]), which are where the values belong.
//!
//! A justified literal carries a marker on the line directly above it, naming the rule and the
//! reason; it allows those rules on that one line:
//!
//! ```text
//! // design-allow(size): the tray icon's size is the StatusNotifierItem spec's, not ours.
//! ```
//!
//! # Owns
//!
//! - Finding the files ([`files`]), the rules and markers, and scanning ([`scan`]).
//!
//! # Not here
//!
//! - What each language's rules match: [`qml`] and [`rust`]. The baseline: [`crate::baseline`].

pub mod qml;
pub mod rust;
pub mod source;

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::Error;
use crate::lint::source::Language;
use crate::lint::source::Line;

/// A kind of literal or stray component the lint finds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// A colour written as hex, channels or a name.
    Colour,
    /// A gap, padding or margin.
    Space,
    /// A width, height, position, stroke or icon size.
    Size,
    /// A corner radius.
    Radius,
    /// An animation's duration.
    Duration,
    /// A font family, size, weight, line height or letter spacing.
    Font,
    /// A toolkit's own control where the library has the component.
    Component,
}

impl Rule {
    pub const ALL: [Rule; 7] = [
        Rule::Colour,
        Rule::Space,
        Rule::Size,
        Rule::Radius,
        Rule::Duration,
        Rule::Font,
        Rule::Component,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Rule::Colour => "colour",
            Rule::Space => "space",
            Rule::Size => "size",
            Rule::Radius => "radius",
            Rule::Duration => "duration",
            Rule::Font => "font",
            Rule::Component => "component",
        }
    }

    pub fn from_name(name: &str) -> Option<Rule> {
        Rule::ALL.into_iter().find(|rule| rule.name() == name)
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A literal or a component that breaks a rule, where no marker allows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Relative to the repo's root, with `/`.
    pub path: String,
    /// 1-based.
    pub line: usize,
    /// 1-based, in characters.
    pub column: usize,
    pub rule: Rule,
    /// What was found: `size 22`, `#1e1e2e`, `iced::widget::button`.
    pub what: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: `{}`",
            self.path, self.line, self.column, self.rule, self.what
        )
    }
}

/// A `design-allow` marker that is malformed or allows nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadMarker {
    pub path: String,
    pub line: usize,
    pub problem: String,
}

impl fmt::Display for BadMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.path, self.line, self.problem)
    }
}

/// What a scan of the repo found.
#[derive(Debug, Default)]
pub struct Scan {
    /// Ordered by file, then line.
    pub violations: Vec<Violation>,
    pub bad_markers: Vec<BadMarker>,
    /// Every file scanned, relative to the root.
    pub files: BTreeSet<String>,
}

/// A rule broken at a column of a line, before markers are applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// 0-based, in characters.
    pub column: usize,
    pub rule: Rule,
    pub what: String,
}

/// The text that opens a marker.
pub const MARKER: &str = "design-allow(";

/// Directories never scanned: build outputs, other checkouts, dependencies.
const SKIPPED_DIRS: &[&str] = &[".git", "target", "node_modules", ".claude", "out", "iso", "result"];

/// Hand-written token files, which hold the values a toolkit's code reads until `kubaos-tokens`
/// generates that toolkit's copy. Like the generated files, they are where the values belong.
// TODO(#147): drop Theme.qml once the generated QML singleton replaces it.
pub const TOKEN_SOURCES: &[&str] = &["config/quickshell/theme/Theme.qml"];

/// The QML module directory of the component library: `KubaOS.Components`. Files in it build
/// components out of Qt's controls, so the component rule spares them.
pub const QML_LIBRARY: &str = "KubaOS/Components/";

/// The Rust component library, which the component rule spares for the same reason.
pub const RUST_LIBRARY: &str = "crates/kubaos-ui/";

/// Every file the lint reads under `root`, with its language, relative paths sorted.
///
/// # Errors
///
/// When a directory or a crate's manifest can't be read.
pub fn files(root: &Path) -> Result<Vec<(String, Language)>, Error> {
    let generated: BTreeSet<&str> = kubaos_tokens::target::TARGETS.iter().map(|t| t.path()).collect();
    let mut found = Vec::new();
    for path in walk(root, "qml")? {
        found.push((path, Language::Qml));
    }
    for krate in ui_crates(root)? {
        for path in walk(&root.join(&krate).join("src"), "rs")? {
            let path = format!("{krate}/src/{path}");
            let is_test = path.ends_with("/tests.rs") || path.contains("/tests/");
            if !is_test {
                found.push((path, Language::Rust));
            }
        }
    }
    found.retain(|(path, _)| !generated.contains(path.as_str()) && !TOKEN_SOURCES.contains(&path.as_str()));
    found.sort();
    Ok(found)
}

/// The crates under `crates/` that depend on iced, as `crates/NAME`.
fn ui_crates(root: &Path) -> Result<Vec<String>, Error> {
    let mut crates = Vec::new();
    let dir = root.join("crates");
    for entry in fs::read_dir(&dir).map_err(|source| Error::read(&dir, source))? {
        let entry = entry.map_err(|source| Error::read(&dir, source))?;
        let manifest = entry.path().join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&manifest) else {
            continue;
        };
        let parsed: toml::Table = toml::from_str(&text).map_err(|source| Error::Manifest {
            path: manifest.clone(),
            source,
        })?;
        let depends_on_iced = parsed
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .is_some_and(|deps| deps.keys().any(|name| name == "iced"));
        if depends_on_iced {
            crates.push(format!("crates/{}", entry.file_name().to_string_lossy()));
        }
    }
    Ok(crates)
}

/// Files under `dir` ending in `.extension`, relative to `dir`.
fn walk(dir: &Path, extension: &str) -> Result<Vec<String>, Error> {
    let mut found = Vec::new();
    let mut pending = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let absolute = dir.join(&relative);
        let entries = match fs::read_dir(&absolute) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(Error::read(&absolute, source)),
        };
        for entry in entries {
            let entry = entry.map_err(|source| Error::read(&absolute, source))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let kind = entry.file_type().map_err(|source| Error::read(&entry.path(), source))?;
            if kind.is_dir() {
                if !SKIPPED_DIRS.contains(&name.as_str()) && !name.starts_with("result-") {
                    pending.push(relative.join(&name));
                }
            } else if Path::new(&name).extension().is_some_and(|e| e == extension) {
                found.push(relative.join(&name).to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(found)
}

/// Scans every file under `root`.
///
/// # Errors
///
/// When the tokens or a file can't be read.
pub fn scan(root: &Path) -> Result<Scan, Error> {
    let tokens = kubaos_tokens::load(root).map_err(Error::Tokens)?;
    let longest_ms = tokens.duration.values().map(|d| d.ms).max().unwrap_or(0);
    let mut scan = Scan::default();
    for (path, language) in files(root)? {
        let absolute = root.join(&path);
        let text = fs::read_to_string(&absolute).map_err(|source| Error::read(&absolute, source))?;
        let (violations, bad_markers) = scan_text(&path, &text, language, longest_ms);
        scan.violations.extend(violations);
        scan.bad_markers.extend(bad_markers);
        scan.files.insert(path);
    }
    Ok(scan)
}

/// The violations and bad markers in one file's `text`; `path` is where it is, relative to the root,
/// and `longest_ms` the longest duration token.
pub fn scan_text(path: &str, text: &str, language: Language, longest_ms: u32) -> (Vec<Violation>, Vec<BadMarker>) {
    let lines = source::lines(text, language);
    let in_library = match language {
        Language::Qml => path.contains(QML_LIBRARY),
        Language::Rust => path.starts_with(RUST_LIBRARY),
    };
    let hits: Vec<Vec<Hit>> = match language {
        Language::Qml => qml::hits(&lines),
        Language::Rust => rust::hits(&lines, longest_ms),
    };
    let mut violations = Vec::new();
    let mut bad_markers = Vec::new();
    let mut used: BTreeSet<(usize, Rule)> = BTreeSet::new();
    let markers = markers(path, &lines, &mut bad_markers);
    for (line, line_hits) in lines.iter().zip(&hits) {
        for hit in line_hits {
            if in_library && hit.rule == Rule::Component {
                continue;
            }
            let allowed_by = marker_line(line.number).filter(|above| {
                markers
                    .iter()
                    .any(|(at, rules)| at == above && rules.contains(&hit.rule))
            });
            if let Some(at) = allowed_by {
                used.insert((at, hit.rule));
                continue;
            }
            violations.push(Violation {
                path: path.to_owned(),
                line: line.number,
                column: hit.column + 1,
                rule: hit.rule,
                what: hit.what.clone(),
            });
        }
    }
    for (at, rules) in &markers {
        for rule in rules {
            if !used.contains(&(*at, *rule)) {
                bad_markers.push(BadMarker {
                    path: path.to_owned(),
                    line: *at,
                    problem: format!("design-allow({rule}) allows nothing on the line below it; remove it"),
                });
            }
        }
    }
    bad_markers.sort_by_key(|m| m.line);
    (violations, bad_markers)
}

/// The number of the line directly above line `number`, where a marker for it sits.
fn marker_line(number: usize) -> Option<usize> {
    number.checked_sub(1).filter(|&above| above >= 1)
}

/// Every marker in `lines`, by line, with the rules it names. A marker without a known rule or a
/// reason is a bad marker, and allows nothing.
fn markers(path: &str, lines: &[Line], bad: &mut Vec<BadMarker>) -> Vec<(usize, Vec<Rule>)> {
    let mut found = Vec::new();
    for line in lines {
        let Some(start) = line.comment.find(MARKER) else {
            continue;
        };
        let rest = &line.comment[start + MARKER.len()..];
        let mut problem = |problem: String| {
            bad.push(BadMarker {
                path: path.to_owned(),
                line: line.number,
                problem,
            });
        };
        if !line.is_comment_only() {
            problem("a design-allow marker goes on its own line, above the code it allows".to_owned());
            continue;
        }
        let Some((names, reason)) = rest.split_once(')') else {
            problem("a design-allow marker reads `design-allow(rule): reason`".to_owned());
            continue;
        };
        let mut rules = Vec::new();
        for name in names.split(',').map(str::trim) {
            match Rule::from_name(name) {
                Some(rule) => rules.push(rule),
                None => problem(format!(
                    "{name:?} isn't a lint rule; the rules are {}",
                    Rule::ALL.map(Rule::name).join(", ")
                )),
            }
        }
        let reason = reason.strip_prefix(':').map(str::trim).unwrap_or_default();
        if reason.is_empty() {
            problem("a design-allow marker says why after a colon: `design-allow(size): why`".to_owned());
            continue;
        }
        if !rules.is_empty() {
            found.push((line.number, rules));
        }
    }
    found
}

#[cfg(test)]
mod tests;
