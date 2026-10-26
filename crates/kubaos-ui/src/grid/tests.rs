use iced::Element;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::Span;
use super::columns;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens::Mode;
use crate::tokens::Space;

fn device(span: Span, scale: Scale) -> (f32, f32) {
    (scale.to_device(span.x), scale.to_device(span.width))
}

#[test]
fn every_column_edge_is_on_a_device_pixel() {
    for scale in Scale::SUPPORTED {
        for width in [997.0, 1000.0, 1001.6, 333.0] {
            for count in 1..=7 {
                for span in columns(width, count, Space::S16, scale) {
                    let (x, w) = device(span, scale);
                    assert!(
                        (x - x.round()).abs() < 1e-3 && (w - w.round()).abs() < 1e-3,
                        "{width} / {count} at {}: {span:?}",
                        scale.factor()
                    );
                }
            }
        }
    }
}

#[test]
fn columns_differ_by_at_most_one_device_pixel() {
    for scale in Scale::SUPPORTED {
        let widths: Vec<f32> = columns(1000.0, 7, Space::S12, scale)
            .into_iter()
            .map(|span| device(span, scale).1)
            .collect();
        let (min, max) = widths
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &w| (lo.min(w), hi.max(w)));
        assert!(max - min <= 1.0 + 1e-3, "{widths:?} at {}", scale.factor());
    }
}

#[test]
fn columns_and_gaps_fill_the_width_exactly() {
    for scale in Scale::SUPPORTED {
        let spans = columns(1000.0, 3, Space::S16, scale);
        let last = spans.last().expect("three columns");

        let end = scale.to_device(last.x + last.width);

        assert!(
            (end - (1000.0 * scale.factor()).floor()).abs() < 1e-3,
            "{}",
            scale.factor()
        );
    }
}

#[test]
fn the_gap_between_columns_is_the_step() {
    let spans = columns(1000.0, 4, Space::S24, Scale::X1);

    for pair in spans.windows(2) {
        assert!((pair[1].x - (pair[0].x + pair[0].width) - 24.0).abs() < 1e-4);
    }
}

#[test]
fn the_grid_places_its_children_on_its_columns() {
    let look = Look::new(Mode::Dark, Scale::X1_25);
    let cell = |id: &'static str| -> Element<'static, ()> {
        widget::container(widget::Space::new().height(30.0))
            .width(iced::Length::Fill)
            .id(id)
            .into()
    };
    let grid = crate::layout::grid(&look, 3, Space::S16)
        .push(cell("a"))
        .push(cell("b"))
        .push(cell("c"))
        .push(cell("d"));
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(1001.0, 400.0), grid);
    let spans = columns(1001.0, 3, Space::S16, Scale::X1_25);

    let b = ui.find(selector::id("b")).expect("b").bounds();
    let d = ui.find(selector::id("d")).expect("d").bounds();

    assert!((b.x - spans[1].x).abs() < 1e-4 && (b.width - spans[1].width).abs() < 1e-4);
    // The second row starts a whole device pixel past the first's 30, plus the gap.
    assert!((d.y - (Scale::X1_25.snap_up(30.0) + 16.0)).abs() < 1e-4, "{d:?}");
}
