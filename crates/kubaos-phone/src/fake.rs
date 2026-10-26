//! KDE Connect made of fields, for the tests here and in the crates that drive the phone link.

use std::time::Duration;

use crate::device::Battery;
use crate::device::Device;
use crate::link::Action;
use crate::link::Link;
use crate::link::LinkError;

/// The machine, as fields the tests set and read back.
#[derive(Clone, Debug)]
pub struct Fake {
    pub on: bool,
    pub running: bool,
    /// Whether starting the unit brings kdeconnectd up.
    pub starts: bool,
    pub name: String,
    pub hostname: Option<String>,
    pub addresses: Vec<String>,
    pub devices: Vec<Device>,
    pub clipboard: String,
    /// (device id, plugin, on): the plugins each device supports.
    pub plugins: Vec<(String, String, bool)>,
    /// The plugins the user allows on top of the link's own.
    pub extra: Vec<String>,
    /// What the next call to kdeconnectd fails with, once.
    pub refuse: Option<String>,
    /// Every change asked of the machine, in order: `start`, `stop`, `on`, `off`, `copy TEXT`,
    /// and `<Action> <device id>`.
    pub calls: Vec<String>,
}

impl Default for Fake {
    fn default() -> Self {
        Fake {
            on: false,
            running: false,
            starts: true,
            name: "kubaos".into(),
            hostname: Some("kubaos".into()),
            addresses: vec!["192.168.1.23".into()],
            devices: vec![],
            clipboard: String::new(),
            plugins: vec![],
            extra: vec![],
            refuse: None,
            calls: vec![],
        }
    }
}

/// The user's iPhone, paired and connected.
pub fn iphone() -> Device {
    Device {
        id: "897224eb_f278_4be9_bd3f_f76230491def".into(),
        name: "iPhone".into(),
        kind: "phone".into(),
        paired: true,
        reachable: true,
        wants_to_pair: false,
        verification_key: Some("807704F2".into()),
        battery: Some(Battery {
            charge: 82,
            charging: false,
        }),
    }
}

/// The user's iPhone asking to pair.
pub fn iphone_asking() -> Device {
    Device {
        paired: false,
        wants_to_pair: true,
        battery: None,
        ..iphone()
    }
}

/// Someone else's computer on the same network.
pub fn stranger() -> Device {
    Device {
        id: "site24".into(),
        name: "site24".into(),
        kind: "desktop".into(),
        reachable: true,
        verification_key: None,
        ..Device::default()
    }
}

impl Fake {
    pub fn new() -> Self {
        Self::default()
    }

    /// On, with kdeconnectd running and these devices.
    #[must_use]
    pub fn running(devices: Vec<Device>) -> Self {
        Fake {
            on: true,
            running: true,
            devices,
            ..Fake::default()
        }
    }

    fn bus(&mut self) -> Result<(), LinkError> {
        if !self.running {
            return Err(LinkError::Failed {
                program: "busctl".into(),
                message: "The name org.kde.kdeconnect was not provided by any .service files".into(),
            });
        }
        match self.refuse.take() {
            Some(message) => Err(LinkError::Failed {
                program: "busctl".into(),
                message,
            }),
            None => Ok(()),
        }
    }

    fn device(&mut self, id: &str) -> Option<&mut Device> {
        self.devices.iter_mut().find(|d| d.id == id)
    }
}

impl Link for Fake {
    fn is_on(&self) -> bool {
        self.on
    }

    fn set_on(&mut self, on: bool) -> Result<(), LinkError> {
        self.calls.push(if on { "on" } else { "off" }.into());
        self.on = on;
        Ok(())
    }

    fn start(&mut self) -> Result<(), LinkError> {
        self.calls.push("start".into());
        self.running = self.on && self.starts;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), LinkError> {
        self.calls.push("stop".into());
        self.running = false;
        Ok(())
    }

    fn running(&mut self) -> bool {
        self.running
    }

    fn name(&mut self) -> Result<String, LinkError> {
        self.bus()?;
        Ok(self.name.clone())
    }

    fn hostname(&mut self) -> Option<String> {
        self.hostname.clone()
    }

    fn devices(&mut self) -> Result<Vec<Device>, LinkError> {
        self.bus()?;
        Ok(self.devices.clone())
    }

    fn addresses(&mut self) -> Vec<String> {
        self.addresses.clone()
    }

    fn call(&mut self, id: &str, action: &Action) -> Result<(), LinkError> {
        self.bus()?;
        self.calls.push(format!("{action:?} {id}"));
        let Some(device) = self.device(id) else {
            return Err(LinkError::Failed {
                program: "busctl".into(),
                message: format!("Unknown object '/modules/kdeconnect/devices/{id}'."),
            });
        };
        match action {
            Action::Accept => {
                device.paired = true;
                device.wants_to_pair = false;
            }
            Action::Unpair => device.paired = false,
            Action::SendClipboard | Action::Share(_) | Action::Ring => {}
        }
        Ok(())
    }

    fn plugins(&mut self, id: &str) -> Result<Vec<(String, bool)>, LinkError> {
        self.bus()?;
        Ok(self
            .plugins
            .iter()
            .filter(|(d, _, _)| d == id)
            .map(|(_, p, on)| (p.clone(), *on))
            .collect())
    }

    fn set_plugin(&mut self, id: &str, plugin: &str, on: bool) -> Result<(), LinkError> {
        self.bus()?;
        for entry in self.plugins.iter_mut().filter(|(d, p, _)| d == id && p == plugin) {
            entry.2 = on;
        }
        Ok(())
    }

    fn extra_plugins(&self) -> Vec<String> {
        self.extra.clone()
    }

    fn copy(&mut self, text: &str) -> Result<(), LinkError> {
        self.calls.push(format!("copy {text}"));
        self.clipboard = text.into();
        Ok(())
    }

    fn paste(&mut self) -> Result<String, LinkError> {
        Ok(self.clipboard.clone())
    }

    fn pause(&mut self, _how_long: Duration) {}
}
