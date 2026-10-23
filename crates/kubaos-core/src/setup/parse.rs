//! Pure functions over what the tools print. Kept apart so the tests can
//! feed them real output without a machine.

use super::MokState;

/// `SecureBoot-8be4df61-…` from efivarfs: 4 bytes of attributes, then the
/// value (1 = on).
pub fn parse_secure_boot(efivar: &[u8]) -> Option<bool> {
    efivar.get(4).map(|b| *b == 1)
}

/// `mokutil --test-key FILE` (stdout and stderr): it exits 1 both when the
/// key is enrolled and when it's queued, so only the words count.
pub fn parse_mok(out: &str) -> MokState {
    if out.contains("is already enrolled") || out.contains("is already in db") {
        MokState::Enrolled
    } else if out.contains("already in the enrollment request") {
        MokState::Pending
    } else if out.contains("is not enrolled") {
        MokState::NotEnrolled
    } else {
        MokState::Unknown
    }
}

/// TUXEDO's kernel modules from `/proc/modules`.
pub fn tuxedo_modules(proc_modules: &str) -> Vec<String> {
    proc_modules
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|m| {
            ["tuxedo", "uniwill", "clevo"]
                .iter()
                .any(|p| m.starts_with(p))
        })
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FprintList {
    /// The reader's name, `None` when fprintd has no device.
    pub device: Option<String>,
    pub fingers: Vec<String>,
}

/// `fprintd-list USER`:
///
/// ```text
/// found 1 devices
/// Device at /net/reactivated/Fprint/Device/0
/// Using device /net/reactivated/Fprint/Device/0
/// Fingerprints for user kuba on Goodix MOC Fingerprint Sensor (press):
///  - #0: right-index-finger
/// ```
///
/// or "No devices available", or "User kuba has no fingers enrolled for
/// <device>."
pub fn parse_fprintd_list(out: &str) -> FprintList {
    let mut l = FprintList::default();
    for line in out.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Fingerprints for user ") {
            if let Some((_, dev)) = rest.split_once(" on ") {
                let dev = dev.trim_end_matches(':');
                let dev = dev.rsplit_once(" (").map_or(dev, |(d, _)| d);
                l.device = Some(dev.trim().to_string());
            }
        } else if let Some(rest) = t.strip_prefix("- #") {
            if let Some((_, f)) = rest.split_once(": ") {
                l.fingers.push(f.trim().to_string());
            }
        } else if t.starts_with("User ") && t.contains(" has no fingers enrolled for ") {
            let dev = t
                .rsplit_once(" enrolled for ")
                .map(|(_, d)| d.trim_end_matches('.').trim().to_string());
            l.device = dev;
        } else if t.starts_with("Using device ") && l.device.is_none() {
            l.device = Some("Fingerprint reader".into());
        }
    }
    l
}

/// fprintd's finger names in the order setup offers them.
pub const FINGERS: [&str; 10] = [
    "right-index-finger",
    "left-index-finger",
    "right-middle-finger",
    "left-middle-finger",
    "right-thumb",
    "left-thumb",
    "right-ring-finger",
    "left-ring-finger",
    "right-little-finger",
    "left-little-finger",
];

/// "right-index-finger" -> "right index".
pub fn finger_name(f: &str) -> String {
    f.trim_end_matches("-finger").replace('-', " ")
}

/// The first finger not enrolled yet.
pub fn next_finger(enrolled: &[String]) -> &'static str {
    FINGERS
        .iter()
        .find(|f| !enrolled.iter().any(|e| e == *f))
        .copied()
        .unwrap_or(FINGERS[0])
}

#[derive(Debug, Clone, PartialEq)]
pub enum Enroll {
    /// One good touch.
    Stage,
    /// Try that touch again, with fprintd's reason.
    Retry(&'static str),
    Completed,
    Failed(String),
    Other,
}

/// One line of `fprintd-enroll` ("Enroll result: enroll-stage-passed").
pub fn enroll_line(line: &str) -> Enroll {
    let Some(r) = line.trim().strip_prefix("Enroll result: ") else {
        return Enroll::Other;
    };
    match r.trim() {
        "enroll-stage-passed" => Enroll::Stage,
        "enroll-completed" => Enroll::Completed,
        "enroll-retry-scan" => Enroll::Retry("Didn't catch that. Same finger, again."),
        "enroll-swipe-too-short" => Enroll::Retry("Too quick. Again, a bit slower."),
        "enroll-finger-not-centered" => Enroll::Retry("Off-centre. Put it flat in the middle."),
        "enroll-remove-and-retry" => Enroll::Retry("Lift your finger and put it back."),
        "enroll-duplicate" => Enroll::Failed("that finger is enrolled already".into()),
        "enroll-data-full" => Enroll::Failed("the reader is full".into()),
        "enroll-disconnected" => Enroll::Failed("the reader went away".into()),
        other => Enroll::Failed(other.trim_start_matches("enroll-").replace('-', " ")),
    }
}

/// What `dot-u2f-enroll --info` says about the plugged-in key.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeyInfo {
    pub pin: Option<bool>,
    /// `None`: no fingerprint reader on the key.
    pub fingerprints: Option<bool>,
    pub enrolled_here: bool,
}

pub fn parse_key_info(out: &str) -> KeyInfo {
    let mut k = KeyInfo::default();
    for line in out.lines() {
        let Some((key, val)) = line.trim().split_once(':') else {
            continue;
        };
        let val = val.trim();
        match key.trim() {
            "PIN" => {
                k.pin = if val.starts_with("set") {
                    Some(true)
                } else if val.starts_with("NOT set") {
                    Some(false)
                } else {
                    None
                }
            }
            "fingerprints" => {
                k.fingerprints = if val.starts_with("enrolled") {
                    Some(true)
                } else if val.starts_with("none yet") {
                    Some(false)
                } else {
                    None
                }
            }
            "this host" => k.enrolled_here = val.starts_with("enrolled"),
            _ => {}
        }
    }
    k
}

/// Credentials for `user` in `/etc/u2f_mappings` (`user:cred1:cred2`).
pub fn u2f_count(mappings: &str, user: &str) -> usize {
    mappings
        .lines()
        .find_map(|l| l.strip_prefix(user)?.strip_prefix(':'))
        .map_or(0, |rest| rest.split(':').filter(|c| !c.is_empty()).count())
}

/// One `pamu2fcfg -n` credential (`:KeyHandle,PublicKey,CoseType,Options`,
/// the leading colon already gone): base64 fields, never a `:` or a newline.
pub fn valid_credential(c: &str) -> bool {
    let parts: Vec<&str> = c.split(',').collect();
    let b64 = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "+/=_-".contains(ch))
    };
    parts.len() == 4
        && (b64(parts[0]) || parts[0] == "*")
        && b64(parts[1])
        && parts[2].chars().all(|ch| ch.is_ascii_alphanumeric())
        && !parts[2].is_empty()
        && parts[3]
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch == '+')
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BwStatus {
    /// unauthenticated, locked, unlocked.
    pub status: String,
    pub email: Option<String>,
    pub server: Option<String>,
}

/// `bw status` (JSON on stdout).
pub fn parse_bw_status(json: &str) -> Option<BwStatus> {
    let v: serde_json::Value = serde_json::from_str(json.trim()).ok()?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_string);
    Some(BwStatus {
        status: s("status")?,
        email: s("userEmail"),
        server: s("serverUrl"),
    })
}

/// A Bitwarden server as `kubaos bitwarden setup` takes it: com, eu, or an https URL.
pub fn server_url(s: &str) -> Result<String, String> {
    let s = s.trim();
    match s {
        "" | "com" | "bitwarden.com" | "https://bitwarden.com" | "https://vault.bitwarden.com" => {
            Ok("https://vault.bitwarden.com".into())
        }
        "eu" | "bitwarden.eu" | "https://bitwarden.eu" | "https://vault.bitwarden.eu" => {
            Ok("https://vault.bitwarden.eu".into())
        }
        u if u.starts_with("https://") && u.len() > 8 && !u.contains(char::is_whitespace) => {
            Ok(u.trim_end_matches('/').to_string())
        }
        _ => Err(format!("{s} isn't a server: com, eu or an https:// URL")),
    }
}

/// bw's error text -> (what failed, what to do).
pub fn bw_error(stderr: &str) -> (String, Option<String>) {
    let s = stderr.to_lowercase();
    let pick = |e: &str, h: &str| (e.to_string(), Some(h.to_string()));
    if s.contains("two-step") && (s.contains("invalid") || s.contains("incorrect")) {
        pick(
            "Bitwarden didn't take that code",
            "Codes last 30 seconds. Type the next one, quicker.",
        )
    } else if s.contains("username or password is incorrect")
        || s.contains("invalid master password")
    {
        pick(
            "wrong email or master password",
            "Check both and try again, slower this time.",
        )
    } else if s.contains("two-step") || s.contains("two step") {
        pick(
            "Bitwarden wants a two-step code",
            "Pick the authenticator app and type its code.",
        )
    } else if s.contains("client_id") || s.contains("invalid_client") {
        pick(
            "Bitwarden refused the API key",
            "Web vault: Settings, Security, Keys, View API key. Copy both again.",
        )
    } else if s.contains("enotfound") || s.contains("econnrefused") || s.contains("fetch failed") {
        pick(
            "can't reach the Bitwarden server",
            "Check the network and the server address.",
        )
    } else {
        let last = stderr
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("bw failed without saying why")
            .trim()
            .to_string();
        (last, None)
    }
}

/// USB vendors that make fingerprint readers libfprint drives. Used only
/// before fprintd is installed; afterwards fprintd itself answers.
pub const FINGERPRINT_VENDORS: [(&str, &str); 10] = [
    ("27c6", "Goodix"),
    ("06cb", "Synaptics"),
    ("04f3", "Elan"),
    ("1c7a", "Egis"),
    ("2808", "FocalTech"),
    ("298d", "Next Biometrics"),
    ("138a", "Validity"),
    ("147e", "Upek"),
    ("10a5", "FPC"),
    ("08ff", "AuthenTec"),
];

/// Whether a USB device looks like a fingerprint reader. Elan and Synaptics
/// also make touchpads, but those sit on I2C, not USB.
pub fn fingerprint_usb(vendor: &str, product: &str) -> Option<String> {
    let p = product.to_lowercase();
    let known = FINGERPRINT_VENDORS
        .iter()
        .find(|(v, _)| v.eq_ignore_ascii_case(vendor))
        .map(|(_, n)| *n);
    if known.is_some() || p.contains("finger") {
        Some(known.unwrap_or("Fingerprint reader").to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_boot_var() {
        assert_eq!(parse_secure_boot(&[6, 0, 0, 0, 1]), Some(true));
        assert_eq!(parse_secure_boot(&[6, 0, 0, 0, 0]), Some(false));
        assert_eq!(parse_secure_boot(&[6, 0]), None);
    }

    #[test]
    fn mok_words() {
        assert_eq!(
            parse_mok("/var/lib/dkms/mok.pub is already enrolled"),
            MokState::Enrolled
        );
        assert_eq!(
            parse_mok("SKIP: /var/lib/dkms/mok.pub is already in the enrollment request"),
            MokState::Pending
        );
        assert_eq!(
            parse_mok("/var/lib/dkms/mok.pub is not enrolled"),
            MokState::NotEnrolled
        );
        assert_eq!(
            parse_mok("EFI variables are not supported"),
            MokState::Unknown
        );
    }

    #[test]
    fn modules() {
        let m = "tuxedo_keyboard 45056 0 - Live 0x0\nuniwill_wmi 16384 1 tuxedo_keyboard, Live 0x0\nbtrfs 1 0 - Live\n";
        assert_eq!(tuxedo_modules(m), ["tuxedo_keyboard", "uniwill_wmi"]);
    }

    #[test]
    fn fprintd_list_shapes() {
        let full = "found 1 devices\nDevice at /net/reactivated/Fprint/Device/0\nUsing device /net/reactivated/Fprint/Device/0\nFingerprints for user kuba on Goodix MOC Fingerprint Sensor (press):\n - #0: right-index-finger\n - #1: left-index-finger\n";
        let l = parse_fprintd_list(full);
        assert_eq!(l.device.as_deref(), Some("Goodix MOC Fingerprint Sensor"));
        assert_eq!(l.fingers, ["right-index-finger", "left-index-finger"]);

        let none = "found 1 devices\nDevice at /net/reactivated/Fprint/Device/0\nUsing device /net/reactivated/Fprint/Device/0\nUser kuba has no fingers enrolled for Goodix MOC Fingerprint Sensor.\n";
        let l = parse_fprintd_list(none);
        assert_eq!(l.device.as_deref(), Some("Goodix MOC Fingerprint Sensor"));
        assert!(l.fingers.is_empty());

        assert_eq!(
            parse_fprintd_list("No devices available\n"),
            FprintList::default()
        );
    }

    #[test]
    fn fingers() {
        assert_eq!(finger_name("right-index-finger"), "right index");
        assert_eq!(finger_name("left-thumb"), "left thumb");
        assert_eq!(next_finger(&[]), "right-index-finger");
        assert_eq!(
            next_finger(&["right-index-finger".into()]),
            "left-index-finger"
        );
    }

    #[test]
    fn enroll_results() {
        assert_eq!(
            enroll_line("Enroll result: enroll-stage-passed"),
            Enroll::Stage
        );
        assert_eq!(
            enroll_line("Enroll result: enroll-completed"),
            Enroll::Completed
        );
        assert!(matches!(
            enroll_line("Enroll result: enroll-retry-scan"),
            Enroll::Retry(_)
        ));
        assert_eq!(
            enroll_line("Enroll result: enroll-duplicate"),
            Enroll::Failed("that finger is enrolled already".into())
        );
        assert_eq!(
            enroll_line("Enrolling right-index-finger finger."),
            Enroll::Other
        );
    }

    #[test]
    fn key_info() {
        let out = "Key: /dev/hidraw3  Yubico YubiKey FIDO\n  model:        YubiKey C Bio - FIDO Edition\n  PIN:          set, 8 tries left\n  fingerprints: enrolled, 3 scans left before the PIN is needed\n  this host:    not enrolled (pam://vm)\n";
        assert_eq!(
            parse_key_info(out),
            KeyInfo {
                pin: Some(true),
                fingerprints: Some(true),
                enrolled_here: false
            }
        );
        let out = "  PIN:          NOT set\n  fingerprints: none yet\n  this host:    enrolled as key 2 (pam://vm)\n";
        assert_eq!(
            parse_key_info(out),
            KeyInfo {
                pin: Some(false),
                fingerprints: Some(false),
                enrolled_here: true
            }
        );
        let out = "  PIN:          set\n  fingerprints: no reader\n";
        assert_eq!(parse_key_info(out).fingerprints, None);
    }

    #[test]
    fn mappings() {
        let m = "other:a,b,es256,+presence\nkuba:k1,p1,es256,+presence+verification:k2,p2,es256,+presence+verification\n";
        assert_eq!(u2f_count(m, "kuba"), 2);
        assert_eq!(u2f_count(m, "jak"), 0);
        assert_eq!(u2f_count("", "kuba"), 0);
    }

    #[test]
    fn credentials() {
        assert!(valid_credential(
            "owBYtdK4lPq+/A==,pDtKMa6S1nf+Q==,es256,+presence+verification"
        ));
        assert!(!valid_credential("a,b,es256,+presence\nroot:x,y,es256,"));
        assert!(!valid_credential("a,b:c,es256,+presence"));
        assert!(!valid_credential("a,b,es256"));
    }

    #[test]
    fn bw_status_and_server() {
        let s = parse_bw_status(
            r#"{"serverUrl":"https://vault.bitwarden.eu","lastSync":null,"userEmail":"k@x.eu","userId":"1","status":"locked"}"#,
        )
        .unwrap();
        assert_eq!(s.status, "locked");
        assert_eq!(s.email.as_deref(), Some("k@x.eu"));
        let s = parse_bw_status(r#"{"serverUrl":null,"status":"unauthenticated"}"#).unwrap();
        assert_eq!((s.status.as_str(), s.server), ("unauthenticated", None));
        assert!(parse_bw_status("not json").is_none());

        assert_eq!(server_url("eu").unwrap(), "https://vault.bitwarden.eu");
        assert_eq!(server_url("").unwrap(), "https://vault.bitwarden.com");
        assert_eq!(
            server_url("https://bw.home.lan/").unwrap(),
            "https://bw.home.lan"
        );
        assert!(server_url("http://insecure").is_err());
    }

    #[test]
    fn bw_errors_name_the_cause() {
        let (e, h) = bw_error("Username or password is incorrect. Try again.");
        assert_eq!(e, "wrong email or master password");
        assert!(h.is_some());
        let (e, _) = bw_error("Two-step token is invalid. Try again.");
        assert!(e.contains("code"));
        let (e, h) = bw_error("something odd\n");
        assert_eq!((e.as_str(), h), ("something odd", None));
    }

    #[test]
    fn usb_readers() {
        assert_eq!(fingerprint_usb("27c6", "").as_deref(), Some("Goodix"));
        assert_eq!(
            fingerprint_usb("dead", "USB Fingerprint Sensor").as_deref(),
            Some("Fingerprint reader")
        );
        assert_eq!(fingerprint_usb("046d", "USB Receiver"), None);
    }
}
