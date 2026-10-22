use iced::widget::{Id, Space, column, container, text};
use iced::{Element, Length, Rectangle, Size};
use iced_test::simulator::Simulator;

use super::grid;

fn card(id: &'static str, lines: usize) -> Element<'static, ()> {
    let rows = (0..lines).map(|i| text(format!("line {i}")).into());
    container(column(rows))
        .id(id)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn bounds_at(width: f32, cards: Vec<Element<'static, ()>>, ids: &[&'static str]) -> Vec<Rectangle> {
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        Size::new(width, 2000.0),
        column![grid(2, 10.0, cards), Space::new().height(Length::Fill)],
    );
    ids.iter()
        .map(|id| {
            ui.find(Id::new(id))
                .unwrap_or_else(|e| panic!("no {id}: {e}"))
                .bounds()
        })
        .collect()
}

#[test]
fn every_cell_is_as_wide_as_its_share_and_as_tall_as_the_tallest() {
    for width in [640.0, 1024.0, 1920.0] {
        let cards = vec![card("a", 1), card("b", 4), card("c", 2), card("d", 6)];

        let b = bounds_at(width, cards, &["a", "b", "c", "d"]);

        for cell in &b[1..] {
            assert_eq!(cell.width, b[0].width, "at {width}");
        }
        assert_eq!(b[0].width * 2.0 + 10.0, width, "at {width}");
        for cell in &b[..3] {
            assert_eq!(cell.height, b[3].height, "at {width}");
        }
        assert_eq!(b[1].x, b[0].x + b[0].width + 10.0, "at {width}");
        assert_eq!(b[2].y, b[0].y + b[0].height + 10.0, "at {width}");
    }
}

#[test]
fn a_row_short_of_cells_keeps_the_cell_width() {
    let cards = vec![card("a", 1), card("b", 1), card("c", 1)];

    let b = bounds_at(1000.0, cards, &["a", "c"]);

    assert_eq!(b[1].width, b[0].width);
    assert_eq!(b[1].x, b[0].x);
}
