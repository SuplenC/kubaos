//! What the power stop does and which CPU scheduler runs, as Views read live when they are shown:
//! the stop now, with every layer it sets as a row, what it wants and what the machine says; the
//! scheduler that runs, why, and what is behind it. The power profile's and the CPU scheduler's
//! Items show them (`view = "power"`, `view = "cpu-scheduler"`).
//!
//! # Owns
//!
//! - The two Views.
//!
//! # Not here
//!
//! - The rows: `kubaos_power::status`, read by `kubaos_power::live`; the scheduler's report:
//!   `kubaos_power::scheduler`. The Items, with the stops, the charge limit, the low-battery drop,
//!   LAVD on and off and the measurement: actions of the registry (`config/kotodama/kubaos.toml`),
//!   which run `kubaos-power`.

use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;
use kubaos_power::live::Now;
use kubaos_power::scheduler::Report;
use kubaos_power::status::Row;

/// The Item kind of a row in either View.
pub const ROW_KIND: &str = "kubaos.power-row";
/// The id of the power stop's View.
pub const VIEW_ID: &str = "power-status";
/// The id of the CPU scheduler's View.
pub const SCHEDULER_VIEW_ID: &str = "cpu-scheduler-status";

const ICON: &str = "battery";
const SCHEDULER_ICON: &str = "cpu";

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
        weak_keywords: Vec::new(),
        yields: false,
        actions: Vec::new(),
    };
    let running = row(
        format!("{SCHEDULER_VIEW_ID}:running"),
        format!("Running: {}", report.running),
        Some(report.why.clone()),
    );
    let details = report.details.iter().map(|(label, value)| {
        row(
            format!("{SCHEDULER_VIEW_ID}:{label}"),
            format!("{label}: {value}"),
            None,
        )
    });
    View {
        id: SCHEDULER_VIEW_ID.to_owned(),
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
            id: format!("{VIEW_ID}:{}", row.name),
            kind: kind(ROW_KIND),
            identity: None,
            title: format!("{}: {}", row.name, row.wanted),
            subtitle: Some(subtitle(row)),
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
    View {
        id: VIEW_ID.to_owned(),
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
