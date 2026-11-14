use iced::Element;
use iced::Size;
use iced::widget;
use iced_test::simulator::Simulator;

use super::ChipKind;
use super::chip;
use crate::Look;
use crate::Mode;
use crate::Scale;

/// The messages a click on the middle of `view`'s chip sends.
fn clicked(view: Element<'_, &'static str>) -> Vec<&'static str> {
    let page: Element<'_, &'static str> = widget::container(view).into();
    let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(400.0, 100.0), page);
    let on = iced::Point::new(20.0, 12.0);

    ui.point_at(on);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: on }),
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
    ]);
    ui.into_messages().collect()
}

#[test]
fn a_pressable_chip_sends_its_press() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let sent = clicked(chip(ChipKind::Neutral, "Italian").on_press("pick").view(&look));

    assert_eq!(sent, ["pick"]);
}

#[test]
fn a_disabled_chip_takes_no_press() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let sent = clicked(
        chip(ChipKind::Neutral, "Italian")
            .on_press("pick")
            .disabled(true)
            .view(&look),
    );

    assert!(sent.is_empty(), "{sent:?}");
}
