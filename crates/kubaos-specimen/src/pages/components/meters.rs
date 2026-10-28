//! The components page's "Meters and sparklines" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Meters.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::meter::meter;
use kubaos_ui::component::meter::sparkline;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Meters and sparklines",
    lede: Some("Readings at a glance: a reading in parts on a progress bar's track, and the last minute of one as a line."),
    view,
};

/// A minute of a reading, a sample every two seconds: a wave the QML page draws too.
fn wave(level: f32, swing: f32, phase: f32) -> Vec<f32> {
    (0..30_u8)
        .map(|i| {
            let t = f32::from(i);
            level + swing * (t * 0.4 + phase).sin() + swing * 0.4 * (t * 1.3 + phase).sin()
        })
        .collect()
}

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let meters = stack(look, Space::S16)
        .width(Extent::Fill)
        .push(meter(look, &[(0.55, c.primary), (0.15, c.tertiary)]))
        .push(meter(look, &[(0.92, c.warning)]))
        .push(meter(look, &[(0.3, c.secondary)]));
    let graphs = stack(look, Space::S16)
        .width(Extent::Fill)
        .push(sparkline(wave(40.0, 20.0, 0.0)).view(look))
        .push(
            sparkline(wave(1200.0, 600.0, 1.0))
                .second(wave(300.0, 150.0, 2.0))
                .max(0.0)
                .view(look),
        );
    row(look, Space::S48).push(meters).push(graphs).into()
}
