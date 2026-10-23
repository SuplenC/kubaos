//! The system toggles: Wi-Fi and Bluetooth over D-Bus (NetworkManager and
//! BlueZ), audio through `wpctl`, brightness through `brightnessctl`, and the
//! power profile over D-Bus (power-profiles-daemon). Each talks to the
//! service or program itself, never to the shell, so it works with
//! Quickshell down; the shell sees the change as it would any other.
//!
//! # Owns
//!
//! - The toggles' Items, and what each of their Actions does.
//! - The audio device list View and picking a device in it.
//!
//! # Not here
//!
//! - Calling D-Bus: [`crate::bus`]. Running programs: [`crate::run`].
//! - Power and session (power off, reboot, suspend, lock, log out): Kotodama's Bundled `system`
//!   Extension.

use std::collections::BTreeMap;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Argument;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;

use crate::audio;
use crate::audio::Direction;
use crate::bus::Bus;
use crate::bus::Property;
use crate::run::Runner;

/// The Item kind of every toggle.
pub const KIND: &str = "kubaos.toggle";
/// The Item kind of an audio device in its list View.
pub const DEVICE_KIND: &str = "kubaos.audio-device";

/// The primary Action of Wi-Fi and Bluetooth: on if off, off if on.
pub const TOGGLE: &str = "toggle";
pub const ON: &str = "on";
pub const OFF: &str = "off";
/// The one Action of the other toggles.
pub const RUN: &str = "run";
/// The one Action of an audio device: make it the default.
pub const USE: &str = "use";

/// The Argument of Volume and Brightness.
const PERCENT: &str = "percent";

/// The step of Volume up and down, and Brightness up and down, as the media
/// keys take (`config/hypr/keybinds.lua`).
const STEP: &str = "5%";

const WIFI: Property = Property {
    service: "org.freedesktop.NetworkManager",
    path: "/org/freedesktop/NetworkManager",
    interface: "org.freedesktop.NetworkManager",
    name: "WirelessEnabled",
};

const BLUEZ: &str = "org.bluez";
const BLUEZ_ROOT: &str = "/org/bluez";

/// power-profiles-daemon's name since 0.20; `net.hadess.PowerProfiles` is the
/// old one, kept for older clients.
const POWER_PROFILE: Property = Property {
    service: "org.freedesktop.UPower.PowerProfiles",
    path: "/org/freedesktop/UPower/PowerProfiles",
    interface: "org.freedesktop.UPower.PowerProfiles",
    name: "ActiveProfile",
};

/// One system toggle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    Wifi,
    Bluetooth,
    Mute,
    MuteMicrophone,
    VolumeUp,
    VolumeDown,
    Volume,
    AudioOutput,
    AudioInput,
    BrightnessUp,
    BrightnessDown,
    Brightness,
    PowerSaver,
    Balanced,
    Performance,
}

/// What running a toggle came to.
#[derive(Clone, Debug, PartialEq)]
pub enum Ran {
    /// Done; the menu closes.
    Close,
    /// Done; the menu stays open, for another step.
    KeepOpen,
    /// Show this View.
    Push(View),
    /// Done from inside the View with this id, which closes with the menu.
    Pop(String),
}

impl Toggle {
    /// Every one, in the order the Items are sent.
    pub const ALL: [Toggle; 15] = [
        Toggle::Wifi,
        Toggle::Bluetooth,
        Toggle::Mute,
        Toggle::MuteMicrophone,
        Toggle::VolumeUp,
        Toggle::VolumeDown,
        Toggle::Volume,
        Toggle::AudioOutput,
        Toggle::AudioInput,
        Toggle::BrightnessUp,
        Toggle::BrightnessDown,
        Toggle::Brightness,
        Toggle::PowerSaver,
        Toggle::Balanced,
        Toggle::Performance,
    ];

    /// Its Item's id, which `hide` names it by.
    pub fn id(self) -> &'static str {
        match self {
            Toggle::Wifi => "wifi",
            Toggle::Bluetooth => "bluetooth",
            Toggle::Mute => "mute",
            Toggle::MuteMicrophone => "mute-microphone",
            Toggle::VolumeUp => "volume-up",
            Toggle::VolumeDown => "volume-down",
            Toggle::Volume => "volume",
            Toggle::AudioOutput => "audio-output",
            Toggle::AudioInput => "audio-input",
            Toggle::BrightnessUp => "brightness-up",
            Toggle::BrightnessDown => "brightness-down",
            Toggle::Brightness => "brightness",
            Toggle::PowerSaver => "power-saver",
            Toggle::Balanced => "balanced",
            Toggle::Performance => "performance",
        }
    }

    /// The one whose Item has the id `id`.
    pub fn from_id(id: &str) -> Option<Toggle> {
        Toggle::ALL.into_iter().find(|toggle| toggle.id() == id)
    }

    fn title(self) -> &'static str {
        match self {
            Toggle::Wifi => "Wi-Fi",
            Toggle::Bluetooth => "Bluetooth",
            Toggle::Mute => "Mute audio",
            Toggle::MuteMicrophone => "Mute microphone",
            Toggle::VolumeUp => "Volume up",
            Toggle::VolumeDown => "Volume down",
            Toggle::Volume => "Set volume",
            Toggle::AudioOutput => "Audio output",
            Toggle::AudioInput => "Audio input",
            Toggle::BrightnessUp => "Brightness up",
            Toggle::BrightnessDown => "Brightness down",
            Toggle::Brightness => "Set brightness",
            Toggle::PowerSaver => "Power saver",
            Toggle::Balanced => "Balanced power",
            Toggle::Performance => "Performance",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Toggle::Wifi => "Turn Wi-Fi on or off",
            Toggle::Bluetooth => "Turn Bluetooth on or off",
            Toggle::Mute => "Mute or unmute the speakers",
            Toggle::MuteMicrophone => "Mute or unmute the microphone",
            Toggle::VolumeUp => "Raise the volume by 5%",
            Toggle::VolumeDown => "Lower the volume by 5%",
            Toggle::Volume => "Set the volume, from 0 to 100%",
            Toggle::AudioOutput => "Choose the speakers or headphones",
            Toggle::AudioInput => "Choose the microphone",
            Toggle::BrightnessUp => "Raise the screen's brightness by 5%",
            Toggle::BrightnessDown => "Lower the screen's brightness by 5%",
            Toggle::Brightness => "Set the screen's brightness, from 1 to 100%",
            Toggle::PowerSaver => "Power profile: longer battery, slower",
            Toggle::Balanced => "Power profile: the default, boring and fine",
            Toggle::Performance => "Power profile: faster, hotter, and the battery can go to hell",
        }
    }

    /// The freedesktop icon names icon themes ship for these.
    fn icon(self) -> &'static str {
        match self {
            Toggle::Wifi => "network-wireless",
            Toggle::Bluetooth => "bluetooth",
            Toggle::Mute | Toggle::VolumeUp | Toggle::VolumeDown | Toggle::Volume => "audio-volume-high",
            Toggle::MuteMicrophone | Toggle::AudioInput => "audio-input-microphone",
            Toggle::AudioOutput => "audio-speakers",
            Toggle::BrightnessUp | Toggle::BrightnessDown | Toggle::Brightness => "display-brightness",
            Toggle::PowerSaver | Toggle::Balanced | Toggle::Performance => "power-profile-balanced",
        }
    }

    /// The other words people type for it.
    fn keywords(self) -> &'static [&'static str] {
        match self {
            Toggle::Wifi => &["wifi", "wireless", "wlan", "network", "wifi on", "wifi off", "airplane"],
            Toggle::Bluetooth => &["bt", "bluetooth on", "bluetooth off"],
            Toggle::Mute => &["sound", "audio", "unmute", "speakers", "silence"],
            Toggle::MuteMicrophone => &["mic", "microphone", "unmute"],
            Toggle::VolumeUp => &["louder", "sound", "audio"],
            Toggle::VolumeDown => &["quieter", "softer", "sound", "audio"],
            Toggle::Volume => &["sound", "audio"],
            Toggle::AudioOutput => &["speakers", "headphones", "sink", "sound output", "hdmi"],
            Toggle::AudioInput => &["microphone", "mic", "source"],
            Toggle::BrightnessUp => &["brighter", "screen", "backlight"],
            Toggle::BrightnessDown => &["darker", "dim", "screen", "backlight"],
            Toggle::Brightness => &["screen", "backlight", "dim"],
            Toggle::PowerSaver => &["power profile", "battery", "eco", "power saving"],
            Toggle::Balanced => &["power profile", "balanced"],
            Toggle::Performance => &["power profile", "fast", "turbo"],
        }
    }

    /// The word typed before its Argument, for the two that take one:
    /// `volume 40`, `brightness 70`.
    fn keyword(self) -> Option<&'static str> {
        match self {
            Toggle::Volume => Some("volume"),
            Toggle::Brightness => Some("brightness"),
            _ => None,
        }
    }

    fn actions(self) -> Vec<Action> {
        match self {
            Toggle::Wifi | Toggle::Bluetooth => vec![
                Action::new(TOGGLE, format!("Turn {} on or off", self.title())),
                Action::new(ON, "Turn on"),
                Action::new(OFF, "Turn off"),
            ],
            Toggle::Volume | Toggle::Brightness => vec![Action::new(RUN, self.title()).arguments(vec![Argument {
                name: PERCENT.to_owned(),
                placeholder: Some("50".to_owned()),
                optional: false,
            }])],
            _ => vec![Action::new(RUN, self.title())],
        }
    }

    /// Its Item.
    pub fn item(self) -> Item {
        Item {
            id: self.id().to_owned(),
            kind: kind(KIND),
            identity: None,
            title: self.title().to_owned(),
            subtitle: Some(self.subtitle().to_owned()),
            icon: Some(self.icon().to_owned()),
            keyword: self.keyword().map(str::to_owned),
            keywords: self.keywords().iter().map(|&keyword| keyword.to_owned()).collect(),
            hidden: false,
            app_only: false,
            actions: self.actions(),
        }
    }

    /// Runs `action` with `arguments`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When the service or program fails, or an Argument is out of range.
    pub fn run(
        self,
        action: &str,
        arguments: &BTreeMap<String, String>,
        bus: &mut impl Bus,
        runner: &impl Runner,
    ) -> Result<Ran, String> {
        match self {
            Toggle::Wifi => switch(bus, &WIFI, action, "Wi-Fi"),
            Toggle::Bluetooth => {
                let adapter = adapter(bus)?;
                let powered = Property {
                    service: BLUEZ,
                    path: &adapter,
                    interface: "org.bluez.Adapter1",
                    name: "Powered",
                };
                switch(bus, &powered, action, "Bluetooth")
            }
            Toggle::Mute => wpctl(runner, &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"], Ran::Close),
            Toggle::MuteMicrophone => wpctl(runner, &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"], Ran::Close),
            Toggle::VolumeUp => set_volume(runner, &format!("{STEP}+"), Ran::KeepOpen),
            Toggle::VolumeDown => set_volume(runner, &format!("{STEP}-"), Ran::KeepOpen),
            Toggle::Volume => set_volume(runner, &format!("{}%", percent(arguments, 0)?), Ran::Close),
            Toggle::AudioOutput => device_list(runner, Direction::Output),
            Toggle::AudioInput => device_list(runner, Direction::Input),
            Toggle::BrightnessUp => brightnessctl(runner, &format!("{STEP}+"), Ran::KeepOpen),
            Toggle::BrightnessDown => brightnessctl(runner, &format!("{STEP}-"), Ran::KeepOpen),
            Toggle::Brightness => brightnessctl(runner, &format!("{}%", percent(arguments, 1)?), Ran::Close),
            Toggle::PowerSaver => power_profile(bus, "power-saver"),
            Toggle::Balanced => power_profile(bus, "balanced"),
            Toggle::Performance => power_profile(bus, "performance"),
        }
    }
}

/// Every toggle's id, which no action of the registry may take.
pub fn ids() -> Vec<&'static str> {
    Toggle::ALL.into_iter().map(Toggle::id).collect()
}

/// Makes the device whose Item is `id` the default, and closes its list View.
///
/// # Errors
///
/// When `id` is no device's Item, or `wpctl` fails.
pub fn use_device(id: &str, runner: &impl Runner) -> Result<Ran, String> {
    let not_a_device = || format!("`{id}` isn't an audio device");
    let (view, object) = id.split_once(':').ok_or_else(not_a_device)?;
    let list = [Toggle::AudioOutput, Toggle::AudioInput]
        .into_iter()
        .find(|list| list.id() == view)
        .ok_or_else(not_a_device)?;
    let object: u32 = object.parse().map_err(|_| not_a_device())?;
    wpctl(
        runner,
        &["set-default", &object.to_string()],
        Ran::Pop(list.id().to_owned()),
    )
}

fn kind(name: &str) -> ItemKind {
    ItemKind::Namespaced(NamespacedKind::new(name).expect("the Extension's kinds are valid"))
}

/// Sets the boolean `property` as `action` asks: on, off, or the other way
/// from now.
fn switch(bus: &mut impl Bus, property: &Property, action: &str, what: &str) -> Result<Ran, String> {
    let on = match action {
        ON => true,
        OFF => false,
        TOGGLE => !bus
            .get_bool(property)
            .map_err(|err| format!("Couldn't tell whether {what} is on: {}", Chain(&err)))?,
        _ => return Err(format!("{what} has no Action `{action}`")),
    };
    bus.set_bool(property, on).map_err(|err| {
        let state = if on { "on" } else { "off" };
        format!("Couldn't turn {what} {state}: {}", Chain(&err))
    })?;
    Ok(Ran::Close)
}

/// The object path of the first Bluetooth adapter.
fn adapter(bus: &mut impl Bus) -> Result<String, String> {
    let children = bus
        .children(BLUEZ, BLUEZ_ROOT)
        .map_err(|err| format!("Couldn't find the Bluetooth adapter: {}", Chain(&err)))?;
    children
        .into_iter()
        .find(|child| child.starts_with("hci"))
        .map(|adapter| format!("{BLUEZ_ROOT}/{adapter}"))
        .ok_or_else(|| "There's no Bluetooth adapter, so there's nothing to turn on".to_owned())
}

fn power_profile(bus: &mut impl Bus, profile: &str) -> Result<Ran, String> {
    bus.set_string(&POWER_PROFILE, profile)
        .map_err(|err| format!("Couldn't switch to the {profile} profile: {}", Chain(&err)))?;
    Ok(Ran::Close)
}

/// The `percent` Argument, from `min` to 100, written with or without `%`.
fn percent(arguments: &BTreeMap<String, String>, min: u32) -> Result<u32, String> {
    let typed = arguments.get(PERCENT).map_or("", |typed| typed.trim());
    typed
        .trim_end_matches('%')
        .trim_end()
        .parse()
        .ok()
        .filter(|percent| (min..=100).contains(percent))
        .ok_or_else(|| format!("Type a percentage from {min} to 100, not `{typed}`"))
}

fn run_program(runner: &impl Runner, argv: &[&str], done: Ran) -> Result<Ran, String> {
    let argv: Vec<String> = argv.iter().map(|&word| word.to_owned()).collect();
    runner.output(&argv).map_err(|err| Chain(&err).to_string())?;
    Ok(done)
}

fn wpctl(runner: &impl Runner, words: &[&str], done: Ran) -> Result<Ran, String> {
    let argv: Vec<&str> = std::iter::once("wpctl").chain(words.iter().copied()).collect();
    run_program(runner, &argv, done)
}

/// Sets the default sink's volume to `volume`, as `wpctl` writes it (`40%`,
/// `5%+`), at most 100%.
fn set_volume(runner: &impl Runner, volume: &str, done: Ran) -> Result<Ran, String> {
    wpctl(runner, &["set-volume", "-l", "1", "@DEFAULT_AUDIO_SINK@", volume], done)
}

/// Sets the backlight to `value`, as `brightnessctl` writes it. `-n2` keeps
/// it from going fully dark, and `-e4` makes the steps even to the eye, as
/// the brightness keys do.
fn brightnessctl(runner: &impl Runner, value: &str, done: Ran) -> Result<Ran, String> {
    run_program(runner, &["brightnessctl", "-e4", "-n2", "set", value], done)
}

/// The list View of the audio devices of `direction`, the default first.
fn device_list(runner: &impl Runner, direction: Direction) -> Result<Ran, String> {
    let status = runner
        .output(&["wpctl".to_owned(), "status".to_owned()])
        .map_err(|err| Chain(&err).to_string())?;
    let mut devices = audio::devices(&status, direction);
    if devices.is_empty() {
        return Err("PipeWire lists no audio devices. Hello? Anyone?".to_owned());
    }
    devices.sort_by_key(|device| !device.default);
    let list = match direction {
        Direction::Output => Toggle::AudioOutput,
        Direction::Input => Toggle::AudioInput,
    };
    let items = devices
        .into_iter()
        .map(|device| Item {
            id: format!("{}:{}", list.id(), device.id),
            kind: kind(DEVICE_KIND),
            identity: None,
            title: device.name,
            subtitle: device.default.then(|| "In use".to_owned()),
            icon: Some(list.icon().to_owned()),
            keyword: None,
            keywords: Vec::new(),
            hidden: false,
            app_only: false,
            actions: vec![Action::new(USE, "Use")],
        })
        .collect();
    Ok(Ran::Push(View {
        id: list.id().to_owned(),
        title: list.title().to_owned(),
        content: ViewContent::List {
            items,
            filters_itself: false,
        },
    }))
}

#[cfg(test)]
mod tests;
