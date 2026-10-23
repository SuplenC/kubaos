//! The answers so far, their validation, and turning them into
//! `install.toml` plus the secrets.

use std::collections::BTreeSet;

use kubaos_core::config::{Disk, DiskTarget, Git, Host, Locale, Payload, User, Wallpapers, Wifi};
use kubaos_core::keyboard::{self, Layout};
use kubaos_core::probe::{DiskInfo, Region, WallpaperFile, WallpaperSet};
use kubaos_core::{InstallConfig, Probe, Secrets};

/// Where the live ISO keeps the payload (docs/INSTALL-INTERNALS.md, `[payload]`).
pub const PAYLOAD_DIR: &str = "/run/kubaos/payload";

/// A wallpaper in [`Form::wallpapers`] and in `[wallpapers] files`: `<set>/<file>`.
pub fn wallpaper_key(set: &WallpaperSet, file: &WallpaperFile) -> String {
    format!("{}/{}", set.id, file.name)
}

/// Every wallpaper of the set as [`Form::wallpapers`] holds them: the set's id alone when the
/// payload lists none of its files.
fn wallpaper_keys(set: &WallpaperSet) -> Vec<String> {
    if set.files.is_empty() {
        return vec![set.id.clone()];
    }
    set.files.iter().map(|f| wallpaper_key(set, f)).collect()
}

/// The wallpapers in a set: its listed files, or the count of an older payload.
fn wallpapers_in(set: &WallpaperSet) -> u32 {
    if set.files.is_empty() {
        set.images
    } else {
        u32::try_from(set.files.len()).unwrap_or(u32::MAX)
    }
}
/// Smallest free region offered for a dual-boot install, on top of the payload size.
const HEADROOM: u64 = 8_000_000_000;
/// What a hostname may be: one RFC 1123 label, as
/// `kubaos_core::engine::validate::valid_hostname` checks it.
pub const HOSTNAME_RULE: &str = "Lowercase a-z, 0-9 and -, up to 63, with no - at either end.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskMode {
    Whole,
    FreeSpace,
}

#[derive(Debug, Clone, Default)]
pub struct Form {
    pub locale: String,
    /// The layouts picked, as `layout` or `layout(variant)`, the first the default.
    pub keymaps: Vec<String>,
    pub keyboard_test: String,
    pub timezone: String,

    pub wifi_ssid: Option<String>,
    pub wifi_psk: String,

    pub full_name: String,
    pub username: String,
    /// The machine's name, `/etc/hostname`. The home config doesn't depend on it.
    pub hostname: String,
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

    /// The wallpapers picked, as `<set>/<file>`; a set the payload lists no files for (an older
    /// payload) by its id alone.
    pub wallpapers: BTreeSet<String>,
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
            keymaps: vec![pick(&probe.keymaps, "us")],
            timezone: pick(&probe.timezones, "Europe/Rome"),
            username: manifest.map(|m| m.username.clone()).unwrap_or_default(),
            hostname: manifest.map(|m| m.hostname.clone()).unwrap_or_default(),
            encrypt: true,
            luks_same_as_password: true,
            disk,
            wallpapers: manifest
                .map(|m| m.wallpaper_sets.iter().flat_map(wallpaper_keys).collect())
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

    pub fn keymap_picked(&self, keymap: &str) -> bool {
        self.keymaps.iter().any(|k| k == keymap)
    }

    /// Picks `keymap`, or unpicks it if it was picked.
    ///
    /// # Errors
    ///
    /// Why it can't: it is the last one, or there are already as many as XKB holds.
    pub fn toggle_keymap(&mut self, keymap: &str) -> Result<(), &'static str> {
        if self.keymap_picked(keymap) {
            if self.keymaps.len() == 1 {
                return Err("Keep one layout at least.");
            }
            self.keymaps.retain(|k| k != keymap);
        } else {
            if self.keymaps.len() >= keyboard::MAX_LAYOUTS {
                return Err("Four layouts at most: XKB holds no more.");
            }
            self.keymaps.push(keymap.to_string());
        }
        Ok(())
    }

    /// Makes `keymap` the default, the first one, picking it if it wasn't.
    ///
    /// # Errors
    ///
    /// When it wasn't picked and there are already as many as XKB holds.
    pub fn make_default_keymap(&mut self, keymap: &str) -> Result<(), &'static str> {
        if !self.keymap_picked(keymap) {
            self.toggle_keymap(keymap)?;
        }
        self.keymaps.retain(|k| k != keymap);
        self.keymaps.insert(0, keymap.to_string());
        Ok(())
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
        use kubaos_core::engine::validate::{reserved_username, valid_hostname, valid_username};
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
        if self.hostname.is_empty() {
            return Err("Name this machine.");
        }
        if !valid_hostname(&self.hostname) {
            return Err(HOSTNAME_RULE);
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

    pub fn wallpaper_on(&self, set: &WallpaperSet, file: &WallpaperFile) -> bool {
        self.wallpapers.contains(&wallpaper_key(set, file))
    }

    pub fn toggle_wallpaper(&mut self, set: &WallpaperSet, file: &WallpaperFile) {
        let key = wallpaper_key(set, file);
        if !self.wallpapers.remove(&key) {
            self.wallpapers.insert(key);
        }
    }

    /// How many of the set's wallpapers are picked.
    pub fn wallpapers_on(&self, set: &WallpaperSet) -> u32 {
        if set.files.is_empty() {
            return if self.wallpapers.contains(&set.id) {
                set.images
            } else {
                0
            };
        }
        let on = set.files.iter().filter(|f| self.wallpaper_on(set, f)).count();
        u32::try_from(on).unwrap_or(u32::MAX)
    }

    /// The set's header: drops every wallpaper of a set that's all picked, picks all of any other.
    pub fn toggle_wallpaper_set(&mut self, set: &WallpaperSet) {
        let all = self.wallpapers_on(set) == wallpapers_in(set);
        for key in wallpaper_keys(set) {
            if all {
                self.wallpapers.remove(&key);
            } else {
                self.wallpapers.insert(key);
            }
        }
    }

    /// The wallpapers picked over every set.
    pub fn wallpapers_picked(&self, sets: &[WallpaperSet]) -> u32 {
        sets.iter().map(|s| self.wallpapers_on(s)).sum()
    }

    /// The picker needs something to show, and the first boot a wallpaper to start on.
    pub fn wallpapers_ok(&self, sets: &[WallpaperSet]) -> Result<(), &'static str> {
        if !sets.is_empty() && self.wallpapers_picked(sets) == 0 {
            return Err("Pick at least one wallpaper.");
        }
        Ok(())
    }

    /// `[wallpapers]`: a set picked whole goes in by its id, so the wallpapers a release adds to
    /// it show up too; the others' picks one by one. Both in the payload's order.
    fn wallpapers_config(&self, sets: &[WallpaperSet]) -> Wallpapers {
        let mut w = Wallpapers {
            sets: Vec::new(),
            files: Vec::new(),
        };
        for set in sets {
            if self.wallpapers_on(set) == wallpapers_in(set) && wallpapers_in(set) > 0 {
                w.sets.push(set.id.clone());
            } else {
                w.files.extend(
                    set.files
                        .iter()
                        .filter(|f| self.wallpaper_on(set, f))
                        .map(|f| wallpaper_key(set, f)),
                );
            }
        }
        w
    }

    /// `secured`: the picked network wants a password.
    pub fn wifi_ok(&self, secured: bool) -> Result<(), &'static str> {
        match self.wifi_psk.chars().count() {
            _ if !secured || self.wifi_ssid.is_none() => Ok(()),
            0 => Err("Type its password, or go with the cable."),
            // WPA's minimum; anything shorter can't be right.
            1..8 => Err("Wi-Fi passwords are at least 8 characters."),
            _ => Ok(()),
        }
    }

    pub fn to_config(&self, probe: &Probe) -> InstallConfig {
        let manifest = probe.payload.as_ref();
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
        let layouts: Vec<Layout> = self.keymaps.iter().map(|k| Layout::parse(k)).collect();
        let git =
            (!self.git_name.trim().is_empty() && !self.git_email.trim().is_empty()).then(|| Git {
                name: self.git_name.trim().to_string(),
                email: self.git_email.trim().to_string(),
            });
        InstallConfig {
            locale: Locale {
                lang: self.locale.clone(),
                keyboard_layout: layouts.iter().map(|l| l.layout.clone()).collect(),
                keyboard_variant: layouts.iter().map(|l| l.variant.clone()).collect(),
                timezone: self.timezone.clone(),
            },
            host: Host {
                hostname: self.hostname.clone(),
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
                .map(|m| self.wallpapers_config(&m.wallpaper_sets)),
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
        assert_eq!(f.hostname, "tuxedo", "the payload's");
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
        f.keymaps = vec!["us(intl)".into(), "it".into()];
        let cfg = f.to_config(&p);
        assert_eq!(cfg.locale.keyboard_layout, ["us", "it"]);
        assert_eq!(cfg.locale.keyboard_variant, ["intl", ""]);
        assert!(matches!(cfg.disk.target, DiskTarget::FreeSpace { .. }));
        let toml = toml::to_string(&cfg).unwrap();
        assert_eq!(InstallConfig::from_toml(&toml).unwrap(), cfg);
        let s = f.to_secrets();
        assert_eq!(s.luks_passphrase.as_deref(), Some("hunter2"));
        assert_eq!(s.wifi_psk, None);
    }

    #[test]
    fn layouts_are_picked_in_order_one_to_four() {
        let p = demo_probe();
        let mut f = Form::new(&p);
        assert_eq!(f.keymaps, ["us"], "the probe's layout is picked");

        assert_eq!(f.toggle_keymap("us"), Err("Keep one layout at least."));
        f.toggle_keymap("it").unwrap();
        f.toggle_keymap("de").unwrap();
        f.toggle_keymap("pl").unwrap();
        assert_eq!(
            f.toggle_keymap("fr"),
            Err("Four layouts at most: XKB holds no more.")
        );
        f.toggle_keymap("de").unwrap();

        assert_eq!(f.keymaps, ["us", "it", "pl"]);
    }

    #[test]
    fn making_a_layout_the_default_moves_it_first() {
        let p = demo_probe();
        let mut f = Form::new(&p);
        f.toggle_keymap("it").unwrap();

        f.make_default_keymap("it").unwrap();
        assert_eq!(f.keymaps, ["it", "us"]);
        f.make_default_keymap("de").unwrap();
        assert_eq!(
            f.keymaps,
            ["de", "it", "us"],
            "a layout not yet picked is picked first"
        );
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
    fn hostname_is_yours_to_pick() {
        let p = demo_probe();
        let mut f = Form::new(&p);
        f.full_name = "Alice Example".into();
        f.password = "correct horse battery".into();
        f.password_confirm = f.password.clone();
        for good in ["tuxedo", "alice-laptop", "x", "a1-b2", &"a".repeat(63)] {
            f.hostname = good.into();
            assert!(f.account_ok().is_ok(), "{good:?}");
            assert_eq!(f.to_config(&p).host.hostname, good);
        }
        for bad in [
            "",
            "Tuxedo",
            "-box",
            "box-",
            "a.b",
            "my box",
            "zażółć",
            &"a".repeat(64),
        ] {
            f.hostname = bad.into();
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
        let sets = &p.payload.as_ref().unwrap().wallpaper_sets;
        let mut f = Form::new(&p);
        let config = |f: &Form| f.to_config(&p).wallpapers.unwrap();
        let all = ["cyberpunk", "japanese", "shinjuku"]
            .map(String::from)
            .to_vec();
        assert_eq!(config(&f).sets, all);
        assert_eq!(config(&f).files, Vec::<String>::new());
        assert_eq!(f.wallpapers_picked(sets), 10);
        f.toggle_wallpaper_set(&sets[1]);
        assert_eq!(config(&f).sets, ["cyberpunk", "shinjuku"]);
        f.toggle_wallpaper_set(&sets[0]);
        f.toggle_wallpaper_set(&sets[2]);
        assert_eq!(config(&f).sets, Vec::<String>::new());
        assert_eq!(f.wallpapers_picked(sets), 0);
        f.toggle_wallpaper_set(&sets[2]);
        f.toggle_wallpaper_set(&sets[1]);
        f.toggle_wallpaper_set(&sets[0]);
        assert_eq!(config(&f).sets, all, "the payload's order, not the clicks'");
    }

    #[test]
    fn single_wallpapers_go_in_one_by_one_and_a_set_picked_whole_by_its_id() {
        let p = demo_probe();
        let sets = &p.payload.as_ref().unwrap().wallpaper_sets;
        let mut f = Form::new(&p);

        f.toggle_wallpaper(&sets[2], &sets[2].files[1]);
        f.toggle_wallpaper(&sets[2], &sets[2].files[3]);
        f.toggle_wallpaper_set(&sets[1]);
        f.toggle_wallpaper(&sets[1], &sets[1].files[1]);

        let w = f.to_config(&p).wallpapers.unwrap();
        assert_eq!(w.sets, ["cyberpunk"]);
        assert_eq!(
            w.files,
            [
                "japanese/tokyo-tower-blue-hour.jpg",
                "shinjuku/kabukicho-red-gate.jpg",
                "shinjuku/omoide-yokocho-lanterns.jpg",
                "shinjuku/train-neon-streak.jpg",
            ]
        );
        assert_eq!(f.wallpapers_on(&sets[2]), 3);
        assert_eq!(f.wallpapers_picked(sets), 7);
        assert!(!f.wallpaper_on(&sets[2], &sets[2].files[1]));
    }

    #[test]
    fn a_set_header_picks_the_rest_of_a_set_and_drops_a_whole_one() {
        let p = demo_probe();
        let sets = &p.payload.as_ref().unwrap().wallpaper_sets;
        let mut f = Form::new(&p);
        f.toggle_wallpaper(&sets[2], &sets[2].files[0]);

        f.toggle_wallpaper_set(&sets[2]);
        assert_eq!(f.wallpapers_on(&sets[2]), 5, "partly picked: all of it");
        f.toggle_wallpaper_set(&sets[2]);
        assert_eq!(f.wallpapers_on(&sets[2]), 0, "all picked: none of it");
    }

    #[test]
    fn at_least_one_wallpaper_must_be_picked() {
        let p = demo_probe();
        let sets = &p.payload.as_ref().unwrap().wallpaper_sets;
        let mut f = Form::new(&p);
        for s in sets {
            f.toggle_wallpaper_set(s);
        }
        assert_eq!(f.wallpapers_ok(sets), Err("Pick at least one wallpaper."));

        f.toggle_wallpaper(&sets[1], &sets[1].files[0]);

        assert_eq!(f.wallpapers_ok(sets), Ok(()));
        assert_eq!(
            f.to_config(&p).wallpapers.unwrap().files,
            ["japanese/shinjuku-purple-hour.jpg"]
        );
    }

    #[test]
    fn a_set_an_older_payload_lists_no_files_for_is_picked_whole() {
        let mut p = demo_probe();
        p.payload.as_mut().unwrap().wallpaper_sets[0].files.clear();
        let sets = p.payload.as_ref().unwrap().wallpaper_sets.clone();
        let mut f = Form::new(&p);
        assert_eq!(f.wallpapers_on(&sets[0]), 3);

        f.toggle_wallpaper_set(&sets[0]);

        assert_eq!(f.wallpapers_on(&sets[0]), 0);
        assert_eq!(f.to_config(&p).wallpapers.unwrap().sets, ["japanese", "shinjuku"]);
    }

    #[test]
    fn no_wallpaper_sets_in_an_older_payload() {
        let mut p = demo_probe();
        p.payload.as_mut().unwrap().wallpaper_sets.clear();
        let f = Form::new(&p);
        assert_eq!(f.to_config(&p).wallpapers, None);
        assert_eq!(f.wallpapers_ok(&[]), Ok(()), "nothing to pick from");
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
            hostname: "j-laptop".into(),
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
