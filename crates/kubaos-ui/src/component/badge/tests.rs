use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::badge;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens::Mode;
use crate::tokens::Size as SizeToken;

fn settings() -> iced::Settings {
    iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    }
}

fn bounds(element: Element<'_, ()>) -> Rectangle {
    let probe = widget::container(element).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(400.0, 400.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

#[test]
fn a_one_digit_badge_is_a_circle_and_a_longer_one_a_pill_as_tall() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let tall = f32::from(SizeToken::Badge.px());

    let one = bounds(badge(&look, Some(3)));
    let many = bounds(badge(&look, Some(128)));

    assert!(
        (one.width - tall).abs() < 1e-4 && (one.height - tall).abs() < 1e-4,
        "{one:?}"
    );
    assert!(many.width > tall && (many.height - tall).abs() < 1e-4, "{many:?}");
}
