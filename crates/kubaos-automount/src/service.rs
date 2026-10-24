//! The automount service: it follows udisks2, mounts each removable drive's filesystems when they
//! appear, tells the user with Open, Import photos and Eject on the notification, and keeps the
//! drives file the Command menu lists for Eject.
//!
//! It mounts only on a filesystem's arrival (and at start), never on a change: a filesystem the
//! user unmounted stays unmounted.
//!
//! # Owns
//!
//! - [`run`], the loop, and what each notification button does.
//! - [`eject`], shared with the `eject` command.
//!
//! # Not here
//!
//! - Which filesystems to mount, and what ejecting takes: [`crate::block`].
//! - The calls to udisks2: [`crate::udisks`].
//! - The user unit: `home/automount.nix`.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::pin::pin;

use futures_lite::StreamExt;
use kotodama_extension::Chain;
use kotodama_extension::launch;
use kotodama_extension::log;
use zbus::Connection;
use zbus::MatchRule;
use zbus::MessageStream;
use zbus::message::Type;

use crate::block;
use crate::block::Verdict;
use crate::drives;
use crate::import;
use crate::notify;
use crate::notify::Note;
use crate::notify::Notifier;
use crate::notify::Signal;
use crate::settings::Settings;
use crate::udisks;
use crate::udisks::Snapshot;
use crate::udisks::UDisks;
use crate::udisks::UDisksError;

/// The file manager the Open button starts.
const FILE_MANAGER: [&str; 2] = ["nautilus", "--new-window"];
const ICON: &str = "drive-removable-media";
const EJECT_ICON: &str = "media-eject";

/// The service couldn't start.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("XDG_RUNTIME_DIR isn't set, so there's nowhere to put the drives file")]
    NoRuntimeDir,
    #[error("connecting to the system bus")]
    System(#[source] zbus::Error),
    #[error("connecting to the session bus")]
    Session(#[source] zbus::Error),
    #[error("following udisks2 on the system bus")]
    Follow(#[source] zbus::Error),
}

/// What happened, from either bus.
enum Event {
    /// An object appeared, or gained an interface, which is a block device's when the flag says.
    Added(String, bool),
    /// Anything else udisks2 reports: a mount, an unmount, an object gone.
    Changed,
    Note(Signal),
}

/// What a notification's buttons act on.
#[derive(Clone, Debug)]
enum Offer {
    Mounted {
        drive: String,
        mount: PathBuf,
        /// The `DCIM` folder on it, for Import photos.
        dcim: Option<PathBuf>,
    },
    Encrypted,
}

struct Daemon {
    udisks: UDisks,
    notifier: Notifier,
    file: PathBuf,
    settings: Option<Settings>,
    /// The open notifications by id, until they close.
    offers: HashMap<u32, Offer>,
}

/// Runs the service until a bus goes.
///
/// # Errors
///
/// When either bus can't be reached or udisks2 can't be followed.
pub async fn run() -> Result<(), RunError> {
    let file = drives::path().ok_or(RunError::NoRuntimeDir)?;
    let system = Connection::system().await.map_err(RunError::System)?;
    let session = Connection::session().await.map_err(RunError::Session)?;
    let udisks = UDisks::new(system);
    let notifier = Notifier::new(session);

    let manager = udisks.object_manager().await.map_err(RunError::Follow)?;
    let added = manager
        .receive_interfaces_added()
        .await
        .map_err(RunError::Follow)?
        .filter_map(|signal| {
            let args = signal.args().ok()?;
            let block = args
                .interfaces_and_properties()
                .keys()
                .any(|name| matches!(name.as_str(), udisks::BLOCK | udisks::FILESYSTEM));
            Some(Event::Added(args.object_path().to_string(), block))
        });
    let removed = manager
        .receive_interfaces_removed()
        .await
        .map_err(RunError::Follow)?
        .map(|_| Event::Changed);
    let changed = properties_changed(udisks.connection())
        .await
        .map_err(RunError::Follow)?
        .map(|_| Event::Changed);
    let signals = notifier
        .signals()
        .await
        .map_err(RunError::Session)?
        .filter_map(|message| message.ok().as_ref().and_then(notify::signal))
        .map(Event::Note);

    let mut daemon = Daemon {
        udisks,
        notifier,
        file,
        settings: Settings::from_env(),
        offers: HashMap::new(),
    };
    daemon.start().await;
    let mut events = pin!(added.or(removed).or(changed).or(signals));
    while let Some(event) = events.next().await {
        match event {
            Event::Added(path, true) => daemon.appeared(&path).await,
            Event::Added(_, false) | Event::Changed => daemon.refresh().await,
            Event::Note(signal) => daemon.answer(signal).await,
        }
    }
    Ok(())
}

/// Every `PropertiesChanged` from udisks2's objects, which is how a mount or an unmount shows.
async fn properties_changed(conn: &Connection) -> zbus::Result<MessageStream> {
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender(udisks::SERVICE)?
        .interface("org.freedesktop.DBus.Properties")?
        .member("PropertiesChanged")?
        .path_namespace(udisks::ROOT)?
        .build();
    MessageStream::for_match_rule(rule, conn, None).await
}

impl Daemon {
    /// Mounts what is plugged in already, as if it had just appeared.
    async fn start(&mut self) {
        let Some(snapshot) = self.snapshot().await else {
            return;
        };
        let paths: Vec<String> = snapshot.blocks.iter().map(|block| block.path.clone()).collect();
        for path in paths {
            self.consider(&snapshot, &path).await;
        }
        self.refresh().await;
    }

    async fn appeared(&mut self, path: &str) {
        if let Some(snapshot) = self.snapshot().await {
            self.consider(&snapshot, path).await;
        }
        self.refresh().await;
    }

    /// Mounts the block at `path` if [`block::verdict`] says so, and tells the user.
    async fn consider(&mut self, snapshot: &Snapshot, path: &str) {
        if !self.settings.as_ref().is_none_or(Settings::automount) {
            return;
        }
        let Some(block) = snapshot.block(path) else {
            return;
        };
        let drive = block.drive.as_deref().and_then(|drive| snapshot.drive(drive));
        let name = block::block_name(block, drive);
        match block::verdict(block, drive) {
            Verdict::Mount => match self.udisks.mount(path).await {
                Ok(mount) => self.mounted(&name, block.drive.clone().unwrap_or_default(), mount).await,
                Err(err) => {
                    log!("mounting {}: {err}", block.device);
                    self.tell(Note::new(ICON, format!("Couldn't mount {name}"), err.message)).await;
                }
            },
            Verdict::Encrypted => {
                let note = Note::new(ICON, format!("{name} is encrypted"), "Unlock it from the sidebar in Files.")
                    .action("default", "Open Files")
                    .action("open", "Open Files");
                self.offer(note, Offer::Encrypted).await;
            }
            Verdict::Skip(_) => {}
        }
    }

    async fn mounted(&mut self, name: &str, drive: String, mount: PathBuf) {
        let dcim = import::dcim(&mount);
        let body = if dcim.is_some() {
            format!("At {}. It has photos.", mount.display())
        } else {
            format!("At {}.", mount.display())
        };
        let mut note = Note::new(ICON, format!("{name} is ready"), body)
            .action("default", "Open")
            .action("open", "Open");
        if dcim.is_some() {
            note = note.action("import", "Import photos");
        }
        note = note.action("eject", "Eject");
        self.offer(note, Offer::Mounted { drive, mount, dcim }).await;
    }

    async fn answer(&mut self, signal: Signal) {
        let (id, key) = match signal {
            Signal::Closed(id) => {
                self.offers.remove(&id);
                return;
            }
            Signal::Action(id, key) => (id, key),
        };
        let Some(offer) = self.offers.remove(&id) else {
            return;
        };
        match (offer, key.as_str()) {
            (Offer::Mounted { mount, .. }, "default" | "open") => open(Some(&mount)),
            (Offer::Encrypted, "default" | "open") => open(None),
            (Offer::Mounted { dcim: Some(dcim), .. }, "import") => import_from(&dcim),
            (Offer::Mounted { drive, .. }, "eject") => {
                let note = eject_note(eject(&self.udisks, &drive).await);
                self.tell(note).await;
                self.offers
                    .retain(|_, offer| !matches!(offer, Offer::Mounted { drive: other, .. } if *other == drive));
                self.refresh().await;
            }
            _ => {}
        }
    }

    /// Rewrites the drives file from udisks2's objects now.
    async fn refresh(&mut self) {
        let Some(snapshot) = self.snapshot().await else {
            return;
        };
        let mounted = block::mounted_drives(&snapshot.blocks, &snapshot.drives);
        if let Err(err) = drives::write(&self.file, &mounted) {
            log!("writing {}: {err}", self.file.display());
        }
    }

    async fn snapshot(&self) -> Option<Snapshot> {
        self.udisks
            .snapshot()
            .await
            .map_err(|err| log!("reading udisks2's drives: {err}"))
            .ok()
    }

    async fn offer(&mut self, note: Note, offer: Offer) {
        match self.notifier.send(&note).await {
            Ok(id) => {
                self.offers.insert(id, offer);
            }
            Err(err) => log!("notifying: {}", Chain(&err)),
        }
    }

    async fn tell(&self, note: Note) {
        if let Err(err) = self.notifier.send(&note).await {
            log!("notifying: {}", Chain(&err));
        }
    }
}

/// Why a drive wasn't ejected.
#[derive(Debug, thiserror::Error)]
pub enum EjectError {
    #[error("{0} isn't a removable drive, one of its devices or mount points")]
    NoSuchDrive(String),
    /// A program still has a file open on it.
    #[error("{name} is still in use")]
    Busy {
        name: String,
        #[source]
        source: UDisksError,
    },
    #[error("couldn't eject {name}")]
    Refused {
        name: String,
        #[source]
        source: UDisksError,
    },
    #[error("reading udisks2's drives")]
    Read(#[source] UDisksError),
}

/// Ejects the drive `target` names (its object path, a device file, or a mount point), and
/// returns its name.
///
/// # Errors
///
/// When no removable drive matches, or udisks2 refuses a step.
pub async fn eject(udisks: &UDisks, target: &str) -> Result<String, EjectError> {
    let snapshot = udisks.snapshot().await.map_err(EjectError::Read)?;
    let drive = block::find_drive(target, &snapshot.blocks, &snapshot.drives)
        .filter(|drive| drive.removable)
        .ok_or_else(|| EjectError::NoSuchDrive(target.to_owned()))?;
    let name = block::mounted_drives(&snapshot.blocks, std::slice::from_ref(drive))
        .into_iter()
        .next()
        .map_or_else(|| block::drive_name(drive).unwrap_or_else(|| "The drive".to_owned()), |mounted| mounted.name);
    for step in block::eject_plan(drive, &snapshot.blocks) {
        udisks.run(&step).await.map_err(|source| {
            if source.is_busy() {
                EjectError::Busy {
                    name: name.clone(),
                    source,
                }
            } else {
                EjectError::Refused {
                    name: name.clone(),
                    source,
                }
            }
        })?;
    }
    Ok(name)
}

/// What the user reads once an eject is over.
pub fn eject_note(result: Result<String, EjectError>) -> Note {
    match result {
        Ok(name) => Note::new(EJECT_ICON, format!("{name} can be removed"), ""),
        Err(EjectError::Busy { name, .. }) => Note::new(
            EJECT_ICON,
            format!("{name} is still in use"),
            "Close the files open on it, then eject it again.",
        ),
        Err(EjectError::Refused { name, source }) => {
            Note::new(EJECT_ICON, format!("Couldn't eject {name}"), source.message)
        }
        Err(err) => Note::new(EJECT_ICON, "Couldn't eject the drive", Chain(&err).to_string()),
    }
}

/// Opens the file manager on `folder`, or on its last place without one.
fn open(folder: Option<&Path>) {
    let mut argv: Vec<String> = FILE_MANAGER.map(str::to_owned).to_vec();
    if let Some(folder) = folder {
        argv.push(folder.to_string_lossy().into_owned());
    }
    if let Err(err) = launch::launch(&argv, None) {
        log!("opening the file manager: {err}");
    }
}

/// Starts `kubaos-automount import` on `dcim`, in a scope of its own, so a long copy neither
/// holds up the service nor dies with it.
fn import_from(dcim: &Path) {
    let program = std::env::current_exe().map_or_else(|_| "kubaos-automount".to_owned(), |exe| {
        exe.to_string_lossy().into_owned()
    });
    let argv = [program, "import".to_owned(), dcim.to_string_lossy().into_owned()];
    if let Err(err) = launch::launch(&argv, None) {
        log!("starting the photo import: {err}");
    }
}
