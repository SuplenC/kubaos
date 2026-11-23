use serde_json::json;

use super::GitHubPin;
use super::HELM_FINGERPRINTS;
use super::HELM_KEYS;
use super::Moved;
use super::Pin;
use super::Tag;
use super::helm_download;
use super::helm_sha256;
use super::helm_signature;
use super::newest_android_studio;
use super::newest_file;
use super::newest_proton_pass;
use super::newest_tag;
use super::percent_decoded;
use crate::apt;

#[test]
fn a_release_download_reads_as_its_repository_tag_and_file() {
    let pin =
        GitHubPin::parse("https://github.com/Kong/insomnia/releases/download/core%4013.4.0/Insomnia.Core-13.4.0.deb");

    assert_eq!(
        pin,
        Some(GitHubPin {
            owner: "Kong",
            repo: "insomnia",
            tag: "core%4013.4.0",
            name: "Insomnia.Core-13.4.0.deb",
        })
    );
    assert_eq!(
        GitHubPin::parse("https://github.com/Kong/insomnia/archive/v1.tar.gz"),
        None
    );
    assert_eq!(GitHubPin::parse("https://proton.me/download/pass/linux/x.deb"), None);
}

#[test]
fn a_tag_splits_at_its_first_digit_and_its_escapes_decode() {
    assert_eq!(
        Tag::parse(&percent_decoded("core%4013.4.0")).unwrap(),
        Tag {
            prefix: "core@".into(),
            version: "13.4.0".into(),
        }
    );
    assert_eq!(Tag::parse("desktop-v2026.11.0").unwrap().prefix, "desktop-v");
    assert_eq!(Tag::parse("1.24.2b").unwrap().prefix, "");
    assert!(Tag::parse("nightly").is_err());
}

#[test]
fn the_newest_tag_skips_drafts_prereleases_and_other_prefixes() {
    let releases = json!([
        {"tag_name": "desktop-v2026.12.0", "draft": true},
        {"tag_name": "desktop-v2026.11.1", "prerelease": true},
        {"tag_name": "cli-v2026.12.0"},
        {"tag_name": "desktop-v2026.11.0"},
        {"tag_name": "desktop-v2026.9.10"},
        {"tag_name": "desktop-vnext"}
    ]);

    assert_eq!(newest_tag(&releases, "desktop-v").as_deref(), Some("2026.11.0"));
    assert_eq!(newest_tag(&releases, "web-v"), None);
}

#[test]
fn versions_compare_as_dpkg_orders_them() {
    let releases = json!([{"tag_name": "1.24.10b"}, {"tag_name": "1.24.9b"}]);

    assert_eq!(newest_tag(&releases, "").as_deref(), Some("1.24.10b"));
}

#[test]
fn a_pin_with_a_suffix_of_its_own_keeps_it_and_its_file_follows() {
    let moved = Moved::from("2026.914.233613-1+ubuntu26.04", "2026.914.233613", "2026.1101.12")
        .unwrap()
        .unwrap();

    assert_eq!(moved.to, "2026.1101.12-1+ubuntu26.04");
    assert_eq!(
        moved.rename("sunshine_2026.914.233613-1+ubuntu26.04_amd64.deb"),
        "sunshine_2026.1101.12-1+ubuntu26.04_amd64.deb"
    );
}

#[test]
fn a_file_named_by_the_release_alone_or_by_nothing_follows_the_release() {
    let moved = Moved::from("0.40.16", "0.40.16", "0.40.17").unwrap().unwrap();

    assert_eq!(moved.rename("T3-Code-0.40.16-amd64.deb"), "T3-Code-0.40.17-amd64.deb");
    assert_eq!(moved.rename("ollama-linux-amd64.tar.zst"), "ollama-linux-amd64.tar.zst");
}

#[test]
fn no_newer_release_is_no_move() {
    assert_eq!(Moved::from("2.0.2", "2.0.2", "2.0.2"), None);
    assert_eq!(Moved::from("2.0.2", "2.0.2", "2.0.1"), None);
}

#[test]
fn a_pin_whose_version_isnt_its_releases_is_an_error() {
    assert!(Moved::from("5.0", "5.1", "5.2").unwrap().is_err());
}

#[test]
fn android_studios_newest_is_its_newest_release_or_patch() {
    let list = json!({"content": {"item": [
        {"version": "2026.3.1.1", "channel": "Canary"},
        {"version": "2026.2.2.10", "channel": "Patch"},
        {"version": "2026.2.2.8", "channel": "Release"},
        {"version": "2026.2.2.7", "channel": "RC"}
    ]}});

    assert_eq!(newest_android_studio(&list).unwrap()["version"], "2026.2.2.10");
}

#[test]
fn proton_pass_waits_for_a_release_out_to_everyone() {
    let feed = json!({"Releases": [
        {"CategoryName": "Stable", "Version": "1.44.1", "RolloutPercentage": 0.05,
         "File": [{"Url": "https://proton.me/download/pass/linux/proton-pass_1.44.1_amd64.deb"}]},
        {"CategoryName": "Beta", "Version": "1.45.0", "RolloutPercentage": 1,
         "File": [{"Url": "https://proton.me/download/pass/linux/proton-pass_1.45.0_amd64.deb"}]},
        {"CategoryName": "Stable", "Version": "1.44.0", "RolloutPercentage": 1,
         "File": [
           {"Url": "https://proton.me/download/pass/linux/proton-pass-1.44.0-1.x86_64.rpm"},
           {"Url": "https://proton.me/download/pass/linux/proton-pass_1.44.0_amd64.deb"}
         ]}
    ]});

    assert_eq!(
        newest_proton_pass(&feed),
        Some((
            "1.44.0".to_owned(),
            "https://proton.me/download/pass/linux/proton-pass_1.44.0_amd64.deb".to_owned()
        ))
    );
}

#[test]
fn a_folder_listings_newest_file_of_the_pins_shape() {
    let listing = r#"<a href="../">../</a>
<a href="blender-5.3.0-linux-x64.tar.xz">blender-5.3.0-linux-x64.tar.xz</a>
<a href="blender-5.3.1-linux-x64.tar.xz">blender-5.3.1-linux-x64.tar.xz</a>
<a href="blender-5.3.1-macos-arm64.dmg">blender-5.3.1-macos-arm64.dmg</a>
<a href="blender-5.3.1.sha256">blender-5.3.1.sha256</a>"#;

    assert_eq!(
        newest_file(listing, "blender-", "-linux-x64.tar.xz").as_deref(),
        Some("5.3.1")
    );
    assert_eq!(newest_file(listing, "blender-", "-windows-x64.zip"), None);
}

#[test]
fn helms_newer_release_is_its_tarball_on_get_helm_sh_and_an_older_one_no_move() {
    let pin = Pin {
        version: "4.3.2",
        arch: "arm64",
        url: "https://get.helm.sh/helm-v4.3.2-linux-arm64.tar.gz",
        sha256: "",
    };

    assert_eq!(
        helm_download(&pin, "4.4.0"),
        Some(Ok("https://get.helm.sh/helm-v4.4.0-linux-arm64.tar.gz".to_owned()))
    );
    assert_eq!(helm_download(&pin, "4.3.2"), None);
    assert_eq!(helm_download(&pin, "3.22.0"), None, "Helm 3's patches are no move");
}

/// Helm 4.3.2's amd64 `.sha256sum` and its signature as the release carries it, made with one of
/// Helm's keys; the same file signed with a throwaway key of no one's.
const HELM_NAME: &str = "helm-v4.3.2-linux-amd64.tar.gz";
const HELM_SUMS: &[u8] = include_bytes!("fixtures/helm-v4.3.2-linux-amd64.tar.gz.sha256sum");
const HELM_SIGNATURE: &[u8] = include_bytes!("fixtures/helm-v4.3.2-linux-amd64.tar.gz.sha256sum.asc");
const OTHER_SIGNATURE: &[u8] = include_bytes!("fixtures/helm-v4.3.2-linux-amd64.tar.gz.sha256sum.other.asc");

#[test]
fn helms_key_file_holds_exactly_the_pinned_keys() {
    let mut found = apt::fingerprints(HELM_KEYS).unwrap();
    let mut pinned = HELM_FINGERPRINTS.map(str::to_owned).to_vec();
    found.sort();
    pinned.sort();

    assert_eq!(found, pinned);
}

#[test]
fn a_helm_checksum_signed_by_helms_key_gives_its_sha256() {
    assert_eq!(
        helm_sha256(HELM_SUMS, Some(HELM_SIGNATURE), HELM_NAME).as_deref(),
        Ok("d1c0278d46beab79582d7e50f5b3dc206944ba8e577cde618513fd07daa53983")
    );
}

#[test]
fn a_helm_checksum_changed_after_signing_is_refused() {
    let tampered = String::from_utf8_lossy(HELM_SUMS).replacen("d1c0", "d1c1", 1);

    let refused = helm_sha256(tampered.as_bytes(), Some(HELM_SIGNATURE), HELM_NAME).unwrap_err();

    assert!(refused.contains("didn't verify"), "{refused}");
}

#[test]
fn a_helm_checksum_signed_by_a_key_helm_doesnt_publish_is_refused() {
    let refused = helm_sha256(HELM_SUMS, Some(OTHER_SIGNATURE), HELM_NAME).unwrap_err();

    assert!(refused.contains("didn't verify"), "{refused}");
}

#[test]
fn a_helm_checksum_with_no_signature_is_refused() {
    let refused = helm_sha256(HELM_SUMS, None, HELM_NAME).unwrap_err();

    assert!(refused.contains("has no"), "{refused}");
}

#[test]
fn helms_signature_is_the_releases_asset_beside_the_checksum() {
    let releases = json!([
        {"tag_name": "v4.4.0", "assets": [
            {"name": "helm-v4.4.0-linux-amd64.tar.gz.sha256sum.asc", "browser_download_url": "https://example.org/amd64.asc"},
            {"name": "helm-v4.4.0-linux-arm64.tar.gz.asc", "browser_download_url": "https://example.org/arm64-tarball.asc"}
        ]},
        {"tag_name": "v4.3.2", "assets": []}
    ]);

    assert_eq!(
        helm_signature(&releases, "4.4.0", "helm-v4.4.0-linux-amd64.tar.gz"),
        Some("https://example.org/amd64.asc")
    );
    assert_eq!(
        helm_signature(&releases, "4.4.0", "helm-v4.4.0-linux-arm64.tar.gz"),
        None,
        "the tarball's own signature isn't the checksum's"
    );
    assert_eq!(
        helm_signature(&releases, "4.5.0", "helm-v4.5.0-linux-amd64.tar.gz"),
        None
    );
}
