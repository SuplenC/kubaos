//! The Bitwarden card's own parts: the vault's folders to pull from, and the SSH keys the last pull
//! found.

use kubaos_ui::Look;
use kubaos_ui::component::check::Tick;
use kubaos_ui::component::check::checkbox;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::flow;
use kubaos_ui::tokens::Space;

use super::notice;
use crate::app::Message;
use crate::setup::SF;
use crate::setup::Setup;
use crate::ui::El;

/// The SSH keys the last pull found, read-only: what each became, and what serves them, which
/// is nothing without Bitwarden desktop.
pub(super) fn keys_overview<'a>(
    folders: &[String],
    overview: &kubaos_core::bitwarden::Overview,
    desktop: bool,
    agent: Option<&kubaos_core::bitwarden::Agent>,
    look: &Look,
) -> El<'a> {
    use kubaos_core::bitwarden::Agent;
    let scope = kubaos_core::bitwarden::scope(folders);
    let mut lines: Vec<String> = overview
        .keys
        .iter()
        .map(|k| {
            let what = match (&k.note, k.block) {
                (_, true) => format!("Host block for {}", k.hosts.join(", ")),
                (Some(why), false) => format!("key only: {why}"),
                (None, false) => "key only".to_string(),
            };
            let signs = if k.signing { "; signs your commits" } else { "" };
            format!("{} ({}): {what}{signs}.", k.name, k.folder)
        })
        .collect();
    if lines.is_empty() {
        lines.push(format!("No SSH keys in {scope}."));
    }
    lines.push(match agent {
        None | Some(Agent::Off) if !desktop => {
            "Bitwarden desktop isn't installed, so nothing serves these keys: kubaos app add bitwarden.".into()
        }
        None | Some(Agent::Off) => {
            "The desktop app's SSH agent is off. In Bitwarden desktop: Settings, Enable SSH agent.".into()
        }
        Some(Agent::Locked) => "The agent runs with no keys: unlock Bitwarden desktop.".into(),
        Some(Agent::Serving { keys }) => format!("The agent serves {} key(s).", keys.len()),
        Some(Agent::Dead) => "The agent's socket doesn't answer: restart Bitwarden desktop.".into(),
    });
    lines.push(
        "A key gets a Host block from a text field hosts (comma-separated; hostname, user and port optional), and \
         signs commits with a field signing set to true."
            .into(),
    );
    notice(look, Icon::Key, "SSH keys", lines)
}

/// The vault's folders as checkboxes: none ticked is the whole vault.
pub(super) fn folder_ticks<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let focused = s.is_focused(SF::Folders) && !s.running();
    let boxes = s.bw_folders().into_iter().enumerate().map(|(i, name)| {
        let on = s.form.folders.iter().any(|f| f == name);
        checkbox(
            look,
            name.to_string(),
            if on { Tick::Full } else { Tick::Empty },
            (!s.running()).then(|| Message::Setup(super::Msg::Folder(name.to_string()))),
            focused && i == s.form.folder_at,
        )
    });
    flow(look, Space::S20, boxes)
}
