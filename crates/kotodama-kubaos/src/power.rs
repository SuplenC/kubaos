//! "Power: what this stop does" in the menu: the stop now, with every layer it sets as a row,
//! what it wants and what the machine says, read live when the Item runs. "CPU scheduler: status"
//! beside it: which scheduler runs, why, and what is behind it.
//!
//! # Owns
//!
//! - The two Items and their Views.
//!
//! # Not here
//!
//! - The rows: `kubaos_power::status`, read by `kubaos_power::live`; the scheduler's report:
//!   `kubaos_power::scheduler`. The stops' own Items, the charge limit, the low-battery drop, LAVD
//!   on and off and the measurement: actions of the registry (`config/kotodama/kubaos.toml`),
//!   which run `kubaos-power`.

use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;
use kubaos_power::live::Now;
use kubaos_power::scheduler::Report;
use kubaos_power::status::Row;

/// The Item kind of "Power: what this stop does".
pub const KIND: &str = "kubaos.power";
/// The Item kind of a row in its View.
pub const ROW_KIND: &str = "kubaos.power-row";
/// The Item's id, which `hide` names it by.
pub const ID: &str = "power-status";
/// The one Action of either Item.
pub const SHOW: &str = "show";
/// The Item kind of "CPU scheduler: status".
pub const SCHEDULER_KIND: &str = "kubaos.cpu-scheduler";
/// Its id, which `hide` names it by.
pub const SCHEDULER_ID: &str = "cpu-scheduler-status";

const ICON: &str = "battery";
const SCHEDULER_ICON: &str = "cpu";

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

/// "CPU scheduler: status".
pub fn scheduler_item() -> Item {
    Item {
        id: SCHEDULER_ID.to_owned(),
        kind: kind(SCHEDULER_KIND),
        identity: None,
        title: "CPU scheduler: status".to_owned(),
        subtitle: Some("Which CPU scheduler runs, Linux's own or LAVD, and why".to_owned()),
        icon: Some(SCHEDULER_ICON.to_owned()),
        keyword: None,
        keywords: [
            "cpu scheduler",
            "scheduler status",
            "lavd",
            "scx",
            "sched_ext",
            "sched ext",
            "eevdf",
            "power stop",
        ]
        .map(str::to_owned)
        .to_vec(),
        hidden: false,
        app_only: false,
        actions: vec![Action::new(SHOW, "Show")],
    }
}

/// Every id of an Item this module sends, which no action of the registry may take.
pub fn ids() -> [&'static str; 2] {
    [ID, SCHEDULER_ID]
}

/// The View of the scheduler's `report`: which runs, with why under it, then the details, a line
/// each.
pub fn scheduler_view(report: &Report) -> View {
    let row = |id: String, title: String, subtitle: Option<String>| Item {
        id,
        kind: kind(ROW_KIND),
        identity: None,
        title,
        subtitle,
        icon: Some(SCHEDULER_ICON.to_owned()),
        keyword: None,
        keywords: Vec::new(),
        hidden: false,
        app_only: false,
        actions: Vec::new(),
    };
    let running = row(
        format!("{SCHEDULER_ID}:running"),
        format!("Running: {}", report.running),
        Some(report.why.clone()),
    );
    let details = report
        .details
        .iter()
        .map(|(label, value)| row(format!("{SCHEDULER_ID}:{label}"), format!("{label}: {value}"), None));
    View {
        id: SCHEDULER_ID.to_owned(),
        title: "CPU scheduler".to_owned(),
        content: ViewContent::List {
            items: std::iter::once(running).chain(details).collect(),
            filters_itself: false,
        },
    }
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
