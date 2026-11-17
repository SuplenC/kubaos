use super::*;

#[test]
fn a_sources_file_gives_its_uri_suite_components_and_architectures() {
    let text = "# Comment: ignored\nTypes: deb\nURIs: https://example.org/ubuntu\nSuites: resolute\nComponents: main extra\nArchitectures: @ARCH@\nSigned-By: /k.gpg\n";

    let sources = Sources::parse(text).unwrap();

    assert_eq!(sources.uri, "https://example.org/ubuntu");
    assert_eq!(sources.suite, "resolute");
    assert_eq!(sources.components, ["main", "extra"]);
    assert!(sources.serves("arm64"));
    assert_eq!(sources.dists(), "https://example.org/ubuntu/dists/resolute");
}

#[test]
fn a_sources_file_with_one_architecture_serves_only_it() {
    let text = "URIs: https://dl.example.com/deb/\nSuites: stable\nComponents: main\nArchitectures: amd64\n";

    let sources = Sources::parse(text).unwrap();

    assert!(sources.serves("amd64"));
    assert!(!sources.serves("arm64"));
    assert_eq!(sources.dists(), "https://dl.example.com/deb/dists/stable");
}

#[test]
fn a_sources_file_without_a_suite_is_refused() {
    assert!(Sources::parse("URIs: https://example.org\nComponents: main\n").is_err());
}

#[test]
fn a_release_file_lists_the_sha256_section_only() {
    let text = "Origin: X\nMD5Sum:\n 0123 10 main/binary-amd64/Packages\nSHA256:\n ABCD 10 main/binary-amd64/Packages\n eeff 5 main/binary-amd64/Packages.gz\nAcquire-By-Hash: yes\n";

    let release = Release::parse(text);

    assert_eq!(release.sha256.len(), 2);
    assert_eq!(release.sha256["main/binary-amd64/Packages"], "abcd");
    assert_eq!(release.sha256["main/binary-amd64/Packages.gz"], "eeff");
}

#[test]
fn an_index_gives_the_newest_version_of_each_package_in_dpkgs_order() {
    let text = "Package: tool\nVersion: 1.9\n\nPackage: tool\nVersion: 1.10~rc1\n\nPackage: tool\nVersion: 1.10\nArchitecture: amd64\n\nPackage: other\nVersion: 2:0.1\n";

    let found = newest(text);

    assert_eq!(found["tool"], "1.10");
    assert_eq!(found["other"], "2:0.1");
}

#[test]
fn a_listing_gives_the_primary_fingerprints_not_the_subkeys() {
    let listing = "\
pub:-:4096:1:AAAA:1:::-:::scSC::::::23::0:
fpr:::::::::9DC858229FC7DD38854AE2D88D81803C0EBFCD88:
uid:-::::1::X::Docker Release (CE deb) <docker@docker.com>::::::::::0:
sub:-:4096:1:BBBB:1::::::s::::::23:
fpr:::::::::D3306A018370199E527AE7317EA0A9C3F273FCD8:
";

    assert_eq!(
        primary_fingerprints(listing),
        ["9DC858229FC7DD38854AE2D88D81803C0EBFCD88"]
    );
}
