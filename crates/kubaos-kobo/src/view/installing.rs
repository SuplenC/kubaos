//! The Installing and Failed screens: the steps as they run, the log under them, and what to do
//! when one fails.

use iced::Length;
use iced::widget;
use kubaos_core::Level;
use kubaos_ui::Look;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::error::Level as LogLevel;
use kubaos_ui::component::error::error_block;
use kubaos_ui::component::error::log;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::component::progress::timed_bar;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::motion::Motion;
use kubaos_ui::text::line;
use kubaos_ui::text::text;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::progress::LOG_VIEW_LINES;
use crate::progress::Outcome;
use crate::progress::StepState;
use crate::progress::fmt_duration;
use crate::theme::jp;
use crate::ui::El;

/// Asks for the charger on Last chance when the battery is low. It warns only: an install on a low
/// battery is the owner's call.
pub fn low_battery_note(app: &App) -> Option<String> {
    let b = app.battery.filter(|b| b.is_low())?;
    Some(format!("{}% left. Plug the damn thing in.", b.percent))
}

fn step_lead<'a>(state: StepState, seconds: f32, look: &Look) -> El<'a> {
    let c = look.colors;
    match state {
        StepState::Done => icon(look, Icon::CheckCircle, IconSize::S20, c.primary),
        StepState::Running => spinner(look, IconSize::S20, seconds, c.primary),
        StepState::Failed => icon(look, Icon::Error, IconSize::S20, c.error),
        StepState::Pending => icon(look, Icon::Circle, IconSize::S20, c.outline_variant),
    }
}

fn steps_list<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let seconds = (app.now - app.epoch).as_secs_f32();
    let rows: Vec<El> = inst
        .steps
        .iter()
        .map(|s| {
            let timing = match (s.state, s.elapsed, s.started) {
                (_, Some(e), _) => fmt_duration(e),
                (StepState::Running, None, Some(st)) => fmt_duration(app.now.saturating_duration_since(st)),
                _ => String::new(),
            };
            let (colour, style) = match s.state {
                StepState::Running => (c.on_surface, Type::BodyStrong),
                StepState::Done => (c.on_surface, Type::Body),
                StepState::Failed => (c.error, Type::BodyStrong),
                StepState::Pending => (c.on_surface_variant, Type::Body),
            };
            let mut r = row(look, Space::S12)
                .center()
                .push(step_lead(s.state, seconds, look))
                .push(text(s.info.title.clone(), style, colour))
                .push(fill());
            if s.state == StepState::Running && s.fraction > 0.0 && s.fraction < 1.0 {
                r = r.push(text(format!("{:.0}%", s.fraction * 100.0), Type::Mono, c.primary));
            }
            r.push(text(timing, Type::Mono, c.on_surface_variant)).into()
        })
        .collect();
    // Two columns, down the first then the second: the list is short and the panel is wide.
    let half = rows.len().div_ceil(2);
    let mut first = stack(look, Space::S8);
    let mut second = stack(look, Space::S8);
    for (i, r) in rows.into_iter().enumerate() {
        if i < half {
            first = first.push(r);
        } else {
            second = second.push(r);
        }
    }
    grid(look, 2, Space::S40).push(first).push(second).into()
}

/// The install's log, its newest lines, `height` tall.
fn log_view<'a>(app: &'a App, look: &Look, height: Extent) -> El<'a> {
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let skip = inst.log.len().saturating_sub(LOG_VIEW_LINES);
    let lines: Vec<(LogLevel, &str)> = inst.log[skip..]
        .iter()
        .map(|l| {
            let level = match l.level {
                Level::Error => LogLevel::Error,
                Level::Warn => LogLevel::Warn,
                Level::Info if l.text.starts_with("==>") => LogLevel::Step,
                Level::Info => LogLevel::Info,
                Level::Debug => LogLevel::Debug,
            };
            (level, l.text.as_str())
        })
        .collect();
    log(look, &lines, height)
}

/// How far the log is open, 0 to 1: it opens over medium as things arrive, and closes the same way.
fn log_openness(app: &App) -> f32 {
    let shown = Motion::default().progress(
        tokens::Duration::Medium,
        Easing::EmphasizedDecel,
        app.now - app.log_open.1,
    );
    if app.log_open.0 { shown } else { 1.0 - shown }
}

fn log_toggle<'a>(app: &'a App, look: &Look) -> El<'a> {
    button(if app.log_open.0 { "Hide log" } else { "Show log" })
        .kind(Kind::Text)
        .icon(Icon::Terminal)
        .on_press(Message::Click(F::ToggleLog))
        .focused(app.is_focused(F::ToggleLog))
        .view(look)
}

pub(super) fn installing<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let current = inst
        .current()
        .map_or_else(|| "Starting…".into(), |s| s.info.title.clone());
    let detail = inst.detail.clone().unwrap_or_default();
    let elapsed = fmt_duration(app.now.saturating_duration_since(inst.started));
    let open = look.scale.snap(log_openness(app) * f32::from(Size::Log.px()));

    let mut page = stack(look, Gap::NONE)
        .height(Extent::Fill)
        .push(
            row(look, Space::S16)
                .bottom()
                .push(header(
                    look,
                    jp::INSTALLING,
                    "Installing",
                    "Go stare at a wall, it'll be done before you get bored.",
                ))
                .push(fill())
                .push(text(
                    format!("{:.0}%", inst.shown * 100.0),
                    Type::HeroNumber,
                    c.on_surface,
                )),
        )
        .push(gap(Space::S20))
        // `shown` already eases toward the work on every tick.
        .push(timed_bar(look, inst.shown))
        .push(gap(Space::S12))
        .push(
            row(look, Space::S12)
                .center()
                .push(text(current, Type::BodyStrong, c.on_surface))
                .push(text(detail, Type::Mono, c.primary))
                .push(fill())
                .push(icon(look, Icon::Timer, IconSize::S16, c.on_surface_variant))
                .push(text(elapsed, Type::Mono, c.on_surface_variant)),
        )
        .push(gap(Space::S32))
        .push(widget::container(steps_list(app, look)).height(Length::Fill).clip(true));
    if open > 0.0 {
        page = page
            .push(
                widget::container(log_view(app, look, Extent::Fill))
                    .height(open)
                    .clip(true),
            )
            .push(gap(Space::S12));
    }
    let last = inst
        .log
        .last()
        .filter(|_| !app.log_open.0)
        .map(|l| l.text.clone())
        .unwrap_or_default();
    page.push(row(look, Space::S12).center().push(log_toggle(app, look)).push(line(
        last,
        Type::Mono,
        c.on_surface_variant,
    )))
    .into()
}

pub(super) fn failed<'a>(app: &'a App, look: &Look) -> El<'a> {
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let (step, error, hint) = match &inst.outcome {
        Some(Outcome::Failed { step, error, hint }) => (*step, error.as_str(), hint.clone()),
        _ => (None, "No idea what happened", None),
    };
    let at = step
        .and_then(|id| inst.steps.iter().find(|s| s.info.id == id))
        .map_or_else(
            || "The install stopped before it got anywhere.".into(),
            |s| {
                format!(
                    "“{}” died{}. Nothing after it ran.",
                    s.info.title,
                    s.elapsed
                        .map(|e| format!(" after {}", fmt_duration(e)))
                        .unwrap_or_default()
                )
            },
        );
    let hint = hint.unwrap_or_else(|| "The log has the gory details. Try again, or save it and go read it.".into());
    let open = log_openness(app);
    let mut page = stack(look, Gap::NONE)
        .height(Extent::Fill)
        .push(header(look, jp::FAILED, "Fuck.", at))
        .push(gap(Space::S20))
        .push(error_block(look, error, hint))
        .push(gap(Space::S16));
    // The log takes its share of the room as it opens, and gives it back as it closes. Open or
    // closed, one of the two has all of it, so its edges are whole device pixels.
    let share = (open * 100.0) as u16;
    let (log_room, rest) = match share {
        0 => (None, Length::Fill),
        100.. => (Some(Length::Fill), Length::Shrink),
        _ => (Some(Length::FillPortion(share)), Length::FillPortion(100 - share)),
    };
    if let Some(room) = log_room {
        page = page.push(widget::container(log_view(app, look, Extent::Fill)).height(room));
    }
    page.push(widget::Space::new().height(rest))
        .push(gap(Space::S16))
        .push(
            row(look, Space::S12)
                .center()
                .push(
                    button("Back to the review")
                        .kind(Kind::Text)
                        .icon(Icon::ArrowBack)
                        .on_press(Message::Click(F::Back))
                        .focused(app.is_focused(F::Back))
                        .view(look),
                )
                .push(log_toggle(app, look))
                .push(fill())
                .push(
                    button("Save log")
                        .kind(Kind::Tonal)
                        .icon(Icon::Save)
                        .on_press(Message::Click(F::SaveLog))
                        .focused(app.is_focused(F::SaveLog))
                        .view(look),
                )
                .push(
                    button("Try again")
                        .icon(Icon::RestartAlt)
                        .on_press(Message::Click(F::Retry))
                        .focused(app.is_focused(F::Retry))
                        .view(look),
                ),
        )
        .into()
}
