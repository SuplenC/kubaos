//! The fixed step list. Titles are what Kōbō shows; weights are rough
//! shares of total time on the laptop.

use serde::{Deserialize, Serialize};

use crate::event::StepInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepId {
    Preflight,
    Partition,
    Encrypt,
    Filesystems,
    Unpack,
    Configure,
    User,
    Bootloader,
    Home,
    Snapshot,
    Finish,
}

impl StepId {
    pub const ALL: [StepId; 11] = [
        StepId::Preflight,
        StepId::Partition,
        StepId::Encrypt,
        StepId::Filesystems,
        StepId::Unpack,
        StepId::Configure,
        StepId::User,
        StepId::Bootloader,
        StepId::Home,
        StepId::Snapshot,
        StepId::Finish,
    ];

    pub fn title(self) -> &'static str {
        match self {
            StepId::Preflight => "Checking the machine",
            StepId::Partition => "Partitioning",
            StepId::Encrypt => "Encrypting",
            StepId::Filesystems => "Creating filesystems",
            StepId::Unpack => "Copying the system",
            StepId::Configure => "Configuring",
            StepId::User => "Creating your account",
            StepId::Bootloader => "Installing the bootloader",
            StepId::Home => "Activating your home",
            StepId::Snapshot => "Taking the factory snapshot",
            StepId::Finish => "Finishing",
        }
    }

    pub fn weight(self) -> u32 {
        match self {
            StepId::Unpack => 60,
            StepId::Bootloader | StepId::Home => 8,
            StepId::Encrypt | StepId::Configure => 5,
            _ => 2,
        }
    }

    pub fn info(self) -> StepInfo {
        StepInfo {
            id: self,
            title: self.title().into(),
            weight: self.weight(),
        }
    }
}
