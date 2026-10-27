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
