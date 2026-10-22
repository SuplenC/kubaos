//! The answers so far, their validation, and turning them into
//! `install.toml` plus the secrets.

use kubaos_core::config::{Disk, DiskTarget, Git, Host, Locale, Payload, User, Wallpapers, Wifi};
use kubaos_core::probe::{DiskInfo, Region};
use kubaos_core::{InstallConfig, Probe, Secrets};

use crate::data;

/// Where the live ISO keeps the payload (docs/INSTALL-INTERNALS.md, `[payload]`).
pub const PAYLOAD_DIR: &str = "/run/kubaos/payload";
/// Smallest free region offered for a dual-boot install, on top of the payload size.
const HEADROOM: u64 = 8_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskMode {
    Whole,
    FreeSpace,
}

#[derive(Debug, Clone, Default)]
pub struct Form {
    pub locale: String,
    pub keymap: String,
    pub keyboard_test: String,
    pub timezone: String,

    pub wifi_ssid: Option<String>,
    pub wifi_psk: String,

    pub full_name: String,
    pub username: String,
    pub password: String,
    pub password_confirm: String,

    pub disk: Option<String>,
    pub disk_mode: Option<DiskMode>,
    pub encrypt: bool,
    pub luks_same_as_password: bool,
    pub luks: String,
    pub luks_confirm: String,

    pub git_name: String,
    pub git_email: String,
    /// The user edited the git name; stop mirroring the full name into it.
    pub git_name_touched: bool,

    /// Ids of the wallpaper sets picked, in no particular order.
    pub wallpaper_sets: Vec<String>,
}

impl Form {
    pub fn new(probe: &Probe) -> Self {
        let pick = |list: &[String], want: &str| {
            list.iter()
                .find(|s| s.as_str() == want)
                .or(list.first())
                .cloned()
                .unwrap_or_else(|| want.to_string())
        };
        let manifest = probe.payload.as_ref();
        let disk = probe
            .disks
            .iter()
            .find(|d| d.is_target())
            .map(|d| d.path.clone());
        let mut f = Self {
            locale: pick(&probe.locales, "en_US.UTF-8"),
            keymap: pick(&probe.keymaps, "us"),
            timezone: pick(&probe.timezones, "Europe/Rome"),
            username: manifest.map(|m| m.username.clone()).unwrap_or_default(),
            encrypt: true,
            luks_same_as_password: true,
            disk,
            wallpaper_sets: manifest
                .map(|m| m.wallpaper_sets.iter().map(|s| s.id.clone()).collect())
                .unwrap_or_default(),
            ..Self::default()
        };
        f.disk_mode = f.selected_disk(probe).map(|d| d.default_mode(probe));
        f
    }

    pub fn selected_disk<'a>(&self, probe: &'a Probe) -> Option<&'a DiskInfo> {
        let path = self.disk.as_deref()?;
        probe.disks.iter().find(|d| d.path == path)
    }

    /// The free region a dual-boot install would use: the largest one.
    pub fn free_region(&self, probe: &Probe) -> Option<Region> {
        self.selected_disk(probe)?.largest_free()
    }

    pub fn password_ok(&self) -> Result<(), &'static str> {
        if self.password.is_empty() {
            Err("Pick a password.")
        } else if still_typing(&self.password, &self.password_confirm) {
            Err("Type it again to confirm.")
        } else if self.password != self.password_confirm {
            Err("The passwords don't match.")
        } else {
            Ok(())
        }
    }

    pub fn account_ok(&self) -> Result<(), &'static str> {
        use kubaos_core::engine::validate::{reserved_username, valid_username};
        if self.full_name.trim().is_empty() {
            return Err("Type your name.");
        }
        if self.username.is_empty() {
            return Err("Pick a username.");
        }
        if !valid_username(&self.username) {
            return Err("Usernames are lowercase: a-z first, then a-z, 0-9, - or _.");
        }
        if reserved_username(&self.username) {
            return Err("That one's a system account. Pick another.");
        }
        self.password_ok()
    }

    pub fn luks_passphrase(&self) -> Option<&str> {
        if !self.encrypt {
            None
        } else if self.luks_same_as_password {
            Some(&self.password)
        } else {
            Some(&self.luks)
        }
    }

    pub fn disk_ok(&self, probe: &Probe) -> Result<(), &'static str> {
        let Some(disk) = self.selected_disk(probe) else {
            return Err("Pick a disk.");
        };
        match self.disk_mode {
            None => return Err("Pick: alongside or erase."),
            Some(DiskMode::FreeSpace) if !disk.fits_dual_boot(probe) => {
                return Err(
                    "Not enough free space to live alongside. Erase it, or shrink something first.",
                );
            }
            Some(DiskMode::Whole) if !disk.fits(probe) => {
                return Err("This disk is too small for KubaOS.");
            }
            _ => {}
        }
        if self.encrypt && !self.luks_same_as_password {
            if self.luks.is_empty() {
                return Err("Pick an encryption passphrase.");
            }
            if still_typing(&self.luks, &self.luks_confirm) {
                return Err("Type the passphrase again to confirm.");
            }
            if self.luks != self.luks_confirm {
                return Err("The passphrases don't match.");
            }
        }
        Ok(())
    }

    pub fn git_ok(&self) -> Result<(), &'static str> {
        // No email means "later": the name alone (mirrored from the full
        // name) isn't worth blocking on.
        let (n, e) = (self.git_name.trim(), self.git_email.trim());
        if n.is_empty() && !e.is_empty() {
            return Err("Git wants a name to go with that email.");
        }
        if !e.is_empty() && !looks_like_email(e) {
            return Err("That isn't an email address.");
        }
        Ok(())
    }

    pub fn wallpaper_set_on(&self, id: &str) -> bool {
        self.wallpaper_sets.iter().any(|s| s == id)
    }

    pub fn toggle_wallpaper_set(&mut self, id: &str) {
        if self.wallpaper_set_on(id) {
            self.wallpaper_sets.retain(|s| s != id);
        } else {
            self.wallpaper_sets.push(id.to_string());
        }
    }

    /// `secured`: the picked network wants a password.
    pub fn wifi_ok(&self, secured: bool) -> Result<(), &'static str> {
        match self.wifi_psk.chars().count() {
            _ if !secured || self.wifi_ssid.is_none() => Ok(()),
            0 => Err("Type its password, or pick No Wi-Fi."),
            // WPA's minimum; anything shorter can't be right.
            1..8 => Err("Wi-Fi passwords are at least 8 characters."),
            _ => Ok(()),
        }
    }

    pub fn to_config(&self, probe: &Probe) -> InstallConfig {
        let manifest = probe.payload.as_ref();
        let (layout, variant) = data::split_keymap(&self.keymap);
        let target = match self.disk_mode {
            Some(DiskMode::FreeSpace) => match self.free_region(probe) {
                Some(r) => DiskTarget::FreeSpace {
                    start: r.start,
                    end: r.end,
                },
                None => DiskTarget::Whole,
            },
            _ => DiskTarget::Whole,
        };
        let git =
            (!self.git_name.trim().is_empty() && !self.git_email.trim().is_empty()).then(|| Git {
                name: self.git_name.trim().to_string(),
                email: self.git_email.trim().to_string(),
            });
        InstallConfig {
            locale: Locale {
                lang: self.locale.clone(),
                keyboard_layout: layout,
                keyboard_variant: variant,
                timezone: self.timezone.clone(),
            },
            host: Host {
                hostname: manifest
                    .map(|m| m.hostname.clone())
                    .unwrap_or_else(|| "tuxedo".into()),
            },
            user: User {
                username: self.username.clone(),
                full_name: self.full_name.trim().to_string(),
            },
            disk: Disk {
                device: self.disk.clone().unwrap_or_default(),
                target,
                encrypt: self.encrypt,
            },
            git,
            wifi: self.wifi_ssid.clone().map(|ssid| Wifi { ssid }),
            // An older payload has no sets to pick from.
            wallpapers: manifest
                .filter(|m| !m.wallpaper_sets.is_empty())
                .map(|m| Wallpapers {
                    sets: m
                        .wallpaper_sets
                        .iter()
                        .filter(|s| self.wallpaper_set_on(&s.id))
                        .map(|s| s.id.clone())
                        .collect(),
                }),
            payload: Payload {
                dir: PAYLOAD_DIR.into(),
            },
        }
    }

    pub fn to_secrets(&self) -> Secrets {
        Secrets {
            luks_passphrase: self.luks_passphrase().map(str::to_string),
            user_password: self.password.clone(),
            wifi_psk: self
                .wifi_ssid
                .as_ref()
                .filter(|_| !self.wifi_psk.is_empty())
                .map(|_| self.wifi_psk.clone()),
        }
    }
}

/// The confirmation is empty or so far a prefix of the original: not wrong
/// yet, just not finished.
fn still_typing(orig: &str, confirm: &str) -> bool {
    confirm != orig && orig.starts_with(confirm)
}

pub fn looks_like_email(s: &str) -> bool {
    match s.split_once('@') {
        Some((user, host)) => {
            !user.is_empty() && host.contains('.') && !host.starts_with('.') && !host.ends_with('.')
        }
        None => false,
    }
}

pub trait DiskExt {
    fn is_target(&self) -> bool;
    fn largest_free(&self) -> Option<Region>;
    fn fits(&self, probe: &Probe) -> bool;
    fn fits_dual_boot(&self, probe: &Probe) -> bool;
    fn default_mode(&self, probe: &Probe) -> DiskMode;
    fn other_os(&self) -> Option<&str>;
}

/// Smallest disk (or free region) KubaOS fits in.
pub fn needed(probe: &Probe) -> u64 {
    probe
        .payload
        .as_ref()
        .map(|m| m.installed_bytes)
        .unwrap_or(0)
        + HEADROOM
}

impl DiskExt for DiskInfo {
    fn is_target(&self) -> bool {
        !self.is_boot_medium
    }

    fn largest_free(&self) -> Option<Region> {
        self.free_regions
            .iter()
            .copied()
            .max_by_key(|r| r.end - r.start)
    }

    fn fits(&self, probe: &Probe) -> bool {
        self.size_bytes >= needed(probe)
    }

    fn fits_dual_boot(&self, probe: &Probe) -> bool {
        self.largest_free()
            .is_some_and(|r| r.end - r.start >= needed(probe))
    }

    /// Keep another OS when there's room next to it.
    fn default_mode(&self, probe: &Probe) -> DiskMode {
        if self.other_os().is_some() && self.fits_dual_boot(probe) {
            DiskMode::FreeSpace
        } else {
            DiskMode::Whole
        }
    }

    fn other_os(&self) -> Option<&str> {
        self.partitions
            .iter()
            .filter_map(|p| p.os_hint.as_deref())
            .find(|h| !h.contains("EFI") && !h.contains("reserved") && !h.contains("recovery"))
    }
}

/// A rough password strength: 0 (empty) to 4 (strong).
pub fn strength(pw: &str) -> u8 {
    if pw.is_empty() {
        return 0;
    }
    let classes = [
        pw.chars().any(|c| c.is_ascii_lowercase()),
        pw.chars().any(|c| c.is_ascii_uppercase()),
        pw.chars().any(|c| c.is_ascii_digit()),
        pw.chars().any(|c| !c.is_ascii_alphanumeric()),
    ]
    .iter()
    .filter(|b| **b)
    .count();
    let len = pw.chars().count();
    // Long passphrases are strong regardless of character classes.
    match (len, classes) {
        (20.., _) => 4,
        (14.., c) if c >= 2 => 4,
        (12.., c) if c >= 3 => 4,
        (10.., c) if c >= 2 => 3,
        (8.., c) if c >= 2 => 2,
        _ => 1,
    }
}

pub fn strength_label(s: u8) -> &'static str {
    match s {
        0 => "",
        1 => "Weak",
        2 => "Fair",
        3 => "Good",
        _ => "Strong",
    }
}

/// `human_bytes` that never breaks between number and unit.
pub fn human_bytes_nb(b: u64) -> String {
    human_bytes(b).replace(' ', "\u{a0}")
}

pub fn human_bytes(b: u64) -> String {
    let gb = b as f64 / 1e9;
    if gb >= 1000.0 {
        format!("{:.1} TB", gb / 1000.0)
    } else if gb >= 10.0 {
        format!("{gb:.0} GB")
    } else if gb >= 1.0 {
        format!("{gb:.1} GB")
    } else {
        format!("{:.0} MB", b as f64 / 1e6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::demo_probe;

    #[test]
    fn defaults_from_demo_probe() {
        let p = demo_probe();
        let f = Form::new(&p);
        assert_eq!(f.timezone, "Europe/Rome");
        assert_eq!(f.username, "kuba");
        assert_eq!(f.disk.as_deref(), Some("/dev/nvme0n1"));
        assert_eq!(f.disk_mode, Some(DiskMode::FreeSpace));
        assert!(f.encrypt);
    }

    #[test]
    fn config_round_trips_through_toml() {
        let p = demo_probe();
        let mut f = Form::new(&p);
        f.full_name = "Kuba".into();
        f.password = "hunter2".into();
        f.password_confirm = "hunter2".into();
        f.keymap = "us(intl)".into();
        let cfg = f.to_config(&p);
        assert_eq!(cfg.locale.keyboard_layout, "us");
        assert_eq!(cfg.locale.keyboard_variant, "intl");
        assert!(matches!(cfg.disk.target, DiskTarget::FreeSpace { .. }));
        let toml = toml::to_string(&cfg).unwrap();
        assert_eq!(InstallConfig::from_toml(&toml).unwrap(), cfg);
        let s = f.to_secrets();
        assert_eq!(s.luks_passphrase.as_deref(), Some("hunter2"));
        assert_eq!(s.wifi_psk, None);
    }

    #[test]
    fn username_is_yours_to_pick() {
        let p = demo_probe();
        let mut f = Form::new(&p);
        f.full_name = "Alice Example".into();
        f.password = "correct horse battery".into();
        f.password_confirm = f.password.clone();
        assert!(f.account_ok().is_ok());
        f.username = "alice".into();
        assert!(f.account_ok().is_ok());
        assert_eq!(f.to_config(&p).user.username, "alice");
        for bad in ["", "Alice", "1alice", "al ice", "root", "nixbld3"] {
            f.username = bad.into();
            assert!(f.account_ok().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn git_skips_without_email() {
        let mut f = Form::default();
        assert!(f.git_ok().is_ok());
        f.git_name = "A".into();
        assert!(f.git_ok().is_ok(), "a name alone means: set it up later");
        f.git_name.clear();
        f.git_email = "a@b.c".into();
        assert!(f.git_ok().is_err());
        f.git_name = "A".into();
        f.git_email = "a@b.c".into();
        assert!(f.git_ok().is_ok());
        f.git_email = "nope".into();
        assert!(f.git_ok().is_err());
    }

    #[test]
    fn every_wallpaper_set_by_default() {
        let p = demo_probe();
        let mut f = Form::new(&p);
        let sets = |f: &Form| f.to_config(&p).wallpapers.map(|w| w.sets);
        let all = ["cyberpunk", "japanese", "shinjuku"]
            .map(String::from)
            .to_vec();
        assert_eq!(sets(&f), Some(all.clone()));
        f.toggle_wallpaper_set("japanese");
        assert_eq!(sets(&f), Some(vec!["cyberpunk".into(), "shinjuku".into()]));
        f.toggle_wallpaper_set("cyberpunk");
        f.toggle_wallpaper_set("shinjuku");
        assert_eq!(sets(&f), Some(vec![]), "none: only your own folder");
        f.toggle_wallpaper_set("shinjuku");
        f.toggle_wallpaper_set("japanese");
        f.toggle_wallpaper_set("cyberpunk");
        assert_eq!(sets(&f), Some(all), "the payload's order, not the clicks'");
    }

    #[test]
    fn no_wallpaper_sets_in_an_older_payload() {
        let mut p = demo_probe();
        p.payload.as_mut().unwrap().wallpaper_sets.clear();
        assert_eq!(Form::new(&p).to_config(&p).wallpapers, None);
    }

    #[test]
    fn wifi_password_rules() {
        let mut f = Form::default();
        assert!(f.wifi_ok(true).is_ok(), "no network picked");
        f.wifi_ssid = Some("Nerv HQ".into());
        assert!(f.wifi_ok(false).is_ok(), "open network");
        assert!(f.wifi_ok(true).is_err());
        f.wifi_psk = "short".into();
        assert!(f.wifi_ok(true).is_err());
        f.wifi_psk = "misato-katsuragi".into();
        assert!(f.wifi_ok(true).is_ok());
    }

    #[test]
    fn confirmation_still_typing_is_not_a_mismatch() {
        let mut f = Form {
            full_name: "J".into(),
            username: "j".into(),
            password: "hunter22".into(),
            password_confirm: "hunt".into(),
            ..Form::default()
        };
        assert_eq!(f.account_ok(), Err("Type it again to confirm."));
        f.password_confirm = "hunx".into();
        assert_eq!(f.account_ok(), Err("The passwords don't match."));
        f.password_confirm = "hunter22".into();
        assert!(f.account_ok().is_ok());
    }

    #[test]
    fn strength_scale() {
        assert_eq!(strength(""), 0);
        assert_eq!(strength("abc"), 1);
        assert_eq!(strength("correct horse battery staple"), 4);
    }
}
