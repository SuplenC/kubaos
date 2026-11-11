use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;

use super::Pending;
use super::UPDATE;
use super::UPGRADE;

fn item(id: &str, title: &str) -> Item {
    Item {
        id: id.to_owned(),
        kind: ItemKind::Namespaced(NamespacedKind::new("kubaos.action").expect("a valid kind")),
        identity: None,
        title: title.to_owned(),
        subtitle: Some("Default".to_owned()),
        icon: None,
        keyword: None,
        keywords: Vec::new(),
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        yields: false,
        actions: Vec::new(),
    }
}

#[test]
fn a_pending_update_shows_in_the_update_subtitle() {
    let pending = Pending::parse("update=main at 1a2b3c4 (0.1.1)\nupdate-commit=1a2b3c4d\n");

    let update = pending.apply(item(UPDATE, "Update KubaOS")).expect("the update Item");

    assert_eq!(
        update.subtitle.as_deref(),
        Some("KubaOS main at 1a2b3c4 (0.1.1) is ready. Update when you feel like it")
    );
}

#[test]
fn with_nothing_pending_the_update_keeps_its_subtitle_and_upgrade_is_not_listed() {
    let pending = Pending::parse("");

    assert_eq!(
        pending
            .apply(item(UPDATE, "Update KubaOS"))
            .and_then(|i| i.subtitle)
            .as_deref(),
        Some("Default")
    );
    assert!(pending.apply(item(UPGRADE, "Upgrade KubaOS")).is_none());
}

#[test]
fn a_next_series_lists_upgrade_by_its_codename() {
    let pending = Pending::parse("upgrade=0.2.0\nupgrade-commit=5e6f\nupgrade-codename=akihabara\n");

    let upgrade = pending
        .apply(item(UPGRADE, "Upgrade KubaOS"))
        .expect("the upgrade Item");

    assert_eq!(upgrade.title, "Upgrade to Akihabara");
    assert_eq!(upgrade.subtitle.as_deref(), Some("The next KubaOS release, 0.2.0"));
}

#[test]
fn other_items_pass_through() {
    let pending = Pending::parse("update=0.1.2\n");

    assert_eq!(
        pending.apply(item("theme-next", "Next")),
        Some(item("theme-next", "Next"))
    );
}

#[test]
fn a_missing_file_is_nothing_pending() {
    let dir = tempfile::tempdir().expect("a temporary folder");

    assert_eq!(Pending::read(&dir.path().join("update-available")), Pending::default());
}

#[test]
fn pending_apt_security_updates_show_in_the_update_subtitle_with_or_without_a_release() {
    let both = Pending::parse("update=0.1.1\nupdate-commit=aa\napt-security=2\napt-security-packages=a=1,b=2\n");
    let only = Pending::parse("apt-security=1\napt-security-packages=a=1\n");

    let with_release = both.apply(item(UPDATE, "Update KubaOS")).and_then(|i| i.subtitle);
    let alone = only.apply(item(UPDATE, "Update KubaOS")).and_then(|i| i.subtitle);

    assert_eq!(
        with_release.as_deref(),
        Some("KubaOS 0.1.1 is ready, 2 apt security updates waiting")
    );
    assert_eq!(
        alone.as_deref(),
        Some("Packages are due: 1 apt security update waiting")
    );
}
