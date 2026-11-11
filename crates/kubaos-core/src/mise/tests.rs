use std::path::PathBuf;

use super::ListedVersion;
use super::MiseError;
use super::OutdatedTool;
use super::ToolRequest;
use super::is_rate_limited;
use super::last_line;
use super::parse_list;
use super::parse_outdated;
use super::resets_at;

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

#[test]
fn a_failure_is_the_line_before_the_version_and_the_hint_mise_ends_with() {
    let stderr = "mise ERROR error parsing config file: ~/p/mise.toml\n\
                  mise ERROR Config files in ~/p/mise.toml are not trusted.\n\
                  Trust them with `mise trust`. See https://mise.jdx.dev/cli/trust.html for more information.\n\
                  mise ERROR Version: 2026.11.6 linux-x64 (1980-01-01)\n\
                  mise ERROR Run with --verbose or MISE_VERBOSE=1 for more information\n";

    assert_eq!(
        last_line(stderr),
        "Trust them with `mise trust`. See https://mise.jdx.dev/cli/trust.html for more information."
    );
    assert_eq!(
        last_line("mise ERROR no version of x\n\n"),
        "mise ERROR no version of x"
    );
}

#[test]
fn a_rate_limited_run_reads_when_the_limit_resets_from_mise_s_warning() {
    let log = "mise WARN  GitHub rate limit exceeded. Resets at 2026-11-11 13:36:35 +01:00\n\
               mise WARN  HTTP GET https://api.github.com/repos/o/r attempt 1 failed\n\
               github rate limit: 0/60 (core), resets at 1794400595\n";

    assert!(is_rate_limited(log));
    assert_eq!(resets_at(log).as_deref(), Some("2026-11-11 13:36:35 +01:00"));
    assert!(!is_rate_limited("mise ERROR no version of demo-lang\n"));
}

#[test]
fn a_rate_limit_says_when_it_resets_and_how_a_token_lifts_it() {
    let err = MiseError::RateLimited {
        command: "lock --global".into(),
        resets: Some("2026-11-11 13:36:35 +01:00".into()),
    };

    let text = err.to_string();

    assert!(text.contains("resets at 2026-11-11 13:36:35 +01:00"), "{text}");
    assert!(text.contains("GITHUB_TOKEN=$(gh auth token)"), "{text}");
}
