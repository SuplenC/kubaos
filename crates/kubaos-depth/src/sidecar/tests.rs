use super::*;

#[test]
fn a_sidecar_reads_back_as_written() {
    let dir = tempfile::tempdir().unwrap();
    let written = Sidecar::new(
        Some(Method::Object),
        0.912_34,
        Place { x: 0.445, y: 0.47 },
        "#9fd0ff".to_owned(),
    );

    written.write(dir.path()).unwrap();

    assert_eq!(Sidecar::read(dir.path()).unwrap(), written);
}

#[test]
fn the_sidecar_is_the_documented_toml() {
    let dir = tempfile::tempdir().unwrap();
    let sidecar = Sidecar::new(None, 0.4, Place { x: 0.5, y: 0.56 }, "#cacaca".to_owned());

    sidecar.write(dir.path()).unwrap();

    let text = std::fs::read_to_string(dir.path().join(SIDECAR_FILE)).unwrap();
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        text,
        format!(
            "format = 1\ntool = \"{version}\"\nmethod = \"none\"\nscore = 0.4\n\n[clock]\nx = 0.5\ny = 0.56\ntint = \"#cacaca\"\n"
        )
    );
}

#[test]
fn a_sidecar_of_another_format_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(SIDECAR_FILE),
        "format = 2\ntool = \"9\"\nmethod = \"sky\"\nscore = 1.0\n\n[clock]\nx = 0.5\ny = 0.5\ntint = \"#ffffff\"\n",
    )
    .unwrap();

    assert!(matches!(Sidecar::read(dir.path()), Err(ReadError::Format(2))));
}

#[test]
fn places_are_kept_to_four_places() {
    let sidecar = Sidecar::new(
        Some(Method::Sky),
        0.123_456,
        Place { x: 0.123_456, y: 0.5 },
        String::new(),
    );

    assert!((sidecar.clock.x - 0.1235).abs() < 1e-7);
    assert!((sidecar.score - 0.1235).abs() < 1e-7);
}
