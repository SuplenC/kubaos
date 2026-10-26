//! What the toasts, the menu and `kubaos-idle status` say about the settings: "Lock after 15 min
//! idle on battery, 30 min plugged in".
//!
//! # Owns
//!
//! - The words for a wait, a timer's settings, the lid's, and a time of day.
//!
//! # Not here
//!
//! - The settings themselves: [`crate::settings`].

use jiff::Zoned;
use kubaos_clock::settings::TimeFormat;

use crate::settings::LidAction;
use crate::settings::Settings;
use crate::settings::Timer;
use crate::settings::Wait;

/// A wait as a person says it: `30 s`, `2 min`, `1 h 30 min`, `never`.
pub fn wait(wait: Wait) -> String {
    if wait.is_never() {
        return "never".to_owned();
    }
    let secs = wait.secs();
    let (hours, minutes, seconds) = (secs / 3600, secs / 60 % 60, secs % 60);
    let parts: Vec<String> = [(hours, "h"), (minutes, "min"), (seconds, "s")]
        .into_iter()
        .filter(|&(amount, _)| amount > 0)
        .map(|(amount, unit)| format!("{amount} {unit}"))
        .collect();
    parts.join(" ")
}

/// What `timer` does, as a sentence: "Lock after 2 min idle, on battery and plugged in", "Suspend
/// after 30 min idle on battery, never plugged in", "Never lock by itself".
pub fn timer(settings: &Settings, timer: Timer) -> String {
    let (battery, ac) = settings.waits(timer);
    let verb = match timer {
        Timer::Lock => "Lock",
        Timer::ScreensOff => "Screens off",
        Timer::Suspend => "Suspend",
    };
    if battery == ac {
        if battery.is_never() {
            return match timer {
                Timer::Lock => "Never lock by itself".to_owned(),
                Timer::ScreensOff => "Screens never go off by themselves".to_owned(),
                Timer::Suspend => "Never suspend by itself".to_owned(),
            };
        }
        return format!("{verb} after {} idle, on battery and plugged in", wait(battery));
    }
    // The source with a time comes first, so the sentence never starts "after never".
    let ((first, first_place), (second, second_place)) = if battery.is_never() {
        ((ac, "plugged in"), (battery, "on battery"))
    } else {
        ((battery, "on battery"), (ac, "plugged in"))
    };
    format!(
        "{verb} after {} idle {first_place}, {} {second_place}",
        wait(first),
        wait(second)
    )
}

/// The lid's settings as a sentence: "Closing the lid suspends on battery, does nothing plugged
/// in or docked".
pub fn lid(settings: &Settings) -> String {
    let verb = |action: LidAction| match action {
        LidAction::Suspend => "suspends",
        LidAction::Lock => "locks",
        LidAction::Nothing => "does nothing",
    };
    let mut doing: Vec<(LidAction, Vec<&str>)> = Vec::new();
    let mut nothing = Vec::new();
    for (action, place) in [(settings.battery.lid, "on battery"), (settings.ac.lid, "plugged in")] {
        if action == LidAction::Nothing {
            nothing.push(place);
        } else if let Some((_, places)) = doing.iter_mut().find(|(done, _)| *done == action) {
            places.push(place);
        } else {
            doing.push((action, vec![place]));
        }
    }
    if doing.is_empty() {
        return "Closing the lid does nothing".to_owned();
    }
    nothing.push("docked");
    let mut parts: Vec<String> = doing
        .into_iter()
        .map(|(action, places)| format!("{} {}", verb(action), places.join(" and ")))
        .collect();
    parts.push(format!("does nothing {}", nothing.join(" or ")));
    format!("Closing the lid {}", parts.join(", "))
}

/// Every setting, a sentence a line, as `kubaos-idle status` prints them.
pub fn all(settings: &Settings) -> String {
    let mut lines: Vec<String> = Timer::ALL.into_iter().map(|t| timer(settings, t)).collect();
    lines.push(lid(settings));
    lines.join("\n")
}

/// How long until something, said roughly: `30 s`, `10 min`, `2 h 5 min`, `3 days`. Seconds
/// count only under a minute, minutes are rounded to the nearest, and from two days on, days.
pub fn left(secs: i64) -> String {
    let secs = secs.max(0);
    if secs < 60 {
        return format!("{secs} s");
    }
    let minutes = (secs + 30) / 60;
    match minutes {
        ..60 => format!("{minutes} min"),
        60..2880 if minutes % 60 == 0 => format!("{} h", minutes / 60),
        60..2880 => format!("{} h {} min", minutes / 60, minutes % 60),
        _ => format!("{} days", (minutes + 720) / 1440),
    }
}

/// When something comes, to follow a verb: "at 17:05" today, else "tomorrow at 08:00", "Friday at
/// 17:00", "9 November at 09:00", as [`time_of_day`].
pub fn when(at: &Zoned, now: &Zoned, format: Option<TimeFormat>) -> String {
    let said = time_of_day(at, now, format);
    if at.date() == now.date() {
        format!("at {said}")
    } else {
        said
    }
}

/// A time of day as the shell writes it: the clock's time format, 24-hour unless it says 12.
/// Another day than `now`'s gets its weekday, and a week or more away its date.
pub fn time_of_day(at: &Zoned, now: &Zoned, format: Option<TimeFormat>) -> String {
    let time = match format {
        Some(TimeFormat::TwelveHour) => at.strftime("%-I:%M %p").to_string(),
        _ => at.strftime("%H:%M").to_string(),
    };
    let days = (at.date() - now.date()).get_days();
    match days {
        0 => time,
        1 => format!("tomorrow at {time}"),
        2..=6 => format!("{} at {time}", at.strftime("%A")),
        _ => format!("{} at {time}", at.strftime("%-d %B")),
    }
}

#[cfg(test)]
mod tests;
