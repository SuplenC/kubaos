use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::Day;
use super::Month;
use super::Week;
use super::calendar;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens::Mode;
use crate::tokens::Size as SizeToken;
use crate::tokens::Space;

fn settings() -> iced::Settings {
    iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    }
}

fn bounds(element: Element<'_, ()>) -> Rectangle {
    let probe = widget::container(element).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(800.0, 800.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

/// Six weeks of 1s, all in the month; enough to lay a month out.
fn month() -> Month {
    let day = Day {
        date: "2026-10-01".to_owned(),
        day: 1,
        in_month: true,
        today: false,
        weekend: false,
        holiday: Some("A holiday".to_owned()),
    };
    Month {
        year: 2026,
        month: 10,
        title: "October 2026".to_owned(),
        weekdays: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(str::to_owned),
        weeks: (40..46)
            .map(|number| Week {
                number,
                days: std::array::from_fn(|_| day.clone()),
            })
            .collect(),
    }
}

#[test]
fn the_month_and_the_year_take_the_same_box_at_every_scale() {
    let month = month();
    let width = f32::from(SizeToken::ControlSm.px() + SizeToken::ControlMd.px() * 7);
    let height =
        f32::from(SizeToken::ControlSm.px() + Space::S8.px() + Space::S24.px() + SizeToken::ControlMd.px() * 6);
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);

        let days = bounds(calendar(&month).selected("2026-10-01").on_pick(|_| ()).view(&look));
        let year = bounds(calendar(&month).year(true).on_month(|_| ()).view(&look));

        assert!(
            (days.width - width).abs() < 1e-4 && (days.height - height).abs() < 1e-4,
            "{scale:?}: {days:?}"
        );
        assert_eq!(days, year, "{scale:?}");
    }
}
