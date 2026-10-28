//! The components page's "Calendars" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Calendars.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use std::sync::LazyLock;

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::calendar::Day;
use kubaos_ui::component::calendar::Month;
use kubaos_ui::component::calendar::Week;
use kubaos_ui::component::calendar::calendar;
use kubaos_ui::layout::row;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Calendars",
    lede: Some(
        "A month as six weeks led by their week numbers, today filled and the picked day ringed; the title opens the year.",
    ),
    view,
};

/// The specimen's October 2026, weeks from Monday, as the shots' clock has it: Tuesday the 27th
/// is today, and All Saints' Day falls in the next month's first week.
fn october() -> Month {
    let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(str::to_owned);
    let weeks = (0..6_u8)
        .map(|week| Week {
            number: 40 + u32::from(week),
            days: std::array::from_fn(|weekday| {
                let weekday = u8::try_from(weekday).expect("a week has seven days");
                let index = week * 7 + weekday;
                // Monday the 28th of September starts the grid.
                let (month, day) = match index {
                    0..=2 => (9, 28 + index),
                    3..=33 => (10, index - 2),
                    _ => (11, index - 33),
                };
                Day {
                    date: format!("2026-{month:02}-{day:02}"),
                    day,
                    in_month: month == 10,
                    today: month == 10 && day == 27,
                    weekend: weekday >= 5,
                    holiday: (month == 11 && day == 1).then(|| "All Saints' Day".to_owned()),
                }
            }),
        })
        .collect();
    Month {
        year: 2026,
        month: 10,
        title: "October 2026".to_owned(),
        weekdays: names,
        weeks,
    }
}

static OCTOBER: LazyLock<Month> = LazyLock::new(october);

fn view<'a>(look: &Look) -> Element<'a, Message> {
    row(look, Space::S48)
        .push(
            calendar(&OCTOBER)
                .selected("2026-10-30")
                .on_pick(|_| Message::Sample)
                .on_move(|_| Message::Sample)
                .on_title(Message::Sample)
                .view(look),
        )
        .push(
            calendar(&OCTOBER)
                .year(true)
                .on_move(|_| Message::Sample)
                .on_title(Message::Sample)
                .on_month(|_| Message::Sample)
                .view(look),
        )
        .into()
}
