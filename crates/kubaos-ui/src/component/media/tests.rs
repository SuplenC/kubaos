use iced::Element;
use iced::Pixels;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced::advanced::renderer;
use iced::advanced::renderer::Headless;
use iced::widget::image;
use iced_test::selector;
use iced_runtime::UserInterface;
use iced_runtime::user_interface;
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

/// `media`'s picture, a 4 by 4 red one, drawn by the software renderer in a box 64 wide at
/// (12, 8) on a black window 100 by 60, as RGBA rows and their width in device pixels.
fn drawn(scale: Scale) -> (Vec<u8>, u32) {
    let look = Look::new(Mode::Dark, scale);
    let red = image::Handle::from_rgba(4, 4, [255, 0, 0, 255].repeat(16));
    let picture = widget::container(media(&look, Some(red), Aspect::Wallpaper, Radius::Control)).width(64.0);
    let view: Element<'_, ()> = widget::container(picture)
        .padding(iced::Padding::ZERO.left(12.0).top(8.0))
        .into();
    let mut renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        iced::Font::DEFAULT,
        Pixels(14.0),
        Some("tiny-skia"),
    ))
    .expect("a tiny-skia renderer");
    let mut ui = UserInterface::build(view, Size::new(100.0, 60.0), user_interface::Cache::default(), &mut renderer);
    ui.draw(
        &mut renderer,
        &iced::Theme::Dark,
        &renderer::Style {
            text_color: iced::Color::WHITE,
        },
        iced::mouse::Cursor::Unavailable,
    );
    let physical = Size::new((100.0 * scale.factor()) as u32, (60.0 * scale.factor()) as u32);
    let rgba = renderer.screenshot(physical, scale.factor(), iced::Color::BLACK);
    (rgba, physical.width)
}

#[test]
fn the_picture_is_drawn_inside_its_box_with_rounded_corners() {
    for scale in Scale::SUPPORTED {
        let (rgba, width) = drawn(scale);
        // Whether the device pixel at a logical point is mostly red.
        let red = |x: f32, y: f32| {
            let (px, py) = ((x * scale.factor()) as usize, (y * scale.factor()) as usize);
            let at = (py * width as usize + px) * 4;
            rgba[at] > 200 && rgba[at + 1] < 60
        };

        // The box is 64 by 40 at (12, 8), its corners the control radius, 12.
        assert!(red(12.5, 28.0) && red(75.5, 28.0), "{scale:?}: the box's sides are drawn");
        assert!(red(44.0, 8.5) && red(44.0, 47.5), "{scale:?}: its top and foot are drawn");
        assert!(!red(11.0, 28.0) && !red(77.0, 28.0), "{scale:?}: nothing left or right of it");
        assert!(!red(44.0, 7.0) && !red(44.0, 49.0), "{scale:?}: nothing above or below it");
        assert!(!red(12.5, 8.5) && !red(75.5, 47.5), "{scale:?}: the corners are rounded");
    }
}
