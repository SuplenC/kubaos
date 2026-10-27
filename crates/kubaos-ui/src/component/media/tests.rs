use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced::widget::image;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::Aspect;
use super::media;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens::Mode;
use crate::tokens::Radius;

/// The bounds of `element`, laid out alone in a column `width` wide.
fn bounds(element: Element<'_, ()>, width: f32) -> Rectangle {
    let probe = widget::container(element).width(width).id("probe");
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(800.0, 800.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

/// A picture of another shape than the box, so a box sized by its picture would show it.
fn square_picture() -> image::Handle {
    image::Handle::from_rgba(4, 4, vec![255; 4 * 4 * 4])
}

#[test]
fn the_box_is_the_same_before_and_after_the_picture_loads() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);

        let before = bounds(media(&look, None, Aspect::Wallpaper, Radius::Control), 256.0);
        let after = bounds(
            media(&look, Some(square_picture()), Aspect::Wallpaper, Radius::Control),
            256.0,
        );

        assert_eq!(before, after, "{scale:?}");
    }
}

#[test]
fn the_box_keeps_its_aspect() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let frame = bounds(media(&look, None, Aspect::Wallpaper, Radius::Control), 256.0);

    assert!(
        (frame.width - 256.0).abs() < 1e-4 && (frame.height - 160.0).abs() < 1e-4,
        "{frame:?}"
    );
}

#[test]
fn the_box_ends_on_a_device_pixel_at_every_scale() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);

        let frame = bounds(media(&look, None, Aspect::Wallpaper, Radius::Control), 250.0);

        assert!(scale.is_whole(frame.height), "{scale:?}: {frame:?}");
    }
}

/// Where the box after `media` in a column 256 wide is laid out.
fn next_in_column(media: Element<'_, ()>) -> Rectangle {
    let next = widget::container(widget::Space::new().width(10).height(10)).id("next");
    let column = widget::column![media, next].width(256.0);
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(800.0, 800.0), column);
    ui.find(selector::id("next")).expect("next").bounds()
}

#[test]
fn what_follows_the_picture_stays_put_when_it_loads() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);

        let before = next_in_column(media(&look, None, Aspect::Wallpaper, Radius::Control));
        let after = next_in_column(media(&look, Some(square_picture()), Aspect::Wallpaper, Radius::Control));

        assert_eq!(before, after, "{scale:?}");
    }
}

#[test]
fn a_box_with_no_end_across_takes_a_thumbnails_width() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let row: widget::Row<'_, ()> =
        widget::row![widget::container(media(&look, None, Aspect::Wallpaper, Radius::Control)).id("probe")];
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        Size::new(800.0, 800.0),
        widget::scrollable(row).direction(widget::scrollable::Direction::Horizontal(
            widget::scrollable::Scrollbar::default(),
        )),
    );

    let frame = ui.find(selector::id("probe")).expect("probe").bounds();

    assert!(
        frame.height.is_finite() && (frame.height - 160.0).abs() < 1e-4,
        "{frame:?}"
    );
}
