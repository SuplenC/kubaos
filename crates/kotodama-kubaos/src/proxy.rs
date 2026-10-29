//! "Proxy status" in the menu: the work network's switch, each part of the profile and whether it
//! is up, and the VPN's type, server, last handshake and bytes moved, read live when the Item runs.
//!
//! # Owns
//!
//! - The Item and its View.
//!
//! # Not here
//!
//! - The rows: `kubaos_proxy::status`. Proxy on and off, importing IT's VPN config and opening the
//!   profile: actions of the registry (`config/kotodama/kubaos.toml`), which run `kubaos-proxy`.

use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;
use kubaos_proxy::status::Row;

/// The Item kind of "Proxy status".
pub const KIND: &str = "kubaos.proxy";
/// The Item kind of a row in its View.
pub const ROW_KIND: &str = "kubaos.proxy-row";
/// The Item's id, which `hide` names it by.
pub const ID: &str = "proxy-status";
/// Its one Action.
pub const SHOW: &str = "show";

const ICON: &str = "network-vpn";

/// The Item.
pub fn item() -> Item {
    Item {
        id: ID.to_owned(),
        kind: kind(KIND),
        identity: None,
        title: "Proxy status".to_owned(),
        subtitle: Some("The work network: each part, whether it is up, and the VPN's last handshake".to_owned()),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: [
            "proxy",
            "vpn",
            "wireguard",
            "openvpn",
            "work network",
            "tunnel",
            "handshake",
            "kill switch",
            "dns",
        ]
        .map(str::to_owned)
        .to_vec(),
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        yields: false,
        actions: vec![Action::new(SHOW, "Show")],
    }
}

/// Every id of an Item this module sends, which no action of the registry may take.
pub fn ids() -> [&'static str; 1] {
    [ID]
}

/// The View of `rows`, the state first: a row each, its label the title and its value the
/// subtitle.
pub fn view(rows: &[Row]) -> View {
    let items = rows
        .iter()
        .enumerate()
        .map(|(index, row)| Item {
            id: format!("{ID}:{index}"),
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
        id: ID.to_owned(),
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
