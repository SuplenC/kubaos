//! The vault as `bw list folders` and `bw list items` print it, cut down at once to what a pull
//! uses: public keys, fingerprints, custom fields and the gh token. SSH private keys, logins'
//! passwords and notes are never read out of the JSON.

use serde::Deserialize;

use super::Progress;
use super::Sensitive;
use super::list;
use super::one_line;

/// The SSH key item type in `bw list items`.
const SSH_KEY: u8 = 5;
/// A linked custom field: it points at another field and carries no value of its own.
const LINKED: u8 = 3;

#[derive(Debug, Clone, Deserialize)]
pub struct Folder {
    /// `None` for bw's "No Folder" entry.
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
}

/// `bw list folders`.
pub fn folders(json: &str) -> Result<Vec<Folder>, serde_json::Error> {
    serde_json::from_str(json)
}

/// One custom field. The value is `tostring`'d the way bw's JSON would read in a shell: a
/// boolean field is `"true"`.
#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub value: Option<String>,
    pub kind: u8,
}

impl Field {
    /// Whether the field is `name` as people type it in Bitwarden's editor: ssh's own spelling
    /// is `HostName`, and a trailing space is invisible there.
    pub fn is(&self, name: &str) -> bool {
        self.name.trim().eq_ignore_ascii_case(name)
    }
}

/// One item, cut down.
#[derive(Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub kind: u8,
    pub folder_id: Option<String>,
    /// The folder's name, "no folder" for none.
    pub folder: String,
    pub created: String,
    pub fields: Vec<Field>,
    pub public_key: Option<String>,
    pub fingerprint: Option<String>,
    /// Only on the item named `gh-token`: its password, else its `token` field.
    pub token: Option<Sensitive>,
}

impl Item {
    /// The first custom field named `name`, "" when it has none.
    pub fn field(&self, name: &str) -> &str {
        self.fields
            .iter()
            .find(|f| f.is(name))
            .and_then(|f| f.value.as_deref())
            .unwrap_or("")
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f.is(name))
    }

    /// The name and the folder, which tell two keys of the same name apart. It goes into
    /// comments in ssh_config, so it is kept to one line.
    pub fn label(&self) -> String {
        one_line(&format!("{} ({})", self.name, self.folder))
    }

    /// The custom fields that can become env variables: not `bwenv`, not linked, with a value.
    pub fn env_fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields
            .iter()
            .filter(|f| f.name != "bwenv" && f.kind != LINKED)
            .filter_map(|f| Some((f.name.as_str(), f.value.as_deref()?)))
    }
}

/// The whole vault, cut down.
pub struct Vault {
    pub items: Vec<Item>,
    pub folders: Vec<Folder>,
}

#[derive(Deserialize)]
struct RawItem {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    kind: u8,
    #[serde(default, rename = "folderId")]
    folder_id: Option<String>,
    #[serde(default, rename = "creationDate")]
    created: Option<String>,
    #[serde(default)]
    fields: Option<Vec<RawField>>,
    #[serde(default, rename = "sshKey")]
    ssh_key: Option<RawSshKey>,
    #[serde(default)]
    login: Option<RawLogin>,
}

#[derive(Deserialize)]
struct RawField {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    value: Option<serde_json::Value>,
    #[serde(default, rename = "type")]
    kind: u8,
}

/// Only the public half: serde skips `privateKey` without keeping it.
#[derive(Deserialize)]
struct RawSshKey {
    #[serde(default, rename = "publicKey")]
    public_key: Option<String>,
    #[serde(default, rename = "keyFingerprint")]
    fingerprint: Option<String>,
}

#[derive(Deserialize)]
struct RawLogin {
    #[serde(default)]
    password: Option<String>,
}

fn tostring(v: serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Null => None,
        serde_json::Value::String(s) => Some(s),
        other => Some(other.to_string()),
    }
}

impl Vault {
    /// `bw list items`, with the folders' names from `bw list folders`.
    pub fn parse(items: &str, folders: &[Folder]) -> Result<Vault, serde_json::Error> {
        let raw: Vec<RawItem> = serde_json::from_str(items)?;
        let folder_name = |id: &Option<String>| {
            id.as_ref()
                .and_then(|id| folders.iter().find(|f| f.id.as_ref() == Some(id)))
                .map_or_else(|| "no folder".to_string(), |f| f.name.clone())
        };
        let items = raw
            .into_iter()
            .map(|r| {
                let fields: Vec<Field> = r
                    .fields
                    .unwrap_or_default()
                    .into_iter()
                    .map(|f| Field {
                        name: f.name.unwrap_or_default(),
                        value: f.value.and_then(tostring),
                        kind: f.kind,
                    })
                    .collect();
                let token = (r.name == "gh-token")
                    .then(|| {
                        r.login.and_then(|l| l.password).or_else(|| {
                            fields
                                .iter()
                                .find(|f| f.is("token"))
                                .and_then(|f| f.value.clone())
                        })
                    })
                    .flatten()
                    .map(Sensitive);
                let (public_key, fingerprint) = r
                    .ssh_key
                    .map(|k| (k.public_key, k.fingerprint))
                    .unwrap_or_default();
                Item {
                    folder: folder_name(&r.folder_id),
                    id: r.id,
                    name: r.name,
                    kind: r.kind,
                    folder_id: r.folder_id,
                    created: r.created.unwrap_or_default(),
                    fields,
                    public_key,
                    fingerprint,
                    token,
                }
            })
            .collect();
        Ok(Vault {
            items,
            folders: folders.to_vec(),
        })
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// The SSH key items in the filter's folders, oldest first (ties by id), so of two keys
    /// with the same name the older keeps the plain file name.
    pub fn ssh_keys(&self, filter: &Filter) -> Vec<Item> {
        let mut keys: Vec<Item> = self
            .items
            .iter()
            .filter(|i| i.kind == SSH_KEY)
            .filter(|i| filter.ids.is_empty() || i.folder_id.as_ref().is_some_and(|f| filter.ids.contains(f)))
            .cloned()
            .collect();
        keys.sort_by(|a, b| (&a.created, &a.id).cmp(&(&b.created, &b.id)));
        keys
    }

    /// The items named `gh-token` that carry a token, in the vault's order.
    pub fn gh_tokens(&self) -> Vec<&Item> {
        self.items.iter().filter(|i| i.token.is_some()).collect()
    }
}

/// Where the SSH keys come from: folder ids (none for the whole vault) and how messages name it.
#[derive(Debug, Clone)]
pub struct Filter {
    pub ids: Vec<String>,
    pub scope: String,
}

impl Filter {
    /// The ids of the listed folders. A listed folder the vault lacks gets a warning; with none
    /// of them there, the whole vault is used.
    pub fn pick(folders: &[Folder], wanted: &[String], p: &mut Progress) -> Filter {
        let mut ids = vec![];
        let mut found = vec![];
        for name in wanted {
            let matching: Vec<String> = folders
                .iter()
                .filter(|f| &f.name == name)
                .filter_map(|f| f.id.clone())
                .collect();
            if matching.is_empty() {
                p.warn(format!(
                    "no folder \"{}\" in the vault (folders= in ~/.config/dot-bw/config)",
                    one_line(name)
                ));
            } else {
                found.push(name.clone());
                ids.extend(matching);
            }
        }
        if !wanted.is_empty() && found.is_empty() {
            p.warn("none of the folders in ~/.config/dot-bw/config is in the vault: using every SSH key in the vault");
        }
        Filter {
            ids,
            scope: scope(&found),
        }
    }
}

/// How messages name where the SSH keys come from: "your vault", `the folder "Work"`, or
/// `the folders "Personal", "Work"`.
pub fn scope(folders: &[String]) -> String {
    match folders {
        [] => "your vault".to_string(),
        [one] => format!("the folder \"{}\"", one_line(one)),
        more => {
            let quoted: Vec<String> = more.iter().map(|n| format!("\"{}\"", one_line(n))).collect();
            format!("the folders {}", list(&quoted))
        }
    }
}
