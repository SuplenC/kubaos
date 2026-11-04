use super::fallback_paths_url;
use super::installed_version;
use super::listing_url;
use super::newest_patch;
use super::parse_listing;
use super::store_path;
use super::system_of;
use crate::update::version::Version;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

/// The S3 listing of a minor, as the bucket returns it.
const LISTING: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Name>nix-releases</Name><Prefix>nix/nix-2.35.</Prefix><Marker></Marker><MaxKeys>1000</MaxKeys><Delimiter>/</Delimiter><IsTruncated>false</IsTruncated><CommonPrefixes><Prefix>nix/nix-2.35.0/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>nix/nix-2.35.1/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>nix/nix-2.35.10/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>nix/nix-2.35.4/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>nix/nix-2.35.11pre20261101_0123abc/</Prefix></CommonPrefixes></ListBucketResult>"#;

const FALLBACK_PATHS: &str = r#"{
  x86_64-linux = "/nix/store/jsx1mm71yz1n5k4zybqlsmajp416c5iy-nix-2.35.4";
  i686-linux = "/nix/store/4wnfc3l7hgx29bv4v9bl98dziicmmq0r-nix-2.35.4";
  aarch64-linux = "/nix/store/yv1zf3y6all4j99aad30cb8i55dw9zmp-nix-2.35.4";
}
"#;

#[test]
fn the_installed_version_is_the_last_word_of_nix_version() {
    assert_eq!(installed_version("nix (Nix) 2.35.3\n"), Some(v("2.35.3")));
    assert_eq!(installed_version(""), None);
    assert_eq!(installed_version("nix (Nix) 2.36.0pre20261101_0123abc"), None);
}

#[test]
fn the_listing_names_the_releases_and_leaves_out_prereleases() {
    assert_eq!(
        parse_listing(LISTING),
        [v("2.35.0"), v("2.35.1"), v("2.35.10"), v("2.35.4")]
    );
}

#[test]
fn the_update_is_the_newest_patch_of_the_installed_minor() {
    let releases = parse_listing(LISTING);

    assert_eq!(newest_patch(&v("2.35.3"), &releases), Some(v("2.35.10")));
}

#[test]
fn a_new_minor_is_never_the_update() {
    let releases = [v("2.36.0"), v("3.0.0"), v("2.35.3")];

    assert_eq!(newest_patch(&v("2.35.3"), &releases), None);
}

#[test]
fn nothing_older_or_equal_is_an_update() {
    assert_eq!(newest_patch(&v("2.35.10"), &parse_listing(LISTING)), None);
}

#[test]
fn the_urls_name_the_minor_and_the_release() {
    assert_eq!(
        listing_url(2, 35),
        "https://nix-releases.s3.amazonaws.com/?delimiter=/&prefix=nix/nix-2.35."
    );
    assert_eq!(
        fallback_paths_url(&v("2.35.4")),
        "https://releases.nixos.org/nix/nix-2.35.4/fallback-paths.nix"
    );
}

#[test]
fn the_store_path_is_the_systems_and_must_be_the_releases() {
    assert_eq!(
        store_path(FALLBACK_PATHS, "x86_64-linux", &v("2.35.4")).as_deref(),
        Some("/nix/store/jsx1mm71yz1n5k4zybqlsmajp416c5iy-nix-2.35.4")
    );
    assert_eq!(store_path(FALLBACK_PATHS, "x86_64-linux", &v("2.35.5")), None);
    assert_eq!(store_path(FALLBACK_PATHS, "riscv64-linux", &v("2.35.4")), None);
    assert_eq!(store_path("<html>Not found</html>", "x86_64-linux", &v("2.35.4")), None);
}

#[test]
fn a_debian_architecture_has_its_nix_system() {
    assert_eq!(system_of("amd64\n"), Some("x86_64-linux"));
    assert_eq!(system_of("arm64"), Some("aarch64-linux"));
    assert_eq!(system_of("riscv64"), None);
}
