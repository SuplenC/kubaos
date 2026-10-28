use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::Column;
use super::table_row;
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
    let probe = widget::container(element).width(320.0).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(800.0, 800.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

#[test]
fn a_row_is_compact_and_as_tall_picked_or_not_whatever_its_words() {
    let columns = [Column::new("Name", 4), Column::new("CPU", 1).numeric()];
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        let row = |name: &str, selected: bool| {
            bounds(
                table_row(vec![name.to_owned(), "38.4%".to_owned()])
                    .selected(selected)
                    .on_press(())
                    .view(&look, &columns),
            )
        };

        let resting = row("firefox", false);

        assert!(
            (resting.height - f32::from(SizeToken::RowCompact.px())).abs() < 1e-4,
            "{scale:?}: {resting:?}"
        );
        assert_eq!(
            resting,
            row("a process with a name far too long for its column", true),
            "{scale:?}"
        );
    }
}
