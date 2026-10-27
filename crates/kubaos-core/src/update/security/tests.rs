use std::path::Path;

use super::apt_options;
use super::pending;

#[test]
fn only_packages_from_a_security_pocket_count() {
    let simulated = "\
Inst openssl [3.0.13-0ubuntu3] (3.0.13-0ubuntu3.5 Ubuntu:24.04/noble-updates, Ubuntu:24.04/noble-security [amd64])
Inst vim [2:9.1.0016] (2:9.1.0017 Ubuntu:24.04/noble-updates [amd64])
Inst linux-image-6.8.0-60-generic (6.8.0-60.63 Ubuntu:24.04/noble-security [amd64])
Inst esm-lib [1.0] (1.1 UbuntuESMApps:24.04/noble-apps-security [amd64])
Conf openssl (3.0.13-0ubuntu3.5 Ubuntu:24.04/noble-updates, Ubuntu:24.04/noble-security [amd64])
Remv oldthing [1.0]
";

    assert_eq!(
        pending(simulated),
        [
            "esm-lib=1.1",
            "linux-image-6.8.0-60-generic=6.8.0-60.63",
            "openssl=3.0.13-0ubuntu3.5"
        ]
    );
}

#[test]
fn nothing_is_pending_when_apt_has_nothing_to_install() {
    assert!(pending("Reading package lists...\nBuilding dependency tree...\n").is_empty());
}

#[test]
fn apt_is_pointed_at_the_private_lists_and_nothing_else() {
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
        ]
    );
}
