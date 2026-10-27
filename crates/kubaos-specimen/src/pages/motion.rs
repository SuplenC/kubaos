//! The motion page: the durations, the curves as graphs and as onion-skinned frames, and the
//! components' own animations frame by frame. In the window, a dot runs along each curve.

use iced::Color;
use iced::Element;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as IcedSize;
use iced::Theme;
use iced::mouse;
use iced::widget;
use iced::widget::canvas;
use iced::widget::canvas::Path;
use iced::widget::canvas::Stroke;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::progress::timed_bar;
use kubaos_ui::component::switch::switch;
use kubaos_ui::component::toast::DEFAULT_STAY;
use kubaos_ui::component::toast::toast_at;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::row;
use kubaos_ui::layout::section;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::motion::Motion;
use kubaos_ui::motion::ease;
use kubaos_ui::snap::centre_x;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Duration;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;

const DURATIONS: [Duration; 7] = [
    Duration::Instant,
    Duration::Micro,
    Duration::Short,
    Duration::Medium,
    Duration::Long,
    Duration::Slow,
    Duration::Scene,
];

const EASINGS: [Easing; 6] = [
    Easing::Standard,
    Easing::StandardDecel,
    Easing::StandardAccel,
    Easing::EmphasizedDecel,
    Easing::EmphasizedAccel,
    Easing::Linear,
];

/// The frames the onion skins and filmstrips show: nine, evenly spaced in time.
const FRAMES: usize = 9;

fn use_of(duration: Duration) -> &'static str {
    match duration {
        Duration::Instant => "The caret, typed text, reduce motion's moves.",
        Duration::Micro => "Hover and press layers, a colour under the pointer.",
        Duration::Short => "A switch's knob, a check mark, focus moving, a toast arriving.",
        Duration::Medium => "A panel or popup leaving, the log opening, a toast leaving.",
        Duration::Long => "A panel or dialog arriving, a page sliding in.",
        Duration::Slow => "A wallpaper fading in, the light/dark crossfade.",
        Duration::Scene => "A whole scene once: the lock lifting, Kōbō's welcome settling.",
    }
}

fn curve_use(easing: Easing) -> &'static str {
    match easing {
        Easing::Standard => "Small changes that stay: toggles, colour, focus.",
        Easing::StandardDecel => "Something moving settling in: a list reordering.",
        Easing::StandardAccel => "Something small leaving for good: a toast.",
        Easing::EmphasizedDecel => "Arriving: panels, dialogs, pages, the log.",
        Easing::EmphasizedAccel => "Leaving: panels and dialogs closing.",
        Easing::Linear => "Real quantities, spinners.",
    }
}

pub fn view<'a>(look: &Look, seconds: f32, live: bool) -> Element<'a, Message> {
    stack(look, Space::S48)
        .push(section(
            look,
            "Durations",
            Some("The bar is the time, 32 px a 100 ms. Speed 0.5× to 2× scales them all; reduce motion caps them at short and turns moves into fades."),
            durations(look),
        ))
        .push(section(
            look,
            "Curves",
            Some("Each curve as a graph (time across, distance up) and as nine evenly timed frames of a dot crossing."),
            curves(look, seconds, live),
        ))
        .push(section(
            look,
            "Frame by frame",
            Some("A switch turning on (short, standard) and a toast arriving (short, emphasized decel), at the nine frames."),
            frames(look),
        ))
        .push(section(
            look,
            "Held",
            Some("A button held instead of clicked, Kōbō's Install: the bar beside it fills as the hold goes on, here at its nine frames."),
            held(look),
        ))
        .into()
}

fn durations<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut table = split(look, &[1, 2, 2], Space::S12).center();
    for duration in DURATIONS {
        // 32 px a 100 ms: every duration is a whole number of hundreds, so every bar is on the
        // grid.
        let width = (duration.ms() / 100 * u32::from(Space::S32.px())) as f32;
        table = table
            .push(
                row(look, Space::S8)
                    .center()
                    .push(text(format!("{duration:?}"), Type::BodyStrong, c.on_surface))
                    .push(text(format!("{} ms", duration.ms()), Type::Mono, c.on_surface_variant)),
            )
            .push(
                widget::container(widget::Space::new())
                    .width(Length::Fixed(width.max(look.hairline())))
                    .height(f32::from(Space::S8.px()))
                    .style(move |_theme| widget::container::Style::default().background(c.primary)),
            )
            .push(text(use_of(duration), Type::Small, c.on_surface_variant));
    }
    table.into()
}

fn curves<'a>(look: &Look, seconds: f32, live: bool) -> Element<'a, Message> {
    let c = look.colors;
    let graph_side = f32::from(Space::S64.px() * 2);
    let mut table = split(look, &[2, 1, 3], Space::S24).center();
    for easing in EASINGS {
        let [x1, y1, x2, y2] = easing.curve();
        // In the window the dot loops: the long duration, then a rest as long.
        let cycle = 2.0 * Duration::Long.ms() as f32 / 1000.0;
        let phase = live.then(|| ((seconds % cycle) / (cycle / 2.0)).min(1.0));
        table = table
            .push(
                stack(look, Space::S4)
                    .push(text(format!("{easing:?}"), Type::BodyStrong, c.on_surface))
                    .push(text(
                        format!("({x1}, {y1}, {x2}, {y2})"),
                        Type::Mono,
                        c.on_surface_variant,
                    ))
                    .push(text(curve_use(easing), Type::Small, c.on_surface_variant)),
            )
            .push(
                widget::canvas(Graph {
                    easing,
                    line: c.primary,
                    grid: look.outline_soft(),
                    width: look.scale.thin(),
                    hairline: look.hairline(),
                })
                .width(graph_side)
                .height(graph_side),
            )
            .push(
                widget::canvas(Onion {
                    easing,
                    dot: c.primary,
                    track: color::alpha(c.on_surface, Opacity::Hover.value()),
                    phase,
                })
                .width(Length::Fill)
                .height(f32::from(Space::S32.px())),
            );
    }
    table.into()
}

fn frames<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let times = (0..FRAMES).map(|i| i as f32 / (FRAMES - 1) as f32);
    let switches = grid(look, 9, Space::S16).extend(times.clone().map(|t| {
        stack(look, Space::S8)
            .center()
            .push(switch(
                look,
                true,
                ease(Easing::Standard, t),
                Some(Message::Sample),
                false,
            ))
            .push(text(
                format!("{:.0} ms", t * Duration::Short.ms() as f32),
                Type::Mono,
                c.on_surface_variant,
            ))
            .into()
    }));
    let arriving = Motion::default().duration(Duration::Short);
    let toasts = grid(look, 9, Space::S16).extend(times.map(|t| {
        centre_x(
            look.scale,
            toast_at(look, "Copied.", None, arriving.mul_f32(t), DEFAULT_STAY),
        )
    }));
    stack(look, Space::S24).push(switches).push(toasts).into()
}

fn held<'a>(look: &Look) -> Element<'a, Message> {
    let times = (0..FRAMES).map(|i| i as f32 / (FRAMES - 1) as f32);
    stack(look, Space::S24)
        .push(
            row(look, Space::S16)
                .center()
                .push(
                    button("Keep holding…")
                        .kind(Kind::Danger)
                        .size(ButtonSize::Lg)
                        .icon(Icon::Download)
                        .on_hold(Message::Sample, Message::Sample)
                        .view(look),
                )
                .push(text("or hold Enter", Type::Caption, look.colors.outline)),
        )
        .push(grid(look, 9, Space::S16).extend(times.map(|t| timed_bar(look, t))))
        .into()
}

/// An easing curve plotted: time across, distance up, with the diagonal for reference.
struct Graph {
    easing: Easing,
    line: Color,
    grid: Color,
    width: f32,
    hairline: f32,
}

impl canvas::Program<Message> for Graph {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let (w, h) = (bounds.width, bounds.height);
        let half = self.hairline / 2.0;
        frame.stroke(
            &Path::rectangle(
                Point::new(half, half),
                IcedSize::new(w - self.hairline, h - self.hairline),
            ),
            Stroke::default().with_width(self.hairline).with_color(self.grid),
        );
        frame.stroke(
            &Path::line(Point::new(0.0, h), Point::new(w, 0.0)),
            Stroke::default().with_width(self.hairline).with_color(self.grid),
        );
        let curve = Path::new(|b| {
            b.move_to(Point::new(0.0, h));
            for i in 1..=64 {
                let x = i as f32 / 64.0;
                b.line_to(Point::new(x * w, h - ease(self.easing, x) * h));
            }
        });
        frame.stroke(
            &curve,
            Stroke::default()
                .with_width(self.width)
                .with_color(self.line)
                .with_line_cap(canvas::LineCap::Round),
        );
        vec![frame.into_geometry()]
    }
}

/// A dot crossing a track: its nine evenly timed frames, faint to solid, and in the window one more
/// dot at `phase`.
struct Onion {
    easing: Easing,
    dot: Color,
    track: Color,
    phase: Option<f32>,
}

impl canvas::Program<Message> for Onion {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let half = bounds.height / 4.0;
        let middle = bounds.height / 2.0;
        let travel = bounds.width - 2.0 * half;
        frame.fill(
            &Path::rounded_rectangle(
                Point::new(0.0, middle - half),
                IcedSize::new(bounds.width, 2.0 * half),
                half.into(),
            ),
            self.track,
        );
        // Each frame a little more solid than the last, so the order reads at a glance.
        for i in 0..FRAMES {
            let time = i as f32 / (FRAMES - 1) as f32;
            let x = half + travel * ease(self.easing, time);
            let opacity = 0.15 + 0.85 * time;
            frame.fill(
                &Path::circle(Point::new(x, middle), half * 0.8),
                color::alpha(self.dot, opacity),
            );
        }
        if let Some(phase) = self.phase {
            let x = half + travel * ease(self.easing, phase);
            frame.fill(&Path::circle(Point::new(x, middle), half * 1.2), self.dot);
        }
        vec![frame.into_geometry()]
    }
}
