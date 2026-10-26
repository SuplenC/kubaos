use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::ButtonSize;
use super::Kind;
use super::button;
use crate::component::field::field;
use crate::icon::Icon;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens::Mode;

fn settings() -> iced::Settings {
    iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    }
}

/// The bounds of `element`, laid out alone in a wide window.
fn bounds(element: Element<'_, ()>) -> Rectangle {
    let probe = widget::container(element).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(800.0, 200.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

fn look() -> Look {
    Look::new(Mode::Dark, Scale::X1)
}

#[test]
fn a_regular_button_is_forty_tall_and_a_small_one_thirty_two() {
    for kind in Kind::ALL {
        let md = bounds(button("Continue").kind(kind).on_press(()).view(&look()));
        let sm = bounds(
            button("Show")
                .kind(kind)
                .size(ButtonSize::Sm)
                .on_press(())
                .view(&look()),
        );

        assert!((md.height - 40.0).abs() < 1e-4, "{kind:?} md {md:?}");
        assert!((sm.height - 32.0).abs() < 1e-4, "{kind:?} sm {sm:?}");
    }
}

#[test]
fn an_icon_doesnt_change_a_buttons_height() {
    let plain = bounds(button("Save log").kind(Kind::Tonal).on_press(()).view(&look()));
    let with_icon = bounds(
        button("Save log")
            .kind(Kind::Tonal)
            .icon(Icon::Save)
            .on_press(())
            .view(&look()),
    );

    assert!((plain.height - with_icon.height).abs() < 1e-4);
}

#[test]
fn focus_never_moves_or_resizes_a_button() {
    let resting = bounds(button("Try again").on_press(()).view(&look()));
    let focused = bounds(button("Try again").on_press(()).focused(true).view(&look()));

    assert_eq!(resting, focused);
}

#[test]
fn a_field_is_as_tall_as_a_regular_button() {
    let field = bounds(field::<()>("Kuba", "Your name").icon(Icon::Person).view(&look()));

    assert!((field.height - 40.0).abs() < 1e-4, "{field:?}");
}
