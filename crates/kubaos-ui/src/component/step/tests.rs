use iced::Element;
use iced::Rectangle;
use iced::Size;
use iced::widget;
use iced_test::selector;
use iced_test::simulator::Simulator;

use super::StepState;
use super::step;
use crate::icon::Icon;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens::Mode;
use crate::tokens::Size as SizeToken;
use crate::tokens::Type;

fn settings() -> iced::Settings {
    iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    }
}

/// The bounds of `element`, laid out in a column 320 wide.
fn bounds(element: Element<'_, ()>) -> Rectangle {
    let probe = widget::container(element).width(320.0).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(800.0, 800.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

#[test]
fn a_step_is_a_list_rows_height_whatever_its_state() {
    let reboot = StepState::Action {
        label: "Reboot",
        glyph: Icon::RestartAlt,
    };
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        for state in [StepState::Todo, StepState::Current, StepState::Done, reboot] {
            let one = bounds(step("Git").icon(Icon::Code).state(state).view(&look));
            let two = bounds(
                step("Git")
                    .subtitle("Not set up yet.")
                    .icon(Icon::Code)
                    .state(state)
                    .view(&look),
            );

            assert!(
                (one.height - f32::from(SizeToken::Row.px())).abs() < 1e-4,
                "{scale:?} {state:?}: {one:?}"
            );
            assert!(
                (two.height - f32::from(SizeToken::RowTwoLine.px())).abs() < 1e-4,
                "{scale:?} {state:?}: {two:?}"
            );
        }
    }
}

#[test]
fn a_long_subtitle_wraps_and_the_step_grows_a_line_at_a_time() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let short = bounds(step::<()>("TUXEDO drivers").subtitle("Not installed.").view(&look));

    let long = bounds(
        step::<()>("TUXEDO drivers")
            .subtitle(
                "Keyboard backlight, fans and power limits need TUXEDO's drivers, built on this machine and signed \
                 with a key made here.",
            )
            .view(&look),
    );

    let line = f32::from(Type::Small.line());
    let grown = long.height - short.height;
    assert!(
        grown >= line - 1e-4 && (grown / line).fract().abs() < 1e-4,
        "{short:?} {long:?}"
    );
}
