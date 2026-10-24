//! udisks2 on the system bus: its block devices and drives as [`Block`] and [`Drive`], and the
//! calls that mount, unmount, lock, power off and eject.
//!
//! The interfaces: <https://storaged.org/doc/udisks2-api/latest/>. An active local session may
//! mount and eject removable drives without a password under udisks2's own polkit policy; every
//! call says `auth.no_user_interaction`, so nothing here ever opens a password prompt.
//!
//! # Owns
//!
//! - [`UDisks`]: reading the objects ([`UDisks::snapshot`]) and making the calls.
//! - Turning udisks2's properties into [`Block`] and [`Drive`].
//!
//! # Not here
//!
//! - Which block to mount or how to eject: [`crate::block`].

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use zbus::Connection;
use zbus::fdo::ManagedObjects;
use zbus::fdo::ObjectManagerProxy;
use zbus::zvariant::ObjectPath;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value;

use crate::block::Block;
use crate::block::Drive;
use crate::block::Step;

/// udisks2's bus name.
pub const SERVICE: &str = "org.freedesktop.UDisks2";
/// The object its object manager is on, and under which every block and drive is.
pub const ROOT: &str = "/org/freedesktop/UDisks2";

pub const BLOCK: &str = "org.freedesktop.UDisks2.Block";
pub const FILESYSTEM: &str = "org.freedesktop.UDisks2.Filesystem";
const DRIVE: &str = "org.freedesktop.UDisks2.Drive";
const ENCRYPTED: &str = "org.freedesktop.UDisks2.Encrypted";

/// The properties of one object, by interface.
pub type Interfaces = HashMap<String, HashMap<String, OwnedValue>>;

/// udisks2's block devices and drives at one moment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub blocks: Vec<Block>,
    pub drives: Vec<Drive>,
}

impl Snapshot {
    /// The objects of `GetManagedObjects`, sorted by path so the drives keep their order.
    pub fn from_objects(objects: impl IntoIterator<Item = (String, Interfaces)>) -> Self {
        let mut snapshot = Snapshot::default();
        for (path, interfaces) in objects {
            if let Some(drive) = interfaces.get(DRIVE) {
                snapshot.drives.push(drive_from(&path, drive));
            }
            if let Some(block) = interfaces.get(BLOCK) {
                snapshot
                    .blocks
                    .push(block_from(&path, block, interfaces.get(FILESYSTEM)));
            }
        }
        snapshot.blocks.sort_by(|a, b| a.path.cmp(&b.path));
        snapshot.drives.sort_by(|a, b| a.path.cmp(&b.path));
        snapshot
    }

    pub fn block(&self, path: &str) -> Option<&Block> {
        self.blocks.iter().find(|block| block.path == path)
    }

    pub fn drive(&self, path: &str) -> Option<&Drive> {
        self.drives.iter().find(|drive| drive.path == path)
    }
}

/// A call to udisks2 failed: the D-Bus error's name and message.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct UDisksError {
    /// As `org.freedesktop.UDisks2.Error.DeviceBusy`.
    pub name: String,
    pub message: String,
}

impl UDisksError {
    fn zbus(err: zbus::Error) -> Self {
        match err {
            zbus::Error::MethodError(name, message, _) => UDisksError {
                name: name.to_string(),
                message: message.unwrap_or_default(),
            },
            other => UDisksError {
                name: "zbus".to_owned(),
                message: other.to_string(),
            },
        }
    }

    /// Whether a program still has a file open on the drive.
    pub fn is_busy(&self) -> bool {
        self.name.ends_with(".DeviceBusy")
    }
}

/// udisks2 on a system bus connection.
#[derive(Clone, Debug)]
pub struct UDisks {
    conn: Connection,
}

impl UDisks {
    #[must_use]
    pub fn new(conn: Connection) -> Self {
        UDisks { conn }
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// The object manager, whose signals say when a block or drive comes or goes.
    ///
    /// # Errors
    ///
    /// When the proxy can't be made.
    pub async fn object_manager(&self) -> zbus::Result<ObjectManagerProxy<'static>> {
        ObjectManagerProxy::builder(&self.conn)
            .destination(SERVICE)?
            .path(ROOT)?
            .build()
            .await
    }

    /// Every block device and drive now.
    ///
    /// # Errors
    ///
    /// When udisks2 doesn't answer.
    pub async fn snapshot(&self) -> Result<Snapshot, UDisksError> {
        let manager = self.object_manager().await.map_err(UDisksError::zbus)?;
        let objects: ManagedObjects = manager
            .get_managed_objects()
            .await
            .map_err(|err| UDisksError::zbus(err.into()))?;
        Ok(Snapshot::from_objects(objects.into_iter().map(|(path, interfaces)| {
            let interfaces = interfaces
                .into_iter()
                .map(|(name, properties)| (name.to_string(), properties))
                .collect();
            (path.to_string(), interfaces)
        })))
    }

    /// Mounts the filesystem on `block` where udisks2 chooses, `/media/<user>/<label>`.
    ///
    /// # Errors
    ///
    /// When udisks2 refuses.
    pub async fn mount(&self, block: &str) -> Result<PathBuf, UDisksError> {
        let reply = self.call(block, FILESYSTEM, "Mount").await?;
        reply
            .body()
            .deserialize::<String>()
            .map(PathBuf::from)
            .map_err(UDisksError::zbus)
    }

    /// Takes one step of ejecting a drive.
    ///
    /// # Errors
    ///
    /// When udisks2 refuses, as with a filesystem a program still has a file open on.
    pub async fn run(&self, step: &Step) -> Result<(), UDisksError> {
        let (path, interface, method) = match step {
            Step::Unmount(block) => (block, FILESYSTEM, "Unmount"),
            Step::Lock(block) => (block, ENCRYPTED, "Lock"),
            Step::PowerOff(drive) => (drive, DRIVE, "PowerOff"),
            Step::Eject(drive) => (drive, DRIVE, "Eject"),
        };
        self.call(path, interface, method).await.map(drop)
    }

    /// Calls `method`, whose one argument is the options, on the udisks2 object `path`.
    async fn call(&self, path: &str, interface: &str, method: &str) -> Result<zbus::Message, UDisksError> {
        let object = ObjectPath::try_from(path).map_err(|err| UDisksError::zbus(err.into()))?;
        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert("auth.no_user_interaction", Value::from(true));
        self.conn
            .call_method(Some(SERVICE), object, Some(interface), method, &(options,))
            .await
            .map_err(UDisksError::zbus)
    }
}

fn block_from(
    path: &str,
    block: &HashMap<String, OwnedValue>,
    filesystem: Option<&HashMap<String, OwnedValue>>,
) -> Block {
    Block {
        path: path.to_owned(),
        device: string_from_bytes(block.get("PreferredDevice").or_else(|| block.get("Device"))),
        drive: object(block.get("Drive")),
        usage: string(block.get("IdUsage")),
        label: string(block.get("IdLabel")),
        filesystem: filesystem.is_some(),
        mount_points: filesystem
            .and_then(|filesystem| filesystem.get("MountPoints"))
            .map(|mounts| byte_strings(mounts).into_iter().map(PathBuf::from).collect())
            .unwrap_or_default(),
        hint_ignore: flag(block.get("HintIgnore")),
        hint_system: flag(block.get("HintSystem")),
        crypto_backing: object(block.get("CryptoBackingDevice")),
        configured: block
            .get("Configuration")
            .is_some_and(|configuration| matches!(&**configuration, Value::Array(entries) if !entries.is_empty())),
    }
}

fn drive_from(path: &str, drive: &HashMap<String, OwnedValue>) -> Drive {
    Drive {
        path: path.to_owned(),
        vendor: string(drive.get("Vendor")),
        model: string(drive.get("Model")),
        removable: flag(drive.get("Removable")) || flag(drive.get("MediaRemovable")),
        ejectable: flag(drive.get("Ejectable")),
        can_power_off: flag(drive.get("CanPowerOff")),
    }
}

fn flag(value: Option<&OwnedValue>) -> bool {
    value.is_some_and(|value| matches!(&**value, Value::Bool(true)))
}

fn string(value: Option<&OwnedValue>) -> String {
    match value.map(|value| &**value) {
        Some(Value::Str(text)) => text.to_string(),
        _ => String::new(),
    }
}

/// An object path, none for udisks2's `/`, which stands for no object.
fn object(value: Option<&OwnedValue>) -> Option<String> {
    match value.map(|value| &**value) {
        Some(Value::ObjectPath(path)) if path.as_str() != "/" => Some(path.to_string()),
        _ => None,
    }
}

/// An `ay` holding a NUL-terminated path, as udisks2 sends device files and mount points.
fn bytes(value: &Value<'_>) -> Option<Vec<u8>> {
    let Value::Array(array) = value else {
        return None;
    };
    let mut bytes: Vec<u8> = array
        .iter()
        .filter_map(|byte| match byte {
            Value::U8(byte) => Some(*byte),
            _ => None,
        })
        .collect();
    while bytes.last() == Some(&0) {
        bytes.pop();
    }
    Some(bytes)
}

fn string_from_bytes(value: Option<&OwnedValue>) -> String {
    value
        .and_then(|value| bytes(value))
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

/// An `aay` of paths.
fn byte_strings(value: &Value<'_>) -> Vec<OsString> {
    let Value::Array(array) = value else {
        return Vec::new();
    };
    array
        .iter()
        .filter_map(bytes)
        .filter(|bytes| !bytes.is_empty())
        .map(OsString::from_vec)
        .collect()
}

#[cfg(test)]
mod tests;
