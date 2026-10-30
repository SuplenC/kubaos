//! The Proxy's status, as a View read live when it is shown: the work network's switch, each part
//! of the profile and whether it is up, and the VPN's type, server, last handshake and bytes moved.
//! The Proxy's Item shows it (`view = "proxy"`).
//!
//! # Owns
//!
//! - The View, and reading its rows from the machine.
//!
//! # Not here
//!
//! - The rows: `kubaos_proxy::status`. The Proxy's Item, with on and off, importing IT's VPN config
//!   and opening the profile: an action of the registry (`config/kotodama/kubaos.toml`), which runs
//!   `kubaos-proxy`.

use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;
use kubaos_proxy::status::Row;

/// The Item kind of a row in the View.
pub const ROW_KIND: &str = "kubaos.proxy-row";
/// The View's id.
pub const VIEW_ID: &str = "proxy-status";

const ICON: &str = "network-vpn";

/// The View of `rows`, the state first: a row each, its label the title and its value the
/// subtitle.
pub fn view(rows: &[Row]) -> View {
    let items = rows
        .iter()
        .enumerate()
        .map(|(index, row)| Item {
            id: format!("{VIEW_ID}:{index}"),
            kind: kind(ROW_KIND),
            identity: None,
            title: row.label.to_owned(),
            subtitle: Some(row.value.clone()),
            icon: Some(ICON.to_owned()),
            keyword: None,
            keywords: Vec::new(),
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: Vec::new(),
        })
        .collect();
    let title = rows
        .first()
        .map_or_else(|| "Proxy".to_owned(), |state| format!("Proxy: {}", state.value));
    View {
        id: VIEW_ID.to_owned(),
        title,
        content: ViewContent::List {
            items,
            filters_itself: false,
        },
    }
}

/// The rows as the machine has them now: the profile, what is up, and the VPN read live.
pub fn now() -> Vec<Row> {
    let Some(paths) = kubaos_proxy::Paths::from_env() else {
        return vec![Row {
            label: "State",
            value: "unknown: neither HOME nor XDG_CONFIG_HOME is set".to_owned(),
        }];
    };
    let profile = kubaos_proxy::Profile::load(&paths.network);
    let mut machine = kubaos_proxy::real::Real::new();
    let status = kubaos_proxy::status::read(&mut machine, &paths, &profile, true);
    kubaos_proxy::status::rows(&status, kubaos_proxy::status::now())
}

fn kind(name: &str) -> ItemKind {
    ItemKind::Namespaced(NamespacedKind::new(name).expect("the Extension's kinds are valid"))
}

#[cfg(test)]
mod tests;
