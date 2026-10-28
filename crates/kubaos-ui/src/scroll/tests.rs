use iced::Element;
use iced::Length;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::scroll;
use crate::Look;
use crate::Mode;
use crate::Scale;
use crate::tokens;
use crate::tokens::Space;

const GLOW: f32 = tokens::FOCUS_FAR as f32;
const GUTTER: f32 = Space::S16.px() as f32;

/// The box the content scrolls in, and the content's own box, in a column under a band 20 tall.
fn laid_out(scroll: super::Scroll<'_, ()>) -> (Rectangle, Rectangle, Rectangle) {
    let above = widget::container(widget::Space::new().width(Length::Fill).height(20.0)).id(widget::Id::new("above"));
    let view: Element<'_, ()> = widget::column![above, scroll.id(widget::Id::new("scroll"))].into();
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(400.0, 300.0), view);

    let above = ui.find(selector::id("above")).expect("above").bounds();
    let scrolled = ui.find(selector::id("scroll")).expect("scroll").bounds();
    let inside = ui.find(selector::id("inside")).expect("inside").bounds();
    (above, scrolled, inside)
}

fn inside<'a>() -> Element<'a, ()> {
    widget::container(widget::Space::new().width(Length::Fill).height(40.0))
        .id(widget::Id::new("inside"))
        .into()
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "Layout puts boxes on whole device pixels, so the sizes are exact: any difference is the bug."
)]
fn a_pages_scroll_lines_its_content_up_with_what_is_around_it() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let (above, _, inside) = laid_out(scroll(&look, inside()));

    assert_eq!((inside.x, inside.width), (above.x, above.width));
    assert_eq!(inside.y, above.y + above.height);
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "Layout puts boxes on whole device pixels, so the sizes are exact: any difference is the bug."
)]
fn a_pages_scroll_reaches_past_its_box_as_far_as_a_glow_does() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let (_, scrolled, inside) = laid_out(scroll(&look, inside()));

    assert_eq!(inside.x - scrolled.x, GLOW, "left");
    assert_eq!(inside.y - scrolled.y, GLOW, "top");
    assert_eq!(
        scrolled.y + scrolled.height - (inside.y + inside.height),
        GLOW,
        "bottom"
    );
    // Content that runs to the edge leaves the gutter on its right, so a glow there reaches the
    // gutter less past the box.
    assert_eq!(
        scrolled.x + scrolled.width - (inside.x + inside.width),
        GLOW - GUTTER,
        "right"
    );
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "Layout puts boxes on whole device pixels, so the sizes are exact: any difference is the bug."
)]
fn a_pages_scroll_reaches_no_farther_than_the_room_left_to_its_neighbours() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let (_, scrolled, inside) = laid_out(scroll(&look, inside()).room_above(Space::S12).room_below(Space::S16));

    assert_eq!(inside.y - scrolled.y, f32::from(Space::S12.px()), "top");
    assert_eq!(
        scrolled.y + scrolled.height - (inside.y + inside.height),
        f32::from(Space::S16.px()),
        "bottom"
    );
    assert_eq!(inside.x - scrolled.x, GLOW, "left");
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "Layout puts boxes on whole device pixels, so the sizes are exact: any difference is the bug."
)]
fn a_pages_scroll_reaches_all_the_room_up_to_its_neighbour_when_that_is_more_than_a_glow() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let (_, scrolled, inside) = laid_out(scroll(&look, inside()).room_above(Space::S32));

    assert_eq!(inside.y - scrolled.y, f32::from(Space::S32.px()));
}

#[test]
fn a_pages_scroll_of_a_set_height_tells_its_parent_that_height_whatever_it_reaches() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let log = f32::from(crate::tokens::Size::Log.px());

    let scrolled: Element<'_, ()> = scroll(&look, inside())
        .height(crate::layout::Extent::Fixed(crate::tokens::Size::Log))
        .room_above(Space::S12)
        .into();

    assert_eq!(scrolled.as_widget().size().height, Length::Fixed(log));
}
