use iced::Element;
use iced::Length;
use iced::Size;
use iced::widget;
use iced::widget::Id;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::Caret;
use super::Pointer;
use super::State;
use super::field;
use crate::Look;
use crate::Mode;
use crate::Scale;
use crate::icon::Icon;
use crate::tokens::Size as SizeToken;

#[test]
fn a_caret_held_to_a_shorter_value_ends_at_its_end() {
    let caret = Caret { pos: 9, anchor: 2 };

    let held = caret.clamp("Kuba");

    assert_eq!(held, Caret { pos: 4, anchor: 2 });
}

#[test]
fn a_selection_runs_from_its_lower_end_to_its_higher() {
    assert_eq!(Caret { pos: 1, anchor: 5 }.range(), (1, 5));
    assert!(Caret::all("Kuba").has_selection());
    assert!(!Caret::at(3).has_selection());
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "Layout puts boxes on whole device pixels, so the sizes are exact: any difference is the bug."
)]
fn a_field_is_as_tall_as_a_regular_control_whatever_its_room() {
    let look = Look::new(Mode::Dark, Scale::X1);
    for room in [Size::new(400.0, 100.0), Size::new(1600.0, 900.0)] {
        let view: Element<'_, ()> = widget::column![
            widget::container(
                field("Kuba Example", "Your name")
                    .icon(Icon::Search)
                    .caret(Caret::all("Kuba Example"))
                    .view(&look)
            )
            .id(Id::new("field"))
        ]
        .height(Length::Fill)
        .into();
        let mut ui = Simulator::with_size(iced::Settings::default(), room, view);

        let bounds = ui.find(selector::id("field")).expect("the field").bounds();

        assert_eq!(bounds.width, room.width);
        assert_eq!(bounds.height, f32::from(SizeToken::ControlMd.px()));
    }
}

#[test]
fn a_field_scrolled_up_a_page_takes_a_press_where_it_is_drawn() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let above = widget::Space::new().height(400.0);
    let input = field::<Pointer>("Kuba", "Your name")
        .on_pointer(|pointer| pointer)
        .view(&look);
    let page: Element<'_, Pointer> = widget::scrollable(widget::column![above, input])
        .anchor_bottom()
        .height(Length::Fill)
        .into();
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(400.0, 200.0), page);
    // Scrolled to its end, the field is drawn on the window's last 40 px, 400 below where it is
    // laid out.
    let drawn = iced::Point::new(300.0, 180.0);

    ui.point_at(drawn);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: drawn }),
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
    ]);

    let pressed: Vec<Pointer> = ui.into_messages().collect();
    assert_eq!(pressed, [Pointer::Press { pos: 4, extend: false }]);
}

#[test]
fn a_disabled_field_takes_no_press_and_shows_no_caret() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let input = field::<Pointer>("Kuba", "Your name")
        .state(State::Disabled)
        .caret(Caret::at(2))
        .on_pointer(|pointer| pointer)
        .view(&look);
    let page: Element<'_, Pointer> = widget::column![input].height(Length::Fill).into();
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(400.0, 200.0), page);
    let on = iced::Point::new(20.0, 20.0);

    ui.point_at(on);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: on }),
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
    ]);

    assert_eq!(ui.into_messages().count(), 0);
}
