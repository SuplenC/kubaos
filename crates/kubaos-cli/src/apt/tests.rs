use kubaos_core::apt::Status;
use kubaos_core::apt::Switch;
use kubaos_core::apt::plan::GroupStatus;
use kubaos_core::apt::plan::PackageState;
use kubaos_core::apt::plan::PackageStatus;

use super::table;

fn package(name: &str, group: &str, state: PackageState, version: Option<&str>) -> PackageStatus {
    PackageStatus {
        name: name.into(),
        group: group.into(),
        state,
        version: version.map(str::to_owned),
        kubaos: true,
    }
}

fn group(name: &str, inactive: Option<&str>, optional: Option<Switch>, packages: Vec<PackageStatus>) -> GroupStatus {
    GroupStatus {
        name: name.into(),
        summary: format!("The {name} group"),
        active: inactive.is_none(),
        inactive: inactive.map(str::to_owned),
        optional,
        packages,
    }
}

#[test]
fn the_table_lines_up_each_groups_packages_and_says_how_to_change_a_group() {
    let status = Status {
        arch: "amd64".into(),
        groups: vec![
            group(
                "tools",
                None,
                None,
                vec![
                    package("curl", "tools", PackageState::Installed, Some("8.14.1-2")),
                    package("git", "tools", PackageState::Missing, None),
                ],
            ),
            group(
                "hyprland",
                None,
                None,
                vec![package("hyprland", "hyprland", PackageState::Held, Some("0.56.0-1"))],
            ),
            group(
                "docker",
                Some("off"),
                Some(Switch::On),
                vec![package("docker-ce", "docker", PackageState::Skipped, None)],
            ),
            group(
                "u2f",
                None,
                Some(Switch::On),
                vec![package("libpam-u2f", "u2f", PackageState::Installed, Some("1.4.0-1"))],
            ),
            group(
                "hyprland-ubuntu",
                Some("arm64 only"),
                None,
                vec![package(
                    "hyprlock",
                    "hyprland-ubuntu",
                    PackageState::Installed,
                    Some("0.9"),
                )],
            ),
        ],
        extra: vec![package("old-tool", "tools", PackageState::Extra, Some("2.0"))],
    };

    assert_eq!(
        table(&status),
        "\
KubaOS's apt packages (amd64)

tools: The tools group
  installed   curl        8.14.1-2
  missing     git

hyprland: The hyprland group
  held        hyprland    0.56.0-1

docker: The docker group [off: kubaos packages enable docker]

u2f: The u2f group [on: kubaos packages disable u2f]
  installed   libpam-u2f  1.4.0-1

hyprland-ubuntu: The hyprland-ubuntu group [arm64 only]
  installed   hyprlock    0.9

extra: KubaOS installed them, no group lists them; kubaos update removes them
  extra       old-tool    2.0  (was tools)

1 package missing: kubaos update installs it.
"
    );
}
