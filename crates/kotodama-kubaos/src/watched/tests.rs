use std::fs;
use std::sync::mpsc;
use std::time::Duration;

use kotodama_extension::watch::EventKind;
use kotodama_extension::watch::RecursiveMode;
use kotodama_extension::watch::Watcher;
use kotodama_extension::watch::notify::event::AccessKind;
use kotodama_extension::watch::notify::event::AccessMode;
use kotodama_extension::watch::notify::event::CreateKind;
use kotodama_extension::watch::notify::event::ModifyKind;
use kotodama_extension::watch::notify::event::RenameMode;
use tempfile::TempDir;

use super::*;

const FILE: &str = "/run/user/1000/kubaos-drives";

fn event(kind: EventKind, path: &str) -> Event {
    Event::new(kind).add_path(path.into())
}

#[test]
fn a_write_a_rename_onto_and_a_creation_of_the_file_change_it() {
    let file = Path::new(FILE);

    for kind in [
        EventKind::Modify(ModifyKind::Any),
        EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        EventKind::Create(CreateKind::File),
    ] {
        assert!(changes(&event(kind, FILE), file), "{kind:?}");
    }
}

#[test]
fn opening_or_reading_the_file_doesnt_change_it() {
    let file = Path::new(FILE);

    for kind in [
        EventKind::Access(AccessKind::Open(AccessMode::Any)),
        EventKind::Access(AccessKind::Read),
        EventKind::Access(AccessKind::Close(AccessMode::Read)),
    ] {
        assert!(!changes(&event(kind, FILE), file), "{kind:?}");
    }
}

#[test]
fn a_change_of_another_file_in_the_folder_doesnt_count() {
    let other = event(EventKind::Modify(ModifyKind::Any), "/run/user/1000/kubaos-other");

    assert!(!changes(&other, Path::new(FILE)));
}

#[test]
fn a_watcher_reading_its_file_once_it_settles_isnt_set_off_again() {
    // Regression: SuplenC/kotodama#240.
    let root = TempDir::new().unwrap();
    let dir = root.path().canonicalize().unwrap();
    let file = dir.join("desk");
    fs::write(&file, "1").unwrap();
    let (settled, reads) = mpsc::channel();
    let changed = file.clone();
    let read = file.clone();
    let watcher = Watcher::start(
        "watch",
        RecursiveMode::NonRecursive,
        Duration::from_millis(50),
        move |event, _| changes(event, &changed),
        move |_| {
            let _ = settled.send(fs::read_to_string(&read).unwrap());
        },
    )
    .unwrap();
    watcher.watch(vec![dir]);

    fs::write(&file, "2").unwrap();

    assert_eq!(reads.recv_timeout(Duration::from_secs(5)).unwrap(), "2");
    assert!(reads.recv_timeout(Duration::from_millis(500)).is_err());
}
