//! "Power: what this stop does" in the menu: the stop now, with every layer it sets as a row,
//! what it wants and what the machine says, read live when the Item runs.
//!
//! # Owns
//!
//! - The Item and its View.
//!
//! # Not here
//!
//! - The rows: `kubaos_power::status`, read by `kubaos_power::live`. The stops' own Items, the
//!   charge limit, the low-battery drop and the measurement: actions of the registry
//!   (`config/kotodama/kubaos.toml`), which run `kubaos-power`.

use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;
use kubaos_power::live::Now;
use kubaos_power::status::Row;

/// The Item kind of "Power: what this stop does".
pub const KIND: &str = "kubaos.power";
/// The Item kind of a row in its View.
pub const ROW_KIND: &str = "kubaos.power-row";
/// The Item's id, which `hide` names it by.
pub const ID: &str = "power-status";
/// Its one Action.
pub const SHOW: &str = "show";

const ICON: &str = "battery";

/// The Item.
pub fn item() -> Item {
    Item {
        id: ID.to_owned(),
        kind: kind(KIND),
        identity: None,
        title: "Power: what this stop does".to_owned(),
        subtitle: Some("Every layer of the power stop you're on, as the machine has it now".to_owned()),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: [
            "power status",
            "power stop",
            "power profile",
            "battery",
            "super saver",
            "max performance",
            "epp",
            "aspm",
            "vrr",
        ]
        .map(str::to_owned)
        .to_vec(),
        hidden: false,
        app_only: false,
        actions: vec![Action::new(SHOW, "Show")],
    }
}

/// Every id of an Item this module sends, which no action of the registry may take.
pub fn ids() -> [&'static str; 1] {
    [ID]
}

/// The View of `now`: a row a layer, what it wants as the title and what the machine says as the
/// subtitle, marked where the two differ.
pub fn view(now: &Now) -> View {
    let items = now
        .rows
        .iter()
        .map(|row| Item {
            id: format!("{ID}:{}", row.name),
            kind: kind(ROW_KIND),
            identity: None,
            title: format!("{}: {}", row.name, row.wanted),
            subtitle: Some(subtitle(row)),
            icon: Some(ICON.to_owned()),
            keyword: None,
            keywords: Vec::new(),
            hidden: false,
            app_only: false,
            actions: Vec::new(),
        })
        .collect();
    View {
        id: ID.to_owned(),
        title: now.header.lines().next().unwrap_or("Power").to_owned(),
        content: ViewContent::List {
            items,
            filters_itself: false,
        },
    }
}

fn subtitle(row: &Row) -> String {
    match (&row.now, row.agrees) {
        (None, _) => "Not on this machine".to_owned(),
        (Some(now), Some(false)) => format!("Now {now}, not what the stop sets"),
        (Some(now), _) => format!("Now {now}"),
    }
}

fn kind(name: &str) -> ItemKind {
    ItemKind::Namespaced(NamespacedKind::new(name).expect("the Extension's kinds are valid"))
}

#[cfg(test)]
mod tests;
