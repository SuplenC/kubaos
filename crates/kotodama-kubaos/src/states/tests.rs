use std::fs;
use std::path::Path;

use kubaos_power::settings::CpuScheduler;
use kubaos_power::settings::Settings as PowerSettings;
use tempfile::TempDir;

use super::*;

const MANIFEST: &str = r#"
[group.base]
summary = "Base"
packages = ["zsh"]

[group.docker]
summary = "Docker"
optional = "on"
packages = ["docker-ce"]

[group.chrome]
summary = "Chrome"
optional = "on"
packages = ["google-chrome-stable"]

[group.piv]
summary = "PIV"
optional = "off"
packages = ["pcscd"]
"#;

/// A machine in a temporary folder: every place a state is read from under it, none of it there
/// yet.
struct Machine {
    dir: TempDir,
}

impl Machine {
    fn new() -> Self {
        Machine {
            dir: tempfile::tempdir().expect("a temporary folder"),
        }
    }

    fn places(&self) -> Places {
        let root = self.dir.path();
        Places {
            apt_choices: root.join("etc/kubaos/apt-groups"),
            apt_manifest: root.join("usr/share/kubaos/system/packages.toml"),
            config_home: Some(root.join("config")),
            keep_awake: Some(root.join("run/kubaos-keep-awake.json")),
            proxy: Some(root.join("run/kubaos-proxy.json")),
            dnd: Some(root.join("run/kubaos-dnd.json")),
            preview: Some(root.join("state/kubaos/preview.json")),
            shell_state: Some(root.join("state/quickshell/kubaos.json")),
        }
    }

    fn write(&self, path: impl AsRef<Path>, text: &str) {
        let path = self.dir.path().join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("make the folder");
        fs::write(path, text).expect("write the file");
    }

    fn read(&self) -> States {
        States::read(&self.places())
    }
}

fn on(value: bool) -> Reading {
    Reading::Bool(value)
}

fn word(value: &str) -> Reading {
    Reading::Word(value.to_owned())
}

#[test]
fn an_optional_package_group_follows_the_users_choice_else_its_default() {
    let machine = Machine::new();
    machine.write("usr/share/kubaos/system/packages.toml", MANIFEST);
    machine.write("etc/kubaos/apt-groups", "# Optional groups\nchrome=off\npiv=on\n");

    let states = machine.read();

    assert_eq!(states.get("packages.chrome"), Some(&on(false)));
    assert_eq!(states.get("packages.piv"), Some(&on(true)));
    assert_eq!(states.get("packages.docker"), Some(&on(true)));
    assert_eq!(states.get("packages.base"), None);
}

#[test]
fn without_a_choices_file_every_optional_group_is_at_its_default() {
    let machine = Machine::new();
    machine.write("usr/share/kubaos/system/packages.toml", MANIFEST);

    let states = machine.read();

    assert_eq!(states.get("packages.chrome"), Some(&on(true)));
    assert_eq!(states.get("packages.piv"), Some(&on(false)));
}

#[test]
fn without_the_manifest_only_the_groups_the_user_chose_are_known() {
    let machine = Machine::new();
    machine.write("etc/kubaos/apt-groups", "chrome=on\n");

    let states = machine.read();

    assert_eq!(states.get("packages.chrome"), Some(&on(true)));
    assert_eq!(states.get("packages.docker"), None);
}

#[test]
fn a_manifest_that_does_not_parse_leaves_only_the_choices() {
    let machine = Machine::new();
    machine.write("usr/share/kubaos/system/packages.toml", "[group.chrome\n");
    machine.write("etc/kubaos/apt-groups", "piv=on\n");

    let states = machine.read();

    assert_eq!(states.get("packages.piv"), Some(&on(true)));
    assert_eq!(states.get("packages.chrome"), None);
}

#[test]
fn the_files_settings_are_their_flag_files() {
    let machine = Machine::new();

    let before = machine.read();
    machine.write("config/kubaos/desktop/files/tidy-downloads", "");
    machine.write("config/kubaos/desktop/files/no-automount", "");
    let after = machine.read();

    assert_eq!(before.get(TIDY_DAILY), Some(&on(false)));
    assert_eq!(before.get(AUTOMOUNT), Some(&on(true)));
    assert_eq!(after.get(TIDY_DAILY), Some(&on(true)));
    assert_eq!(after.get(AUTOMOUNT), Some(&on(false)));
}

#[test]
fn the_greeting_is_on_until_its_off_file_exists() {
    let machine = Machine::new();

    let before = machine.read();
    machine.write("config/kubaos/terminal/zsh/no-greeting", "");
    let after = machine.read();

    assert_eq!(before.get(GREETING), Some(&on(true)));
    assert_eq!(after.get(GREETING), Some(&on(false)));
}

#[test]
fn the_phone_link_is_on_while_its_file_exists() {
    let machine = Machine::new();

    let before = machine.read();
    machine.write("config/kubaos/desktop/phone/phone-link", "");
    let after = machine.read();

    assert_eq!(before.get(PHONE_LINK), Some(&on(false)));
    assert_eq!(after.get(PHONE_LINK), Some(&on(true)));
}

#[test]
fn the_mac_accents_are_the_keyboard_settings_and_unknown_when_they_do_not_read() {
    let machine = Machine::new();

    let missing = machine.read();
    machine.write("config/kubaos/desktop/keyboard/keyboard.toml", "mac_accents = false\n");
    let off = machine.read();
    machine.write("config/kubaos/desktop/keyboard/keyboard.toml", "mac_accents = 3\n");
    let broken = machine.read();

    assert_eq!(missing.get(MAC_ACCENTS), Some(&on(true)));
    assert_eq!(off.get(MAC_ACCENTS), Some(&on(false)));
    assert_eq!(broken.get(MAC_ACCENTS), None);
}

#[test]
fn keep_awake_is_its_state_files_active_and_unknown_when_it_does_not_read() {
    let machine = Machine::new();

    let missing = machine.read();
    machine.write("run/kubaos-keep-awake.json", r#"{"active": true, "until": 0}"#);
    let held = machine.read();
    machine.write("run/kubaos-keep-awake.json", "{\"active\": tr");
    let broken = machine.read();

    assert_eq!(missing.get(KEEP_AWAKE), Some(&on(false)));
    assert_eq!(held.get(KEEP_AWAKE), Some(&on(true)));
    assert_eq!(broken.get(KEEP_AWAKE), None);
}

#[test]
fn the_proxy_is_on_while_the_user_wants_it_and_unknown_without_its_file() {
    let machine = Machine::new();

    let missing = machine.read();
    machine.write(
        "run/kubaos-proxy.json",
        r#"{"configured": true, "wanted": true, "on": false}"#,
    );
    let wanted = machine.read();
    machine.write(
        "run/kubaos-proxy.json",
        r#"{"configured": true, "wanted": false, "on": false}"#,
    );
    let unwanted = machine.read();

    assert_eq!(missing.get(PROXY), None);
    assert_eq!(wanted.get(PROXY), Some(&on(true)));
    assert_eq!(unwanted.get(PROXY), Some(&on(false)));
}

#[test]
fn do_not_disturb_is_its_state_files_on_and_unknown_without_the_shell() {
    let machine = Machine::new();

    let missing = machine.read();
    machine.write(
        "run/kubaos-dnd.json",
        r#"{"on": true, "manual": false, "reason": "sharing"}"#,
    );
    let sharing = machine.read();
    machine.write("run/kubaos-dnd.json", r#"{"on": false, "manual": false, "reason": ""}"#);
    let off = machine.read();

    assert_eq!(missing.get(DND), None);
    assert_eq!(sharing.get(DND), Some(&on(true)));
    assert_eq!(off.get(DND), Some(&on(false)));
}

#[test]
fn a_live_preview_is_on_while_its_record_names_one() {
    let machine = Machine::new();

    let missing = machine.read();
    machine.write(
        "state/kubaos/preview.json",
        r#"{"path": "/h/Projects/kubaos", "branch": "feature/x", "commit": "1a2b3c4d", "dirty": false, "since": 1}"#,
    );
    let previewing = machine.read();
    machine.write("state/kubaos/preview.json", "");
    let released = machine.read();
    machine.write("state/kubaos/preview.json", "{ not a record");
    let broken = machine.read();

    assert_eq!(missing.get(PREVIEW), Some(&on(false)));
    assert_eq!(previewing.get(PREVIEW), Some(&on(true)));
    assert_eq!(released.get(PREVIEW), Some(&on(false)));
    assert_eq!(broken.get(PREVIEW), None);
}

#[test]
fn unnamed_bluetooth_devices_are_hidden_until_the_shell_remembers_them_shown() {
    let machine = Machine::new();

    let missing = machine.read();
    machine.write("state/quickshell/kubaos.json", r#"{"animate": "ac"}"#);
    let unset = machine.read();
    machine.write(
        "state/quickshell/kubaos.json",
        r#"{"animate": "ac", "bluetoothUnnamed": true}"#,
    );
    let shown = machine.read();
    machine.write(
        "state/quickshell/kubaos.json",
        r#"{"animate": "ac", "bluetoothUnnamed": false}"#,
    );
    let hidden = machine.read();
    machine.write("state/quickshell/kubaos.json", "{ not json");
    let broken = machine.read();

    assert_eq!(missing.get(BLUETOOTH_UNNAMED), Some(&on(false)));
    assert_eq!(unset.get(BLUETOOTH_UNNAMED), Some(&on(false)));
    assert_eq!(shown.get(BLUETOOTH_UNNAMED), Some(&on(true)));
    assert_eq!(hidden.get(BLUETOOTH_UNNAMED), Some(&on(false)));
    assert_eq!(broken.get(BLUETOOTH_UNNAMED), None);
}

#[test]
fn the_power_settings_are_read_from_their_file_with_its_defaults() {
    let machine = Machine::new();

    let defaults = machine.read();
    machine.write(
        "config/kubaos/desktop/power/power.toml",
        "panel_power_saving = false\nscheduler = \"lavd\"\nlow_battery_stop = \"off\"\n",
    );
    let set = machine.read();
    machine.write("config/kubaos/desktop/power/power.toml", "scheduler = \"fifo\"\n");
    let broken = machine.read();

    assert_eq!(defaults.get(PANEL_POWER_SAVING), Some(&on(true)));
    assert_eq!(defaults.get(SCHEDULER), Some(&word("kernel")));
    assert_eq!(defaults.get(LOW_BATTERY), Some(&word("power-saver")));
    assert_eq!(set.get(PANEL_POWER_SAVING), Some(&on(false)));
    assert_eq!(set.get(SCHEDULER), Some(&word("lavd")));
    assert_eq!(set.get(LOW_BATTERY), Some(&word("off")));
    assert_eq!(broken.get(SCHEDULER), None);
}

#[test]
fn unfree_packages_are_on_until_the_user_flakes_setting_turns_them_off() {
    let machine = Machine::new();

    let missing = machine.read();
    machine.write("config/kubaos/home/kubaos.d/nixpkgs.toml", "allow_unfree = false\n");
    let off = machine.read();
    machine.write("config/kubaos/home/kubaos.d/nixpkgs.toml", "allow_unfree = 0\n");
    let broken = machine.read();

    assert_eq!(missing.get(UNFREE), Some(&on(true)));
    assert_eq!(off.get(UNFREE), Some(&on(false)));
    assert_eq!(broken.get(UNFREE), None);
}

#[test]
fn without_a_config_home_or_runtime_dir_their_states_are_unknown() {
    let machine = Machine::new();
    let places = Places {
        config_home: None,
        keep_awake: None,
        proxy: None,
        dnd: None,
        preview: None,
        shell_state: None,
        ..machine.places()
    };

    let states = States::read(&places);

    for name in [
        TIDY_DAILY,
        AUTOMOUNT,
        GREETING,
        MAC_ACCENTS,
        PHONE_LINK,
        PANEL_POWER_SAVING,
        UNFREE,
        KEEP_AWAKE,
        PROXY,
        DND,
        PREVIEW,
        BLUETOOTH_UNNAMED,
    ] {
        assert_eq!(states.get(name), None, "{name}");
    }
}

#[test]
fn every_file_a_state_is_read_from_is_watched() {
    let machine = Machine::new();
    let places = machine.places();
    let root = machine.dir.path();

    let files = places.files();

    for file in [
        "etc/kubaos/apt-groups",
        "usr/share/kubaos/system/packages.toml",
        "config/kubaos/desktop/files/tidy-downloads",
        "config/kubaos/desktop/files/no-automount",
        "config/kubaos/terminal/zsh/no-greeting",
        "config/kubaos/desktop/keyboard/keyboard.toml",
        "config/kubaos/desktop/phone/phone-link",
        "config/kubaos/desktop/power/power.toml",
        "config/kubaos/home/kubaos.d/nixpkgs.toml",
        "run/kubaos-keep-awake.json",
        "run/kubaos-proxy.json",
        "run/kubaos-dnd.json",
        "state/kubaos/preview.json",
        "state/quickshell/kubaos.json",
    ] {
        assert!(files.contains(&root.join(file)), "{file} isn't watched: {files:?}");
    }
}

#[test]
fn a_state_name_is_one_of_the_probes_or_a_package_group() {
    assert_eq!(values("packages.chrome"), Some(Values::Bool));
    assert_eq!(values(PROXY), Some(Values::Bool));
    assert_eq!(values(DND), Some(Values::Bool));
    assert_eq!(values(UNFREE), Some(Values::Bool));
    assert_eq!(values(PREVIEW), Some(Values::Bool));
    assert_eq!(values(BLUETOOTH_UNNAMED), Some(Values::Bool));
    assert_eq!(values(SCHEDULER), Some(Values::Words(&["kernel", "lavd"])));
    assert_eq!(values("packages."), None);
    assert_eq!(values("chrome"), None);
}

#[test]
fn the_word_states_take_the_words_the_power_settings_have() {
    let schedulers: Vec<&str> = CpuScheduler::ALL.into_iter().map(CpuScheduler::name).collect();
    let mut stops: Vec<&str> = PowerSettings::LOW_BATTERY_STOPS
        .into_iter()
        .map(kubaos_power::stop::Stop::id)
        .collect();
    stops.push("off");

    let words = |name| match values(name) {
        Some(Values::Words(words)) => words.to_vec(),
        other => panic!("{name} reads as {other:?}"),
    };

    assert_eq!(words(SCHEDULER), schedulers);
    assert_eq!(words(LOW_BATTERY), stops);
}

#[test]
fn a_reading_is_allowed_only_of_its_states_kind() {
    let words = Values::Words(&["kernel", "lavd"]);

    assert!(Values::Bool.allow(&Reading::Bool(false)));
    assert!(!Values::Bool.allow(&Reading::Word("on".to_owned())));
    assert!(words.allow(&Reading::Word("lavd".to_owned())));
    assert!(!words.allow(&Reading::Word("fifo".to_owned())));
    assert!(!words.allow(&Reading::Bool(true)));
}
