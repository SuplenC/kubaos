use super::DigestError;
use super::Release;
use super::check_digest;
use super::choose;
use super::installed_version;
use super::parse_releases;
use crate::update::version::Version;

const DIGEST: &str = "02cb6bc6a429d8d0da9fdf9d23739358165cf7b98a245783d6f641526f75b0a7";

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

fn release(version: &str) -> Release {
    Release {
        version: v(version),
        digest: Some(format!("sha256:{DIGEST}")),
    }
}

/// GitHub's release list, as the API returns it, cut to the fields the phase reads.
const RELEASES: &str = r#"[
  {"tag_name": "desktop-v2027.1.0", "draft": false, "prerelease": false,
   "assets": [{"name": "Bitwarden-2027.1.0-amd64.deb", "digest": "sha256:aa"}]},
  {"tag_name": "desktop-v2026.12.1", "draft": false, "prerelease": true,
   "assets": [{"name": "Bitwarden-2026.12.1-amd64.deb", "digest": "sha256:bb"}]},
  {"tag_name": "desktop-v2026.12.2", "draft": true, "prerelease": false, "assets": []},
  {"tag_name": "cli-v2026.12.0", "draft": false, "prerelease": false, "assets": []},
  {"tag_name": "desktop-v2026.12.0", "draft": false, "prerelease": false,
   "assets": [
     {"name": "Bitwarden-2026.12.0-x86_64.AppImage", "digest": "sha256:cc"},
     {"name": "Bitwarden-2026.12.0-amd64.deb", "digest": "sha256:dd"}
   ]},
  {"tag_name": "desktop-v2026.11.0", "draft": false, "prerelease": false,
   "assets": [{"name": "Bitwarden-2026.11.0-amd64.deb", "digest": null}]},
  {"tag_name": "web-v2026.12.0", "draft": false, "prerelease": false, "assets": []}
]"#;

#[test]
fn the_list_keeps_published_desktop_releases_with_their_debs_digest() {
    let releases = parse_releases(RELEASES).unwrap();

    assert_eq!(
        releases,
        [
            Release {
                version: v("2027.1.0"),
                digest: Some("sha256:aa".into()),
            },
            Release {
                version: v("2026.12.0"),
                digest: Some("sha256:dd".into()),
            },
            Release {
                version: v("2026.11.0"),
                digest: None,
            },
        ]
    );
}

#[test]
fn text_that_is_no_release_list_is_an_error() {
    assert!(parse_releases(r#"{"message": "API rate limit exceeded"}"#).is_err());
}

#[test]
fn the_update_is_the_newest_newer_release_of_the_same_series() {
    let releases = [release("2026.11.0"), release("2026.12.0"), release("2026.10.1")];

    let choice = choose(&v("2026.10.1"), &releases);

    assert_eq!(choice.update.unwrap().version, v("2026.12.0"));
    assert_eq!(choice.waiting, None);
}

#[test]
fn a_new_series_is_never_the_update_and_waits() {
    let releases = [release("2027.1.0"), release("2027.2.0"), release("2026.10.1")];

    let choice = choose(&v("2026.10.1"), &releases);

    assert_eq!(choice.update, None);
    assert_eq!(choice.waiting, Some(v("2027.2.0")));
}

#[test]
fn nothing_older_or_equal_is_an_update() {
    let releases = [release("2026.9.0"), release("2026.10.1")];

    assert_eq!(choose(&v("2026.10.1"), &releases).update, None);
}

#[test]
fn a_prerelease_is_never_chosen() {
    let releases = parse_releases(RELEASES).unwrap();

    let choice = choose(&v("2026.12.0"), &releases);

    assert_eq!(choice.update, None);
    assert_eq!(choice.waiting, Some(v("2027.1.0")));
}

#[test]
fn the_download_url_is_the_desktop_tags_deb() {
    assert_eq!(
        release("2026.12.0").url(),
        "https://github.com/bitwarden/clients/releases/download/desktop-v2026.12.0/Bitwarden-2026.12.0-amd64.deb"
    );
}

#[test]
fn the_installed_version_is_dpkgs_or_none() {
    assert_eq!(installed_version("2026.10.1"), Some(v("2026.10.1")));
    assert_eq!(installed_version(""), None);
    assert_eq!(installed_version("installed hyprland\n"), None);
}

#[test]
fn a_download_with_the_published_digest_passes() {
    let published = format!("sha256:{DIGEST}");
    let sha256sum = format!("{}  /tmp/Bitwarden-2026.12.0-amd64.deb\n", DIGEST.to_uppercase());

    assert_eq!(check_digest(Some(&published), &sha256sum), Ok(()));
}

#[test]
fn a_download_with_another_digest_fails() {
    let published = format!("sha256:{DIGEST}");
    let other = "f".repeat(64);

    assert_eq!(
        check_digest(Some(&published), &format!("{other}  bitwarden.deb")),
        Err(DigestError::Mismatch {
            published: DIGEST.into(),
            actual: other,
        })
    );
}

#[test]
fn a_release_without_a_sha256_digest_fails() {
    let sha256sum = format!("{DIGEST}  bitwarden.deb");

    assert_eq!(check_digest(None, &sha256sum), Err(DigestError::Missing));
    assert_eq!(
        check_digest(Some(&format!("sha512:{DIGEST}")), &sha256sum),
        Err(DigestError::Missing)
    );
    assert_eq!(check_digest(Some("sha256:"), &sha256sum), Err(DigestError::Missing));
}

#[test]
fn sha256sum_output_without_a_digest_fails() {
    let published = format!("sha256:{DIGEST}");

    assert!(matches!(
        check_digest(Some(&published), ""),
        Err(DigestError::Unreadable(_))
    ));
}
