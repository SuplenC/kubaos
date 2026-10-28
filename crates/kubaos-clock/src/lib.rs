//! KubaOS's date engine, offline: date search and maths, the month calendar with holidays, world
//! clocks and countdowns. The clock popup reaches it through `kubaos-clock serve`, a long-lived
//! process speaking a line of JSON each way; the Command menu's KubaOS Extension links it.
//!
//! ```text
//! bar clock ─click─▶ ClockPopup.qml ─{"op":"query",...}─▶ kubaos-clock serve ─▶ query, calendar
//!                         ▲                                       │
//!                         └──────────── {"id":1,"answer":...} ◀───┘
//! menu ─"days until christmas"─▶ kotodama-kubaos ─▶ query::evaluate ─▶ the answer row
//! ```
//!
//! # Owns
//!
//! - Reading queries ([`query`]), the calendar's months and days ([`calendar`]), holidays
//!   ([`holidays`]), zone names ([`zones`]), and the settings in the user layer ([`settings`]).
//! - The popup's protocol ([`serve`]).
//!
//! # Not here
//!
//! - The popup itself: `config/quickshell/modules/clock/`. The menu's answers and Items:
//!   `crates/kotodama-kubaos/src/clock.rs` and `config/kotodama/kubaos.toml`.

pub mod calendar;
pub mod holidays;
mod parse;
pub mod query;
pub mod serve;
pub mod settings;
mod words;
pub mod zones;

pub use self::parse::alarm;
pub use self::parse::duration;
pub use self::query::Answer;
pub use self::query::Context;
pub use self::query::evaluate;
