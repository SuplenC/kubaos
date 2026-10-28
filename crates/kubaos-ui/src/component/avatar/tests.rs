use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced::widget::image;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::TileSize;
use super::badge;
use super::icon_tile;
use crate::icon::Icon;
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

fn picture() -> image::Handle {
    image::Handle::from_rgba(4, 4, vec![255; 4 * 4 * 4])
}

#[test]
fn a_tile_is_its_size_square_whatever_it_shows() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        for size in TileSize::ALL {
            let side = f32::from(size.side().px());

            let symbol = bounds(icon_tile(Icon::Notifications, size).view(&look));
            let app = bounds(
                icon_tile(Icon::Notifications, size)
                    .picture(Some(picture()))
                    .view(&look),
            );
            let photo = bounds(icon_tile(Icon::Notifications, size).photo(Some(picture())).view(&look));

            assert!(
                (symbol.width - side).abs() < 1e-4 && (symbol.height - side).abs() < 1e-4,
                "{scale:?} {size:?}: {symbol:?}"
            );
            assert_eq!(symbol, app, "{scale:?} {size:?}");
            assert_eq!(symbol, photo, "{scale:?} {size:?}");
        }
    }
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
