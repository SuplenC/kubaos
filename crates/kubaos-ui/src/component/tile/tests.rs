use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::tile;
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

/// The bounds of `element`, laid out in a column 192 wide.
fn bounds(element: Element<'_, ()>) -> Rectangle {
    let probe = widget::container(element).width(192.0).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(800.0, 800.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

#[test]
fn a_tile_is_a_two_line_row_tall_on_or_off_with_or_without_its_arrow() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        let tile = |checked: bool, detail: bool| {
            let t = tile(Icon::Wifi, "Wi-Fi")
                .subtitle("A network name far too long for the tile")
                .checked(checked)
                .on_press(());
            bounds(if detail { t.on_detail(()) } else { t }.view(&look))
        };

        let resting = tile(false, false);

        assert!(
            (resting.height - f32::from(SizeToken::RowTwoLine.px())).abs() < 1e-4,
            "{scale:?}: {resting:?}"
        );
        assert_eq!(resting, tile(true, false), "{scale:?}");
        assert_eq!(resting, tile(true, true), "{scale:?}");
    }
}
