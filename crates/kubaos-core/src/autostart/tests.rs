use std::fs;

use tempfile::TempDir;

use super::*;

/// A machine of folders under a temporary one: the user's entries, one system folder, the apps,
/// and a `bin` on `PATH`, on a Hyprland session.
struct Machine {
    root: TempDir,
    dirs: Dirs,
}

impl Machine {
    fn new() -> Self {
        let root = TempDir::new().expect("make a temporary folder");
        let dirs = Dirs {
            user: root.path().join("config/autostart"),
            system: vec![root.path().join("etc/xdg/autostart")],
            applications: vec![root.path().join("share/applications")],
            desktops: vec!["Hyprland".to_owned()],
            path: vec![root.path().join("bin")],
        };
        Machine { root, dirs }
    }

    fn write(&self, path: &str, text: &str) {
        let file = self.root.path().join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }

    fn read(&self, path: &str) -> Option<String> {
        fs::read_to_string(self.root.path().join(path)).ok()
    }

    fn app(&self, id: &str, extra: &str) {
        self.write(
            &format!("share/applications/{id}"),
            &format!("[Desktop Entry]\nType=Application\nName=App {id}\nExec=/usr/bin/app %U\n{extra}"),
        );
    }

    fn ids(&self) -> Vec<String> {
        self.dirs.startup_apps().into_iter().map(|app| app.id).collect()
    }
}

fn entry(name: &str, extra: &str) -> String {
    format!("[Desktop Entry]\nType=Application\nName={name}\nExec={name}\n{extra}")
}

#[test]
fn the_user_entry_replaces_a_system_one_of_the_same_name() {
    let machine = Machine::new();
    machine.write("etc/xdg/autostart/solaar.desktop", &entry("Solaar", ""));
    machine.write("config/autostart/solaar.desktop", &entry("Solaar", "Hidden=true"));

    assert!(!machine.dirs.starts("solaar.desktop"));
    assert!(machine.ids().is_empty());
}

#[test]
fn an_entry_kept_from_this_desktop_does_not_start() {
    let machine = Machine::new();
    machine.write(
        "etc/xdg/autostart/keyring.desktop",
        &entry("Keyring", "OnlyShowIn=GNOME;Unity;"),
    );
    machine.write("etc/xdg/autostart/kde.desktop", &entry("Kde", "NotShowIn=Hyprland;"));
    machine.write("etc/xdg/autostart/dirs.desktop", &entry("Dirs", "NotShowIn=KDE;LXQt;"));
    machine.write("etc/xdg/autostart/ours.desktop", &entry("Ours", "OnlyShowIn=Hyprland;"));

    assert_eq!(machine.ids(), ["dirs.desktop", "ours.desktop"]);
}

#[test]
fn an_entry_turned_off_by_its_own_keys_does_not_start() {
    let machine = Machine::new();
    machine.write(
        "etc/xdg/autostart/a.desktop",
        &entry("A", "X-GNOME-Autostart-enabled=false"),
    );
    machine.write(
        "etc/xdg/autostart/b.desktop",
        &entry("B", "X-GNOME-HiddenUnderSystemd=true"),
    );
    machine.write(
        "etc/xdg/autostart/c.desktop",
        "[Desktop Entry]\nType=Application\nName=C\n",
    );
    machine.write("etc/xdg/autostart/d.desktop", &entry("D", ""));

    assert_eq!(machine.ids(), ["d.desktop"]);
}

#[test]
fn an_entry_whose_try_exec_is_missing_does_not_start() {
    let machine = Machine::new();
    machine.write(
        "etc/xdg/autostart/bitwarden.desktop",
        &entry("Bitwarden", "TryExec=bitwarden"),
    );
    assert!(!machine.dirs.starts("bitwarden.desktop"));

    machine.write("bin/bitwarden", "");

    assert!(machine.dirs.starts("bitwarden.desktop"));
}

#[test]
fn the_startup_apps_are_sorted_by_name_and_named_by_their_entries() {
    let machine = Machine::new();
    machine.write("etc/xdg/autostart/z.desktop", &entry("alpha", ""));
    machine.write("config/autostart/a.desktop", &entry("Zulu", ""));
    machine.write("config/autostart/nameless.desktop", "[Desktop Entry]\nExec=x\n");

    let apps = machine.dirs.startup_apps();

    let names: Vec<&str> = apps.iter().map(|app| app.name.as_str()).collect();
    assert_eq!(names, ["alpha", "nameless", "Zulu"]);
    assert_eq!(apps[2].file, machine.root.path().join("config/autostart/a.desktop"));
}

#[test]
fn keys_outside_the_desktop_entry_group_are_not_read() {
    let parsed = Entry::parse(
        "# Hidden=true\n[Desktop Entry]\nName=App\nName[pl]=Aplikacja\n[Desktop Action new]\nHidden=true\n",
    );

    assert_eq!(parsed.get("Name"), Some("App"));
    assert_eq!(parsed.get("Name[pl]"), Some("Aplikacja"));
    assert!(!parsed.is_true("Hidden"));
}

#[test]
fn turning_an_app_on_writes_its_entry_from_its_desktop_file() {
    let machine = Machine::new();
    machine.app("app.desktop", "Icon=app\nMimeType=text/plain;\n");

    machine.dirs.turn_on("app.desktop").unwrap();

    assert_eq!(
        machine.read("config/autostart/app.desktop").as_deref(),
        Some("[Desktop Entry]\nType=Application\nName=App app.desktop\nIcon=app\nExec=/usr/bin/app %U\n")
    );
    assert!(machine.dirs.starts("app.desktop"));
}

#[test]
fn turning_an_app_on_brings_back_the_system_entry_the_user_hid() {
    let machine = Machine::new();
    machine.app("solaar.desktop", "");
    machine.write("etc/xdg/autostart/solaar.desktop", &entry("Solaar", ""));
    machine.dirs.turn_off("solaar.desktop").unwrap();
    assert!(!machine.dirs.starts("solaar.desktop"));

    machine.dirs.turn_on("solaar.desktop").unwrap();

    assert!(machine.read("config/autostart/solaar.desktop").is_none());
    assert!(machine.dirs.starts("solaar.desktop"));
}

#[test]
fn turning_an_app_on_over_a_system_entry_for_other_desktops_writes_the_users() {
    let machine = Machine::new();
    machine.app("keyring.desktop", "");
    machine.write(
        "etc/xdg/autostart/keyring.desktop",
        &entry("Keyring", "OnlyShowIn=GNOME;"),
    );

    machine.dirs.turn_on("keyring.desktop").unwrap();

    assert!(machine.dirs.starts("keyring.desktop"));
    assert!(machine.read("config/autostart/keyring.desktop").is_some());
}

#[test]
fn turning_off_an_app_with_a_system_entry_hides_it() {
    let machine = Machine::new();
    machine.write("etc/xdg/autostart/solaar.desktop", &entry("Solaar", ""));

    machine.dirs.turn_off("solaar.desktop").unwrap();

    assert_eq!(
        machine.read("config/autostart/solaar.desktop").as_deref(),
        Some("[Desktop Entry]\nType=Application\nName=Solaar\nHidden=true\n")
    );
    assert!(!machine.dirs.starts("solaar.desktop"));
}

#[test]
fn turning_off_an_app_only_the_user_starts_removes_its_entry() {
    let machine = Machine::new();
    machine.write("config/autostart/discord.desktop", &entry("Discord", ""));

    machine.dirs.turn_off("discord.desktop").unwrap();

    assert!(machine.read("config/autostart/discord.desktop").is_none());
}

#[test]
fn turning_on_an_app_that_starts_or_off_one_that_does_not_changes_nothing() {
    let machine = Machine::new();
    machine.write("etc/xdg/autostart/solaar.desktop", &entry("Solaar", ""));
    machine.app("app.desktop", "");

    machine.dirs.turn_on("solaar.desktop").unwrap();
    machine.dirs.turn_off("app.desktop").unwrap();

    assert!(!machine.root.path().join("config/autostart").exists());
}

#[test]
fn turning_on_an_app_that_is_not_there_says_so() {
    let machine = Machine::new();

    let err = machine.dirs.turn_on("gone.desktop").unwrap_err();

    assert!(matches!(err, AutostartError::NoApp(id) if id == "gone.desktop"));
}

#[test]
fn an_id_that_is_not_a_desktop_file_name_is_refused() {
    let machine = Machine::new();

    for id in ["solaar", "../x.desktop", ".desktop", "a/b.desktop"] {
        assert!(
            matches!(machine.dirs.turn_on(id), Err(AutostartError::InvalidId(_))),
            "{id}"
        );
        assert!(
            matches!(machine.dirs.turn_off(id), Err(AutostartError::InvalidId(_))),
            "{id}"
        );
    }
}

#[test]
fn the_apps_are_the_ones_a_launcher_shows_the_first_of_each_id_counting() {
    let mut machine = Machine::new();
    let local = machine.root.path().join("local/applications");
    machine.dirs.applications.insert(0, local);
    machine.app("shown.desktop", "");
    machine.app("nodisplay.desktop", "NoDisplay=true\n");
    machine.app("gnome.desktop", "OnlyShowIn=GNOME;\n");
    machine.app("overridden.desktop", "");
    machine.write(
        "local/applications/overridden.desktop",
        "[Desktop Entry]\nType=Application\nHidden=true\n",
    );
    machine.write("share/applications/link.desktop", "[Desktop Entry]\nType=Link\nURL=x\n");

    let mut apps = machine.dirs.apps();
    apps.sort();

    assert_eq!(apps, ["shown.desktop"]);
}
