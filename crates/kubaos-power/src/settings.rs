//! The power settings, in the user layer: `~/.config/kubaos/desktop/power/power.toml`.
//!
//! ```toml
//! panel_power_saving = true     # false for colour-accurate work
//! charge_limit = 100            # 80, 90 or 100
//! low_battery = 20              # the drop's level, in percent
//! low_battery_stop = "power-saver"   # "super-saver", or "off" for no drop
//!
//! [usb]
//! skip = ["046d:c547", "1050:*"]
//!
//! [super-saver]
//! panel = "60hz"                # "60hz", "vrr" or "native"
//! cpu_max_mhz = 1600            # 0 for no cap
//! ```
//!
//! The Command menu and the control center change the file through `kubaos-power`, which edits
//! it in place, so the user's comments and layout stay.
//!
//! # Owns
//!
//! - The settings, their defaults and limits, reading the file and every change made to it.
//!
//! # Not here
//!
//! - The seed with its comments: `user-layer/desktop/power/power.toml`. What a stop comes to:
//!   [`crate::plan`].

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;
use serde::Deserialize;
use toml_edit::DocumentMut;
use toml_edit::value;

use crate::stop::Stop;

/// The settings file in the power folder of the user layer.
pub const FILE: &str = "power.toml";

/// The battery level the low-battery drop comes at: the level where the shell's battery turns
/// red, early enough that the saving still buys real time.
pub const DEFAULT_LOW_BATTERY: u8 = 20;
/// The lowest level the drop may be set to: below it the battery has minutes, not hours.
pub const MIN_LOW_BATTERY: u8 = 5;
/// The highest level: above it the drop would come on every unplugged afternoon.
pub const MAX_LOW_BATTERY: u8 = 50;
/// Where the low-battery drop goes: Power Saver, which keeps the panel and the machine usable.
pub const DEFAULT_LOW_BATTERY_STOP: Stop = Stop::PowerSaver;
/// The battery charges full by default: a limit is the user's choice, for a laptop on a desk.
pub const DEFAULT_CHARGE_LIMIT: ChargeLimit = ChargeLimit::Full;
/// Super Saver's clock cap. With boost off the Ryzen AI 9 HX 370 already tops out at its 2.0 GHz
/// base, so a cap there would do nothing; 1.6 GHz keeps the desktop smooth while every core stays
/// in its most efficient range. Settled by `kubaos power measure` on the reference laptop.
pub const DEFAULT_SUPER_SAVER_MHZ: u32 = 1600;

/// How far the battery charges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargeLimit {
    /// To 80 %, for a laptop that lives plugged in.
    Eighty,
    /// To 90 %.
    Ninety,
    /// Full.
    Full,
}

impl ChargeLimit {
    pub const ALL: [ChargeLimit; 3] = [ChargeLimit::Eighty, ChargeLimit::Ninety, ChargeLimit::Full];

    pub fn percent(self) -> u8 {
        match self {
            ChargeLimit::Eighty => 80,
            ChargeLimit::Ninety => 90,
            ChargeLimit::Full => 100,
        }
    }

    /// The limit of `percent`, one of 80, 90 and 100, as the firmware's profiles have them.
    pub fn from_percent(percent: u8) -> Option<ChargeLimit> {
        ChargeLimit::ALL.into_iter().find(|limit| limit.percent() == percent)
    }

    /// TUXEDO's `charging_profile` for it: tuxedo-drivers' three profiles, from the most charge
    /// to the least, as TUXEDO Control Center names them (full, 90 % and 80 %).
    pub fn tuxedo_profile(self) -> &'static str {
        match self {
            ChargeLimit::Eighty => "stationary",
            ChargeLimit::Ninety => "balanced",
            ChargeLimit::Full => "high_capacity",
        }
    }

    /// The limit TUXEDO's `charging_profile` value stands for.
    pub fn from_tuxedo_profile(profile: &str) -> Option<ChargeLimit> {
        ChargeLimit::ALL
            .into_iter()
            .find(|limit| limit.tuxedo_profile() == profile)
    }
}

/// What the laptop panel does in a stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelMode {
    /// 60 Hz, VRR off: half the frames to draw and send.
    Fixed60,
    /// The native refresh rate with VRR, so a still screen refreshes at the panel's lowest rate.
    Vrr,
    /// The native refresh rate, VRR off.
    Native,
}

impl PanelMode {
    pub const ALL: [PanelMode; 3] = [PanelMode::Fixed60, PanelMode::Vrr, PanelMode::Native];

    /// Its name in the file.
    pub fn name(self) -> &'static str {
        match self {
            PanelMode::Fixed60 => "60hz",
            PanelMode::Vrr => "vrr",
            PanelMode::Native => "native",
        }
    }

    pub fn from_name(name: &str) -> Option<PanelMode> {
        PanelMode::ALL.into_iter().find(|mode| mode.name() == name)
    }
}

/// A stop's own settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopSettings {
    pub panel: PanelMode,
    /// The CPU's clock cap in MHz, `None` for none.
    pub cpu_max_mhz: Option<u32>,
}

/// The power settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// AMD's panel power saving in the saver stops; off for colour-accurate work.
    pub panel_power_saving: bool,
    pub charge_limit: ChargeLimit,
    /// The battery level of the low-battery drop, in percent.
    pub low_battery: u8,
    /// Where the low-battery drop goes; `None` turns it off.
    pub low_battery_stop: Option<Stop>,
    /// USB devices never autosuspended, beyond the input devices, which never are.
    pub usb_skip: Vec<UsbId>,
    stops: [StopSettings; 5],
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            panel_power_saving: true,
            charge_limit: DEFAULT_CHARGE_LIMIT,
            low_battery: DEFAULT_LOW_BATTERY,
            low_battery_stop: Some(DEFAULT_LOW_BATTERY_STOP),
            usb_skip: Vec::new(),
            stops: Stop::ALL.map(default_stop),
        }
    }
}

/// KubaOS's settings for `stop`, as the stops' table in `crate::plan` has them.
fn default_stop(stop: Stop) -> StopSettings {
    match stop {
        Stop::SuperSaver => StopSettings {
            panel: PanelMode::Fixed60,
            cpu_max_mhz: Some(DEFAULT_SUPER_SAVER_MHZ),
        },
        Stop::PowerSaver | Stop::Balanced => StopSettings {
            panel: PanelMode::Vrr,
            cpu_max_mhz: None,
        },
        Stop::Performance | Stop::MaxPerformance => StopSettings {
            panel: PanelMode::Native,
            cpu_max_mhz: None,
        },
    }
}

/// A USB device by vendor and product, `*` for any product of the vendor: `1050:*`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsbId {
    pub vendor: String,
    /// `None` for any product.
    pub product: Option<String>,
}

impl UsbId {
    /// `vvvv:pppp` or `vvvv:*`, four hex digits each.
    pub fn parse(text: &str) -> Option<UsbId> {
        let (vendor, product) = text.trim().split_once(':')?;
        let hex = |part: &str| part.len() == 4 && part.chars().all(|c| c.is_ascii_hexdigit());
        if !hex(vendor) {
            return None;
        }
        let product = match product {
            "*" => None,
            product if hex(product) => Some(product.to_ascii_lowercase()),
            _ => return None,
        };
        Some(UsbId {
            vendor: vendor.to_ascii_lowercase(),
            product,
        })
    }

    /// Whether the device `vendor:product` is this one.
    pub fn matches(&self, vendor: &str, product: &str) -> bool {
        self.vendor.eq_ignore_ascii_case(vendor)
            && self
                .product
                .as_deref()
                .is_none_or(|own| own.eq_ignore_ascii_case(product))
    }
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsbToml {
    #[serde(default)]
    skip: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct StopToml {
    panel: Option<String>,
    cpu_max_mhz: Option<i64>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsToml {
    panel_power_saving: Option<bool>,
    charge_limit: Option<i64>,
    low_battery: Option<i64>,
    low_battery_stop: Option<String>,
    #[serde(default)]
    usb: UsbToml,
    #[serde(rename = "super-saver")]
    super_saver: Option<StopToml>,
    #[serde(rename = "power-saver")]
    power_saver: Option<StopToml>,
    balanced: Option<StopToml>,
    performance: Option<StopToml>,
    #[serde(rename = "max-performance")]
    max_performance: Option<StopToml>,
}

/// Why the settings couldn't be read or changed. The messages are what the toast says.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SettingsError {
    #[error("reading the power settings")]
    Read(#[source] io::Error),
    #[error("parsing the power settings")]
    Parse(#[source] toml::de::Error),
    #[error("the power settings aren't TOML that can be edited")]
    Edit(#[source] toml_edit::TomlError),
    #[error("writing the power settings")]
    Write(#[source] io::Error),
    #[error("`{0}` isn't a charge limit: 80, 90 or 100")]
    NotAChargeLimit(i64),
    #[error("the low-battery drop comes between {MIN_LOW_BATTERY} and {MAX_LOW_BATTERY} %, not at {0} %")]
    NotALowBattery(i64),
    #[error("`{0}` isn't where the low-battery drop can go: power-saver, super-saver or off")]
    NotALowBatteryStop(String),
    #[error("`{0}` isn't a USB device: write it as vendor:product, 046d:c547, or 1050:* for any of a vendor's")]
    NotAUsbId(String),
    #[error("`{0}` isn't a panel mode: 60hz, vrr or native")]
    NotAPanelMode(String),
    #[error("a clock cap of {0} MHz is out of range: from 400 to 10000, or 0 for none")]
    NotAClock(i64),
    #[error("in [{stop}]")]
    InStop {
        stop: &'static str,
        #[source]
        problem: Box<SettingsError>,
    },
}

/// The lowest clock cap: below it the desktop stutters on any CPU.
const MIN_MHZ: u32 = 400;
/// The highest: above any laptop's boost clock, so it can only be a typo.
const MAX_MHZ: u32 = 10_000;

impl Settings {
    /// `stop`'s own settings.
    pub fn stop(&self, stop: Stop) -> StopSettings {
        self.stops[stop.index()]
    }

    /// The stops the low-battery drop may go to.
    pub const LOW_BATTERY_STOPS: [Stop; 2] = [Stop::PowerSaver, Stop::SuperSaver];

    /// Parses the settings file's text; a key left out takes its default.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Parse`] for malformed TOML or an unknown key, and the value's own error
    /// for a value out of its range.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let read: SettingsToml = toml::from_str(text).map_err(SettingsError::Parse)?;
        let mut settings = Settings::default();
        if let Some(on) = read.panel_power_saving {
            settings.panel_power_saving = on;
        }
        if let Some(limit) = read.charge_limit {
            settings.charge_limit = u8::try_from(limit)
                .ok()
                .and_then(ChargeLimit::from_percent)
                .ok_or(SettingsError::NotAChargeLimit(limit))?;
        }
        if let Some(level) = read.low_battery {
            settings.low_battery = u8::try_from(level)
                .ok()
                .filter(|level| (MIN_LOW_BATTERY..=MAX_LOW_BATTERY).contains(level))
                .ok_or(SettingsError::NotALowBattery(level))?;
        }
        if let Some(stop) = read.low_battery_stop {
            settings.low_battery_stop = parse_low_battery_stop(&stop)?;
        }
        settings.usb_skip = read
            .usb
            .skip
            .iter()
            .map(|id| UsbId::parse(id).ok_or_else(|| SettingsError::NotAUsbId(id.clone())))
            .collect::<Result<_, _>>()?;
        let tables = [
            read.super_saver,
            read.power_saver,
            read.balanced,
            read.performance,
            read.max_performance,
        ];
        for (stop, table) in Stop::ALL.into_iter().zip(tables) {
            let Some(table) = table else { continue };
            let own = &mut settings.stops[stop.index()];
            let in_stop = |problem| SettingsError::InStop {
                stop: stop.id(),
                problem: Box::new(problem),
            };
            if let Some(panel) = table.panel {
                own.panel = PanelMode::from_name(&panel).ok_or_else(|| in_stop(SettingsError::NotAPanelMode(panel)))?;
            }
            if let Some(mhz) = table.cpu_max_mhz {
                own.cpu_max_mhz = match u32::try_from(mhz) {
                    Ok(0) => None,
                    Ok(capped) if (MIN_MHZ..=MAX_MHZ).contains(&capped) => Some(capped),
                    _ => return Err(in_stop(SettingsError::NotAClock(mhz))),
                };
            }
        }
        Ok(settings)
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
            Err(err) => Err(SettingsError::Read(err)),
        }
    }
}

/// `power-saver`, `super-saver` or `off`.
///
/// # Errors
///
/// [`SettingsError::NotALowBatteryStop`] for anything else.
pub fn parse_low_battery_stop(text: &str) -> Result<Option<Stop>, SettingsError> {
    if text.trim() == "off" {
        return Ok(None);
    }
    Stop::from_id(text)
        .filter(|stop| Settings::LOW_BATTERY_STOPS.contains(stop))
        .map(Some)
        .ok_or_else(|| SettingsError::NotALowBatteryStop(text.to_owned()))
}

/// The power folder in the user layer: `~/.config/kubaos/desktop/power`.
pub fn dir() -> Option<PathBuf> {
    kubaos_user_layer::config_home().map(|config| kubaos_user_layer::dir(&config, App::Power))
}

/// The settings file: `~/.config/kubaos/desktop/power/power.toml`.
pub fn path() -> Option<PathBuf> {
    dir().map(|dir| dir.join(FILE))
}

/// Turns AMD's panel power saving on or off in the saver stops.
///
/// # Errors
///
/// As reading and writing the file.
pub fn set_panel_power_saving(path: &Path, on: bool) -> Result<Settings, SettingsError> {
    edit(path, |doc| doc["panel_power_saving"] = value(on))
}

/// Sets how far the battery charges.
///
/// # Errors
///
/// As reading and writing the file.
pub fn set_charge_limit(path: &Path, limit: ChargeLimit) -> Result<Settings, SettingsError> {
    edit(path, |doc| doc["charge_limit"] = value(i64::from(limit.percent())))
}

/// Sets where the low-battery drop goes, `None` for no drop.
///
/// # Errors
///
/// As reading and writing the file.
pub fn set_low_battery_stop(path: &Path, stop: Option<Stop>) -> Result<Settings, SettingsError> {
    let name = stop.map_or("off", Stop::id);
    edit(path, |doc| doc["low_battery_stop"] = value(name))
}

/// Sets the battery level the low-battery drop comes at.
///
/// # Errors
///
/// [`SettingsError::NotALowBattery`] out of its range, else as reading and writing the file.
pub fn set_low_battery(path: &Path, percent: u8) -> Result<Settings, SettingsError> {
    if !(MIN_LOW_BATTERY..=MAX_LOW_BATTERY).contains(&percent) {
        return Err(SettingsError::NotALowBattery(i64::from(percent)));
    }
    edit(path, |doc| doc["low_battery"] = value(i64::from(percent)))
}

/// Sets what the laptop panel does in `stop`.
///
/// # Errors
///
/// As reading and writing the file.
pub fn set_panel(path: &Path, stop: Stop, panel: PanelMode) -> Result<Settings, SettingsError> {
    edit(path, |doc| {
        if !doc.contains_key(stop.id()) {
            doc.insert(stop.id(), toml_edit::Item::Table(toml_edit::Table::new()));
        }
        doc[stop.id()]["panel"] = value(panel.name());
    })
}

/// Applies `change` to the file, which must parse before and after.
fn edit(path: &Path, change: impl FnOnce(&mut DocumentMut)) -> Result<Settings, SettingsError> {
    let mut doc = document(path)?;
    Settings::parse(&doc.to_string())?;
    change(&mut doc);
    save(path, &doc)
}

/// The settings file as a document to edit, empty when missing.
fn document(path: &Path) -> Result<DocumentMut, SettingsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(SettingsError::Read(err)),
    };
    text.parse().map_err(SettingsError::Edit)
}

/// Writes `doc` over the file at `path` through a temporary file and a rename, once it parses.
fn save(path: &Path, doc: &DocumentMut) -> Result<Settings, SettingsError> {
    let text = doc.to_string();
    let settings = Settings::parse(&text)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(SettingsError::Write)?;
    }
    let tmp = path.with_extension("toml.new");
    fs::write(&tmp, text).map_err(SettingsError::Write)?;
    fs::rename(&tmp, path).map_err(SettingsError::Write)?;
    Ok(settings)
}

#[cfg(test)]
mod tests;
