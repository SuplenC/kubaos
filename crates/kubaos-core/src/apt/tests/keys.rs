//! `fetch_key` in `system/lib.sh`, which sets up a vendor repository's key: kept while it lasts,
//! fetched again as it nears its expiry, the pinned fingerprint required either way. bash runs it
//! in a sandbox, with `curl` a stand-in serving a fixture key, `date` one saying when it is, and
//! the real `gpg` reading the keys.
//!
//! The keys in `test/fixtures/apt-keys/` are ed25519 keys made for these tests, with no secret
//! half kept: `vendor-2027.asc`, a certifying primary key with a signing subkey expiring
//! 2027-03-15 (as Element's does), and `vendor-2029.asc`, the same key with that subkey extended
//! to 2029-03-15; `primary-2027.asc`, a primary key that signs, expiring 2027-01-10;
//! `rotated.asc`, a signing subkey that expired in 2024 beside one lasting to 2029; `other.asc`,
//! another fingerprint.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;

use crate::sandbox::Sandbox;

const VENDOR: &str = "24E7C936DBEAB46D47A625C9F09D7A26C32D5EE1";
const PRIMARY: &str = "B6B52A5E689F0AD5B75B2D7B94935B3AF335D657";
const ROTATED: &str = "5D5C2D82F75F4A01F4A4D27A323569DE411A8227";

/// 2026-11-20: every fixture key but `other.asc`'s lasts more than 30 days.
const EARLY: u64 = 1_795_132_800;
/// 2027-03-05, ten days before the vendor's subkey expires, after the primary key's expiry.
const NEAR: u64 = 1_804_204_800;
/// 2027-04-01, past the vendor's subkey's expiry.
const PAST: u64 = 1_806_537_600;
/// 2029-03-01, two weeks before the vendor's extended subkey expires.
const LATE: u64 = 1_867_017_600;

/// The expiry of the vendor's extended subkey, as `gpg --with-colons` gives it.
const EXTENDED: &str = "1868266800";

struct Machine {
    dir: crate::sandbox::TempDir,
    sandbox: Sandbox,
}

impl Machine {
    /// A sandbox whose `date +%s` is `now`, with the real gpg, and `stored` at the keyring when
    /// given.
    fn new(now: u64, stored: Option<&str>) -> Self {
        let dir = crate::sandbox::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(home.join(".gnupg")).unwrap();
        let sandbox = Sandbox::new(&dir.path().join("sandbox"), &home, "kuba");
        let gpg = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|dir| dir.join("gpg"))
            .find(|path| path.is_file())
            .expect("gpg on PATH");
        symlink(gpg, sandbox.bin().join("gpg")).unwrap();
        sandbox.stub("date", &format!("echo {now}"));
        let machine = Self { dir, sandbox };
        if let Some(name) = stored {
            fs::create_dir_all(machine.keyring().parent().unwrap()).unwrap();
            fs::copy(fixture(name), machine.keyring()).unwrap();
        }
        machine
    }

    fn keyring(&self) -> PathBuf {
        self.dir.path().join("keyrings/vendor.gpg")
    }

    /// `curl -o` writes the fixture `name`.
    fn serves(&self, name: &str) {
        let key = fixture(name);
        self.sandbox.stub(
            "curl",
            &format!(
                "while [ $# -gt 0 ]; do [ \"$1\" = -o ] && exec cp '{}' \"$2\"; shift; done; exit 2",
                key.display()
            ),
        );
    }

    /// `fetch_key` of the repository pinned to `fingerprint`, its sudo running as the test.
    fn fetch(&self, fingerprint: &str) -> Output {
        let system = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../system");
        let script = format!(
            "source \"$SYSTEM/lib.sh\"\nsudo_run() {{ run \"$@\"; }}\n\
             fetch_key https://vendor.example/key.asc '{}' {fingerprint}\n",
            self.keyring().display()
        );
        self.sandbox
            .command("bash")
            .args(["-c", &script])
            .env("SYSTEM", system)
            .env("GNUPGHOME", self.dir.path().join("home/.gnupg"))
            .env("SUDO_USER", "")
            .output()
            .unwrap()
    }

    /// `system/apt-repo.sh --keys-due` of the repository pinned to `fingerprint` with its key at
    /// the keyring, and of `gone`, a repository the manifest lacks.
    fn due(&self, fingerprint: &str) -> Output {
        let system = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../system");
        let manifest = self.dir.path().join("packages.toml");
        fs::write(
            &manifest,
            format!(
                "[repo.vendor]\nkey_url = \"https://vendor.example/key.asc\"\nfingerprint = \"{fingerprint}\"\n\
                 keyring = \"{}\"\n",
                self.keyring().display()
            ),
        )
        .unwrap();
        self.sandbox
            .command("bash")
            .arg(system.join("apt-repo.sh"))
            .args(["--keys-due", "vendor", "gone"])
            .env("APT_MANIFEST", manifest)
            .env_remove("GNUPGHOME")
            .output()
            .unwrap()
    }

    fn fetched(&self) -> bool {
        self.sandbox.calls().iter().any(|call| call.starts_with("curl "))
    }

    /// The expiry of each key of the stored keyring, `pub` and `sub` lines in order.
    fn expiries(&self) -> Vec<String> {
        let out = self
            .sandbox
            .command("gpg")
            .args(["--show-keys", "--with-colons"])
            .arg(self.keyring())
            .env("GNUPGHOME", self.dir.path().join("home/.gnupg"))
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| line.starts_with("pub:") || line.starts_with("sub:"))
            .map(|line| line.split(':').nth(6).unwrap_or_default().to_owned())
            .collect()
    }
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test/fixtures/apt-keys")
        .join(name)
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn a_stored_key_that_lasts_is_kept_and_nothing_is_fetched() {
    let m = Machine::new(EARLY, Some("vendor-2027.asc"));
    m.serves("vendor-2029.asc");
    let before = fs::read(m.keyring()).unwrap();

    let out = m.fetch(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert!(!m.fetched());
    assert_eq!(fs::read(m.keyring()).unwrap(), before);
}

#[test]
fn a_stored_key_whose_signing_subkey_expires_within_30_days_is_fetched_again() {
    let m = Machine::new(NEAR, Some("vendor-2027.asc"));
    m.serves("vendor-2029.asc");

    let out = m.fetch(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert!(m.fetched());
    assert_eq!(m.expiries(), ["", EXTENDED], "{}", text(&out));
}

#[test]
fn a_stored_primary_key_that_has_expired_is_fetched_again_and_a_failed_fetch_keeps_it() {
    let m = Machine::new(NEAR, Some("primary-2027.asc"));
    m.sandbox.stub("curl", "exit 22");
    let before = fs::read(m.keyring()).unwrap();

    let out = m.fetch(PRIMARY);

    assert!(out.status.success(), "{}", text(&out));
    assert!(m.fetched());
    assert!(text(&out).contains("keeping the key"), "{}", text(&out));
    assert!(text(&out).contains("2027-01-10"), "{}", text(&out));
    assert_eq!(fs::read(m.keyring()).unwrap(), before);
}

#[test]
fn a_refetched_key_that_still_expires_is_installed_with_a_warning() {
    let m = Machine::new(PAST, Some("vendor-2027.asc"));
    m.serves("vendor-2027.asc");

    let out = m.fetch(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert!(m.fetched());
    assert!(text(&out).contains("expires 2027-03-15"), "{}", text(&out));
}

#[test]
fn a_refetched_key_of_another_fingerprint_is_refused_with_a_warning_and_the_stored_one_kept() {
    let m = Machine::new(NEAR, Some("vendor-2027.asc"));
    m.serves("other.asc");
    let before = fs::read(m.keyring()).unwrap();

    let out = m.fetch(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("refusing to trust it"), "{}", text(&out));
    assert!(text(&out).contains("keeping the key"), "{}", text(&out));
    assert!(text(&out).contains("2027-03-15"), "{}", text(&out));
    assert_eq!(fs::read(m.keyring()).unwrap(), before);
}

#[test]
fn a_refetched_copy_expiring_before_the_stored_one_is_not_installed() {
    let m = Machine::new(LATE, Some("vendor-2029.asc"));
    m.serves("vendor-2027.asc");
    let before = fs::read(m.keyring()).unwrap();

    let out = m.fetch(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert!(m.fetched());
    assert!(text(&out).contains("keeping that"), "{}", text(&out));
    assert_eq!(fs::read(m.keyring()).unwrap(), before);
}

#[test]
fn an_expired_subkey_beside_one_that_lasts_needs_no_fetch() {
    let m = Machine::new(EARLY, Some("rotated.asc"));
    m.serves("rotated.asc");

    let out = m.fetch(ROTATED);

    assert!(out.status.success(), "{}", text(&out));
    assert!(!m.fetched());
}

#[test]
fn a_key_not_yet_stored_is_fetched_and_stored_dearmored() {
    let m = Machine::new(EARLY, None);
    m.serves("vendor-2029.asc");

    let out = m.fetch(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(m.expiries(), ["", EXTENDED]);
    assert!(!fs::read(m.keyring()).unwrap().starts_with(b"-----BEGIN"));
}

#[test]
fn a_global_keyring_holding_only_the_vendors_keys_is_removed_and_one_with_another_key_stays() {
    let m = Machine::new(EARLY, None);
    let trusted = m.dir.path().join("trusted.gpg.d");
    fs::create_dir_all(&trusted).unwrap();
    fs::copy(fixture("vendor-2027.asc"), trusted.join("vendor.gpg")).unwrap();
    fs::copy(fixture("primary-2027.asc"), trusted.join("vendor-old.gpg")).unwrap();
    fs::copy(fixture("other.asc"), trusted.join("someone.gpg")).unwrap();
    let both = [
        fs::read(fixture("vendor-2027.asc")).unwrap(),
        fs::read(fixture("other.asc")).unwrap(),
    ]
    .concat();
    fs::write(trusted.join("shared.gpg"), both).unwrap();
    let system = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../system");
    let script = format!(
        "source \"$SYSTEM/lib.sh\"\nsudo_run() {{ run \"$@\"; }}\n\
         for k in vendor vendor-old someone shared absent; do\n\
         untrust_global_keyring \"$TRUSTED/$k.gpg\" {VENDOR} {PRIMARY}\ndone\n"
    );

    let out = m
        .sandbox
        .command("bash")
        .args(["-c", &script])
        .env("SYSTEM", system)
        .env("TRUSTED", &trusted)
        .env("GNUPGHOME", m.dir.path().join("home/.gnupg"))
        .env("SUDO_USER", "")
        .output()
        .unwrap();

    assert!(out.status.success(), "{}", text(&out));
    let mut left: Vec<String> = fs::read_dir(&trusted)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(left, ["shared.gpg", "someone.gpg"], "{}", text(&out));
    assert!(text(&out).contains("shared.gpg trusts"), "{}", text(&out));
}

#[test]
fn a_stored_key_nearing_its_expiry_is_due_and_saying_so_changes_nothing() {
    let m = Machine::new(NEAR, Some("vendor-2027.asc"));
    m.serves("vendor-2029.asc");
    let before = fs::read(m.keyring()).unwrap();

    let out = m.due(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "vendor\n");
    assert_eq!(fs::read(m.keyring()).unwrap(), before);
    let calls = m.sandbox.calls();
    assert!(
        !calls.iter().any(|c| c.starts_with("curl ") || c.starts_with("sudo ")),
        "{calls:?}"
    );
    assert!(!m.dir.path().join("home/.gnupg/pubring.kbx").exists());
}

#[test]
fn a_stored_key_that_lasts_is_not_due() {
    let m = Machine::new(EARLY, Some("vendor-2027.asc"));

    let out = m.due(VENDOR);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

#[test]
fn a_missing_key_or_one_of_another_fingerprint_is_due() {
    let missing = Machine::new(EARLY, None);
    let other = Machine::new(EARLY, Some("other.asc"));

    let (missing, other) = (missing.due(VENDOR), other.due(VENDOR));

    assert_eq!(
        String::from_utf8_lossy(&missing.stdout),
        "vendor\n",
        "{}",
        text(&missing)
    );
    assert_eq!(String::from_utf8_lossy(&other.stdout), "vendor\n", "{}", text(&other));
}
