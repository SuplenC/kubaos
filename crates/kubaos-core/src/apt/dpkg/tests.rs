use super::Dpkg;
use super::candidates;
use super::removals;

#[test]
fn installed_and_held_packages_come_from_the_query() {
    let dpkg = Dpkg::parse(
        "installed install curl 8.14.1-2\n\
         installed hold hyprland 0.56.0-1\n\
         config-files deinstall vim 2:9.1\n\
         not-installed hold hyprlock \n\
         installed install libc6 2.42-0\n\
         not-installed unknown libc6 \n",
    );

    assert_eq!(dpkg.get("curl").unwrap().version, "8.14.1-2");
    assert!(!dpkg.get("curl").unwrap().held);
    assert!(dpkg.get("hyprland").unwrap().held);
    assert!(!dpkg.has("vim"));
    assert!(dpkg.has("libc6"));
    assert!(dpkg.held_missing("hyprlock"));
    assert!(!dpkg.held_missing("hyprland"));
}

#[test]
fn candidates_are_the_packages_apt_cache_policy_can_install() {
    let policy = "\
docker-ce:
  Installed: (none)
  Candidate: 5:28.0.1-1~ubuntu.26.04~resolute
  Version table:
     5:28.0.1-1~ubuntu.26.04~resolute 500
hyprland-guiutils:
  Installed: (none)
  Candidate: (none)
  Version table:
curl:
  Installed: 8.14.1-2
  Candidate: 8.14.1-2
";

    let found = candidates(policy);

    assert_eq!(found.into_iter().collect::<Vec<_>>(), ["curl", "docker-ce"]);
}

#[test]
fn removals_are_the_simulations_remv_lines() {
    let simulated = "\
NOTE: This is only a simulation!
Remv gnome-sushi [46.0-1]
Remv nautilus:amd64 [49.0-1] []
Inst something [1]
";

    assert_eq!(
        removals(simulated).into_iter().collect::<Vec<_>>(),
        ["gnome-sushi", "nautilus"]
    );
}
