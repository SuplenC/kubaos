use crate::update::changelog::Changelog;
use crate::update::event::Command;
use crate::update::event::PackageChange;
use crate::update::event::Program;
use crate::update::event::ProgramChange;
use crate::update::event::ProgramResult;
use crate::update::event::Release;
use crate::update::event::Restart;
use crate::update::event::SuiteChanges;
use crate::update::event::SuiteTool;
use crate::update::event::Summary;
use crate::update::firmware::Firmware;
use crate::update::firmware::FirmwareOutcome;
use crate::update::firmware::FirmwareUpdate;
use crate::update::record::Channel;

fn release(version: &str, commit: &str) -> Release {
    Release {
        version: version.into(),
        codename: Some("shinjuku".into()),
        commit: commit.into(),
    }
}

/// An update that stayed at its release and changed nothing.
fn quiet() -> Summary {
    Summary {
        command: Command::Update,
        from: release("0.1.0", "1a2b3c4"),
        to: release("0.1.0", "1a2b3c4"),
        channel: Channel::Stable,
        changelog: Changelog::default(),
        packages: 0,
        upgraded: Vec::new(),
        apt: Vec::new(),
        inputs: Vec::new(),
        nix_packages: Some(Vec::new()),
        migrations: Vec::new(),
        programs: vec![ProgramChange {
            program: Program::Nix,
            result: ProgramResult::Current {
                version: "2.35.4".into(),
            },
            waiting: None,
        }],
        suites: SuiteChanges::default(),
        reloaded: vec!["reloaded Hyprland".into()],
        firmware: Firmware::default(),
        restart: Restart::None,
        restart_reason: None,
        undo: Some("kubaos rollback".into()),
        log: None,
    }
}

/// An update to 0.1.1 that moved a bit of everything and staged firmware for the reboot.
fn busy() -> Summary {
    let mut firmware = Firmware::default();
    firmware.push(
        FirmwareUpdate {
            device: "System Firmware".into(),
            id: "a45d".into(),
            from: "1.2.3".into(),
            to: "1.3.0".into(),
            reboot: true,
        },
        FirmwareOutcome::Staged,
    );
    Summary {
        to: release("0.1.1", "9f8e7d6"),
        packages: 12,
        nix_packages: Some(vec![PackageChange {
            name: "btop".into(),
            from: Some("1.4.7".into()),
            to: Some("1.4.8".into()),
        }]),
        programs: vec![ProgramChange {
            program: Program::Bitwarden,
            result: ProgramResult::Updated {
                from: "2026.10.1".into(),
                to: "2026.12.0".into(),
            },
            waiting: None,
        }],
        firmware,
        restart: Restart::Reboot,
        restart_reason: Some("firmware is staged to install at it".into()),
        ..quiet()
    }
}

#[test]
fn an_update_that_moved_nothing_changed_nothing_whatever_the_session_reloaded() {
    assert!(quiet().changed_nothing());
}

#[test]
fn any_moved_part_or_a_needed_restart_is_a_change() {
    let changes: [fn(&mut Summary); 6] = [
        |s| s.to.commit = "9f8e7d6".into(),
        |s| s.packages = 1,
        |s| s.inputs = vec!["nixpkgs".into()],
        |s| s.migrations = vec!["1792765570-backlight-rule.sh".into()],
        |s| s.programs[0].waiting = Some("2.36.0".into()),
        |s| s.restart = Restart::Relogin,
    ];

    for change in changes {
        let mut summary = quiet();
        change(&mut summary);
        assert!(!summary.changed_nothing(), "{summary:?}");
    }
}

#[test]
fn the_notification_puts_the_reboot_first_and_one_line_a_part_after_it() {
    let notification = busy().notification();

    assert_eq!(notification.title, "KubaOS updated to Shinjuku 0.1.1");
    assert_eq!(
        notification.body,
        "Reboot: firmware is staged to install at it\n\
         12 apt packages upgraded\n\
         1 Nix package changed\n\
         Bitwarden 2026.10.1 → 2026.12.0\n\
         Firmware: 1 staged for the reboot"
    );
    assert!(notification.urgent);
}

#[test]
fn nothing_changed_is_the_title_alone_and_not_urgent() {
    let notification = quiet().notification();

    assert_eq!(notification.title, "KubaOS is up to date: nothing changed");
    assert_eq!(notification.body, "");
    assert!(!notification.urgent);
}

#[test]
fn a_failed_part_asks_for_attention_and_points_at_the_log() {
    let mut summary = quiet();
    summary.programs[0].result = ProgramResult::Failed {
        error: "downloading nix".into(),
    };

    let notification = summary.notification();

    assert_eq!(notification.title, "KubaOS updated, still at Shinjuku 0.1.0");
    assert_eq!(notification.body, "Nix failed to update: the log says why");
    assert!(notification.urgent);
}

fn tool(from: &str, to: &str) -> SuiteTool {
    SuiteTool {
        suite: "go".into(),
        tool: "gopls".into(),
        from: Some(from.into()),
        to: to.into(),
    }
}

#[test]
fn a_moved_suite_tool_or_a_waiting_upgrade_is_a_change() {
    let changes: [fn(&mut Summary); 3] = [
        |s| s.suites.updated = vec![tool("0.22.1", "0.22.3")],
        |s| s.suites.upgrades = vec![tool("0.22", "0.24")],
        |s| s.suites.error = Some("mise failed".into()),
    ];

    for change in changes {
        let mut summary = quiet();
        change(&mut summary);
        assert!(!summary.changed_nothing(), "{summary:?}");
    }
}

#[test]
fn the_notification_counts_the_suite_tools_and_points_at_the_upgrade() {
    let mut summary = quiet();
    summary.suites.updated = vec![tool("0.22.1", "0.22.3"), tool("1.27.0", "1.27.2")];
    summary.suites.upgrades = vec![tool("0.22", "0.24")];

    let notification = summary.notification();

    assert_eq!(
        notification.body,
        "2 Suite tools updated\n1 Suite tool upgrade waits: kubaos suite upgrade"
    );
    assert!(!notification.urgent);
}

#[test]
fn a_suite_tool_reads_as_suite_tool_and_its_versions() {
    assert_eq!(tool("0.22", "0.24").describe(), "go: gopls 0.22 → 0.24");
    let fresh = SuiteTool {
        from: None,
        ..tool("", "0.24.0")
    };
    assert_eq!(fresh.describe(), "go: gopls 0.24.0");
}
