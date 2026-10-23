use super::*;

#[test]
fn the_json_is_what_the_shell_reads() {
    let state = State {
        agent: true,
        prompt: Some(Prompt {
            id: 3,
            kind: Kind::DisplayPasskey,
            device: "/org/bluez/hci0/dev_AA".into(),
            name: "K380".into(),
            code: Some("012345".into()),
            entered: Some(2),
        }),
        pairing: BTreeSet::from(["/org/bluez/hci0/dev_AA".to_owned()]),
        errors: BTreeMap::from([("/org/bluez/hci0/dev_BB".to_owned(), "Timed out".to_owned())]),
    };
    let json: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "agent": true,
            "prompt": {
                "id": 3,
                "kind": "display-passkey",
                "device": "/org/bluez/hci0/dev_AA",
                "name": "K380",
                "code": "012345",
                "entered": 2,
            },
            "pairing": ["/org/bluez/hci0/dev_AA"],
            "errors": {"/org/bluez/hci0/dev_BB": "Timed out"},
        })
    );
}

#[test]
fn a_write_replaces_the_file_whole() {
    let dir = tempfile::tempdir().unwrap();
    let file = StateFile::new(dir.path().join(FILE_NAME));
    file.write(&State {
        agent: true,
        ..State::default()
    })
    .expect("write the first state");
    file.write(&State::default())
        .expect("write the second state");

    let text = fs::read_to_string(dir.path().join(FILE_NAME)).unwrap();
    assert_eq!(
        text,
        r#"{"agent":false,"prompt":null,"pairing":[],"errors":{}}"#
    );
    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        1,
        "no partial file is left behind"
    );
}
