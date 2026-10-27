use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced::widget::image;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::NamePlace;
use super::picture_choice;
use crate::component::media::Aspect;
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

/// The bounds of `element`, laid out in a column 256 wide.
fn bounds(element: Element<'_, ()>) -> Rectangle {
    let probe = widget::container(element).width(256.0).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(800.0, 800.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

fn picture() -> image::Handle {
    image::Handle::from_rgba(4, 4, vec![255; 4 * 4 * 4])
}

#[test]
fn a_name_over_the_picture_makes_the_tile_the_pictures_own_size() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let tile = bounds(
        picture_choice(Some(picture()), Aspect::Wallpaper, "Shinjuku")
            .name(NamePlace::Over)
            .on_press(())
            .view(&look),
    );

    assert!(
        (tile.width - 256.0).abs() < 1e-4 && (tile.height - 160.0).abs() < 1e-4,
        "{tile:?}"
    );
}

#[test]
fn picking_or_loading_moves_nothing() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        let tile = |picture: Option<image::Handle>, picked: bool| {
            bounds(
                picture_choice(picture, Aspect::Wallpaper, "Shinjuku")
                    .picked(picked)
                    .mark(crate::icon::Icon::Check, "Current")
                    .on_press(())
                    .view(&look),
            )
        };

        let resting = tile(None, false);

        assert_eq!(resting, tile(Some(picture()), false), "{scale:?}");
        assert_eq!(resting, tile(Some(picture()), true), "{scale:?}");
    }
}
