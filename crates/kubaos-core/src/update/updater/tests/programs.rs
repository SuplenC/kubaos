use std::fs;

use super::Fake;
use super::World;
use super::notes;
use super::phases;
use super::summary;
use super::warnings;
use super::yes;
use crate::update::event::Outcome;
use crate::update::event::Phase;
use crate::update::event::Program;
use crate::update::event::ProgramChange;
use crate::update::event::ProgramResult;
use crate::update::event::UpdateEvent;
use crate::update::programs::Part;
use crate::update::programs::Switches;
use crate::update::record::Channel;

const DIGEST: &str = "02cb6bc6a429d8d0da9fdf9d23739358165cf7b98a245783d6f641526f75b0a7";
const BITWARDEN_INSTALLED: &str = "dpkg-query -W -f=${Version} bitwarden";
const DEB: &str = "Bitwarden-2026.12.0-amd64.deb";

fn releases() -> String {
    format!(
        r#"[
  {{"tag_name": "desktop-v2027.1.0", "draft": false, "prerelease": false,
    "assets": [{{"name": "Bitwarden-2027.1.0-amd64.deb", "digest": "sha256:{}"}}]}},
  {{"tag_name": "desktop-v2026.12.0", "draft": false, "prerelease": false,
    "assets": [{{"name": "{DEB}", "digest": "sha256:{DIGEST}"}}]}}
]"#,
        "a".repeat(64)
    )
}

/// A machine with Bitwarden desktop 2026.10.1 whose download of 2026.12.0 has `digest`.
fn with_bitwarden(digest: &str) -> Fake {
    let releases = releases();
    let sum = format!("{digest}  /home/{DEB}\n");
    Fake::new()
        .answering(BITWARDEN_INSTALLED, &["2026.10.1"])
        .answering("api.github.com", &[&releases])
        .answering("sha256sum", &[&sum])
}

fn finished(events: &[UpdateEvent], phase: Phase) -> Outcome {
    events
        .iter()
        .find_map(|e| match e {
            UpdateEvent::PhaseFinished { phase: p, outcome, .. } if *p == phase => Some(*outcome),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{phase:?} didn't finish"))
}

#[test]
fn bitwarden_moves_to_the_newest_release_of_its_series_from_a_checked_download() {
    let w = World::new(Channel::Edge, true);
    let mut m = with_bitwarden(DIGEST);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let install = m
        .ran
        .iter()
        .find(|line| line.starts_with("sudo apt-get install"))
        .expect("apt installed the .deb");
    assert!(install.ends_with(&format!("downloads/{DEB}")), "{install}");
    assert!(!w.user.update_downloads().join(DEB).exists());
    let ran = phases(&events);
    let home = ran.iter().position(|p| *p == Phase::Home).unwrap();
    assert_eq!(ran[home + 1], Phase::Bitwarden);
    assert_eq!(finished(&events, Phase::Bitwarden), Outcome::Done);
    assert!(
        notes(&events).contains(&"Bitwarden 2027.1.0 is out, a new series: it comes with a KubaOS release"),
        "{:?}",
        notes(&events)
    );
    assert_eq!(
        summary(&events).programs,
        [ProgramChange {
            program: Program::Bitwarden,
            result: ProgramResult::Updated {
                from: "2026.10.1".into(),
                to: "2026.12.0".into(),
            },
            waiting: Some("2027.1.0".into()),
        }]
    );
}

#[test]
fn a_download_that_isnt_the_published_one_installs_nothing_and_the_update_goes_on() {
    let w = World::new(Channel::Edge, true);
    let mut m = with_bitwarden(&"f".repeat(64));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!m.ran.iter().any(|line| line.contains(DEB) && line.contains("apt-get")));
    assert!(!w.user.update_downloads().join(DEB).exists());
    assert_eq!(finished(&events, Phase::Bitwarden), Outcome::Failed);
    let warning = warnings(&events)
        .into_iter()
        .find(|w| w.starts_with("Bitwarden:"))
        .expect("a warning");
    assert!(
        warning.contains("isn't the one GitHub published, so nothing was installed"),
        "{warning}"
    );
    let programs = &summary(&events).programs;
    assert!(
        matches!(programs[0].result, ProgramResult::Failed { .. }),
        "{programs:?}"
    );
}

#[test]
fn a_release_without_a_digest_installs_nothing() {
    let w = World::new(Channel::Edge, true);
    let releases = format!(r#"[{{"tag_name": "desktop-v2026.12.0", "assets": [{{"name": "{DEB}"}}]}}]"#);
    let mut m = Fake::new()
        .answering(BITWARDEN_INSTALLED, &["2026.10.1"])
        .answering("api.github.com", &[&releases])
        .answering("sha256sum", &[&format!("{DIGEST}  {DEB}")]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!m.ran.iter().any(|line| line.contains(DEB) && line.contains("apt-get")));
    assert!(
        warnings(&events)
            .iter()
            .any(|w| w.contains("publishes no sha256 digest"))
    );
}

#[test]
fn a_program_turned_off_skips_its_phase_and_asks_nothing() {
    let w = World::new(Channel::Edge, true);
    Switches::new(w.user.config_home()).turn_off(Part::Bitwarden).unwrap();
    let mut m = with_bitwarden(DIGEST);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!m.ran.iter().any(|line| line.starts_with("curl")));
    assert_eq!(finished(&events, Phase::Bitwarden), Outcome::Skipped);
    assert_eq!(
        summary(&events).programs,
        [ProgramChange {
            program: Program::Bitwarden,
            result: ProgramResult::Off,
            waiting: None,
        }]
    );
}

#[test]
fn a_machine_without_bitwarden_or_nix_runs_neither_phase() {
    let w = World::new(Channel::Edge, true);
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let ran = phases(&events);
    assert!(!ran.contains(&Phase::Bitwarden) && !ran.contains(&Phase::Nix));
    assert!(summary(&events).programs.is_empty());
}

const LISTING: &str = "<ListBucketResult><CommonPrefixes><Prefix>nix/nix-2.35.3/</Prefix></CommonPrefixes>\
<CommonPrefixes><Prefix>nix/nix-2.35.4/</Prefix></CommonPrefixes></ListBucketResult>";
const NEXT_LISTING: &str =
    "<ListBucketResult><CommonPrefixes><Prefix>nix/nix-2.36.0/</Prefix></CommonPrefixes></ListBucketResult>";
const FALLBACK_PATHS: &str = "{\n  x86_64-linux = \"/nix/store/jsx1mm71yz1n5k4zybqlsmajp416c5iy-nix-2.35.4\";\n}\n";

/// A world whose multi-user profile has Nix, its service unit changed in the new release.
fn with_nix(w: &World) {
    let profile = w.sys.root().join("nix/var/nix/profiles/default");
    fs::create_dir_all(profile.join("bin")).unwrap();
    fs::write(profile.join("bin/nix"), "").unwrap();
    let units = profile.join("lib/systemd/system");
    let etc = w.sys.root().join("etc/systemd/system");
    fs::create_dir_all(&units).unwrap();
    fs::create_dir_all(&etc).unwrap();
    fs::write(units.join("nix-daemon.service"), "new\n").unwrap();
    fs::write(etc.join("nix-daemon.service"), "old\n").unwrap();
    fs::write(units.join("nix-daemon.socket"), "same\n").unwrap();
    fs::write(etc.join("nix-daemon.socket"), "same\n").unwrap();
    fs::create_dir_all(w.sys.root().join("run/systemd/system")).unwrap();
}

#[test]
fn nix_moves_to_the_newest_patch_of_its_minor_and_restarts_the_daemon() {
    let w = World::new(Channel::Edge, true);
    with_nix(&w);
    let mut m = Fake::new()
        .answering("--version", &["nix (Nix) 2.35.3\n", "nix (Nix) 2.35.4\n"])
        .answering("prefix=nix/nix-2.35.", &[LISTING])
        .answering("prefix=nix/nix-2.36.", &[NEXT_LISTING])
        .answering("fallback-paths.nix", &[FALLBACK_PATHS]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let upgrade = m
        .ran
        .iter()
        .find(|line| line.contains("upgrade-nix"))
        .expect("upgrade-nix ran");
    assert!(upgrade.starts_with("sudo "), "{upgrade}");
    assert!(
        upgrade.ends_with("--nix-store-paths-url https://releases.nixos.org/nix/nix-2.35.4/fallback-paths.nix"),
        "{upgrade}"
    );
    let copies: Vec<&String> = m.ran.iter().filter(|line| line.starts_with("sudo cp")).collect();
    assert_eq!(copies.len(), 1, "{copies:?}");
    assert!(copies[0].ends_with("etc/systemd/system/nix-daemon.service"));
    let reload = m
        .ran
        .iter()
        .position(|line| line == "sudo systemctl daemon-reload")
        .unwrap();
    let restart = m
        .ran
        .iter()
        .position(|line| line == "sudo systemctl restart nix-daemon.service")
        .unwrap();
    assert!(reload < restart);
    assert_eq!(finished(&events, Phase::Nix), Outcome::Done);
    assert_eq!(
        summary(&events).programs,
        [ProgramChange {
            program: Program::Nix,
            result: ProgramResult::Updated {
                from: "2.35.3".into(),
                to: "2.35.4".into(),
            },
            waiting: Some("2.36.0".into()),
        }]
    );
}

#[test]
fn nix_at_the_newest_patch_stays_and_the_daemon_keeps_running() {
    let w = World::new(Channel::Edge, true);
    with_nix(&w);
    let mut m = Fake::new()
        .answering("--version", &["nix (Nix) 2.35.4\n"])
        .answering("prefix=nix/nix-2.35.", &[LISTING]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(
        !m.ran
            .iter()
            .any(|line| line.contains("upgrade-nix") || line.contains("systemctl"))
    );
    assert_eq!(
        summary(&events).programs[0].result,
        ProgramResult::Current {
            version: "2.35.4".into()
        }
    );
}

#[test]
fn a_fallback_file_naming_another_release_upgrades_nothing() {
    let w = World::new(Channel::Edge, true);
    with_nix(&w);
    let mut m = Fake::new()
        .answering("--version", &["nix (Nix) 2.35.2\n"])
        .answering("prefix=nix/nix-2.35.", &[LISTING])
        .answering("fallback-paths.nix", &[&FALLBACK_PATHS.replace("2.35.4", "2.35.3")]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!m.ran.iter().any(|line| line.contains("upgrade-nix")));
    assert_eq!(finished(&events, Phase::Nix), Outcome::Failed);
    assert!(
        warnings(&events)
            .iter()
            .any(|w| w.contains("names no x86_64-linux build of Nix 2.35.4")),
        "{:?}",
        warnings(&events)
    );
}
