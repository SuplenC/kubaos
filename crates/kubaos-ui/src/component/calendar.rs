//! Calendars: a month as six weeks, each led by its ISO week number, or the year's twelve months.
//!
//! A day is a control as tall as a regular button, its disc a small one. Today is filled with the
//! accent, the picked day ringed with it, a holiday dotted, and the days of the months around
//! dimmed. The title between the arrows opens the year.
//!
//! # Owns
//!
//! - [`calendar`], built up with [`Calendar`], and the month it shows: [`Month`], [`Week`],
//!   [`Day`].
//!
//! # Not here
//!
//! - Working the month out (week numbers, the first weekday, holidays): the app's, as
//!   `kubaos-clock` does for the shell.

use std::fmt;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::alignment;
use iced::widget;

use crate::color;
use crate::component::button::ButtonSize;
use crate::component::button::Kind;
use crate::component::button::icon_button;
use crate::component::layered;
use crate::icon::Icon;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::fill;
use crate::layout::grid;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::centre;
use crate::snap::centre_x;
use crate::snap::snap;
use crate::text::text;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// A day's cell: a regular control, so a day is as easy to hit as a button.
const CELL: Size = Size::ControlMd;
/// The disc inside a day's cell: a small control, a pill.
const DISC: Size = Size::ControlSm;
/// The week numbers' column: a small control wide.
const WEEK_COLUMN: Size = Size::ControlSm;
/// The weekday names' row.
const NAMES: Space = Space::S24;
/// A holiday's dot under its day's number.
const DOT: Space = Space::S4;

/// The months' names in the year view.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A month as the calendar lays it out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Month {
    pub year: i32,
    /// 1 to 12.
    pub month: u8,
    /// What the title reads: "October 2026".
    pub title: String,
    /// The weekdays' names, from the first weekday: "Mon", "Tue"…
    pub weekdays: [String; 7],
    /// Six weeks, so every month is as tall as every other.
    pub weeks: Vec<Week>,
}

/// One week of a [`Month`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Week {
    /// Its ISO week number.
    pub number: u32,
    pub days: [Day; 7],
}

/// One day of a [`Week`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Day {
    /// The day as the app names it to pick it: "2026-10-27".
    pub date: String,
    /// Its number in its month.
    pub day: u8,
    /// Whether it is in the month shown, rather than the one before or after.
    pub in_month: bool,
    pub today: bool,
    pub weekend: bool,
    /// The holiday's name, if it is one.
    pub holiday: Option<String>,
}

/// A calendar showing `month`, built up then drawn with [`Calendar::view`].
pub struct Calendar<'a, M> {
    month: &'a Month,
    selected: Option<&'a str>,
    year: bool,
    on_pick: Option<Box<dyn Fn(String) -> M + 'a>>,
    on_move: Option<Box<dyn Fn(i32) -> M + 'a>>,
    on_title: Option<M>,
    on_month: Option<Box<dyn Fn(u8) -> M + 'a>>,
}

impl<M> fmt::Debug for Calendar<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Calendar")
            .field("month", &self.month.title)
            .field("selected", &self.selected)
            .field("year", &self.year)
            .finish_non_exhaustive()
    }
}

/// `month`'s days, none picked.
pub fn calendar<M>(month: &Month) -> Calendar<'_, M> {
    Calendar {
        month,
        selected: None,
        year: false,
        on_pick: None,
        on_move: None,
        on_title: None,
        on_month: None,
    }
}

impl<'a, M: Clone + 'a> Calendar<'a, M> {
    /// The picked day, by its [`Day::date`]: ringed, unless it is today.
    #[must_use]
    pub fn selected(self, date: &'a str) -> Self {
        Calendar {
            selected: Some(date),
            ..self
        }
    }

    /// The year's twelve months in place of the month's days.
    #[must_use]
    pub fn year(self, year: bool) -> Self {
        Calendar { year, ..self }
    }

    /// What a press on a day sends, with its date.
    #[must_use]
    pub fn on_pick(self, on_pick: impl Fn(String) -> M + 'a) -> Self {
        Calendar {
            on_pick: Some(Box::new(on_pick)),
            ..self
        }
    }

    /// What the arrows send: how many months to move, a year at a time in the year view.
    #[must_use]
    pub fn on_move(self, on_move: impl Fn(i32) -> M + 'a) -> Self {
        Calendar {
            on_move: Some(Box::new(on_move)),
            ..self
        }
    }

    /// What a press on the title sends: the year opens, or closes.
    #[must_use]
    pub fn on_title(self, message: M) -> Self {
        Calendar {
            on_title: Some(message),
            ..self
        }
    }

    /// What a press on a month of the year view sends, with its number, 1 to 12.
    #[must_use]
    pub fn on_month(self, on_month: impl Fn(u8) -> M + 'a) -> Self {
        Calendar {
            on_month: Some(Box::new(on_month)),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let width = f32::from(WEEK_COLUMN.px() + CELL.px() * 7);
        let body_height = f32::from(NAMES.px() + CELL.px() * 6);
        let header = self.header(look);
        let body: Element<'a, M> = if self.year {
            centre(look.scale, self.months(look))
        } else {
            self.days(look)
        };
        let view = stack(look, Space::S8)
            .push(header)
            .push(widget::container(body).width(Length::Fill).height(body_height));
        let view = widget::container(view).width(width);
        probe::component(
            "Calendar",
            [if self.year { "year" } else { "" }, "", ""],
            Fixed::BOTH,
            view,
        )
    }

    /// The arrows at either end and the title between them, a small control tall.
    fn header(&self, look: &Look) -> Element<'a, M> {
        let (back, forward, step) = if self.year {
            ("Last year", "Next year", 12)
        } else {
            ("Last month", "Next month", 1)
        };
        let arrow = |glyph: Icon, label: &'static str, months: i32| {
            icon_button(glyph, label)
                .kind(Kind::Text)
                .size(ButtonSize::Sm)
                .on_press_maybe(self.on_move.as_ref().map(|on_move| on_move(months)))
                .view(look)
        };
        let arrows = row(look, Gap::NONE)
            .center()
            .push(arrow(Icon::ChevronLeft, back, -step))
            .push(fill())
            .push(arrow(Icon::ChevronRight, forward, step));
        let words = if self.year {
            self.month.year.to_string()
        } else {
            self.month.title.clone()
        };
        let c = look.colors;
        let (pill, ink) = if self.year {
            (Some(c.secondary_container), c.on_secondary_container)
        } else {
            (None, c.on_surface)
        };
        let enabled = self.on_title.is_some();
        let title = widget::button(snap(look.scale, text(words, Type::Subheading, ink)))
            .height(f32::from(Size::ControlSm.px()))
            .padding(Inset::xy(Space::S12, Space::S4).padding())
            .on_press_maybe(self.on_title.clone())
            .style(face(pill, ink));
        let title = layered(title, Radius::Full, ink, enabled, None);
        widget::stack![arrows, centre_x(look.scale, title)].into()
    }

    /// The weekdays' names, then the weeks, a row each.
    fn days(&self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let mut names = row(look, Gap::NONE).push(cell(
            look,
            text("Wk", Type::Caption, c.on_surface_variant),
            WEEK_COLUMN,
            NAMES.px(),
        ));
        for name in &self.month.weekdays {
            let weekend = name.starts_with("Sat") || name.starts_with("Sun");
            let ink = if weekend { c.tertiary } else { c.on_surface_variant };
            let short: String = name.chars().take(2).collect();
            names = names.push(cell(look, text(short, Type::Label, ink), CELL, NAMES.px()));
        }
        let mut weeks = stack(look, Gap::NONE);
        for week in &self.month.weeks {
            let mut line = row(look, Gap::NONE).push(cell(
                look,
                text(week.number.to_string(), Type::Caption, c.on_surface_variant),
                WEEK_COLUMN,
                CELL.px(),
            ));
            for day in &week.days {
                line = line.push(self.day(look, day));
            }
            weeks = weeks.push(line);
        }
        stack(look, Gap::NONE).push(names).push(weeks).into()
    }

    /// A day's cell: its disc 4 in, the number centred on it.
    fn day(&self, look: &Look, day: &Day) -> Element<'a, M> {
        let c = look.colors;
        let dim = if day.in_month {
            1.0
        } else {
            Opacity::DisabledContent.value()
        };
        let picked = self.selected == Some(day.date.as_str()) && !day.today;
        let (disc, ink, style) = if day.today {
            (Some(c.primary), c.on_primary, Type::BodyStrong)
        } else {
            let ink = if day.holiday.is_some() {
                c.tertiary
            } else if day.weekend {
                c.on_surface_variant
            } else {
                c.on_surface
            };
            (None, ink, Type::Body)
        };
        let number = centred(look, text(day.day.to_string(), style, color::alpha(ink, dim)));
        let mut layers = vec![Element::from(number)];
        if day.holiday.is_some() {
            let dot_ink = color::alpha(if day.today { c.on_primary } else { c.tertiary }, dim);
            let dot = widget::container(widget::Space::new())
                .width(f32::from(DOT.px()))
                .height(f32::from(DOT.px()))
                .style(move |_theme| widget::container::Style {
                    background: Some(Background::Color(dot_ink)),
                    border: Border::default().rounded(radius(Radius::Full)),
                    ..widget::container::Style::default()
                });
            layers.push(
                widget::container(centre_x(look.scale, dot))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_y(alignment::Vertical::Bottom)
                    .padding(Inset::NONE.bottom(DOT).padding())
                    .into(),
            );
        }
        let ring = picked.then_some(c.primary);
        let thin = look.scale.thin();
        let side = f32::from(DISC.px());
        let pressable = widget::button(widget::Stack::with_children(layers))
            .width(side)
            .height(side)
            .padding(0)
            .on_press_maybe(self.on_pick.as_ref().map(|on_pick| on_pick(day.date.clone())))
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: disc.map(Background::Color),
                text_color: ink,
                border: Border {
                    color: ring.unwrap_or(Color::TRANSPARENT),
                    width: if ring.is_some() { thin } else { 0.0 },
                    radius: radius(Radius::Full).into(),
                },
                shadow: Shadow::default(),
                snap: true,
            });
        let enabled = self.on_pick.is_some();
        let content = if day.today { c.on_primary } else { c.on_surface };
        widget::container(layered(pressable, Radius::Full, content, enabled, None))
            .padding(Inset::all(Space::S4).padding())
            .width(f32::from(CELL.px()))
            .height(f32::from(CELL.px()))
            .into()
    }

    /// The year's twelve months, three to a row, the one shown filled.
    fn months(&self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let mut months = grid(look, 3, Space::S8);
        for (index, name) in (1_u8..).zip(MONTHS) {
            let current = index == self.month.month;
            let (fill, ink) = if current {
                (Some(c.primary), c.on_primary)
            } else {
                (Some(c.surface_container_high), c.on_surface)
            };
            let pressable = widget::button(centred(look, text(name, Type::BodyStrong, ink)))
                .width(Length::Fill)
                .height(f32::from(Size::ControlMd.px()))
                .padding(0)
                .on_press_maybe(self.on_month.as_ref().map(|on_month| on_month(index)))
                .style(move |_theme: &Theme, _status| widget::button::Style {
                    background: fill.map(Background::Color),
                    text_color: ink,
                    border: Border::default().rounded(radius(Radius::Control)),
                    shadow: Shadow::default(),
                    snap: true,
                });
            months = months.push(layered(pressable, Radius::Control, ink, self.on_month.is_some(), None));
        }
        months.into()
    }
}

/// `content` centred in a box `width` wide and `height` tall.
fn cell<'a, M: 'a>(look: &Look, content: Element<'a, M>, width: Size, height: u16) -> Element<'a, M> {
    widget::container(centred(look, content))
        .width(f32::from(width.px()))
        .height(f32::from(height))
        .into()
}

/// `content` in the middle of its box: across as a stack centres, an odd spare device pixel after
/// it, so a number lands on the same pixel in both toolkits; down to the nearest device pixel.
fn centred<'a, M: 'a>(look: &Look, content: Element<'a, M>) -> Element<'a, M> {
    centre(look.scale, centre_x(look.scale, content))
}

/// A pill's style: `fill`, or none, whatever the pointer does; the state layer goes over it.
fn face(fill: Option<Color>, ink: Color) -> impl Fn(&Theme, widget::button::Status) -> widget::button::Style {
    move |_theme, _status| widget::button::Style {
        background: fill.map(Background::Color),
        text_color: ink,
        border: Border::default().rounded(radius(Radius::Full)),
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests;
