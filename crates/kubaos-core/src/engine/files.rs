//! Text of the files the engine writes into the target. Pure functions.

use std::path::Path;

use kubaos_user_layer::App;

use crate::config::Wallpapers;

/// Btrfs subvolumes, flat under the top level (system/00-btrfs-subvolumes.md),
/// and where each is mounted. Order matters: parents before children.
pub const SUBVOLUMES: [(&str, &str); 8] = [
    ("@", "/"),
    ("@home", "/home"),
    ("@snapshots", "/.snapshots"),
    ("@log", "/var/log"),
    ("@cache", "/var/cache"),
    ("@nix", "/nix"),
    ("@docker", "/var/lib/docker"),
    ("@swap", "/swap"),
];

pub const BTRFS_OPTS: &str = "noatime,compress=zstd:1";

/// Mount options for a subvolume, as fstab and the install both use them.
pub fn subvol_opts(subvol: &str) -> String {
    if subvol == "@swap" {
        format!("noatime,subvol={subvol}")
    } else {
        format!("{BTRFS_OPTS},subvol={subvol}")
    }
}

pub const LUKS_NAME: &str = "cryptroot";

#[derive(Debug, Clone)]
pub struct FsUuids {
    /// The btrfs filesystem (inside LUKS when encrypted).
    pub root: String,
    pub boot: String,
    /// vfat volume id, `ABCD-1234`.
    pub esp: String,
}

pub fn fstab(u: &FsUuids, swapfile: bool) -> String {
    let mut s = String::from(
        "# /etc/fstab: written by the KubaOS install. Subvolumes as in\n\
         # system/00-btrfs-subvolumes.md; names, not ids, so a rollback that\n\
         # replaces @ keeps working.\n\
         # <file system>  <mount point>  <type>  <options>  <dump>  <pass>\n",
    );
    for (sv, mp) in SUBVOLUMES {
        s.push_str(&format!(
            "UUID={}  {mp}  btrfs  {}  0 0\n",
            u.root,
            subvol_opts(sv)
        ));
    }
    s.push_str(&format!("UUID={}  /boot  ext4  defaults  0 2\n", u.boot));
    s.push_str(&format!(
        "UUID={}  /boot/efi  vfat  umask=0077  0 1\n",
        u.esp
    ));
    if swapfile {
        s.push_str("/swap/swapfile  none  swap  defaults  0 0\n");
    }
    s
}

pub fn crypttab(luks_uuid: &str) -> String {
    // The name must match the mapping that's open while update-initramfs
    // runs, or the cryptsetup hook can't find it (00-btrfs-subvolumes.md).
    // tries=0: ask until the passphrase is right. The default (3) ends in
    // the initramfs giving up on the root device and dropping to a busybox
    // shell, minutes later.
    format!(
        "# <target name> <source device> <key file> <options>\n\
         {LUKS_NAME} UUID={luks_uuid} none luks,discard,tries=0\n"
    )
}

pub fn hosts(hostname: &str) -> String {
    format!(
        "127.0.0.1\tlocalhost\n\
         127.0.1.1\t{hostname}\n\
         \n\
         # The following lines are desirable for IPv6 capable hosts\n\
         ::1     ip6-localhost ip6-loopback\n\
         fe00::0 ip6-localnet\n\
         ff00::0 ip6-mcastprefix\n\
         ff02::1 ip6-allnodes\n\
         ff02::2 ip6-allrouters\n"
    )
}

pub fn locale(lang: &str) -> String {
    format!("LANG={lang}\n")
}

/// NetworkManager keyfile for the Wi-Fi the user picked. `psk` `None` or
/// empty means an open network. Written root:root 0600, as NM does itself.
pub fn nmconnection(ssid: &str, psk: Option<&str>, uuid: &str) -> String {
    let mut s = format!(
        "[connection]\nid={}\nuuid={uuid}\ntype=wifi\n\n[wifi]\nmode=infrastructure\nssid={}\n",
        keyfile_escape(ssid),
        keyfile_ssid(ssid)
    );
    if let Some(psk) = psk.filter(|p| !p.is_empty()) {
        s.push_str(&format!(
            "\n[wifi-security]\nkey-mgmt=wpa-psk\npsk={}\n",
            keyfile_escape(psk)
        ));
    }
    s.push_str("\n[ipv4]\nmethod=auto\n\n[ipv6]\naddr-gen-mode=default\nmethod=auto\n");
    s
}

/// GKeyFile string escaping (what NM's keyfile reader undoes).
fn keyfile_escape(v: &str) -> String {
    let mut out = String::new();
    for (i, c) in v.chars().enumerate() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            ' ' if i == 0 => out.push_str("\\s"),
            c => out.push(c),
        }
    }
    out
}

/// Plain text for ordinary SSIDs; otherwise NM's byte-list form, which
/// can't be misread (`;` is a list separator in keyfiles).
fn keyfile_ssid(ssid: &str) -> String {
    let plain = ssid
        .chars()
        .all(|c| c.is_ascii_graphic() && c != ';' && c != '\\' || c == ' ')
        && !ssid.starts_with(' ')
        && !ssid.ends_with(' ');
    if plain {
        ssid.to_string()
    } else {
        ssid.bytes().map(|b| format!("{b};")).collect()
    }
}

/// File name for a connection: the SSID with anything odd replaced.
pub fn nmconnection_name(ssid: &str) -> String {
    let base: String = ssid
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    let base = base.trim_start_matches('.');
    format!(
        "{}.nmconnection",
        if base.is_empty() { "wifi" } else { base }
    )
}

/// System files the image's scripts render for the build user (template in
/// the dotfiles checkout, file in the target). The engine renders them again
/// for the real user; install/image/check fails on any other file in the
/// payload that names the build user.
pub const USER_TEMPLATES: &[(&str, &str)] = &[
    // system/50-hyprland.sh: tty1 autologin
    (
        "system/files/getty/autologin.conf.in",
        "/etc/systemd/system/getty@tty1.service.d/autologin.conf",
    ),
    // system/40-apps.sh: Chrome's theme color follows the wallpaper
    (
        "system/files/chrome/dot-chrome-theme.path.in",
        "/etc/systemd/system/dot-chrome-theme.path",
    ),
    (
        "system/files/chrome/dot-chrome-theme.service.in",
        "/etc/systemd/system/dot-chrome-theme.service",
    ),
];

/// `render_template` from system/lib.sh: `@USER@` and `@HOME@`.
pub fn render_user_template(template: &str, user: &str, home: &str) -> String {
    template.replace("@USER@", user).replace("@HOME@", home)
}

/// The user flake, under the home (user-flake/ in the repo): the user's own
/// nixpkgs, KubaOS as an input, and home.nix for their own packages.
pub const USER_FLAKE: &str = ".config/kubaos/home";

/// user-flake/flake.nix.in for `user` on `host`. `system` is Nix's, e.g.
/// `x86_64-linux`.
pub fn user_flake(template: &str, user: &str, host: &str, system: &str) -> String {
    template
        .replace("@USER@", user)
        .replace("@HOST@", host)
        .replace("@SYSTEM@", system)
}

/// Where the git identity lives, under the home. Not in ~/.config/git: that
/// holds KubaOS's read-only defaults. config/git/config includes it.
pub const GIT_IDENTITY: &str = ".config/git-identity/identity.gitconfig";

/// The `[user]` section of [`GIT_IDENTITY`]. Values are quoted, so a `"`,
/// `\`, `#` or `;` in a name survives; newlines are the caller's to refuse.
pub fn gitconfig(name: &str, email: &str) -> String {
    let q = |v: &str| format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""));
    format!("[user]\n\tname = {}\n\temail = {}\n", q(name), q(email))
}

/// The wallpapers the user chose, under the home: the user layer, so
/// an update never changes it. scripts/dot-theme reads it (`folders`, `wallpapers`).
pub fn wallpaper_sets_path() -> String {
    let dir = kubaos_user_layer::root(Path::new(".config")).join(App::Wallpapers.relative());
    format!("{}/sets", dir.display())
}

/// [`wallpaper_sets_path`]: one entry per line, the whole sets' ids, then the single wallpapers as
/// `<set>/<file>`; `#` comments. `home/theme.nix` seeds the same header.
pub fn wallpaper_sets(w: &Wallpapers) -> String {
    let mut s = String::from(
        "# The wallpapers the picker shows (from /usr/share/kubaos/wallpapers), one per line:\n\
         # a set's name for all of it, <set>/<file> for one wallpaper.\n\
         # Your own images go in ~/Pictures/wallpapers.\n",
    );
    for entry in w.sets.iter().chain(&w.files) {
        s.push_str(entry);
        s.push('\n');
    }
    s
}

/// `<n>/info.xml` the way snapper writes it, for the raw-btrfs fallback.
pub fn snapper_info(num: u32, date_utc: &str, description: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
         <snapshot>\n  <type>single</type>\n  <num>{num}</num>\n  <date>{date_utc}</date>\n  \
         <description>{}</description>\n  <cleanup>number</cleanup>\n</snapshot>\n",
        xml_escape(description)
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Unix seconds -> `YYYY-MM-DD HH:MM:SS` (UTC), snapper's date format.
pub fn utc_datetime(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Snapper settings `system/20-base.sh` applies; the engine sets them when
/// it creates the config, so the first boot already matches. `SYNC_ACL=yes`
/// is set apart, after the factory snapshot: on a kernel without btrfs ACLs
/// it makes every later snapper call fail.
pub fn snapper_settings(user: &str) -> Vec<String> {
    [
        format!("ALLOW_USERS={user}"),
        "TIMELINE_CREATE=yes".into(),
        "TIMELINE_CLEANUP=yes".into(),
        "TIMELINE_LIMIT_HOURLY=6".into(),
        "TIMELINE_LIMIT_DAILY=7".into(),
        "TIMELINE_LIMIT_WEEKLY=2".into(),
        "TIMELINE_LIMIT_MONTHLY=1".into(),
        "TIMELINE_LIMIT_YEARLY=0".into(),
        "NUMBER_CLEANUP=yes".into(),
        "NUMBER_LIMIT=30".into(),
        "NUMBER_LIMIT_IMPORTANT=10".into(),
    ]
    .into()
}

/// Groups the first user gets, if the target has them: Ubuntu's defaults
/// for the user the install creates plus what `system/*.sh` adds (i2c from
/// 20-base.sh, docker from 40-apps.sh).
pub const USER_GROUPS: [&str; 9] = [
    "adm", "sudo", "users", "plugdev", "dip", "video", "render", "i2c", "docker",
];

/// Names from an `/etc/group` that exist, in `wanted` order.
pub fn existing_groups(group_file: &str, wanted: &[&str]) -> Vec<String> {
    let have: Vec<&str> = group_file
        .lines()
        .filter_map(|l| l.split(':').next())
        .collect();
    wanted
        .iter()
        .filter(|g| have.contains(g))
        .map(|g| g.to_string())
        .collect()
}

/// `(uid, gid, home)` of `user` in an `/etc/passwd`.
pub fn passwd_entry(passwd: &str, user: &str) -> Option<(u32, u32, String)> {
    passwd.lines().find_map(|l| {
        let f: Vec<&str> = l.split(':').collect();
        (f.len() >= 7 && f[0] == user)
            .then(|| Some((f[2].parse().ok()?, f[3].parse().ok()?, f[5].to_string())))
            .flatten()
    })
}

/// Who already has `uid`, if anyone.
pub fn uid_owner(passwd: &str, uid: u32) -> Option<String> {
    passwd.lines().find_map(|l| {
        let f: Vec<&str> = l.split(':').collect();
        (f.len() >= 3 && f[2].parse::<u32>().ok() == Some(uid)).then(|| f[0].to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uuids() -> FsUuids {
        FsUuids {
            root: "11111111-2222-3333-4444-555555555555".into(),
            boot: "66666666-7777-8888-9999-000000000000".into(),
            esp: "ABCD-1234".into(),
        }
    }

    #[test]
    fn fstab_matches_the_documented_layout() {
        let f = fstab(&uuids(), true);
        let lines: Vec<&str> = f.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(
            lines,
            vec![
                "UUID=11111111-2222-3333-4444-555555555555  /  btrfs  noatime,compress=zstd:1,subvol=@  0 0",
                "UUID=11111111-2222-3333-4444-555555555555  /home  btrfs  noatime,compress=zstd:1,subvol=@home  0 0",
                "UUID=11111111-2222-3333-4444-555555555555  /.snapshots  btrfs  noatime,compress=zstd:1,subvol=@snapshots  0 0",
                "UUID=11111111-2222-3333-4444-555555555555  /var/log  btrfs  noatime,compress=zstd:1,subvol=@log  0 0",
                "UUID=11111111-2222-3333-4444-555555555555  /var/cache  btrfs  noatime,compress=zstd:1,subvol=@cache  0 0",
                "UUID=11111111-2222-3333-4444-555555555555  /nix  btrfs  noatime,compress=zstd:1,subvol=@nix  0 0",
                "UUID=11111111-2222-3333-4444-555555555555  /var/lib/docker  btrfs  noatime,compress=zstd:1,subvol=@docker  0 0",
                "UUID=11111111-2222-3333-4444-555555555555  /swap  btrfs  noatime,subvol=@swap  0 0",
                "UUID=66666666-7777-8888-9999-000000000000  /boot  ext4  defaults  0 2",
                "UUID=ABCD-1234  /boot/efi  vfat  umask=0077  0 1",
                "/swap/swapfile  none  swap  defaults  0 0",
            ]
        );
        assert!(!fstab(&uuids(), false).contains("swapfile"));
    }

    #[test]
    fn crypttab_line() {
        let c = crypttab("aaaa-bbbb");
        assert!(c.ends_with("cryptroot UUID=aaaa-bbbb none luks,discard,tries=0\n"));
    }

    #[test]
    fn small_files() {
        assert!(hosts("tuxedo").contains("127.0.1.1\ttuxedo\n"));
        assert_eq!(locale("en_US.UTF-8"), "LANG=en_US.UTF-8\n");
    }

    #[test]
    fn wifi_keyfile() {
        let k = nmconnection("Home 5G", Some("pa ss;word"), "u-1");
        assert!(k.contains("[connection]\nid=Home 5G\nuuid=u-1\ntype=wifi\n"));
        assert!(k.contains("[wifi]\nmode=infrastructure\nssid=Home 5G\n"));
        assert!(k.contains("[wifi-security]\nkey-mgmt=wpa-psk\npsk=pa ss;word\n"));
        assert!(k.contains("[ipv4]\nmethod=auto\n"));
        let open = nmconnection("Cafe", None, "u-2");
        assert!(!open.contains("wifi-security"));
        assert!(!nmconnection("Cafe", Some(""), "u").contains("wifi-security"));
        let odd = nmconnection("a;b", Some("x\\y"), "u");
        assert!(odd.contains("ssid=97;59;98;\n"));
        assert!(odd.contains("psk=x\\\\y\n"));
        assert_eq!(nmconnection_name("Home 5G"), "Home_5G.nmconnection");
        assert_eq!(nmconnection_name("../x"), "_x.nmconnection");
        assert_eq!(nmconnection_name("..."), "wifi.nmconnection");
    }

    #[test]
    fn wallpaper_sets_file() {
        let w = Wallpapers {
            sets: vec!["shinjuku".into(), "cyberpunk".into()],
            files: vec![],
        };
        assert_eq!(wallpaper_sets_path(), ".config/kubaos/desktop/wallpapers/sets");
        assert_eq!(
            wallpaper_sets(&w),
            "# The wallpapers the picker shows (from /usr/share/kubaos/wallpapers), one per line:\n\
             # a set's name for all of it, <set>/<file> for one wallpaper.\n\
             # Your own images go in ~/Pictures/wallpapers.\n\
             shinjuku\n\
             cyberpunk\n"
        );
        let none = Wallpapers {
            sets: vec![],
            files: vec![],
        };
        assert!(
            wallpaper_sets(&none).lines().all(|l| l.starts_with('#')),
            "no sets: only the comment"
        );
    }

    #[test]
    fn single_wallpapers_follow_the_whole_sets_in_the_sets_file() {
        let w = Wallpapers {
            sets: vec!["cyberpunk".into()],
            files: vec![
                "shinjuku/kabukicho-red-gate.jpg".into(),
                "japanese/tokyo-tower-blue-hour.jpg".into(),
            ],
        };

        let lines: Vec<String> = wallpaper_sets(&w)
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(str::to_string)
            .collect();

        assert_eq!(
            lines,
            [
                "cyberpunk",
                "shinjuku/kabukicho-red-gate.jpg",
                "japanese/tokyo-tower-blue-hour.jpg"
            ]
        );
    }

    #[test]
    fn dates() {
        assert_eq!(utc_datetime(0), "1970-01-01 00:00:00");
        assert_eq!(utc_datetime(951_782_400), "2000-02-29 00:00:00");
        assert_eq!(utc_datetime(1_792_150_245), "2026-10-16 11:30:45");
    }

    #[test]
    fn snapper_xml() {
        let x = snapper_info(1, "2026-10-15 10:10:45", "factory");
        assert!(x.contains("<num>1</num>"));
        assert!(x.contains("<description>factory</description>"));
        assert!(x.contains("<date>2026-10-15 10:10:45</date>"));
    }

    #[test]
    fn passwd_and_groups() {
        let passwd = "root:x:0:0:root:/root:/bin/bash\nubuntu:x:1000:1000::/home/ubuntu:/bin/bash\nkuba:x:1001:1001:J:/home/kuba:/usr/bin/zsh\n";
        assert_eq!(
            passwd_entry(passwd, "kuba"),
            Some((1001, 1001, "/home/kuba".into()))
        );
        assert_eq!(passwd_entry(passwd, "nobody"), None);
        assert_eq!(uid_owner(passwd, 1000), Some("ubuntu".into()));
        assert_eq!(uid_owner(passwd, 1002), None);
        let groups = "root:x:0:\nsudo:x:27:\nadm:x:4:syslog\ndocker:x:990:\n";
        assert_eq!(
            existing_groups(groups, &USER_GROUPS),
            vec!["adm", "sudo", "docker"]
        );
    }
}
