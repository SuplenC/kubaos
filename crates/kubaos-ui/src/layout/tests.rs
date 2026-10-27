use iced::Element;
use iced::Length;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::Extent;
use super::Gap;
use super::Inset;
use super::row;
use super::stack;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens::Mode;
use crate::tokens::Size as SizeToken;
use crate::tokens::Space;

fn settings() -> iced::Settings {
    iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    }
}

fn block(id: &'static str) -> Element<'static, ()> {
    widget::container(widget::Space::new())
        .width(Length::Fixed(40.0))
        .height(Length::Fixed(20.0))
        .id(id)
        .into()
}

#[test]
fn an_inset_is_padding_from_the_spacing_scale() {
    let padding = Inset::x(Space::S40).top(Space::S32).bottom(Space::S24).padding();

    assert_eq!(
        (padding.top, padding.right, padding.bottom, padding.left),
        (32.0, 40.0, 24.0, 40.0)
    );
}

#[test]
fn no_gap_is_no_space() {
    assert!(Gap::NONE.px().abs() < f32::EPSILON);
    assert!((Gap::from(Space::S12).px() - 12.0).abs() < f32::EPSILON);
}

#[test]
fn a_fixed_extent_is_a_component_size() {
    assert_eq!(Extent::Fixed(SizeToken::ControlMd).length(), Length::Fixed(40.0));
}

#[test]
fn a_row_puts_its_gap_between_children() {
    let look = &Look::new(Mode::Dark, Scale::X1);
    let view = row(look, Space::S8).push(block("a")).push(block("b"));
    let mut ui = Simulator::with_size(settings(), Size::new(400.0, 100.0), view);

    let b = ui.find(selector::id("b")).expect("b").bounds();

    assert!((b.x - 48.0).abs() < 1e-4, "{b:?}");
}

#[test]
fn a_stack_puts_its_gap_between_children_and_its_inset_around_them() {
    let look = &Look::new(Mode::Dark, Scale::X1);
    let view = stack(look, Space::S12)
        .padding(Inset::all(Space::S16))
        .push(block("a"))
        .push(block("b"));
    let mut ui = Simulator::with_size(settings(), Size::new(400.0, 200.0), view);

    let a = ui.find(selector::id("a")).expect("a").bounds();
    let b = ui.find(selector::id("b")).expect("b").bounds();

    assert!((a.x - 16.0).abs() < 1e-4 && (a.y - 16.0).abs() < 1e-4, "{a:?}");
    assert!((b.y - 48.0).abs() < 1e-4, "{b:?}");
}
