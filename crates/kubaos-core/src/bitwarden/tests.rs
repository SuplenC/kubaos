use std::fmt::Write;
use std::path::Path;

use serde_json::json;

use super::fake::Fake;
use super::fake::PASSWORD;
use super::fake::SESSION;
use super::*;

const HOME: &str = "/home/kuba";
const EU: &str = "https://vault.bitwarden.eu";
const CONFIG: &str = "/home/kuba/.config/dot-bw/config";
const SSH_CONFIG: &str = "/home/kuba/.ssh/config.d/bitwarden";
const SIGNING: &str = "/home/kuba/.config/git-bitwarden/signing.gitconfig";
const SIGNERS: &str = "/home/kuba/.config/git-bitwarden/allowed_signers";
const AGENT: &str = "/home/kuba/.bitwarden-ssh-agent.sock";
const PEM: &str = "-----BEGIN PRIVATE KEY-----\nMIIBmarker9d2e\n-----END PRIVATE KEY-----";

fn paths() -> Paths {
    Paths::under(Path::new(HOME))
}

fn ssh_item(name: &str, slug: &str, folder: Option<&str>, created: u8, fields: serde_json::Value) -> serde_json::Value {
    let mut item = json!({
        "id": format!("i-{slug}"),
        "type": 5,
        "name": name,
        "folderId": folder,
        "creationDate": format!("2026-01-01T00:00:{created:02}.000Z"),
        "notes": null,
        "sshKey": {
            "privateKey": "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjE\n-----END OPENSSH PRIVATE KEY-----",
            "publicKey": format!("ssh-ed25519 AAAA{slug}"),
            "keyFingerprint": format!("SHA256:{slug}"),
        },
    });
    item["fields"] = fields;
    item
}

fn field(name: &str, value: &str) -> serde_json::Value {
    json!({ "name": name, "value": value, "type": 0 })
}

fn folders() -> String {
    json!([
        { "id": "f-personal", "name": "Personal" },
        { "id": "f-work", "name": "Work" },
        { "id": "f-env", "name": "env/demo" },
        { "id": null, "name": "No Folder" },
    ])
    .to_string()
}

/// The harness's vault: five SSH keys in two folders and none, two named GitHub, one hostile;
/// a project env in `env/demo` and a `bwenv` item elsewhere; a gh token.
fn items() -> Vec<serde_json::Value> {
    vec![
        ssh_item(
            "GitHub",
            "github",
            Some("f-personal"),
            1,
            json!([field("hosts", "github.com"), field("user", "git"), { "name": "signing", "value": "true", "type": 2 }]),
        ),
        ssh_item(
            "Work GitHub",
            "work",
            Some("f-work"),
            2,
            json!([
                field("hosts", "github-work"),
                field("hostname", "github.com"),
                field("user", "git")
            ]),
        ),
        ssh_item(
            "Build server",
            "build",
            None,
            3,
            json!([
                field("hosts", "build build.example.com"),
                field("hostname", "192.0.2.10"),
                field("user", "deploy"),
                field("port", "2022"),
            ]),
        ),
        ssh_item("GitHub", "dup", Some("f-work"), 4, json!([])),
        ssh_item(
            "Evil",
            "evil",
            Some("f-work"),
            5,
            json!([field("hosts", "evil\n  ProxyCommand touch /tmp/pwned-ssh")]),
        ),
        json!({
            "id": "i-demo", "type": 2, "name": "demo env", "folderId": "f-env", "notes": null,
            "fields": [
                { "name": "DATABASE_URL", "value": "postgres://demo:pw@db:5432/demo", "type": 1 },
                { "name": "API_TOKEN", "value": "tok-7c1f", "type": 1 },
                { "name": "TLS_KEY", "value": PEM, "type": 1 },
                { "name": "bad-name", "value": "x", "type": 0 },
            ],
        }),
        json!({
            "id": "i-shared", "type": 2, "name": "shared web", "folderId": null, "notes": null,
            "fields": [
                { "name": "bwenv", "value": "demo", "type": 0 },
                { "name": "EXTRA_VAR", "value": "extra-value", "type": 1 },
                { "name": "LINKED", "value": null, "type": 3, "linkedId": 100 },
            ],
        }),
        json!({
            "id": "i-gh", "type": 1, "name": "gh-token", "folderId": null,
            "login": { "username": "kuba", "password": "tok-gh-99" },
            "fields": [field("host", "github.com")],
        }),
    ]
}

fn agent_lists(keys: &[&str]) -> String {
    keys.iter().fold(String::new(), |mut out, k| {
        let _ = writeln!(out, "256 SHA256:{k} {k} (ED25519)");
        out
    })
}

/// Logged in and locked on the EU cloud, with the harness's vault, the desktop agent serving
/// every key, git and gh answering.
fn machine() -> Fake {
    let mut m = Fake::new()
        .locked(EU, "k@x.eu")
        .vault(&folders(), &json!(items()).to_string())
        .file(CONFIG, "server=https://vault.bitwarden.eu\nemail=k@x.eu\n")
        .answer(
            "timeout 20 ssh-add -l -E sha256",
            0,
            &agent_lists(&["github", "work", "build", "dup", "evil"]),
        )
        .answer("git -C / config --get user.email", 0, "k@x.eu\n")
        .answer(
            "git -C / config --get-all include.path",
            0,
            "~/.config/git-bitwarden/signing.gitconfig\n",
        )
        .answer("gh auth token", 1, "")
        .answer("gh auth login", 0, "")
        .answer(
            "gh auth status",
            0,
            "github.com\n  ✓ Logged in to github.com account kuba (keyring)\n",
        );
    m.sockets.insert(AGENT.into());
    m
}

fn with_items(m: Fake, items: &[serde_json::Value]) -> Fake {
    let folders = m.folders.clone();
    m.vault(&folders, &json!(items).to_string())
}

/// What a run printed, as one text, and its report.
struct Out {
    lines: Vec<Line>,
    report: Report,
}

impl Out {
    fn text(&self) -> String {
        let mut s = String::new();
        for l in &self.lines {
            let (Line::Step(t) | Line::Ok(t) | Line::Info(t) | Line::Warn(t)) = l;
            s.push_str(t);
            s.push('\n');
        }
        for l in &self.report.summary {
            s.push_str(l);
            s.push('\n');
        }
        s
    }

    fn warned(&self, part: &str) -> bool {
        self.report.warnings.iter().any(|w| w.contains(part))
    }
}

fn run(f: impl FnOnce(&mut Progress) -> Result<(), Error>) -> (Out, Result<(), Error>) {
    let mut lines = vec![];
    let mut emit = |l: &Line| lines.push(l.clone());
    let mut p = Progress::new(&mut emit);
    let r = f(&mut p);
    let report = p.finish();
    (Out { lines, report }, r)
}

fn pull_parts(m: &mut Fake, parts: Parts) -> Out {
    let (out, r) = run(|p| pull(m, &paths(), &Session::new(SESSION), parts, p));
    r.expect("pull");
    out
}

fn pull_all(m: &mut Fake) -> Out {
    pull_parts(m, Parts::ALL)
}

fn ssh_only() -> Parts {
    Parts::parse("ssh").unwrap()
}

fn host_blocks(m: &Fake) -> usize {
    m.text(SSH_CONFIG)
        .unwrap()
        .lines()
        .filter(|l| l.starts_with("Host "))
        .count()
}

fn public_key(m: &Fake, file: &str) -> Option<String> {
    m.text(&format!("{HOME}/.ssh/{file}"))
        .map(|s| s.split(' ').take(2).collect::<Vec<_>>().join(" "))
}

// ------------------------------------------------------------------ ssh

#[test]
fn a_pull_takes_every_ssh_key_in_the_vault_whatever_its_folder() {
    let mut m = machine();

    let out = pull_all(&mut m);

    assert!(
        out.text().contains("8 items, 5 SSH key(s) in your vault"),
        "{}",
        out.text()
    );
    for (file, key) in [
        ("github.pub", "github"),
        ("github-2.pub", "dup"),
        ("work-github.pub", "work"),
        ("build-server.pub", "build"),
        ("evil.pub", "evil"),
    ] {
        assert_eq!(public_key(&m, file), Some(format!("ssh-ed25519 AAAA{key}")), "{file}");
    }
    assert_eq!(
        m.text("/home/kuba/.ssh/github.pub"),
        Some("ssh-ed25519 AAAAgithub bitwarden:GitHub\n")
    );
    assert_eq!(m.mode(SSH_CONFIG), Some(0o600));
    assert_eq!(m.dirs.get(Path::new("/home/kuba/.ssh")), Some(&0o700));
}

#[test]
fn of_two_keys_with_one_name_the_older_keeps_the_plain_file_and_the_newer_is_named_in_a_warning() {
    let mut m = machine();

    let out = pull_all(&mut m);

    assert!(
        out.warned("~/.ssh/github.pub is taken by \"GitHub\" (Personal): \"GitHub\" (Work) gets ~/.ssh/github-2.pub")
    );
    assert_eq!(public_key(&m, "github.pub").as_deref(), Some("ssh-ed25519 AAAAgithub"));
    assert_eq!(public_key(&m, "github-2.pub").as_deref(), Some("ssh-ed25519 AAAAdup"));
}

#[test]
fn the_summary_names_the_keys_with_a_host_block_and_the_keys_without() {
    let mut m = machine();

    let out = pull_all(&mut m);

    let summary = &out.report.summary;
    assert!(summary.contains(&"ssh    5 public key(s), 3 Host block(s) in ~/.ssh/config.d/bitwarden".to_string()));
    assert!(
        summary.contains(
            &"       Host blocks: GitHub (Personal), Work GitHub (Work), Build server (no folder)".to_string()
        )
    );
    assert!(summary.contains(
        &"       key only: GitHub (Work). A 'hosts' field (hosts=example.com) gives a key its Host block".to_string()
    ));
}

#[test]
fn each_host_block_names_the_agent_and_only_its_own_public_key() {
    let mut m = machine();

    pull_all(&mut m);

    let conf = m.text(SSH_CONFIG).unwrap();
    assert!(conf.starts_with("# Generated by `kubaos bitwarden` from the Bitwarden SSH keys in your vault."));
    assert!(conf.contains(
        "# GitHub (Personal) (SHA256:github)\nHost github.com\n  User git\n  \
         IdentityAgent /home/kuba/.bitwarden-ssh-agent.sock\n  IdentityFile /home/kuba/.ssh/github.pub\n  \
         IdentitiesOnly yes\n"
    ));
    assert!(conf.contains("Host github-work\n  HostName github.com\n  User git\n"));
    assert!(conf.contains(
        "# Build server (no folder) (SHA256:build)\nHost build build.example.com\n  HostName 192.0.2.10\n  User deploy\n  Port 2022\n"
    ));
    assert_eq!(host_blocks(&m), 3);
}

#[test]
fn a_field_smuggling_a_directive_gets_no_host_block() {
    let mut m = machine();

    let out = pull_all(&mut m);

    assert!(!m.text(SSH_CONFIG).unwrap().contains("ProxyCommand"));
    assert!(out.warned(
        "Evil (Work): the field `hosts` holds `evil` followed by a newline, and ssh_config would read the rest as directives; \
         in Bitwarden, make the value one line. No Host block"
    ));
}

#[test]
fn characters_ssh_config_cant_take_get_no_host_block() {
    let evil = ssh_item(
        "Quote",
        "quote",
        None,
        1,
        json!([field("hosts", "x\"y"), field("user", "a b")]),
    );
    let mut m = with_items(machine(), &[evil]);

    let out = pull_all(&mut m);

    assert_eq!(host_blocks(&m), 0);
    assert!(out.warned("characters ssh_config can't take safely"));
}

#[test]
fn a_name_with_a_newline_stays_on_one_line_in_the_ssh_config() {
    let item = ssh_item("Two\nlines", "two", None, 1, json!([field("hosts", "two.example")]));
    let mut m = with_items(machine(), &[item]);

    pull_all(&mut m);

    let conf = m.text(SSH_CONFIG).unwrap();
    assert!(conf.contains("# Two lines (no folder) (SHA256:two)\nHost two.example\n"));
    assert_eq!(
        m.text("/home/kuba/.ssh/two-lines.pub"),
        Some("ssh-ed25519 AAAAtwo bitwarden:Two lines\n")
    );
}

#[test]
fn a_host_in_two_items_warns_that_ssh_offers_both_keys() {
    let a = ssh_item("Personal GitHub", "a", None, 1, json!([field("hosts", "github.com")]));
    let b = ssh_item(
        "Work GitHub",
        "b",
        None,
        2,
        json!([field("hosts", "github.com gist.github.com")]),
    );
    let mut m = with_items(machine(), &[a, b]);

    let out = pull_all(&mut m);

    assert!(out.warned(
        "Host github.com is in both Personal GitHub (no folder) and Work GitHub (no folder), so ssh offers both keys there"
    ));
}

/// One default key for every server, and two GitHub accounts on aliases, in the order a vault
/// lists them: the default first.
fn default_and_aliases() -> Vec<serde_json::Value> {
    vec![
        ssh_item("Alias2k", "alias2k", None, 1, json!([field("hosts", "*")])),
        ssh_item(
            "SuplenC",
            "suplenc",
            None,
            2,
            json!([field("hosts", "github-suplenc"), field("hostname", "github.com")]),
        ),
        ssh_item(
            "JakubWasilczyk",
            "jakubwasilczyk",
            None,
            3,
            json!([field("hosts", "github-jakubwasilczyk"), field("hostname", "github.com")]),
        ),
    ]
}

fn host_lines(m: &Fake) -> Vec<String> {
    m.text(SSH_CONFIG)
        .unwrap()
        .lines()
        .filter(|l| l.starts_with("Host "))
        .map(str::to_string)
        .collect()
}

#[test]
fn blocks_with_a_wildcard_come_after_every_specific_block_in_vault_order() {
    let mut items = default_and_aliases();
    items.insert(
        1,
        ssh_item("Lab", "lab", None, 4, json!([field("hosts", "*.lab lab-?")])),
    );
    let mut m = with_items(machine(), &items);

    pull_all(&mut m);

    let hosts = host_lines(&m);
    assert_eq!(hosts.len(), 4, "{hosts:?}");
    assert_eq!(hosts[0], "Host github-suplenc");
    assert_eq!(hosts[1], "Host github-jakubwasilczyk");
    assert!(hosts[2].starts_with("Host * "), "{hosts:?}");
    assert!(hosts[3].starts_with("Host *.lab lab-? "), "{hosts:?}");
}

#[test]
fn an_alias_offers_only_its_own_key_not_the_default_one_too() {
    let mut m = with_items(machine(), &default_and_aliases());

    pull_all(&mut m);

    let conf = m.text(SSH_CONFIG).unwrap();
    assert!(
        conf.contains(
            "# Alias2k (no folder) (SHA256:alias2k)\nHost * !github-suplenc !github-jakubwasilczyk\n  \
             IdentityAgent /home/kuba/.bitwarden-ssh-agent.sock\n  IdentityFile /home/kuba/.ssh/alias2k.pub\n"
        ),
        "{conf}"
    );
    assert!(conf.contains("Host github-suplenc\n  HostName github.com\n"), "{conf}");
}

#[test]
fn a_wildcard_block_leaves_out_only_the_other_blocks_hosts() {
    let mixed = ssh_item("Mixed", "mixed", None, 1, json!([field("hosts", "* jump-box")]));
    let alias = ssh_item(
        "Alias",
        "alias",
        None,
        2,
        json!([field("hosts", "github-a"), field("hostname", "github.com")]),
    );
    let mut m = with_items(machine(), &[mixed, alias]);

    pull_all(&mut m);

    assert_eq!(host_lines(&m), ["Host github-a", "Host * jump-box !github-a"]);
}

#[test]
fn a_key_without_a_usable_public_key_is_skipped_with_a_warning() {
    let mut bad = ssh_item("Broken", "broken", None, 1, json!([]));
    bad["sshKey"]["publicKey"] = json!("ssh-ed25519 AAAA\nProxyCommand x");
    let mut m = with_items(machine(), &[bad]);

    let out = pull_all(&mut m);

    assert!(out.warned(
        "Broken (no folder): the public key field holds `ssh-ed25519 AAAA` followed by a newline; \
         in Bitwarden, make the public key that one line (or import the key again). Skipped: no .pub, no Host block"
    ));
    assert!(m.text("/home/kuba/.ssh/broken.pub").is_none());
}

fn with_public_key(name: &str, slug: &str, key: &str, fields: serde_json::Value) -> serde_json::Value {
    let mut item = ssh_item(name, slug, Some("f-work"), 1, fields);
    item["sshKey"]["publicKey"] = json!(key);
    item
}

#[test]
fn a_public_key_with_a_comment_is_written_without_the_comment() {
    let item = with_public_key(
        "Mac",
        "mac",
        "ssh-ed25519 AAAAmac jakub@macbook",
        json!([field("hosts", "mac.example")]),
    );
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert_eq!(
        m.text("/home/kuba/.ssh/mac.pub"),
        Some("ssh-ed25519 AAAAmac bitwarden:Mac\n")
    );
    assert_eq!(host_blocks(&m), 1);
    assert!(!out.warned("Mac (Work):"), "{:?}", out.report.warnings);
}

#[test]
fn a_public_key_with_runs_of_spaces_is_written_with_single_ones() {
    let item = with_public_key("Spaced", "spaced", "ssh-ed25519  AAAAspaced  me@host ", json!([]));
    let mut m = with_items(machine(), &[item]);

    pull_all(&mut m);

    assert_eq!(
        m.text("/home/kuba/.ssh/spaced.pub"),
        Some("ssh-ed25519 AAAAspaced bitwarden:Spaced\n")
    );
}

#[test]
fn a_public_key_ending_in_a_line_break_is_written_without_it() {
    let item = with_public_key(
        "Alias2k",
        "alias",
        "ssh-ed25519 AAAAalias me@work\n",
        json!([field("hosts", "alias.example")]),
    );
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert_eq!(
        m.text("/home/kuba/.ssh/alias2k.pub"),
        Some("ssh-ed25519 AAAAalias bitwarden:Alias2k\n")
    );
    assert!(!out.warned("Alias2k (Work):"), "{:?}", out.report.warnings);
}

#[test]
fn a_long_public_key_is_shortened_in_the_warning() {
    let key = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGq7\r\nx";
    let item = with_public_key("Long", "long", key, json!([]));
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(out.warned("Long (Work): the public key field holds `ssh-ed25519 AAAAC3NzaC1l…` followed by a newline"));
}

#[test]
fn a_public_key_of_an_unknown_type_names_the_type() {
    let item = with_public_key("Dss", "dss", "ssh-dss AAAAdss", json!([]));
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(out.warned(
        "Dss (Work): the public key is of type `ssh-dss`; kubaos bitwarden takes ssh-ed25519, ssh-rsa and ecdsa-sha2-nistp*. \
         Create an Ed25519 key in Bitwarden instead. Skipped: no .pub, no Host block"
    ), "{:?}", out.report.warnings);
}

#[test]
fn a_signing_key_with_a_comment_signs() {
    let item = with_public_key(
        "Signer",
        "signer",
        "ssh-ed25519 AAAAsigner me@mac",
        json!([{ "name": "signing", "value": "true", "type": 2 }]),
    );
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(!out.warned("signing is left as it was"), "{:?}", out.report.warnings);
    assert!(
        m.text(SIGNING)
            .unwrap()
            .contains("signingKey = /home/kuba/.ssh/signer.pub")
    );
    assert_eq!(
        m.text(SIGNERS),
        Some("k@x.eu namespaces=\"git\" ssh-ed25519 AAAAsigner\n")
    );
}

#[test]
fn a_pub_on_disk_with_another_comment_counts_as_the_same_key() {
    let mut m = machine()
        .file("/home/kuba/.ssh/github", "-----BEGIN OPENSSH PRIVATE KEY-----")
        .file("/home/kuba/.ssh/github.pub", "ssh-ed25519 AAAAgithub me@laptop\n");

    let out = pull_all(&mut m);

    assert!(!out.warned("different key on disk"), "{:?}", out.report.warnings);
    assert_eq!(
        m.text("/home/kuba/.ssh/github.pub"),
        Some("ssh-ed25519 AAAAgithub bitwarden:GitHub\n")
    );
}

#[test]
fn field_names_match_whatever_their_case_and_surrounding_spaces() {
    let a = ssh_item(
        "A",
        "a",
        None,
        1,
        json!([
            field("Hosts", "a-alias"),
            field("HostName", "a.example"),
            field(" User", "git")
        ]),
    );
    let b = ssh_item(
        "B",
        "b",
        None,
        2,
        json!([field("hosts", "b-alias"), field("hostname ", "b.example")]),
    );
    let mut m = with_items(machine(), &[a, b]);

    let out = pull_all(&mut m);

    let conf = m.text(SSH_CONFIG).unwrap();
    assert!(
        conf.contains("Host a-alias\n  HostName a.example\n  User git\n"),
        "{conf}"
    );
    assert!(conf.contains("Host b-alias\n  HostName b.example\n"), "{conf}");
    assert!(
        !out.warned("A (no folder):") && !out.warned("B (no folder):"),
        "{:?}",
        out.report.warnings
    );
}

#[test]
fn an_alias_without_a_hostname_keeps_its_block_and_warns_with_the_fix() {
    let item = ssh_item(
        "Lavoro GitHub",
        "lavoro",
        None,
        1,
        json!([field("hosts", "github-lavoro"), field("user", "git")]),
    );
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(
        out.warned(
            "Lavoro GitHub (no folder): Host github-lavoro has no `hostname` field, so ssh looks up \"github-lavoro\" \
         as a server name and fails; in Bitwarden, add a custom text field `hostname` = github.com"
        ),
        "{:?}",
        out.report.warnings
    );
    assert!(m.text(SSH_CONFIG).unwrap().contains("Host github-lavoro\n  User git\n"));
}

#[test]
fn a_real_host_name_needs_no_hostname_field() {
    let mut m = machine();

    let out = pull_all(&mut m);

    assert!(!out.warned("has no `hostname` field"), "{:?}", out.report.warnings);
}

#[test]
fn an_unknown_field_close_to_a_known_one_points_to_it() {
    let item = ssh_item(
        "Typo",
        "typo",
        None,
        1,
        json!([
            field("host", "typo.example"),
            field("Hostnme", "x"),
            field("notes2", "y")
        ]),
    );
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(
        out.warned("Typo (no folder): kubaos bitwarden reads no field `host`; in Bitwarden, rename it to `hosts`"),
        "{:?}",
        out.report.warnings
    );
    assert!(
        out.warned(
            "Typo (no folder): kubaos bitwarden reads no field `Hostnme`; in Bitwarden, rename it to `hostname`"
        )
    );
    assert!(!out.warned("notes2"));
}

#[test]
fn a_field_ssh_config_cant_take_is_named_with_its_value() {
    let item = ssh_item(
        "Spaced user",
        "su",
        None,
        1,
        json!([field("hosts", "su.example"), field("user", "a b")]),
    );
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(
        out.warned(
            "Spaced user (no folder): the field `user` holds `a b`, with characters ssh_config can't take safely \
         (it takes letters, digits and . _ @ -); fix the value in Bitwarden. No Host block"
        ),
        "{:?}",
        out.report.warnings
    );
}

#[test]
fn a_private_key_on_disk_under_the_same_name_keeps_its_pub() {
    let mut m = machine()
        .file("/home/kuba/.ssh/github", "-----BEGIN OPENSSH PRIVATE KEY-----")
        .file("/home/kuba/.ssh/github.pub", "ssh-ed25519 AAAAmine me@laptop\n");

    let out = pull_all(&mut m);

    assert!(out.warned("GitHub (Personal): ~/.ssh/github is a different key on disk"));
    assert_eq!(
        m.text("/home/kuba/.ssh/github.pub"),
        Some("ssh-ed25519 AAAAmine me@laptop\n")
    );
}

#[test]
fn a_file_field_picks_the_pub_name() {
    let item = ssh_item("Server", "server", None, 1, json!([field("file", "id_server")]));
    let mut m = with_items(machine(), &[item]);

    pull_all(&mut m);

    assert!(m.text("/home/kuba/.ssh/id_server.pub").is_some());
}

#[test]
fn no_private_key_or_secret_value_reaches_a_file() {
    let mut m = machine();

    pull_all(&mut m);

    for (path, (text, _)) in &m.files {
        for secret in [
            "PRIVATE KEY",
            "b3BlbnNzaC1rZXktdjE",
            "tok-gh-99",
            "tok-7c1f",
            "MIIBmarker9d2e",
        ] {
            assert!(!text.contains(secret), "{} holds {secret}", path.display());
        }
    }
}

// --------------------------------------------------------------- folders

#[test]
fn folder_work_takes_only_works_keys_and_its_github_alone_is_github_pub() {
    let mut m = machine();
    pull_parts(&mut m, ssh_only());
    m.files.insert(
        CONFIG.into(),
        ("server=https://vault.bitwarden.eu\nfolder=Work\n".into(), 0o600),
    );

    let out = pull_parts(&mut m, ssh_only());

    assert!(
        out.text().contains("3 SSH key(s) in the folder \"Work\""),
        "{}",
        out.text()
    );
    assert!(m.text("/home/kuba/.ssh/build-server.pub").is_none());
    assert!(m.text("/home/kuba/.ssh/github-2.pub").is_none());
    assert_eq!(public_key(&m, "github.pub").as_deref(), Some("ssh-ed25519 AAAAdup"));
    assert_eq!(
        public_key(&m, "work-github.pub").as_deref(),
        Some("ssh-ed25519 AAAAwork")
    );
    assert_eq!(host_blocks(&m), 1);
    assert!(
        out.text()
            .contains("removed ~/.ssh/build-server.pub (no longer in the vault)")
    );
    assert!(m.text(SSH_CONFIG).unwrap().contains("SSH keys in the folder \"Work\""));
}

#[test]
fn folders_lists_several_and_warns_about_the_one_the_vault_lacks() {
    let mut m = machine().file(CONFIG, "folders=Personal, Work,Archive\n");

    let out = pull_parts(&mut m, ssh_only());

    assert!(out.warned("no folder \"Archive\" in the vault (folders= in ~/.config/dot-bw/config)"));
    assert!(
        out.text()
            .contains("4 SSH key(s) in the folders \"Personal\", \"Work\"")
    );
    assert!(m.text("/home/kuba/.ssh/build-server.pub").is_none());
    assert_eq!(public_key(&m, "github.pub").as_deref(), Some("ssh-ed25519 AAAAgithub"));
    assert_eq!(public_key(&m, "github-2.pub").as_deref(), Some("ssh-ed25519 AAAAdup"));
    assert_eq!(host_blocks(&m), 2);
}

#[test]
fn a_folder_the_vault_lacks_falls_back_to_the_whole_vault() {
    let mut m = machine().file(CONFIG, "folder=dotfiles\n");

    let out = pull_parts(&mut m, ssh_only());

    assert!(out.warned("no folder \"dotfiles\" in the vault"));
    assert!(
        out.warned("none of the folders in ~/.config/dot-bw/config is in the vault: using every SSH key in the vault")
    );
    assert!(out.text().contains("5 SSH key(s) in your vault"));
    assert_eq!(host_blocks(&m), 3);
}

#[test]
fn a_folder_matches_whatever_spaces_surround_its_name_in_the_vault() {
    let folders = json!([{ "id": "f-personal", "name": "Personal" }, { "id": "f-work", "name": " Work " }]).to_string();
    let mut m = machine()
        .vault(&folders, &json!(items()).to_string())
        .file(CONFIG, "folders=Work\n");

    let out = pull_parts(&mut m, ssh_only());

    assert!(!out.warned("no folder"), "{:?}", out.report.warnings);
    assert!(
        out.text().contains("3 SSH key(s) in the folder \"Work\""),
        "{}",
        out.text()
    );
}

#[test]
fn a_listed_folder_matches_whatever_spaces_surround_it_in_the_list() {
    let folders = vault::folders(&folders()).unwrap();
    let mut filter = None;

    let (out, _) = run(|p| {
        filter = Some(vault::Filter::pick(&folders, &[" Work ".into()], p));
        Ok(())
    });

    assert!(!out.warned("no folder"), "{:?}", out.report.warnings);
    assert_eq!(filter.unwrap().ids, ["f-work"]);
}

#[test]
fn set_folders_writes_the_list_and_an_empty_one_means_the_whole_vault() {
    let mut m = machine().file(CONFIG, "server=a\nfolder=Work\n");

    set_folders(&mut m, &paths(), &["Personal".into(), "Work".into()]).unwrap();
    assert_eq!(m.text(CONFIG), Some("server=a\nfolders=Personal,Work\n"));
    assert_eq!(m.mode(CONFIG), Some(0o600));

    set_folders(&mut m, &paths(), &[]).unwrap();
    assert_eq!(m.text(CONFIG), Some("server=a\n"));
}

// ------------------------------------------------------------------- git

#[test]
fn git_signs_with_the_key_marked_signing_and_trusts_it_for_every_address() {
    let mut m = machine();

    let out = pull_all(&mut m);

    let signing = m.text(SIGNING).unwrap();
    assert!(signing.contains("signingKey = /home/kuba/.ssh/github.pub\n"));
    assert!(signing.contains("[gpg]\n\tformat = ssh\n"));
    assert!(signing.contains("allowedSignersFile = /home/kuba/.config/git-bitwarden/allowed_signers\n"));
    assert!(signing.contains("[commit]\n\tgpgSign = true\n"));
    assert_eq!(
        m.text(SIGNERS),
        Some("k@x.eu namespaces=\"git\" ssh-ed25519 AAAAgithub\n")
    );
    assert!(
        out.report
            .summary
            .contains(&"git    commits signed with GitHub (Personal) (~/.ssh/github.pub)".to_string())
    );
}

#[test]
fn with_two_signing_keys_the_oldest_signs_and_the_other_is_named() {
    let sign = json!([{ "name": "signing", "value": "true", "type": 2 }, field("emails", "old@x.eu,other@x.eu")]);
    let a = ssh_item("Old", "old", None, 1, sign.clone());
    let b = ssh_item("New", "new", None, 2, sign);
    let c = ssh_item("Retired", "retired", None, 3, json!([field("signing", "verify")]));
    let mut m = with_items(machine(), &[b, a, c]);

    let out = pull_all(&mut m);

    assert!(out.warned(
        "2 SSH keys have signing=true: Old (no folder), New (no folder). Signing with the oldest, Old (no folder); set signing=verify on the others"
    ));
    assert!(
        m.text(SIGNING)
            .unwrap()
            .contains("signingKey = /home/kuba/.ssh/old.pub")
    );
    let signers = m.text(SIGNERS).unwrap();
    assert!(signers.contains("k@x.eu namespaces=\"git\" ssh-ed25519 AAAAretired\n"));
    assert!(signers.contains("other@x.eu namespaces=\"git\" ssh-ed25519 AAAAnew\n"));
}

#[test]
fn a_signing_key_whose_pub_belongs_to_a_private_key_on_disk_says_to_give_it_a_file_name() {
    let mut m = machine()
        .file("/home/kuba/.ssh/github", "-----BEGIN OPENSSH PRIVATE KEY-----")
        .file("/home/kuba/.ssh/github.pub", "ssh-ed25519 AAAAmine me@laptop\n");

    let out = pull_all(&mut m);

    assert!(out.warned(
        "GitHub (Personal) has signing=true, but ~/.ssh/github.pub belongs to the private key ~/.ssh/github on disk \
         and holds another key, so signing is left as it was; in Bitwarden, add a text field `file` = \
         `github-bitwarden` to GitHub (Personal), then run `kubaos bitwarden refresh --only ssh,git`"
    ), "{:?}", out.report.warnings);
    assert!(m.text(SIGNING).is_none());
}

#[test]
fn git_alone_with_an_older_pub_says_to_pull_ssh_too() {
    let mut m = machine().file("/home/kuba/.ssh/github.pub", "ssh-ed25519 AAAAold bitwarden:GitHub\n");

    let out = pull_parts(&mut m, Parts::parse("git").unwrap());

    assert!(out.warned(
        "GitHub (Personal) has signing=true, but ~/.ssh/github.pub holds another key, from an older pull, so signing \
         is left as it was; run `kubaos bitwarden refresh --only ssh,git` to write the vault's key"
    ), "{:?}", out.report.warnings);
    assert!(m.text(SIGNING).is_none());
}

#[test]
fn git_alone_without_the_pub_says_to_pull_ssh_too() {
    let mut m = machine();

    let out = pull_parts(&mut m, Parts::parse("git").unwrap());

    assert!(
        out.warned(
            "GitHub (Personal) has signing=true, but ~/.ssh/github.pub isn't on disk, so signing is left as it was; \
         run `kubaos bitwarden refresh --only ssh,git` to write it"
        ),
        "{:?}",
        out.report.warnings
    );
    assert!(m.text(SIGNING).is_none());
}

#[test]
fn a_signing_key_with_an_unusable_public_key_signs_nothing_and_says_to_fix_it() {
    let item = with_public_key(
        "Dss",
        "dss",
        "ssh-dss AAAAdss",
        json!([{ "name": "signing", "value": "true", "type": 2 }]),
    );
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(
        out.warned(
            "Dss (Work) has signing=true, but its public key can't be used (see its warning), so signing is left \
         as it was; fix the public key in Bitwarden, then run `kubaos bitwarden refresh --only ssh,git`"
        ),
        "{:?}",
        out.report.warnings
    );
    assert!(m.text(SIGNING).is_none());
}

#[test]
fn without_a_signing_key_commits_stay_unsigned_and_the_old_file_goes() {
    let item = ssh_item("Plain", "plain", None, 1, json!([]));
    let mut m = with_items(machine().file(SIGNING, "[user]\n"), &[item]);

    let out = pull_all(&mut m);

    assert!(m.text(SIGNING).is_none());
    assert!(out.warned("no SSH key in your vault has the field signing=true: commits stay unsigned"));
}

// -------------------------------------------------------------------- gh

#[test]
fn gh_gets_the_token_on_stdin_and_keeps_it_in_the_keyring() {
    let mut m = machine();

    let out = pull_all(&mut m);

    let login = m
        .ran
        .iter()
        .find(|c| c.args.starts_with(&["auth".into(), "login".into()]))
        .unwrap();
    assert_eq!(login.stdin.as_ref().map(|s| s.0.as_str()), Some("tok-gh-99"));
    assert!(!login.args.iter().any(|a| a.contains("tok-gh")));
    assert!(
        out.report
            .summary
            .contains(&"gh     logged in to github.com (token in the keyring)".to_string())
    );
}

#[test]
fn gh_without_a_keyring_logs_out_again() {
    let mut m = machine().answer("gh auth status", 0, "Token: gho_*** (oauth_token)\n");

    let out = pull_all(&mut m);

    assert!(m.lines().iter().any(|l| l.starts_with("gh auth logout")));
    assert!(out.warned("gh had no keyring and wrote the token to a file"));
}

#[test]
fn without_a_gh_token_item_gh_is_skipped() {
    let mut m = with_items(machine(), &[]);

    let out = pull_all(&mut m);

    assert!(
        out.report
            .summary
            .contains(&"gh     no gh-token item, skipped".to_string())
    );
}

// ------------------------------------------------------------------- env

#[test]
fn a_pull_puts_every_project_in_the_keyring_and_multi_line_values_go_base64() {
    let mut m = machine();

    let out = pull_all(&mut m);

    assert!(
        out.report
            .summary
            .contains(&"env    1 project(s) in the keyring: demo".to_string())
    );
    assert!(out.warned("demo: skipped fields that aren't variable names: bad-name"));
    let tls = &m.keyring[&("demo".to_string(), "TLS_KEY".to_string())];
    assert_eq!(tls.label, "dotfiles-env/demo/TLS_KEY#base64");
    assert!(!tls.value.contains('\n'));
    let vars: Vec<(String, String)> = env::export(&mut m, "demo")
        .unwrap()
        .into_iter()
        .map(|(n, v)| (n, v.0))
        .collect();
    assert_eq!(vars.len(), 4, "{vars:?}");
    assert!(vars.contains(&("TLS_KEY".into(), PEM.into())));
    assert!(vars.contains(&("API_TOKEN".into(), "tok-7c1f".into())));
    assert!(vars.contains(&("EXTRA_VAR".into(), "extra-value".into())));
    assert!(vars.contains(&("DATABASE_URL".into(), "postgres://demo:pw@db:5432/demo".into())));
}

#[test]
fn a_bwenv_field_counts_whatever_its_case_and_surrounding_spaces() {
    let item = json!({
        "id": "i-loose", "type": 2, "name": "loose env", "folderId": null, "notes": null,
        "fields": [
            { "name": " BWEnv ", "value": "loose", "type": 0 },
            { "name": "Api_Token", "value": "tok-loose", "type": 1 },
            { "name": "API_TOKEN", "value": "tok-upper", "type": 1 },
        ],
    });
    let mut m = with_items(machine(), &[item]);

    let out = pull_all(&mut m);

    assert!(
        out.report
            .summary
            .contains(&"env    2 project(s) in the keyring: demo loose".to_string()),
        "{}",
        out.text()
    );
    assert!(!out.warned("skipped fields"), "{:?}", out.report.warnings);
    let mut vars: Vec<(String, String)> = env::export(&mut m, "loose")
        .unwrap()
        .into_iter()
        .map(|(n, v)| (n, v.0))
        .collect();
    vars.sort();
    assert_eq!(
        vars,
        [
            ("API_TOKEN".to_string(), "tok-upper".to_string()),
            ("Api_Token".to_string(), "tok-loose".to_string())
        ]
    );
}

#[test]
fn values_reach_secret_tool_on_stdin_never_in_argv() {
    let mut m = machine();

    pull_all(&mut m);

    let lines = m.lines().join("\n");
    for secret in [
        "tok-7c1f",
        "MIIBmarker9d2e",
        "postgres://",
        "extra-value",
        SESSION,
        PASSWORD,
    ] {
        assert!(!lines.contains(secret), "{secret} in a command line");
    }
}

#[test]
fn a_project_gone_from_the_vault_leaves_the_keyring_on_the_next_pull() {
    let mut m = machine();
    pull_all(&mut m);
    m.keyring.insert(
        ("old".into(), "X".into()),
        fake::Secret {
            label: "dotfiles-env/old/X".into(),
            value: "1".into(),
        },
    );

    let out = pull_all(&mut m);

    assert!(
        out.text()
            .contains("old: gone from the vault, removed from the keyring")
    );
    assert_eq!(env::projects(&mut m).unwrap(), ["demo"]);
}

#[test]
fn env_pull_fetches_only_the_projects_asked_for() {
    let mut m = machine();

    let (out, r) = run(|p| {
        env::pull(
            &mut m,
            &Session::new(SESSION),
            &Names::default(),
            &["demo".into(), "nope".into()],
            p,
        )
    });

    r.unwrap();
    assert!(out.warned("nope: no folder env/nope and no item with bwenv=nope in the vault"));
    assert!(out.text().contains("demo                 4 variable(s)"));
    assert_eq!(env::projects(&mut m).unwrap(), ["demo"]);
}

#[test]
fn a_bad_project_name_is_refused_before_anything_runs() {
    let mut m = machine();

    let (_, r) = run(|p| env::pull(&mut m, &Session::new(SESSION), &Names::default(), &["../x".into()], p));

    assert!(matches!(r, Err(Error::Project(_))));
    assert!(m.bw.is_empty());
    assert!(env::export(&mut m, "a b").is_err());
}

#[test]
fn list_shows_names_only_and_clear_forgets_a_project() {
    let mut m = machine();
    pull_all(&mut m);

    let listed = env::list(&mut m, None).unwrap();
    assert_eq!(
        listed["demo"].iter().map(String::as_str).collect::<Vec<_>>(),
        ["API_TOKEN", "DATABASE_URL", "EXTRA_VAR", "TLS_KEY"]
    );

    env::clear(&mut m, Some("demo"));
    assert!(env::projects(&mut m).unwrap().is_empty());
    assert!(env::export(&mut m, "demo").unwrap().is_empty());
}

#[test]
fn the_vault_names_default_to_gh_token_and_env_and_the_environment_overrides_them() {
    let none = Names::from_lookup(|_| None);
    let blank = Names::from_lookup(|_| Some("  ".into()));
    let set = Names::from_lookup(|var| match var {
        "KUBAOS_BW_GH_ITEM" => Some("GitHub CLI".into()),
        "KUBAOS_BW_ENV_PREFIX" => Some("secrets/".into()),
        _ => None,
    });

    assert_eq!(none, Names::default());
    assert_eq!((none.gh_item.as_str(), none.env_prefix.as_str()), ("gh-token", "env/"));
    assert_eq!(blank, Names::default());
    assert_eq!(
        (set.gh_item.as_str(), set.env_prefix.as_str()),
        ("GitHub CLI", "secrets/")
    );
    assert_eq!(set.project("secrets/demo"), Some("demo"));
    assert_eq!(set.project("env/demo"), None);
    assert_eq!(set.project("secrets/"), None);
}

#[test]
fn a_pull_finds_gh_and_the_projects_under_the_names_it_is_given() {
    let folders = json!([
        { "id": "f-env", "name": "env/old" },
        { "id": "f-secrets", "name": "secrets/demo" },
    ])
    .to_string();
    let items = json!([
        {
            "id": "i-demo", "type": 2, "name": "demo env", "folderId": "f-secrets", "notes": null,
            "fields": [{ "name": "API_TOKEN", "value": "tok-7c1f", "type": 1 }],
        },
        {
            "id": "i-old", "type": 2, "name": "old env", "folderId": "f-env", "notes": null,
            "fields": [{ "name": "OLD", "value": "1", "type": 1 }],
        },
        {
            "id": "i-gh", "type": 1, "name": "GitHub CLI", "folderId": null,
            "login": { "username": "kuba", "password": "tok-gh-99" },
        },
    ])
    .to_string();
    let mut m = machine().vault(&folders, &items);
    let mut paths = paths();
    paths.names = Names {
        gh_item: "GitHub CLI".into(),
        env_prefix: "secrets/".into(),
    };

    let (out, r) = run(|p| pull(&mut m, &paths, &Session::new(SESSION), Parts::ALL, p));

    r.expect("pull");
    assert!(
        out.report
            .summary
            .contains(&"env    1 project(s) in the keyring: demo".to_string()),
        "{}",
        out.text()
    );
    assert_eq!(env::projects(&mut m).unwrap(), ["demo"]);
    let login = m
        .ran
        .iter()
        .find(|c| c.args.starts_with(&["auth".into(), "login".into()]))
        .unwrap();
    assert_eq!(login.stdin.as_ref().map(|s| s.0.as_str()), Some("tok-gh-99"));
}

#[test]
fn a_project_asked_for_by_name_says_which_folder_it_looked_for() {
    let mut m = machine();
    let names = Names {
        gh_item: DEFAULT_GH_ITEM.into(),
        env_prefix: "secrets/".into(),
    };

    let (out, r) = run(|p| {
        env::pull(
            &mut m,
            &Session::new(SESSION),
            &names,
            &["demo".into(), "nope".into()],
            p,
        )
    });

    r.unwrap();
    assert!(out.warned("nope: no folder secrets/nope and no item with bwenv=nope in the vault"));
    assert!(
        out.text().contains("demo                 1 variable(s)"),
        "{}",
        out.text()
    );
}

#[test]
fn names_and_projects_follow_the_shell_rules() {
    assert!(env::name_ok("_A1") && env::name_ok("DATABASE_URL"));
    assert!(!env::name_ok("1A") && !env::name_ok("bad-name") && !env::name_ok(""));
    assert!(env::project_ok("my.app_2-x"));
    assert!(!env::project_ok(".hidden") && !env::project_ok("a/b") && !env::project_ok(""));
}

#[test]
fn base64_round_trips_every_length() {
    for s in ["", "a", "ab", "abc", "abcd", PEM, "tab\there"] {
        let e = base64::encode(s.as_bytes());
        assert_eq!(base64::decode(&e).unwrap(), s.as_bytes(), "{s}");
    }
    assert_eq!(base64::encode(b"hello"), "aGVsbG8=");
    assert!(base64::decode("not*base64").is_none());
}

// ----------------------------------------------------------------- agent

#[test]
fn the_agent_serving_every_key_says_so() {
    let mut m = machine();

    let out = pull_all(&mut m);

    assert!(out.text().contains("agent answers with 5 key(s)"));
    assert!(
        out.report
            .summary
            .contains(&"agent  running, serves every SSH key in your vault".to_string())
    );
    let probe = m.ran.iter().find(|c| c.args.contains(&"ssh-add".to_string())).unwrap();
    assert!(probe.env.contains(&("SSH_AUTH_SOCK".into(), AGENT.into())));
}

#[test]
fn the_agent_missing_a_key_names_it() {
    let mut m = machine().answer("timeout 20 ssh-add", 0, &agent_lists(&["github"]));

    let out = pull_all(&mut m);

    assert!(out.warned("agent doesn't offer SHA256:work (vault not synced in the app?)"));
    assert!(
        out.report
            .summary
            .iter()
            .any(|l| l.contains("some vault keys are missing"))
    );
}

#[test]
fn the_agent_states_each_have_their_hint() {
    let mut off = machine();
    off.sockets.clear();
    assert!(pull_all(&mut off).warned("no agent socket"));

    let mut locked = machine().answer("timeout 20 ssh-add", 1, "");
    assert!(pull_all(&mut locked).warned("agent has no keys: unlock Bitwarden desktop"));

    let mut dead = machine().answer("timeout 20 ssh-add", 2, "");
    assert!(pull_all(&mut dead).warned("agent socket doesn't answer"));
}

#[test]
fn another_ssh_auth_sock_is_a_warning() {
    let mut m = machine();
    let mut p = paths();
    p.auth_sock = Some("/run/user/1000/gcr/ssh".into());

    let (out, r) = run(|pr| pull(&mut m, &p, &Session::new(SESSION), Parts::ALL, pr));

    r.unwrap();
    assert!(out.warned("SSH_AUTH_SOCK is '/run/user/1000/gcr/ssh', not the Bitwarden agent"));
}

#[test]
fn check_asks_the_agent_about_the_keys_on_disk_without_bw() {
    let mut m = machine();
    pull_all(&mut m);
    m.bw.clear();
    for (file, k) in [
        ("github", "github"),
        ("github-2", "dup"),
        ("work-github", "work"),
        ("build-server", "build"),
        ("evil", "evil"),
    ] {
        m = m.answer(
            &format!("ssh-keygen -lf /home/kuba/.ssh/{file}.pub"),
            0,
            &format!("256 SHA256:{k} x (ED25519)\n"),
        );
    }

    let (out, r) = run(|p| {
        check(&mut m, &paths(), p);
        Ok(())
    });

    r.unwrap();
    assert!(m.bw.is_empty());
    assert!(out.text().contains("agent answers with 5 key(s)"));
    assert!(
        out.report
            .summary
            .contains(&"file   ~/.ssh/config.d/bitwarden".to_string())
    );
    assert!(
        out.report
            .summary
            .contains(&"file   ~/.config/git-bitwarden/signing.gitconfig".to_string())
    );
    assert!(out.report.summary.contains(&"env    demo".to_string()));
}

#[test]
fn check_names_the_files_missing() {
    let mut m = machine();

    let (out, _) = run(|p| {
        check(&mut m, &paths(), p);
        Ok(())
    });

    assert!(
        out.report
            .summary
            .contains(&"file   ~/.ssh/config.d/bitwarden missing".to_string())
    );
}

// --------------------------------------------------------------- overview

#[test]
fn a_pull_leaves_an_overview_of_folders_and_keys_for_the_card() {
    let mut m = machine();

    pull_all(&mut m);

    let o = Overview::read(&mut m, &paths());
    assert_eq!(o.folders, ["Personal", "Work", "env/demo"]);
    let github = o.keys.iter().find(|k| k.file.as_deref() == Some("github")).unwrap();
    assert!(github.block && github.signing);
    assert_eq!(github.hosts, ["github.com"]);
    let dup = o.keys.iter().find(|k| k.file.as_deref() == Some("github-2")).unwrap();
    assert!(!dup.block && !dup.signing);
    assert_eq!(dup.note.as_deref(), Some("no 'hosts' field"));
    let evil = o.keys.iter().find(|k| k.name == "Evil").unwrap();
    assert!(!evil.block);
    let text = m.text("/home/kuba/.local/state/dot-bw/overview.json").unwrap();
    assert!(!text.contains("AAAA"), "the overview holds no key material");
}

// ----------------------------------------------------------------- login

struct Script(Vec<&'static str>);

impl Ask for Script {
    fn ask(&mut self, _prompt: &str) -> Option<String> {
        (!self.0.is_empty()).then(|| self.0.remove(0).to_string())
    }

    fn tty(&self) -> bool {
        true
    }
}

fn log_in(m: &mut Fake, req: &LoginRequest, ask: &mut dyn Ask) -> (Out, Result<Session, Error>) {
    let mut session = None;
    let (out, r) = run(|p| {
        session = Some(login(m, &paths(), req, ask, p)?);
        Ok(())
    });
    (out, r.map(|()| session.expect("a session")))
}

#[test]
fn a_first_terminal_login_asks_the_server_the_method_and_the_email() {
    let mut m = Fake::new();

    let (out, r) = log_in(&mut m, &LoginRequest::default(), &mut Script(vec!["2", "", "k@x.eu"]));

    assert_eq!(r.unwrap().expose(), SESSION);
    assert_eq!(
        m.text(CONFIG),
        Some("server=https://vault.bitwarden.eu\nemail=k@x.eu\n")
    );
    assert_eq!(m.server.as_deref(), Some(EU));
    assert!(matches!(&m.logins[0], Login::Totp { email } if email == "k@x.eu"));
    assert!(out.text().contains("vault unlocked for this run"));
    assert_eq!(m.bw, ["status", "config server https://vault.bitwarden.eu", "login"]);
}

#[test]
fn a_self_hosted_server_is_asked_for_its_url() {
    let mut m = Fake::new();

    let (_, r) = log_in(
        &mut m,
        &LoginRequest::default(),
        &mut Script(vec!["3", "https://vault.example.org/", "1", "k@x.eu"]),
    );

    r.unwrap();
    assert_eq!(m.server.as_deref(), Some("https://vault.example.org"));
}

#[test]
fn a_failed_authenticator_login_offers_the_api_key_and_takes_its_warning_back() {
    let mut m = Fake::new().file(CONFIG, "server=https://vault.bitwarden.eu\nemail=k@x.eu\n");
    m.login_error = Some("Two-step token is invalid. Try again.".into());

    let (out, r) = log_in(&mut m, &LoginRequest::default(), &mut Script(vec!["1", "y"]));

    r.unwrap();
    assert!(matches!(m.logins[1], Login::ApiKeyPrompt));
    assert!(out.report.warnings.is_empty());
    assert_eq!(m.bw.last().map(String::as_str), Some("unlock"));
}

#[test]
fn a_failed_authenticator_login_with_login_given_stops() {
    let mut m = Fake::new().file(CONFIG, "server=https://vault.bitwarden.eu\nemail=k@x.eu\n");
    m.login_error = Some("Two-step token is invalid.".into());
    let req = LoginRequest {
        method: Some(Method::Totp),
        ..LoginRequest::default()
    };

    let (_, r) = log_in(&mut m, &req, &mut Script(vec![]));

    let e = r.unwrap_err();
    assert_eq!(e.to_string(), "try again, or: kubaos bitwarden setup --login apikey");
}

#[test]
fn a_refresh_asks_nothing() {
    let refresh = LoginRequest {
        refresh: true,
        ..LoginRequest::default()
    };

    let (_, r) = log_in(&mut Fake::new(), &refresh, &mut NoTerminal);
    assert!(matches!(r, Err(Error::NotSetUp)));

    let mut m = Fake::new().file(CONFIG, "server=https://vault.bitwarden.eu\n");
    let (_, r) = log_in(&mut m, &refresh, &mut NoTerminal);
    assert!(matches!(r, Err(Error::LoggedOut)));
}

#[test]
fn a_question_without_a_terminal_names_it() {
    let (_, r) = log_in(&mut Fake::new(), &LoginRequest::default(), &mut NoTerminal);

    assert_eq!(
        r.unwrap_err().to_string(),
        "need an answer for 'Choice [1-3]:' but stdin isn't a terminal"
    );
}

#[test]
fn a_locked_vault_only_unlocks() {
    let mut m = Fake::new().locked(EU, "k@x.eu");

    let (out, r) = log_in(&mut m, &LoginRequest::default(), &mut NoTerminal);

    r.unwrap();
    assert_eq!(m.bw, ["status", "unlock"]);
    assert!(out.text().contains("logged in as k@x.eu on https://vault.bitwarden.eu"));
    assert_eq!(m.text(CONFIG), Some("server=https://vault.bitwarden.eu\n"));
}

#[test]
fn a_session_from_bw_session_is_reused() {
    let mut m = Fake::new().locked(EU, "k@x.eu");
    let req = LoginRequest {
        session: Some(Session::new(SESSION)),
        ..LoginRequest::default()
    };

    let (out, r) = log_in(&mut m, &req, &mut NoTerminal);

    r.unwrap();
    assert_eq!(m.bw, ["status"]);
    assert!(out.text().contains("unlocked (k@x.eu, session from BW_SESSION)"));
}

#[test]
fn logged_in_to_another_server_says_to_log_out_first() {
    let mut m = Fake::new()
        .locked("https://vault.bitwarden.com", "k@x.eu")
        .file(CONFIG, "server=https://vault.bitwarden.eu\n");

    let (_, r) = log_in(&mut m, &LoginRequest::default(), &mut NoTerminal);

    assert_eq!(
        r.unwrap_err().to_string(),
        "bw is logged in to https://vault.bitwarden.com, not https://vault.bitwarden.eu: run 'bw logout' first"
    );
}

fn form(method: FormMethod) -> Form {
    Form {
        server: "eu".into(),
        email: "k@x.eu".into(),
        password: Sensitive(PASSWORD.into()),
        method,
        code: "123456".into(),
        client_id: String::new(),
        client_secret: Sensitive(String::new()),
    }
}

#[test]
fn the_form_logs_in_with_the_password_in_bws_environment() {
    let mut m = Fake::new();
    let req = LoginRequest {
        form: Some(form(FormMethod::Totp)),
        ..LoginRequest::default()
    };

    let (out, r) = log_in(&mut m, &req, &mut NoTerminal);

    assert_eq!(r.unwrap().expose(), SESSION);
    assert!(matches!(&m.logins[0], Login::Password { method: "0", code, .. } if code == "123456"));
    assert_eq!(
        m.text(CONFIG),
        Some("server=https://vault.bitwarden.eu\nemail=k@x.eu\n")
    );
    assert!(!out.text().contains(PASSWORD) && !out.text().contains(SESSION));
}

#[test]
fn the_form_with_a_yubikey_otp_uses_method_3() {
    let mut m = Fake::new();
    let req = LoginRequest {
        form: Some(form(FormMethod::YubikeyOtp)),
        ..LoginRequest::default()
    };

    log_in(&mut m, &req, &mut NoTerminal).1.unwrap();

    assert!(matches!(&m.logins[0], Login::Password { method: "3", .. }));
}

#[test]
fn the_form_with_an_api_key_logs_in_then_unlocks_with_the_password() {
    let mut m = Fake::new();
    let mut f = form(FormMethod::ApiKey);
    f.client_id = "user.abc".into();
    f.client_secret = Sensitive("s3cret".into());
    let req = LoginRequest {
        form: Some(f),
        ..LoginRequest::default()
    };

    log_in(&mut m, &req, &mut NoTerminal).1.unwrap();

    assert!(matches!(&m.logins[0], Login::ApiKey { client_id, .. } if client_id == "user.abc"));
    assert_eq!(m.bw.last().map(String::as_str), Some("unlock"));
}

#[test]
fn the_form_says_what_it_lacks() {
    let mut f = form(FormMethod::Totp);
    f.code.clear();
    let req = LoginRequest {
        form: Some(f),
        ..LoginRequest::default()
    };
    let (_, r) = log_in(&mut Fake::new(), &req, &mut NoTerminal);
    assert_eq!(r.unwrap_err().to_string(), "email, master password and code, all three");

    let req = LoginRequest {
        form: Some(form(FormMethod::ApiKey)),
        ..LoginRequest::default()
    };
    let (_, r) = log_in(&mut Fake::new(), &req, &mut NoTerminal);
    assert_eq!(
        r.unwrap_err().to_string(),
        "the API key needs client_id and client_secret"
    );

    let mut f = form(FormMethod::Totp);
    f.password = Sensitive(String::new());
    let req = LoginRequest {
        form: Some(f),
        ..LoginRequest::default()
    };
    let (_, r) = log_in(&mut Fake::new().locked(EU, "k@x.eu"), &req, &mut NoTerminal);
    assert_eq!(r.unwrap_err().to_string(), "the vault is locked");
}

#[test]
fn the_form_with_a_wrong_password_keeps_bws_words_as_the_cause() {
    let mut f = form(FormMethod::Totp);
    f.password = Sensitive("wrong".into());
    let req = LoginRequest {
        form: Some(f),
        ..LoginRequest::default()
    };

    let (_, r) = log_in(&mut Fake::new(), &req, &mut NoTerminal);

    let Err(Error::Login { cause, .. }) = r else {
        panic!("not a login error")
    };
    assert!(cause.detail().contains("Username or password is incorrect"));
}

#[test]
fn the_forms_server_counts_only_when_logged_out() {
    let mut m = Fake::new()
        .locked(EU, "k@x.eu")
        .file(CONFIG, "server=https://vault.bitwarden.eu\n");
    let mut f = form(FormMethod::Totp);
    f.server = "com".into();
    let req = LoginRequest {
        form: Some(f),
        ..LoginRequest::default()
    };

    log_in(&mut m, &req, &mut NoTerminal).1.unwrap();

    assert_eq!(m.server.as_deref(), Some(EU));
}

#[test]
fn logging_out_then_in_switches_the_server() {
    let mut m = Fake::new()
        .locked(EU, "k@x.eu")
        .file(CONFIG, "server=https://vault.bitwarden.eu\n");

    logout(&mut m).unwrap();
    let mut f = form(FormMethod::Totp);
    f.server = "https://vault.example.org".into();
    let req = LoginRequest {
        form: Some(f),
        ..LoginRequest::default()
    };
    log_in(&mut m, &req, &mut NoTerminal).1.unwrap();

    assert_eq!(m.server.as_deref(), Some("https://vault.example.org"));
    assert!(m.text(CONFIG).unwrap().contains("server=https://vault.example.org\n"));
}

// ----------------------------------------------------------- preflight

#[test]
fn preflight_names_the_missing_program() {
    let mut m = Fake::new();
    m.installed = false;
    assert_eq!(
        preflight(&mut m, Parts::ALL).unwrap_err().to_string(),
        "bw not found: run system/70-bitwarden.sh"
    );

    let mut m = Fake::new();
    m.programs.remove("secret-tool");
    assert!(preflight(&mut m, Parts::ALL).is_err());
    assert!(preflight(&mut m, ssh_only()).is_ok());
}

#[test]
fn parts_parse_the_list_and_refuse_a_stranger() {
    assert_eq!(Parts::parse("ssh,git,gh,env").unwrap(), Parts::ALL);
    assert!(Parts::parse("ssh,gpg").is_err());
}

// ------------------------------------------------------------ the parts

#[test]
fn slug_makes_a_file_name_from_an_item_name() {
    assert_eq!(ssh::slug("GitHub (work)"), "github-work");
    assert_eq!(ssh::slug("Work GitHub"), "work-github");
    assert_eq!(ssh::slug("..Build  server!"), "build-server");
    assert_eq!(ssh::slug("Zażółć"), "za");
    assert_eq!(ssh::slug("!!!"), "key");
}

#[test]
fn a_public_key_is_one_known_type_and_base64() {
    let key = |k| ssh::public_key(k).ok();
    assert_eq!(
        key("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB+/=").as_deref(),
        Some("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB+/=")
    );
    assert_eq!(
        key("ecdsa-sha2-nistp256 AAAA").as_deref(),
        Some("ecdsa-sha2-nistp256 AAAA")
    );
    assert_eq!(
        key("ssh-ed25519 AAAA jakub@macbook pro").as_deref(),
        Some("ssh-ed25519 AAAA")
    );
    assert_eq!(key("ssh-ed25519  AAAA ").as_deref(), Some("ssh-ed25519 AAAA"));
    assert_eq!(key("ssh-dss AAAA"), None);
    assert_eq!(key("ecdsa-sha2-nistp AAAA"), None);
    assert_eq!(key("ssh-ed25519 AA*A"), None);
    assert_eq!(key("ssh-ed25519"), None);
    assert_eq!(key("ssh-ed25519 AAAA\nProxyCommand x"), None);
    assert_eq!(key("ssh-ed25519 AAAA\r\n").as_deref(), Some("ssh-ed25519 AAAA"));
    assert_eq!(key("ssh-ed25519 AAAA me\tthere"), None);
}

#[test]
fn folders_reads_the_list_and_falls_back_to_the_older_key() {
    assert_eq!(
        Config::parse("folders=Personal, Work,,Archive \n").folders(),
        ["Personal", "Work", "Archive"]
    );
    assert_eq!(Config::parse("folder=Work\n").folders(), ["Work"]);
    assert_eq!(Config::parse("folder=Old\nfolders=New\n").folders(), ["New"]);
    assert!(
        Config::parse("server=https://vault.bitwarden.eu\n")
            .folders()
            .is_empty()
    );
}

#[test]
fn set_replaces_the_key_and_keeps_the_rest_in_order() {
    let mut c = Config::parse("server=a\nemail=k@x.eu\n");

    c.set("server", "b");

    assert_eq!(c.render(), "email=k@x.eu\nserver=b\n");
}

#[test]
fn the_session_and_the_form_never_print_their_secrets() {
    let f = form(FormMethod::Totp);
    let text = format!("{:?} {f:?}", Session::new(SESSION));
    assert!(!text.contains(SESSION) && !text.contains(PASSWORD));
}
