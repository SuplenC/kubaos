//! The five power stops, power-profiles-daemon's three profiles under them, and the two power
//! sources each of which remembers a stop.
//!
//! # Owns
//!
//! - [`Stop`], its names in every place they show, and the profile it sits on.
//! - [`Profile`], power-profiles-daemon's names. [`Source`], battery or plugged in.
//!
//! # Not here
//!
//! - What each stop sets: [`crate::plan`].

use std::fmt;

/// One position of the power slider, from the least power to the most speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stop {
    SuperSaver,
    PowerSaver,
    Balanced,
    Performance,
    MaxPerformance,
}

impl Stop {
    /// Every stop, in slider order.
    pub const ALL: [Stop; 5] = [
        Stop::SuperSaver,
        Stop::PowerSaver,
        Stop::Balanced,
        Stop::Performance,
        Stop::MaxPerformance,
    ];

    /// Its name on the command line, in the files and in the Command menu's ids: `super-saver`.
    pub fn id(self) -> &'static str {
        match self {
            Stop::SuperSaver => "super-saver",
            Stop::PowerSaver => "power-saver",
            Stop::Balanced => "balanced",
            Stop::Performance => "performance",
            Stop::MaxPerformance => "max-performance",
        }
    }

    /// Its name for people: `Super Saver`. The middle three are power-profiles-daemon's own.
    pub fn title(self) -> &'static str {
        match self {
            Stop::SuperSaver => "Super Saver",
            Stop::PowerSaver => "Power Saver",
            Stop::Balanced => "Balanced",
            Stop::Performance => "Performance",
            Stop::MaxPerformance => "Max Performance",
        }
    }

    /// Its position on the slider, 0 for Super Saver.
    pub fn index(self) -> usize {
        match self {
            Stop::SuperSaver => 0,
            Stop::PowerSaver => 1,
            Stop::Balanced => 2,
            Stop::Performance => 3,
            Stop::MaxPerformance => 4,
        }
    }

    /// The stop at `index` on the slider.
    pub fn from_index(index: usize) -> Option<Stop> {
        Stop::ALL.get(index).copied()
    }

    /// The stop named `id`, as [`Stop::id`] writes it; `power_saver` and `Power Saver` too.
    pub fn from_id(id: &str) -> Option<Stop> {
        let wanted = id.trim().to_ascii_lowercase().replace(['_', ' '], "-");
        Stop::ALL.into_iter().find(|stop| stop.id() == wanted)
    }

    /// The power-profiles-daemon profile it sits on, so other tools show something sensible.
    pub fn profile(self) -> Profile {
        match self {
            Stop::SuperSaver | Stop::PowerSaver => Profile::PowerSaver,
            Stop::Balanced => Profile::Balanced,
            Stop::Performance | Stop::MaxPerformance => Profile::Performance,
        }
    }

    /// The stop a profile set from outside lands on: the one named like it.
    pub fn from_profile(profile: Profile) -> Stop {
        match profile {
            Profile::PowerSaver => Stop::PowerSaver,
            Profile::Balanced => Stop::Balanced,
            Profile::Performance => Stop::Performance,
        }
    }
}

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}

/// A profile of power-profiles-daemon, as its `ActiveProfile` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Profile {
    PowerSaver,
    Balanced,
    Performance,
}

impl Profile {
    /// Its name on D-Bus and to `powerprofilesctl`.
    pub fn name(self) -> &'static str {
        match self {
            Profile::PowerSaver => "power-saver",
            Profile::Balanced => "balanced",
            Profile::Performance => "performance",
        }
    }

    /// The profile named `name` on D-Bus.
    pub fn from_name(name: &str) -> Option<Profile> {
        [Profile::PowerSaver, Profile::Balanced, Profile::Performance]
            .into_iter()
            .find(|profile| profile.name() == name)
    }
}

/// Where the power comes from. Each source remembers the stop last chosen on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    Battery,
    Ac,
}

impl Source {
    /// From UPower's `OnBattery`.
    pub fn from_on_battery(on_battery: bool) -> Source {
        if on_battery { Source::Battery } else { Source::Ac }
    }

    /// Its key in the state file.
    pub fn key(self) -> &'static str {
        match self {
            Source::Battery => "battery",
            Source::Ac => "ac",
        }
    }

    /// Where a sentence says it is: "on battery".
    pub fn words(self) -> &'static str {
        match self {
            Source::Battery => "on battery",
            Source::Ac => "plugged in",
        }
    }

    /// The stop a source starts on before anything is chosen on it: Power Saver on battery, so an
    /// unplugged laptop lasts; Balanced plugged in.
    pub fn default_stop(self) -> Stop {
        match self {
            Source::Battery => Stop::PowerSaver,
            Source::Ac => Stop::Balanced,
        }
    }
}

#[cfg(test)]
mod tests;
