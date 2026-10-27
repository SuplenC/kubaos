use iced::Color;
use iced::Element;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::line;
use crate::tokens::Type;

/// How wide `content` is drawn as a line in `room` logical pixels.
fn drawn(content: &str, room: f32) -> f32 {
    let view: Element<'_, ()> =
        widget::container(widget::container(line(content, Type::Body, Color::BLACK)).id(widget::Id::new("line")))
            .width(room)
            .into();
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(800.0, 100.0), view);
    ui.find(selector::id("line")).expect("the line").bounds().width
}

#[test]
fn a_line_that_fits_is_as_wide_as_its_words() {
    let short = drawn("Disk", 400.0);

    assert!(short > 0.0 && short < 100.0, "{short}");
}

#[test]
fn a_line_too_long_for_its_room_is_cut_to_it() {
    let long = "WD_BLACK SN850X 4000GB with Heatsink, Gaming Edition, Firmware 620361WD";

    let cut = drawn(long, 120.0);

    assert!(cut <= 120.0 && cut > 60.0, "{cut}");
}
