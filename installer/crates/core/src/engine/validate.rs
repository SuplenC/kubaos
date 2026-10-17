//! Config vs. machine vs. payload. Every problem is collected, so the GUI
//! (and `dotinstall validate`) can show them all at once.

use super::disk::{self, MIB};
use super::machine::gib;
use crate::config::{DiskTarget, InstallConfig, Secrets};
use crate::probe::Probe;

pub fn problems(c: &InstallConfig, p: &Probe) -> Vec<String> {
    let mut out = Vec::new();
    let mut bad = |s: String| out.push(s);

    // Payload.
    let manifest = p.payload.as_ref();
    match manifest {
        None => bad(format!(
            "no payload found in {} (manifest.json missing or unreadable)",
            c.payload.dir
        )),
        Some(m) => {
            if m.arch != p.arch {
                bad(format!(
                    "the payload is built for {}, this machine is {}",
                    m.arch, p.arch
                ));
            }
            if c.host.hostname != m.hostname {
                bad(format!(
                    "hostname must be \"{}\": the payload is built for the flake output {}@{}",
                    m.hostname, m.username, m.hostname
                ));
            }
            if c.user.username != m.username {
                bad(format!(
                    "username must be \"{}\": the payload's home-manager generation is built for that user",
                    m.username
                ));
            }
        }
    }
    if !p.uefi {
        bad("the machine didn't boot in UEFI mode; dotinstall installs GRUB for EFI only".into());
    }

    // User and host.
    if !valid_username(&c.user.username) {
        bad(format!(
            "username \"{}\" must start with a lowercase letter and use only a-z, 0-9, - and _ (at most 32)",
            c.user.username
        ));
    }
    if c.user.full_name.contains([':', '\n', ',']) {
        bad("the full name can't contain ':', ',' or a newline".into());
    }
    if !valid_hostname(&c.host.hostname) {
        bad(format!(
            "hostname \"{}\" must be 1-63 characters of a-z, 0-9 and -, not starting or ending with -",
            c.host.hostname
        ));
    }

    // Locale. Empty lists mean probe couldn't read them; don't block on that.
    if !p.timezones.is_empty() && !p.timezones.contains(&c.locale.timezone) {
        bad(format!("unknown timezone \"{}\"", c.locale.timezone));
    }
    if !p.keymaps.is_empty() && !p.keymaps.contains(&c.locale.keyboard_layout) {
        bad(format!(
            "unknown keyboard layout \"{}\"",
            c.locale.keyboard_layout
        ));
    }
    if !c
        .locale
        .keyboard_variant
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "_-".contains(ch))
    {
        bad(format!(
            "odd keyboard variant \"{}\"",
            c.locale.keyboard_variant
        ));
    }
    if !p.locales.is_empty() && !p.locales.contains(&c.locale.lang) {
        bad(format!("unknown locale \"{}\"", c.locale.lang));
    }

    // Disk.
    let Some(d) = p.disks.iter().find(|d| d.path == c.disk.device) else {
        bad(format!("{} is not a disk this machine has", c.disk.device));
        return out;
    };
    if d.is_boot_medium {
        bad(format!("{} is the medium the installer runs from", d.path));
    }
    let installed = manifest.map_or(0, |m| m.installed_bytes);
    match c.disk.target {
        DiskTarget::Whole => {
            let need = disk::required_bytes(installed, false);
            if d.size_bytes < need {
                bad(format!(
                    "{} has {}, the install needs at least {}",
                    d.path,
                    gib(d.size_bytes),
                    gib(need)
                ));
            }
        }
        DiskTarget::FreeSpace { start, end } => {
            // Probe's regions are aligned already; allow the GUI to pick a
            // sub-range of one.
            let inside = d
                .free_regions
                .iter()
                .any(|r| r.start <= start && end <= r.end && start < end);
            if !inside {
                bad(format!(
                    "{start}..{end} on {} isn't inside a free region (see `dotinstall probe`)",
                    d.path
                ));
            }
            let has_esp = d
                .partitions
                .iter()
                .any(|p| p.os_hint.as_deref() == Some("EFI system"));
            let need = disk::required_bytes(installed, has_esp);
            let have = end.saturating_sub(start).saturating_sub(MIB);
            if have < need {
                bad(format!(
                    "the free region has {}, the install needs {}",
                    gib(have),
                    gib(need)
                ));
            }
        }
    }
    out
}

pub fn secret_problems(c: &InstallConfig, s: &Secrets) -> Vec<String> {
    let mut out = Vec::new();
    if s.user_password.is_empty() {
        out.push("the user password is empty".into());
    }
    if s.user_password.contains(['\n', '\r']) {
        out.push("the user password can't contain a line break".into());
    }
    if c.disk.encrypt && s.luks_passphrase.as_deref().is_none_or(str::is_empty) {
        out.push("encryption is on but there's no LUKS passphrase".into());
    }
    out
}

pub fn valid_username(u: &str) -> bool {
    let mut ch = u.chars();
    u.len() <= 32
        && ch
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && ch.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

pub fn valid_hostname(h: &str) -> bool {
    (1..=63).contains(&h.len())
        && !h.starts_with('-')
        && !h.ends_with('-')
        && h.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use crate::probe::{DiskInfo, Manifest, PartitionInfo, Region};

    pub fn manifest() -> Manifest {
        Manifest {
            version: "1".into(),
            codename: None,
            arch: "x86_64".into(),
            built_at: "2026-10-16T10:00:00Z".into(),
            commit: "abc".into(),
            username: "kuba".into(),
            hostname: "tuxedo".into(),
            rootfs: "rootfs.sqfs".into(),
            rootfs_bytes: 4 << 30,
            installed_bytes: 12 << 30,
            sha256: "00".into(),
            kernel: "6.17.0-5-generic".into(),
            home_generation: "/nix/store/xxx-home-manager-generation".into(),
        }
    }

    fn probe() -> Probe {
        Probe {
            arch: "x86_64".into(),
            uefi: true,
            disks: vec![
                DiskInfo {
                    path: "/dev/nvme0n1".into(),
                    model: "990".into(),
                    size_bytes: 1000 << 30,
                    removable: false,
                    is_boot_medium: false,
                    partitions: vec![PartitionInfo {
                        path: "/dev/nvme0n1p1".into(),
                        size_bytes: 100 << 20,
                        fstype: Some("vfat".into()),
                        label: None,
                        os_hint: Some("EFI system".into()),
                    }],
                    free_regions: vec![Region {
                        start: 500 << 30,
                        end: 900 << 30,
                    }],
                },
                DiskInfo {
                    path: "/dev/sda".into(),
                    model: "stick".into(),
                    size_bytes: 32 << 30,
                    removable: true,
                    is_boot_medium: true,
                    partitions: vec![],
                    free_regions: vec![],
                },
            ],
            keymaps: vec!["us".into(), "it".into()],
            timezones: vec!["Europe/Rome".into(), "UTC".into()],
            locales: vec!["en_US.UTF-8".into()],
            wifi: vec![],
            online: false,
            payload: Some(manifest()),
        }
    }

    pub fn config() -> InstallConfig {
        InstallConfig::from_toml(
            r#"
            [locale]
            lang = "en_US.UTF-8"
            keyboard_layout = "us"
            timezone = "Europe/Rome"
            [host]
            hostname = "tuxedo"
            [user]
            username = "kuba"
            full_name = "Kuba Example"
            [disk]
            device = "/dev/nvme0n1"
            target = { mode = "whole" }
            [payload]
            dir = "/run/dotinstall/payload"
            "#,
        )
        .unwrap()
    }

    #[test]
    fn good_config_passes() {
        assert_eq!(problems(&config(), &probe()), Vec::<String>::new());
    }

    #[test]
    fn collects_every_problem() {
        let mut c = config();
        c.user.username = "Root".into();
        c.host.hostname = "-bad".into();
        c.locale.timezone = "Mars/Olympus".into();
        c.locale.keyboard_layout = "xx".into();
        c.disk.device = "/dev/sda".into();
        let p = problems(&c, &probe());
        assert_eq!(p.len(), 7, "{p:#?}");
        assert!(p[0].contains("hostname must be \"tuxedo\""));
        assert!(p[1].contains("username must be \"kuba\""));
        assert!(
            p.iter()
                .any(|s| s.contains("medium the installer runs from"))
        );
        assert!(p.iter().any(|s| s.contains("Mars/Olympus")));
    }

    #[test]
    fn disk_checks() {
        let mut c = config();
        c.disk.device = "/dev/sdz".into();
        assert!(problems(&c, &probe())[0].contains("not a disk"));

        let mut p = probe();
        p.disks[0].size_bytes = 16 << 30;
        assert!(problems(&config(), &p)[0].contains("needs at least"));

        let mut c = config();
        c.disk.target = DiskTarget::FreeSpace {
            start: 600 << 30,
            end: 700 << 30,
        };
        assert!(problems(&c, &probe()).is_empty());
        c.disk.target = DiskTarget::FreeSpace {
            start: 400 << 30,
            end: 700 << 30,
        };
        assert!(problems(&c, &probe())[0].contains("isn't inside a free region"));
        c.disk.target = DiskTarget::FreeSpace {
            start: 600 << 30,
            end: (600 << 30) + (10 << 30),
        };
        assert!(problems(&c, &probe())[0].contains("the install needs"));
    }

    #[test]
    fn hostname_must_match_the_payload() {
        let mut c = config();
        c.host.hostname = "vm".into();
        let p = problems(&c, &probe());
        assert_eq!(p.len(), 1);
        assert!(p[0].contains("kuba@tuxedo"), "{}", p[0]);
    }

    #[test]
    fn payload_checks() {
        let mut p = probe();
        p.payload = None;
        assert!(problems(&config(), &p)[0].contains("no payload"));
        let mut p = probe();
        p.uefi = false;
        p.arch = "aarch64".into();
        assert_eq!(problems(&config(), &p).len(), 2);
    }

    #[test]
    fn secrets() {
        let c = config();
        let s = Secrets {
            luks_passphrase: None,
            user_password: "pw".into(),
            wifi_psk: None,
        };
        assert_eq!(secret_problems(&c, &s).len(), 1);
        let s = Secrets {
            luks_passphrase: Some("x".into()),
            user_password: String::new(),
            wifi_psk: None,
        };
        assert_eq!(secret_problems(&c, &s).len(), 1);
        let s = Secrets {
            luks_passphrase: Some("x".into()),
            user_password: "a\nb".into(),
            wifi_psk: None,
        };
        assert_eq!(secret_problems(&c, &s).len(), 1);
        let mut c2 = c.clone();
        c2.disk.encrypt = false;
        let s = Secrets {
            luks_passphrase: None,
            user_password: "pw".into(),
            wifi_psk: None,
        };
        assert!(secret_problems(&c2, &s).is_empty());
    }

    #[test]
    fn names() {
        assert!(valid_username("kuba"));
        assert!(valid_username("_svc-1"));
        assert!(!valid_username("1kuba"));
        assert!(!valid_username(""));
        assert!(valid_hostname("vm"));
        assert!(valid_hostname("tuxedo-2"));
        assert!(!valid_hostname("Tuxedo"));
        assert!(!valid_hostname("a.b"));
        assert!(!valid_hostname(&"a".repeat(64)));
    }
}
