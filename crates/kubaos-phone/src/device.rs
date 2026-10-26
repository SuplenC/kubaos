//! What the phone link reports: whether it is on, the name phones see, and the devices KDE
//! Connect knows. The card and `kubaos-phone status --json` show the same values.

use serde::Deserialize;
use serde::Serialize;

/// The phone link as it stands.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Status {
    /// The user set up a phone and hasn't turned the link off since.
    pub on: bool,
    /// kdeconnectd answers on the session bus.
    pub running: bool,
    /// What phones see this laptop as: KDE Connect's name for it while it runs, else the host
    /// name it starts with.
    pub name: Option<String>,
    /// This laptop's IPv4 addresses, for adding it by address in the phone's app when the
    /// network blocks discovery.
    pub addresses: Vec<String>,
    /// Every device KDE Connect knows: paired ones, and the ones it can see on the network.
    pub devices: Vec<Device>,
}

impl Status {
    /// The first device asking to pair.
    pub fn request(&self) -> Option<&Device> {
        self.devices.iter().find(|d| d.wants_to_pair && !d.paired)
    }

    /// The paired phone the link sends to.
    pub fn paired(&self) -> Option<&Device> {
        preferred(&self.devices)
    }

    /// Devices on the network that are neither paired nor asking to: often someone else's.
    pub fn others(&self) -> impl Iterator<Item = &Device> {
        self.devices.iter().filter(|d| !d.paired && !d.wants_to_pair)
    }
}

/// The paired device a send goes to when none is named: the first in reach, else the first.
pub fn preferred(devices: &[Device]) -> Option<&Device> {
    let mut paired = devices.iter().filter(|d| d.paired);
    let first = paired.clone().next();
    paired.find(|d| d.reachable).or(first)
}

/// A device KDE Connect knows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Device {
    /// KDE Connect's id for it, the last part of its object path.
    pub id: String,
    pub name: String,
    /// `phone`, `tablet`, `desktop`, `laptop` or `tv`.
    pub kind: String,
    pub paired: bool,
    /// Connected now, so it can be sent to.
    pub reachable: bool,
    /// It asked to pair, and waits for an answer here.
    pub wants_to_pair: bool,
    /// The key both sides show while pairing. Comparing it tells the user's phone apart from
    /// another device on the same network asking at the same time.
    pub verification_key: Option<String>,
    /// Known while a paired device is connected and shares it.
    pub battery: Option<Battery>,
}

/// A device's battery.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Battery {
    /// Percent.
    pub charge: u8,
    pub charging: bool,
}
