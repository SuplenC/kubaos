use std::collections::BTreeSet;
use std::path::Path;

use super::apt_options;
use super::candidates;
use super::pending;
use super::with_newer_security;

const SIMULATED: &str = "\
Inst openssl [3.0.13-0ubuntu3] (3.0.13-0ubuntu3.5 Ubuntu:24.04/noble-updates, Ubuntu:24.04/noble-security [amd64])
Inst vim [2:9.1.0016] (2:9.1.0017 Ubuntu:24.04/noble-updates [amd64])
Inst linux-image-6.8.0-60-generic (6.8.0-60.63 Ubuntu:24.04/noble-security [amd64])
Conf openssl (3.0.13-0ubuntu3.5 Ubuntu:24.04/noble-updates, Ubuntu:24.04/noble-security [amd64])
Remv oldthing [1.0]
";

const POLICY: &str = "\
vim:
  Installed: 2:9.1.0016
  Candidate: 2:9.1.0018
  Version table:
     2:9.1.0018 500
        500 http://archive.ubuntu.com/ubuntu noble-updates/main amd64 Packages
     2:9.1.0017 500
        500 http://security.ubuntu.com/ubuntu noble-security/main amd64 Packages
 *** 2:9.1.0016 100
        100 /var/lib/dpkg/status
curl:
  Installed: 8.5.0-2
  Candidate: 8.5.0-3
  Version table:
     8.5.0-3 500
        500 http://archive.ubuntu.com/ubuntu noble-updates/main amd64 Packages
 *** 8.5.0-2 500
        500 http://security.ubuntu.com/ubuntu noble-security/main amd64 Packages
";

#[test]
fn only_packages_from_a_security_pocket_count() {
    let found = pending(&candidates(SIMULATED), &BTreeSet::new());

    assert_eq!(
        found,
        ["linux-image-6.8.0-60-generic=6.8.0-60.63", "openssl=3.0.13-0ubuntu3.5"]
    );
}

#[test]
fn a_security_fix_inside_a_newer_updates_version_counts() {
    let simulated = "\
Inst vim [2:9.1.0016] (2:9.1.0018 Ubuntu:24.04/noble-updates [amd64])
Inst curl [8.5.0-2] (8.5.0-3 Ubuntu:24.04/noble-updates [amd64])
";

    let newer = with_newer_security(POLICY);
    let found = pending(&candidates(simulated), &newer);

    assert_eq!(newer, BTreeSet::from(["vim".to_owned()]));
    assert_eq!(found, ["vim=2:9.1.0018"]);
}

#[test]
fn nothing_is_pending_when_apt_has_nothing_to_install() {
    assert!(candidates("Reading package lists...\nBuilding dependency tree...\n").is_empty());
}

#[test]
fn apt_is_pointed_at_the_private_lists_with_no_package_cache() {
    let options: Vec<String> = apt_options(Path::new("/home/u/.local/state/kubaos/apt"))
        .iter()
        .map(|o| o.to_string_lossy().into_owned())
        .collect();

    assert_eq!(
        options,
        [
            "-o",
            "Dir::State=/home/u/.local/state/kubaos/apt/state",
            "-o",
            "Dir::Cache=/home/u/.local/state/kubaos/apt/cache",
            "-o",
            "Dir::State::status=/var/lib/dpkg/status",
            "-o",
            "Dir::State::extended_states=/var/lib/apt/extended_states",
            "-o",
            "Dir::Cache::srcpkgcache=",
            "-o",
            "Dir::Cache::pkgcache=",
            "-o",
            "Acquire::Languages=none",
        ]
    );
}
