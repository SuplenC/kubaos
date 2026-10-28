use iced::Element;
use iced::Length;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::scroll;
use crate::Look;
use crate::Mode;
use crate::Scale;

#[test]
#[expect(
    clippy::float_cmp,
    reason = "Layout puts boxes on whole device pixels, so the sizes are exact: any difference is the bug."
)]
fn a_pages_scroll_lines_its_content_up_with_what_is_around_it() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let above = widget::container(widget::Space::new().width(Length::Fill).height(20.0)).id(widget::Id::new("above"));
    let inside = widget::container(widget::Space::new().width(Length::Fill).height(40.0)).id(widget::Id::new("inside"));
    let view: Element<'_, ()> = widget::column![above, scroll(&look, inside)].into();
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(400.0, 300.0), view);

    let above = ui.find(selector::id("above")).expect("above").bounds();
    let inside = ui.find(selector::id("inside")).expect("inside").bounds();

    assert_eq!((inside.x, inside.width), (above.x, above.width));
    assert_eq!(inside.y, above.y + above.height);
}
