//! The components page's "Sliders" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Sliders.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::slider::Slider;
use kubaos_ui::component::slider::Value;
use kubaos_ui::component::slider::slider;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Sliders",
    lede: Some(
                "A pill whose fill is the value: its round end marks it. Steps show on both sides, and the mark \
                 the value rests on turns solid. Regular above, compact below.",
            ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let caption = |words: &'static str| text(words, Type::Small, c.on_surface_variant);
    let live = |glyph: Icon, value: Value| -> Slider<'a, Message> {
        slider(glyph, value)
            .on_change(|_| Message::Sample)
            .on_icon(Message::Sample)
    };
    let power = [
        "Super Saver",
        "Power Saver",
        "Balanced",
        "Performance",
        "Max Performance",
    ];
    let cell = |shown: Slider<'a, Message>, compact: bool, words: &'static str| -> Element<'a, Message> {
        let shown = if compact { shown.compact() } else { shown };
        stack(look, Space::S8)
            .push(shown.view(look))
            .push(caption(words))
            .into()
    };
    let samples = |compact: bool| -> Element<'a, Message> {
        grid(look, 3, Space::S32)
            .push(cell(
                live(Icon::VolumeUp, Value::Continuous(0.62)),
                compact,
                "Volume, 62%",
            ))
            .push(cell(
                live(Icon::VolumeUp, Value::Continuous(0.62)).hovered(),
                compact,
                "Under the pointer",
            ))
            .push(cell(
                live(Icon::VolumeUp, Value::Continuous(0.62)).held(),
                compact,
                "Held: the fill lights up",
            ))
            .push(cell(
                live(Icon::VolumeOff, Value::Continuous(0.0)).words("Muted"),
                compact,
                "Muted: the icon mutes",
            ))
            .push(cell(
                live(Icon::MotionPhotosAuto, Value::Continuous(0.55))
                    .words("Auto · 55%")
                    .soft(),
                compact,
                "Auto: the sensor's level",
            ))
            .push(cell(
                live(Icon::Balance, Value::Stop { index: 2, count: 5 }).stops(power),
                compact,
                "Five steps, resting on one",
            ))
            .push(cell(
                live(Icon::VolumeUp, Value::Continuous(2.0 / 3.0))
                    .sticky(2.0 / 3.0)
                    .number("100%")
                    .widest("150%"),
                compact,
                "Sticky at 100% of 150%",
            ))
            .push(cell(
                live(Icon::VolumeUp, Value::Continuous(0.3)).focused(true),
                compact,
                "Focused",
            ))
            .push(cell(
                slider(Icon::VolumeUp, Value::Continuous(0.3)),
                compact,
                "Disabled",
            ))
            .into()
    };
    stack(look, Space::S32).push(samples(false)).push(samples(true)).into()
}
