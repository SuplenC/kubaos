//! What's done and what isn't, read from the machine every time. Nothing is
//! remembered between runs, so a card done by hand (or by the scripts)
//! shows as done here too.

use super::parse::*;
use crate::bitwarden::Config;
use crate::bitwarden::Overview;
use crate::bitwarden::agent;
use super::{CardId, CardStatus, Cmd, Env, Facts, MokState, State, Status, Sys};

const SECURE_BOOT_VAR: &str =
    "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c";
/// Where DKMS keeps its signing certificate (Ubuntu's shim-signed path first).
pub const MOK_KEYS: [&str; 2] = ["/var/lib/shim-signed/mok/MOK.der", "/var/lib/dkms/mok.pub"];
/// shim's enrollment request, which `mokutil --import` writes for MokManager to show on the next boot.
pub const MOK_NEW_VAR: &str =
    "/sys/firmware/efi/efivars/MokNew-605dab50-e046-4300-abb6-3dd810dd8b23";

pub fn status(sys: &mut dyn Sys, env: &Env) -> Status {
    let online = online(sys);
    let cards = CardId::ALL
        .iter()
        .map(|&id| card_status(sys, env, id, online))
        .collect();
    Status { online, cards }
}

pub fn online(sys: &mut dyn Sys) -> bool {
    match sys.run(&Cmd::new("nmcli").args(["-t", "-f", "CONNECTIVITY", "general"])) {
        Ok(o) => o.stdout.trim() == "full",
        Err(_) => sys.reachable(),
    }
}

pub fn card_status(sys: &mut dyn Sys, env: &Env, id: CardId, online: bool) -> CardStatus {
    let mut c = match id {
        CardId::Tuxedo => tuxedo(sys),
        CardId::Fingerprint => fingerprint(sys, env),
        CardId::Yubikey => yubikey(sys, env),
        CardId::Bitwarden => bitwarden(sys, env),
        CardId::Git => git(sys, env),
    };
    if c.state == State::Needed && id.needs_network() && !online && !offline_ok(&c) {
        c.state = State::Waiting;
        c.summary = format!("{} Needs the internet first.", c.summary);
    }
    c
}

/// Cards that can finish without the network from where they are.
fn offline_ok(c: &CardStatus) -> bool {
    match &c.facts {
        // Installed already; only the MOK import is left.
        Facts::Tuxedo { drivers, .. } => *drivers,
        // fprintd is there; enrolling and PAM are local.
        Facts::Fingerprint { fprintd, .. } => *fprintd,
        _ => false,
    }
}

fn trimmed(sys: &mut dyn Sys, p: &str) -> String {
    sys.read(p).unwrap_or_default().trim().to_string()
}

fn pkg_installed(sys: &mut dyn Sys, pkg: &str) -> bool {
    sys.run(&Cmd::new("dpkg-query").args(["-W", "-f=${Status}", pkg]))
        .is_ok_and(|o| o.stdout.contains("install ok installed"))
}

// ------------------------------------------------------------------ TUXEDO

fn tuxedo(sys: &mut dyn Sys) -> CardStatus {
    let vendor = trimmed(sys, "/sys/class/dmi/id/sys_vendor");
    let board = trimmed(sys, "/sys/class/dmi/id/board_vendor");
    let product = trimmed(sys, "/sys/class/dmi/id/product_name");
    let absent = |summary: String, vendor: String| CardStatus {
        id: CardId::Tuxedo,
        state: State::Absent,
        summary,
        facts: Facts::Tuxedo {
            vendor,
            product: String::new(),
            secure_boot: None,
            drivers: false,
            modules: vec![],
            mok: None,
        },
    };
    let is_tuxedo = [&vendor, &board]
        .iter()
        .any(|v| v.to_lowercase().contains("tuxedo"));
    if !is_tuxedo {
        let what = if vendor.is_empty() {
            "this machine".to_string()
        } else {
            vendor.clone()
        };
        return absent(
            format!("Not on this machine: {what} isn't a TUXEDO."),
            vendor,
        );
    }
    let arch = sys
        .run(&Cmd::new("dpkg").arg("--print-architecture"))
        .map(|o| o.stdout.trim().to_string())
        .unwrap_or_default();
    if arch != "amd64" {
        return absent(
            format!("Not on this machine: TUXEDO builds its drivers for amd64, this is {arch}."),
            vendor,
        );
    }

    let secure_boot = sys
        .read(SECURE_BOOT_VAR)
        .and_then(|s| parse_secure_boot(s.as_bytes()));
    let drivers = pkg_installed(sys, "tuxedo-drivers");
    let modules = tuxedo_modules(&sys.read("/proc/modules").unwrap_or_default());
    let mok = if secure_boot == Some(true) && drivers {
        Some(mok_state(sys))
    } else {
        None
    };

    let (state, summary) = if !drivers {
        (
            State::Needed,
            "Keyboard backlight, fans and power limits need TUXEDO's drivers.".to_string(),
        )
    } else {
        match mok {
            Some(MokState::NotEnrolled | MokState::NoKey) => (
                State::Needed,
                "Drivers are in, but Secure Boot won't load them until you enroll their key."
                    .to_string(),
            ),
            Some(MokState::Pending) => (
                State::Reboot,
                "Key queued. Reboot and enroll it on the blue MokManager screen.".to_string(),
            ),
            _ if modules.is_empty() => (
                State::Reboot,
                "Installed. The drivers load after a reboot.".to_string(),
            ),
            _ => (
                State::Done,
                format!("Drivers loaded: {}.", modules.join(", ")),
            ),
        }
    };
    CardStatus {
        id: CardId::Tuxedo,
        state,
        summary,
        facts: Facts::Tuxedo {
            vendor,
            product,
            secure_boot,
            drivers,
            modules,
            mok,
        },
    }
}

fn mok_state(sys: &mut dyn Sys) -> MokState {
    let Some(key) = MOK_KEYS.iter().find(|k| sys.exists(k)) else {
        return MokState::NoKey;
    };
    // Exits 1 for "enrolled" too; read the words, both streams.
    let cmd = Cmd::new("sh").args(["-c", "mokutil --test-key \"$1\" 2>&1; true", "sh", key]);
    let state = match sys.run(&cmd) {
        Ok(o) => parse_mok(&o.stdout),
        Err(_) => MokState::Unknown,
    };
    // HAZARD: mokutil writes MokNew mode 0600, so as the user `--test-key` can't read the request
    // and says "is not enrolled" for a key root just queued. Only the variable's presence is
    // visible; trusting mokutil alone asks for the password again forever.
    if state == MokState::NotEnrolled && sys.exists(MOK_NEW_VAR) {
        MokState::Pending
    } else {
        state
    }
}

// ------------------------------------------------------------- fingerprint

fn fingerprint(sys: &mut dyn Sys, env: &Env) -> CardStatus {
    let fprintd = sys.exists("/usr/bin/fprintd-list");
    let pam = sys
        .read("/etc/pam.d/common-auth")
        .is_some_and(|s| s.contains("pam_fprintd"));
    let mut reader = None;
    let mut fingers = vec![];
    let mut asked = false;
    if fprintd && let Ok(o) = sys.run(&Cmd::new("fprintd-list").arg(&env.user)) {
        asked = true;
        let l = parse_fprintd_list(&o.stdout);
        reader = l.device;
        fingers = l.fingers;
    }
    if !asked {
        reader = usb_reader(sys);
    }
    let facts = |reader: Option<String>, fingers: Vec<String>| Facts::Fingerprint {
        reader,
        fprintd,
        fingers,
        pam,
    };
    let Some(name) = reader.clone() else {
        return CardStatus {
            id: CardId::Fingerprint,
            state: State::Absent,
            summary: "Not on this machine: no fingerprint reader.".into(),
            facts: facts(None, fingers),
        };
    };
    let (state, summary) = if fingers.is_empty() {
        (State::Needed, format!("{name}. No finger enrolled yet."))
    } else if !pam {
        (
            State::Needed,
            format!(
                "{} enrolled, but sudo doesn't ask for it yet.",
                count(fingers.len(), "finger")
            ),
        )
    } else {
        let names: Vec<String> = fingers.iter().map(|f| finger_name(f)).collect();
        (
            State::Done,
            format!(
                "{}: {}. sudo and password dialogs take it.",
                count(fingers.len(), "finger"),
                names.join(", ")
            ),
        )
    };
    CardStatus {
        id: CardId::Fingerprint,
        state,
        summary,
        facts: facts(reader, fingers),
    }
}

fn usb_reader(sys: &mut dyn Sys) -> Option<String> {
    for d in sys.list("/sys/bus/usb/devices") {
        let base = format!("/sys/bus/usb/devices/{d}");
        let v = trimmed(sys, &format!("{base}/idVendor"));
        if v.is_empty() {
            continue;
        }
        let p = trimmed(sys, &format!("{base}/idProduct"));
        let prod = trimmed(sys, &format!("{base}/product"));
        if let Some(name) = fingerprint_usb(&v, &prod) {
            return Some(format!("{name} reader ({v}:{p})"));
        }
    }
    None
}

fn count(n: usize, what: &str) -> String {
    let words = ["No", "One", "Two", "Three", "Four", "Five"];
    let w = words.get(n).map_or(n.to_string(), |s| s.to_string());
    if n == 1 {
        format!("{w} {what}")
    } else {
        format!("{w} {what}s")
    }
}

// ------------------------------------------------------------------ YubiKey

/// Plugged-in YubiKeys (Yubico's USB vendor id), by product name.
pub(super) fn yubikeys(sys: &mut dyn Sys) -> Vec<String> {
    let mut keys = vec![];
    for d in sys.list("/sys/bus/usb/devices") {
        let base = format!("/sys/bus/usb/devices/{d}");
        if trimmed(sys, &format!("{base}/idVendor")) == "1050" {
            let p = trimmed(sys, &format!("{base}/product"));
            keys.push(if p.is_empty() { "YubiKey".into() } else { p });
        }
    }
    keys
}

fn yubikey(sys: &mut dyn Sys, env: &Env) -> CardStatus {
    let plugged = yubikeys(sys);
    let enrolled = u2f_count(
        &sys.read("/etc/u2f_mappings").unwrap_or_default(),
        &env.user,
    );
    let info = if plugged.len() == 1 {
        sys.run(&Cmd::new(env.script("dot-u2f-enroll")).arg("--info"))
            .ok()
            .map(|o| parse_key_info(&o.stdout))
    } else {
        None
    };
    let this_key_enrolled = info.as_ref().is_some_and(|i| i.enrolled_here);
    let pin = info.as_ref().and_then(|i| i.pin);
    let fingerprints = info.as_ref().and_then(|i| i.fingerprints);
    let keys = |n: usize| count(n, "key");
    let (state, summary) = match (plugged.len(), enrolled) {
        (0, 0) => (
            State::Waiting,
            "Plug in your YubiKey to use it for sudo, polkit and the lock screen.".to_string(),
        ),
        (0, n) => (
            State::Done,
            format!("{} enrolled for sudo, polkit and the lock screen.", keys(n)),
        ),
        (1, n) if this_key_enrolled => (
            State::Done,
            format!(
                "This key is enrolled ({} in total).{}",
                keys(n).to_lowercase(),
                if n < 2 {
                    " Enroll a backup key too: keys get lost."
                } else {
                    ""
                }
            ),
        ),
        (1, _) if pin == Some(false) || fingerprints == Some(false) => (
            State::Needed,
            "This key needs a PIN and a fingerprint of its own first.".to_string(),
        ),
        (1, 0) => (
            State::Needed,
            format!("{} plugged in, not enrolled yet.", plugged[0]),
        ),
        (1, n) => (
            State::Needed,
            format!(
                "A new key: {} enrolled already, this one isn't.",
                keys(n).to_lowercase()
            ),
        ),
        (_, _) => (
            State::Waiting,
            "More than one key plugged in. Leave only the one to enroll.".to_string(),
        ),
    };
    CardStatus {
        id: CardId::Yubikey,
        state,
        summary,
        facts: Facts::Yubikey {
            plugged,
            enrolled,
            this_key_enrolled,
            pin,
            fingerprints,
        },
    }
}

// ---------------------------------------------------------------- Bitwarden

fn bitwarden(sys: &mut dyn Sys, env: &Env) -> CardStatus {
    let desktop = sys.exists("/opt/Bitwarden");
    let paths = &env.bitwarden;
    let m = sys.bitwarden();
    let config = Config::read(m, paths);
    let configured = m.exists(&paths.config) && m.exists(&paths.ssh.join("config.d/bitwarden"));
    let folders = config.folders();
    let overview = Overview::read(m, paths);
    if !m.installed() {
        return CardStatus {
            id: CardId::Bitwarden,
            state: State::Absent,
            summary: "Not on this machine: bw isn't installed (system/70-bitwarden.sh does it)."
                .into(),
            facts: Facts::Bitwarden {
                bw: false,
                login: String::new(),
                email: None,
                server: None,
                configured,
                desktop,
                folders,
                overview,
                agent: None,
            },
        };
    }
    let st = m
        .status(None)
        .ok()
        .and_then(|o| parse_bw_status(&o))
        .unwrap_or_default();
    // The config remembers the email the last login used.
    let email = st.email.clone().or_else(|| config.get("email").map(str::to_string));
    let server = st.server.clone().or_else(|| config.get("server").map(str::to_string));
    let agent = Some(agent::probe(m, paths));
    let logged_in = matches!(st.status.as_str(), "locked" | "unlocked");
    let who = email.clone().unwrap_or_else(|| "you".into());
    let (state, summary) = if !logged_in {
        (State::Needed, "Not logged in.".to_string())
    } else if !configured {
        (
            State::Needed,
            format!("Logged in as {who}, but nothing's pulled from the vault yet."),
        )
    } else {
        (
            State::Done,
            format!(
                "Logged in as {who}. SSH keys, commit signing and project env come from the vault."
            ),
        )
    };
    CardStatus {
        id: CardId::Bitwarden,
        state,
        summary,
        facts: Facts::Bitwarden {
            bw: true,
            login: st.status,
            email,
            server,
            configured,
            desktop,
            folders,
            overview,
            agent,
        },
    }
}

// --------------------------------------------------------------------- Git

pub const GIT: &str = "/usr/bin/git";

/// git's answer, not a file: a name set by hand in ~/.gitconfig counts too.
fn git(sys: &mut dyn Sys, env: &Env) -> CardStatus {
    let mut get = |key: &str| {
        sys.run(&Cmd::new(GIT).args(["config", "--global", "--includes", "--get", key]))
            .ok()
            .map(|o| o.stdout.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let name = get("user.name");
    let email = get("user.email");
    let passwd = sys.read("/etc/passwd").unwrap_or_default();
    let full_name = passwd
        .lines()
        .map(|l| l.split(':').collect::<Vec<_>>())
        .find(|f| f.len() > 4 && f[0] == env.user)
        .and_then(|f| f[4].split(',').next().map(str::trim).map(str::to_string))
        .filter(|n| !n.is_empty());
    let (state, summary) = match (&name, &email) {
        (Some(n), Some(e)) => (State::Done, format!("Commits as {n} <{e}>.")),
        _ => (
            State::Needed,
            "No name and email for commits yet.".to_string(),
        ),
    };
    CardStatus {
        id: CardId::Git,
        state,
        summary,
        facts: Facts::Git {
            name,
            email,
            full_name,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::super::fake::FakeSys;
    use super::*;
    use crate::bitwarden::Agent;
    use crate::bitwarden::fake::Fake;
    use std::path::PathBuf;

    pub fn env() -> Env {
        Env {
            home: PathBuf::from("/home/kuba"),
            user: "kuba".into(),
            privileged: vec!["pkexec".into(), "/usr/lib/kubaos/setup-helper".into()],
            bitwarden: crate::bitwarden::Paths::under(std::path::Path::new("/home/kuba")),
        }
    }

    fn vm() -> FakeSys {
        // The bench VM: QEMU's DMI, no readers, no keys, bw installed.
        FakeSys::new()
            .file("/sys/class/dmi/id/sys_vendor", "QEMU\n")
            .file("/sys/bus/usb/devices/1-1/idVendor", "0627\n")
            .file("/sys/bus/usb/devices/1-1/product", "QEMU USB Keyboard\n")
            .file("/sys/bus/usb/devices/usb1/idVendor", "1d6b\n")
            .ok("nmcli", "full\n")
    }

    #[test]
    fn the_vm_has_nothing_but_bitwarden() {
        let mut s = vm();
        let st = status(&mut s, &env());
        assert!(st.online);
        let get = |id| st.card(id).unwrap().state;
        assert_eq!(get(CardId::Tuxedo), State::Absent);
        assert_eq!(get(CardId::Fingerprint), State::Absent);
        assert_eq!(get(CardId::Yubikey), State::Waiting);
        assert_eq!(get(CardId::Bitwarden), State::Needed);
        assert!(st.card(CardId::Tuxedo).unwrap().summary.contains("QEMU"));
        assert!(!st.settled());
    }

    #[test]
    fn offline_turns_needed_into_waiting() {
        let mut s = vm();
        s.responses[0] = ("nmcli".into(), Ok("none\n".into()));
        let st = status(&mut s, &env());
        assert!(!st.online);
        assert_eq!(st.card(CardId::Bitwarden).unwrap().state, State::Waiting);
        // No NetworkManager: the TCP fallback decides.
        let mut s = vm();
        let _ = s.responses.remove(0);
        s.online_fallback = true;
        assert!(status(&mut s, &env()).online);
    }

    fn tuxedo_box() -> FakeSys {
        FakeSys::new()
            .file("/sys/class/dmi/id/sys_vendor", "TUXEDO\n")
            .file(
                "/sys/class/dmi/id/product_name",
                "TUXEDO InfinityBook Pro AMD Gen10\n",
            )
            .ok("dpkg --print-architecture", "amd64\n")
    }

    #[test]
    fn tuxedo_states() {
        // Fresh: no drivers.
        let mut s = tuxedo_box().fail("dpkg-query", "no packages found");
        let c = tuxedo(&mut s);
        assert_eq!(c.state, State::Needed);

        // Drivers in, Secure Boot on, key not enrolled.
        let mut s = tuxedo_box()
            .file(SECURE_BOOT_VAR, "\u{6}\0\0\0\u{1}")
            .file("/var/lib/dkms/mok.pub", "cert")
            .ok(
                "dpkg-query -W -f=${Status} tuxedo-drivers",
                "install ok installed",
            )
            .ok("sh -c mokutil", "/var/lib/dkms/mok.pub is not enrolled\n");
        let c = tuxedo(&mut s);
        assert_eq!(c.state, State::Needed);
        assert!(matches!(
            c.facts,
            Facts::Tuxedo {
                secure_boot: Some(true),
                mok: Some(MokState::NotEnrolled),
                ..
            }
        ));

        // Queued: reboot.
        let mut s = tuxedo_box()
            .file(SECURE_BOOT_VAR, "\u{6}\0\0\0\u{1}")
            .file("/var/lib/dkms/mok.pub", "cert")
            .ok("dpkg-query", "install ok installed")
            .ok(
                "sh -c mokutil",
                "SKIP: /var/lib/dkms/mok.pub is already in the enrollment request\n",
            );
        assert_eq!(tuxedo(&mut s).state, State::Reboot);

        // Secure Boot off, modules loaded.
        let mut s = tuxedo_box()
            .file(SECURE_BOOT_VAR, "\u{6}\0\0\0\0")
            .file("/proc/modules", "tuxedo_keyboard 1 0 - Live\n")
            .ok("dpkg-query", "install ok installed");
        let c = tuxedo(&mut s);
        assert_eq!(c.state, State::Done);
        assert!(c.summary.contains("tuxedo_keyboard"));
        assert!(!s.ran_any("sh -c mokutil"));

        // A TUXEDO running arm64 (it doesn't exist, but the VM says TUXEDO
        // in tests): absent, not a failure.
        let mut s = FakeSys::new()
            .file("/sys/class/dmi/id/board_vendor", "TUXEDO\n")
            .ok("dpkg --print-architecture", "arm64\n");
        assert_eq!(tuxedo(&mut s).state, State::Absent);
    }

    #[test]
    fn a_key_root_queued_shows_as_reboot_although_the_user_cannot_read_the_request() {
        let mut s = tuxedo_box()
            .file(SECURE_BOOT_VAR, "\u{6}\0\0\0\u{1}")
            .file("/var/lib/shim-signed/mok/MOK.der", "cert")
            .file(MOK_NEW_VAR, "")
            .ok("dpkg-query", "install ok installed")
            .ok(
                "sh -c mokutil",
                "/var/lib/shim-signed/mok/MOK.der is not enrolled\n",
            );

        let c = tuxedo(&mut s);

        assert_eq!(c.state, State::Reboot, "{}", c.summary);
        assert!(matches!(
            c.facts,
            Facts::Tuxedo {
                mok: Some(MokState::Pending),
                ..
            }
        ));
    }

    #[test]
    fn fingerprint_states() {
        // Reader on USB, fprintd not installed yet.
        let mut s = FakeSys::new()
            .file("/sys/bus/usb/devices/3-3/idVendor", "27c6\n")
            .file("/sys/bus/usb/devices/3-3/idProduct", "609c\n");
        let c = fingerprint(&mut s, &env());
        assert_eq!(c.state, State::Needed);
        assert!(c.summary.starts_with("Goodix reader (27c6:609c)"));

        // fprintd answers: enrolled, PAM on.
        let mut s = FakeSys::new()
            .file("/usr/bin/fprintd-list", "")
            .file("/etc/pam.d/common-auth", "auth [success=2 default=ignore] pam_fprintd.so max-tries=1 timeout=10\n")
            .ok(
                "fprintd-list kuba",
                "Fingerprints for user kuba on Goodix MOC Fingerprint Sensor (press):\n - #0: right-index-finger\n",
            );
        let c = fingerprint(&mut s, &env());
        assert_eq!(c.state, State::Done);
        assert!(c.summary.contains("right index"));

        // fprintd: no device.
        let mut s = FakeSys::new()
            .file("/usr/bin/fprintd-list", "")
            .ok("fprintd-list", "No devices available\n");
        assert_eq!(fingerprint(&mut s, &env()).state, State::Absent);
    }

    #[test]
    fn yubikey_states() {
        let key = |s: FakeSys| {
            s.file("/sys/bus/usb/devices/1-2/idVendor", "1050\n")
                .file("/sys/bus/usb/devices/1-2/product", "YubiKey FIDO\n")
        };
        // Enrolled earlier, not plugged in.
        let mut s = FakeSys::new().file("/etc/u2f_mappings", "kuba:a,b,es256,+presence\n");
        let c = yubikey(&mut s, &env());
        assert_eq!(c.state, State::Done);
        assert!(c.summary.starts_with("One key enrolled"));

        // Plugged in, ready, not enrolled.
        let mut s = key(FakeSys::new()).ok(
            "/home/kuba/.local/bin/dot-u2f-enroll --info",
            "  PIN:          set\n  fingerprints: enrolled\n  this host:    not enrolled (pam://vm)\n",
        );
        assert_eq!(yubikey(&mut s, &env()).state, State::Needed);

        // Plugged in, no PIN.
        let mut s = key(FakeSys::new()).ok(
            "/home/kuba/.local/bin/dot-u2f-enroll --info",
            "  PIN:          NOT set\n  fingerprints: none yet\n",
        );
        let c = yubikey(&mut s, &env());
        assert_eq!(c.state, State::Needed);
        assert!(c.summary.contains("PIN"));

        // Plugged in and enrolled: done, with the backup nudge.
        let mut s = key(FakeSys::new())
            .file("/etc/u2f_mappings", "kuba:a,b,es256,+presence\n")
            .ok(
                "/home/kuba/.local/bin/dot-u2f-enroll --info",
                "  PIN: set\n  fingerprints: enrolled\n  this host:    enrolled as key 1 (pam://vm)\n",
            );
        let c = yubikey(&mut s, &env());
        assert_eq!(c.state, State::Done);
        assert!(c.summary.contains("backup"));
    }

    #[test]
    fn git_states() {
        // Nothing set: the full name from passwd is there to suggest.
        let mut s = FakeSys::new().file(
            "/etc/passwd",
            "root:x:0:0:root:/root:/bin/bash\nkuba:x:1000:1000:Kuba Example,,,:/home/kuba:/usr/bin/zsh\n",
        );
        let c = card_status(&mut s, &env(), CardId::Git, false);
        assert_eq!(c.state, State::Needed, "offline is fine: {}", c.summary);
        assert_eq!(
            c.facts,
            Facts::Git {
                name: None,
                email: None,
                full_name: Some("Kuba Example".into()),
            }
        );

        // A name without an email isn't an identity yet.
        let mut s = FakeSys::new().ok(
            "/usr/bin/git config --global --includes --get user.name",
            "Kuba Example\n",
        );
        assert_eq!(git(&mut s, &env()).state, State::Needed);

        // Set by setup, the install or by hand: git says so either way.
        let mut s = FakeSys::new()
            .ok(
                "/usr/bin/git config --global --includes --get user.name",
                "Kuba Example\n",
            )
            .ok(
                "/usr/bin/git config --global --includes --get user.email",
                "kuba@example.com\n",
            );
        let c = git(&mut s, &env());
        assert_eq!(c.state, State::Done);
        assert!(
            c.summary.contains("Kuba Example <kuba@example.com>"),
            "{}",
            c.summary
        );
    }

    #[test]
    fn bitwarden_states() {
        let mut s = FakeSys::new();
        s.bw.installed = false;
        assert_eq!(bitwarden(&mut s, &env()).state, State::Absent);

        let mut s = FakeSys::new();
        s.bw = Fake::new()
            .locked("https://vault.bitwarden.eu", "k@x.eu")
            .file("/home/kuba/.config/dot-bw/config", "server=https://vault.bitwarden.eu\nfolders=Work\n")
            .file("/home/kuba/.ssh/config.d/bitwarden", "");
        let c = bitwarden(&mut s, &env());
        assert_eq!(c.state, State::Done);
        assert!(c.summary.contains("k@x.eu"));
        assert!(matches!(c.facts, Facts::Bitwarden { ref folders, agent: Some(Agent::Off), .. } if folders == &["Work"]));

        // Logged out, but the config remembers the email for the form.
        let mut s = FakeSys::new();
        s.bw = Fake::new().file("/home/kuba/.config/dot-bw/config", "email=k@x.eu\n");
        let c = bitwarden(&mut s, &env());
        assert_eq!(c.state, State::Needed);
        assert!(matches!(c.facts, Facts::Bitwarden { email: Some(ref e), .. } if e == "k@x.eu"));
    }
}
