use iced::Element;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use crate::layout::row;
use crate::look::Look;
use crate::scale::Scale;
use crate::text::text;
use crate::tokens::Mode;
use crate::tokens::Space;
use crate::tokens::Type;

fn settings() -> iced::Settings {
    iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    }
}

#[test]
fn what_follows_text_in_a_row_starts_on_a_device_pixel() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        let after: Element<'_, ()> = widget::container(widget::Space::new().width(8.0).height(8.0))
            .id("after")
            .into();
        let view = row(&look, Space::S8)
            .push(text("Kōbō, Shinjuku", Type::Body, look.colors.on_surface))
            .push(after);
        let mut ui = Simulator::with_size(settings(), Size::new(600.0, 100.0), view);

        let x = ui.find(selector::id("after")).expect("after").bounds().x;

        assert!(scale.is_whole(x), "x {x} at {}", scale.factor());
    }
}
