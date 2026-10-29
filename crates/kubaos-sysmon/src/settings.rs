//! The system monitor's settings, in the user layer: `~/.config/kubaos/desktop/sysmon/sysmon.toml`.
//!
//! ```toml
//! pill = true
//! style = "numbers"          # numbers, bars or sparklines
//! metrics = ["cpu", "ram"]   # cpu, ram, temp, gpu, net, power
//! alerts = true
//!
//! [thresholds]
//! cpu_temp = 90              # °C
//! ram = 90                   # % used
//! disk = 95                  # % full
//! battery_draw = 30          # W
//! ```
//!
//! The panel and the Command menu change it through `kubaos-sysmon set`, which edits the file in
//! place, so the user's comments and layout stay.
//!
//! # Owns
//!
//! - Reading the file, the defaults, and every change the panel and the menu make to it.
//!
//! # Not here
//!
//! - The seed with its comments: `user-layer/desktop/sysmon/sysmon.toml`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;
use serde::Deserialize;
use serde::Serialize;
use toml_edit::DocumentMut;
use toml_edit::value;

/// The settings file, in [`dir`].
pub const FILE: &str = "sysmon.toml";

/// The CPU temperature that is hot by default, in °C: Ryzen mobile chips throttle from 95.
pub const DEFAULT_CPU_TEMP: f32 = 90.0;
/// RAM used that is nearly full by default, in percent: past it the kernel starts dropping cache
/// a desktop would rather keep.
pub const DEFAULT_RAM: f32 = 90.0;
/// A disk that is nearly full by default, in percent: Btrfs gets slow and snapper's cleanup can't
/// keep up much past it.
pub const DEFAULT_DISK: f32 = 95.0;
/// The battery draw that is fast by default, in watts: about three times what the laptop draws
/// idle, so a busy build passes and a runaway process doesn't.
pub const DEFAULT_BATTERY_DRAW: f32 = 30.0;

/// How the bar's pill draws its metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    #[default]
    Numbers,
    Bars,
    Sparklines,
}

impl Style {
    const NAMES: &str = "numbers, bars, sparklines";

    fn parse(name: &str) -> Option<Self> {
        match name {
            "numbers" => Some(Style::Numbers),
            "bars" => Some(Style::Bars),
            "sparklines" | "graphs" => Some(Style::Sparklines),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Style::Numbers => "numbers",
            Style::Bars => "bars",
            Style::Sparklines => "sparklines",
        }
    }
}

/// A metric the bar's pill can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Metric {
    Cpu,
    Ram,
    /// The CPU's temperature.
    Temp,
    Gpu,
    /// Network speed, down and up.
    Net,
    /// The battery's draw, in watts.
    Power,
}

impl Metric {
    const NAMES: &str = "cpu, ram, temp, gpu, net, power";

    fn parse(name: &str) -> Option<Self> {
        match name {
            "cpu" => Some(Metric::Cpu),
            "ram" | "memory" | "mem" => Some(Metric::Ram),
            "temp" | "temperature" => Some(Metric::Temp),
            "gpu" => Some(Metric::Gpu),
            "net" | "network" => Some(Metric::Net),
            "power" | "battery" | "draw" => Some(Metric::Power),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Metric::Cpu => "cpu",
            Metric::Ram => "ram",
            Metric::Temp => "temp",
            Metric::Gpu => "gpu",
            Metric::Net => "net",
            Metric::Power => "power",
        }
    }
}

/// When the pill turns red and an alert fires.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thresholds {
    pub cpu_temp: f32,
    pub ram: f32,
    pub disk: f32,
    pub battery_draw: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Thresholds {
            cpu_temp: DEFAULT_CPU_TEMP,
            ram: DEFAULT_RAM,
            disk: DEFAULT_DISK,
            battery_draw: DEFAULT_BATTERY_DRAW,
        }
    }
}

/// The monitor's settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// The bar's pill shows.
    pub pill: bool,
    pub style: Style,
    pub metrics: Vec<Metric>,
    /// Alerts fire as notifications.
    pub alerts: bool,
    pub thresholds: Thresholds,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            pill: true,
            style: Style::default(),
            metrics: vec![Metric::Cpu, Metric::Ram],
            alerts: true,
            thresholds: Thresholds::default(),
        }
    }
}

/// The file as written; every key may be left out.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsToml {
    pill: Option<bool>,
    style: Option<Style>,
    metrics: Option<Vec<Metric>>,
    alerts: Option<bool>,
    #[serde(default)]
    thresholds: ThresholdsToml,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThresholdsToml {
    cpu_temp: Option<f32>,
    ram: Option<f32>,
    disk: Option<f32>,
    battery_draw: Option<f32>,
}

/// Why the settings couldn't be read or changed.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("reading the system monitor settings")]
    Read(#[source] io::Error),
    #[error("parsing the system monitor settings")]
    Parse(#[source] toml::de::Error),
    #[error("the system monitor settings aren't TOML that can be edited")]
    Edit(#[source] toml_edit::TomlError),
    #[error("writing the system monitor settings")]
    Write(#[source] io::Error),
    #[error("{key} can't be {value:?}; {expected}")]
    Value {
        key: &'static str,
        value: String,
        expected: &'static str,
    },
    #[error("no setting {0:?}; one of pill, style, metrics, alerts, cpu-temp, ram, disk, battery-draw")]
    NoSetting(String),
}

impl SettingsError {
    fn read(err: io::Error) -> Self {
        SettingsError::Read(err)
    }

    fn write(err: io::Error) -> Self {
        SettingsError::Write(err)
    }
}

impl Settings {
    /// Parses the settings file's text; a key left out takes its default.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Parse`] for malformed TOML, an unknown key, style or metric.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let read: SettingsToml = toml::from_str(text).map_err(SettingsError::Parse)?;
        let defaults = Settings::default();
        let thresholds = Thresholds {
            cpu_temp: read.thresholds.cpu_temp.unwrap_or(DEFAULT_CPU_TEMP),
            ram: read.thresholds.ram.unwrap_or(DEFAULT_RAM),
            disk: read.thresholds.disk.unwrap_or(DEFAULT_DISK),
            battery_draw: read.thresholds.battery_draw.unwrap_or(DEFAULT_BATTERY_DRAW),
        };
        Ok(Settings {
            pill: read.pill.unwrap_or(defaults.pill),
            style: read.style.unwrap_or(defaults.style),
            metrics: read.metrics.unwrap_or(defaults.metrics),
            alerts: read.alerts.unwrap_or(defaults.alerts),
            thresholds,
        })
    }

    /// Reads the settings file; a missing file is the defaults.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Read`] when the file exists but can't be read, else as in
    /// [`Settings::parse`].
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        match fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(SettingsError::read(err)),
        }
    }
}

/// The monitor's folder in the user layer: `~/.config/kubaos/desktop/sysmon`.
pub fn dir() -> Option<PathBuf> {
    kubaos_user_layer::config_home().map(|config| kubaos_user_layer::dir(&config, App::Sysmon))
}

/// The settings file as a document to edit, empty when missing.
fn document(path: &Path) -> Result<DocumentMut, SettingsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(SettingsError::read(err)),
    };
    text.parse().map_err(SettingsError::Edit)
}

/// Writes `doc` over the file at `path` through a temporary file and a rename, so the shell's
/// watch never reads half a file.
fn save(path: &Path, doc: &DocumentMut) -> Result<(), SettingsError> {
    let text = doc.to_string();
    Settings::parse(&text)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(SettingsError::write)?;
    }
    let tmp = path.with_extension("toml.new");
    fs::write(&tmp, text).map_err(SettingsError::write)?;
    fs::rename(&tmp, path).map_err(SettingsError::write)
}

/// `on`, `off` or `toggle` against `current`.
fn switch(key: &'static str, setting: &str, current: bool) -> Result<bool, SettingsError> {
    match setting {
        "on" | "true" | "yes" | "show" => Ok(true),
        "off" | "false" | "no" | "hide" => Ok(false),
        "toggle" => Ok(!current),
        _ => Err(SettingsError::Value {
            key,
            value: setting.to_owned(),
            expected: "on, off or toggle",
        }),
    }
}

/// Sets one setting by the name the command line and the menu use: `pill` and `alerts` (`on`,
/// `off` or `toggle`), `style` (`numbers`, `bars`, `sparklines`), `metrics` (words, in order:
/// `cpu ram temp`), and the thresholds `cpu-temp`, `ram`, `disk` (percent) and `battery-draw`.
///
/// # Errors
///
/// [`SettingsError::NoSetting`] for another name, [`SettingsError::Value`] for a value it can't
/// take; else as reading and writing the file.
pub fn set(path: &Path, key: &str, setting: &str) -> Result<(), SettingsError> {
    let mut doc = document(path)?;
    let current = Settings::parse(&doc.to_string())?;
    let setting = setting.trim().to_lowercase();
    match key {
        "pill" => doc["pill"] = value(switch("pill", &setting, current.pill)?),
        "alerts" => doc["alerts"] = value(switch("alerts", &setting, current.alerts)?),
        "style" => {
            let style = Style::parse(&setting).ok_or_else(|| SettingsError::Value {
                key: "style",
                value: setting.clone(),
                expected: Style::NAMES,
            })?;
            doc["style"] = value(style.name());
        }
        "metrics" => {
            let metrics = parse_metrics(&setting)?;
            let mut array = toml_edit::Array::new();
            for metric in metrics {
                array.push(metric.name());
            }
            doc["metrics"] = value(array);
        }
        "cpu-temp" | "cpu_temp" => threshold(&mut doc, ("cpu_temp", "cpu-temp"), &setting, CPU_TEMP_RANGE)?,
        "ram" => threshold(&mut doc, ("ram", "ram"), &setting, PERCENT_RANGE)?,
        "disk" => threshold(&mut doc, ("disk", "disk"), &setting, PERCENT_RANGE)?,
        "battery-draw" | "battery_draw" => {
            threshold(&mut doc, ("battery_draw", "battery-draw"), &setting, WATTS_RANGE)?;
        }
        _ => return Err(SettingsError::NoSetting(key.to_owned())),
    }
    save(path, &doc)
}

/// The metrics named in `words`, split on spaces or commas, each once and in order.
fn parse_metrics(words: &str) -> Result<Vec<Metric>, SettingsError> {
    let mut metrics = Vec::new();
    for word in words.split([' ', ',']).filter(|word| !word.is_empty()) {
        let metric = Metric::parse(word).ok_or_else(|| SettingsError::Value {
            key: "metrics",
            value: word.to_owned(),
            expected: Metric::NAMES,
        })?;
        if !metrics.contains(&metric) {
            metrics.push(metric);
        }
    }
    if metrics.is_empty() {
        return Err(SettingsError::Value {
            key: "metrics",
            value: words.to_owned(),
            expected: Metric::NAMES,
        });
    }
    Ok(metrics)
}

/// A threshold's bounds, and how an error names them.
type Range = (f32, f32, &'static str);

const CPU_TEMP_RANGE: Range = (30.0, 110.0, "a temperature from 30 to 110 °C");
const PERCENT_RANGE: Range = (10.0, 100.0, "a percentage from 10 to 100");
const WATTS_RANGE: Range = (1.0, 200.0, "watts from 1 to 200");

/// Sets `[thresholds] field` to `setting`, a number in the range; `key` is how the menu names it.
fn threshold(
    doc: &mut DocumentMut,
    (field, key): (&str, &'static str),
    setting: &str,
    (low, high, expected): Range,
) -> Result<(), SettingsError> {
    let number = setting
        .trim_end_matches(['%', 'w', 'c', '°'])
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|number| (low..=high).contains(number))
        .ok_or_else(|| SettingsError::Value {
            key,
            value: setting.to_owned(),
            expected,
        })?;
    if !doc.contains_table("thresholds") {
        doc["thresholds"] = toml_edit::table();
    }
    let stored = if number.fract() == 0.0 {
        value(i64::from(number as i32))
    } else {
        value(f64::from(number))
    };
    doc["thresholds"][field] = stored;
    Ok(())
}

#[cfg(test)]
mod tests;
