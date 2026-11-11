use std::path::PathBuf;

use super::ListedVersion;
use super::OutdatedTool;
use super::ToolRequest;
use super::parse_list;
use super::parse_outdated;

#[test]
fn a_tool_request_is_tool_at_request() {
    let request = ToolRequest {
        tool: "aqua:astral-sh/uv".into(),
        request: "0.13".into(),
    };

    assert_eq!(request.to_string(), "aqua:astral-sh/uv@0.13");
}

#[test]
fn mise_ls_json_reads_as_one_entry_per_version() {
    let json = br#"{
      "node": [
        {"version": "22.1.0", "install_path": "/x/node/22.1.0", "installed": true, "active": false},
        {"version": "24.22.1", "requested_version": "24.22", "install_path": "/x/node/24.22.1",
         "source": {"type": "mise.toml", "path": "/h/.config/mise/conf.d/kubaos-node.toml"},
         "installed": true, "active": true}
      ],
      "uv": [
        {"version": "0.13.2", "requested_version": "0.13",
         "source": {"type": "mise.toml", "path": "/h/.config/mise/config.toml"}, "installed": false}
      ]
    }"#;

    let listed = parse_list(json).unwrap();

    assert_eq!(
        listed,
        [
            ListedVersion {
                tool: "node".into(),
                version: "22.1.0".into(),
                requested: None,
                source: None,
                installed: true,
            },
            ListedVersion {
                tool: "node".into(),
                version: "24.22.1".into(),
                requested: Some("24.22".into()),
                source: Some(PathBuf::from("/h/.config/mise/conf.d/kubaos-node.toml")),
                installed: true,
            },
            ListedVersion {
                tool: "uv".into(),
                version: "0.13.2".into(),
                requested: Some("0.13".into()),
                source: Some(PathBuf::from("/h/.config/mise/config.toml")),
                installed: false,
            },
        ]
    );
}

#[test]
fn mise_outdated_json_reads_as_one_entry_per_tool() {
    let json = br#"{
      "node": {"name": "node", "requested": "24.22", "current": "24.22.0", "bump": "26.1",
               "latest": "26.1.0",
               "source": {"type": "mise.toml", "path": "/h/.config/mise/conf.d/kubaos-node.toml"}}
    }"#;

    let outdated = parse_outdated(json).unwrap();

    assert_eq!(
        outdated,
        [OutdatedTool {
            tool: "node".into(),
            requested: "24.22".into(),
            current: Some("24.22.0".into()),
            latest: "26.1.0".into(),
            bump: Some("26.1".into()),
            source: Some(PathBuf::from("/h/.config/mise/conf.d/kubaos-node.toml")),
        }]
    );
}

#[test]
fn nothing_installed_reads_as_no_versions() {
    assert_eq!(parse_list(b"{}").unwrap(), []);
    assert_eq!(parse_outdated(b"{}").unwrap(), []);
}
