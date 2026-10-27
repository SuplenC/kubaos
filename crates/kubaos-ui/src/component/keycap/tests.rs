use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::key_hints;
use super::keycap;
use super::keys;
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

#[test]
fn a_combination_is_split_into_its_keys() {
    assert_eq!(keys("Ctrl+K").collect::<Vec<_>>(), ["Ctrl", "K"]);
    assert_eq!(keys("Esc").collect::<Vec<_>>(), ["Esc"]);
    assert_eq!(keys("+").collect::<Vec<_>>(), ["+"]);
}

#[test]
fn a_keycap_is_as_tall_as_a_line_of_body_text_at_every_scale() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);

        let cap = bounds(keycap(&look, "Esc"));

        assert!((cap.height - 20.0).abs() < 1e-4, "{scale:?}: {cap:?}");
        assert!(scale.is_whole(cap.width), "{scale:?}: {cap:?}");
    }
}

#[test]
fn a_one_letter_keycap_is_square() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let cap = bounds(keycap(&look, "K"));

    assert!((cap.width - 20.0).abs() < 1e-4, "{cap:?}");
}

#[test]
fn a_longer_key_grows_the_keycap() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let short = bounds(keycap(&look, "K"));
    let long = bounds(keycap(&look, "Enter"));

    assert!(long.width > short.width, "{short:?} {long:?}");
}

#[test]
fn key_hints_are_as_tall_as_their_keycaps() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let line = bounds(key_hints(&look, &[("Enter", "continue"), ("Ctrl+K", "actions")]));

    assert!((line.height - 20.0).abs() < 1e-4, "{line:?}");
}
