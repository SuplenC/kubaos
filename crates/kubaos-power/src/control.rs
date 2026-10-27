//! Which stop the machine is on, and why: the user's choice on each power source, a profile set
//! from outside, a plug or an unplug, and the low-battery drop with its Undo. Pure: it takes what
//! happened and says what to do, so every path is tested without a bus or a battery.
//!
//! ```text
//! unplug ─▶ battery's stop ─(20 %)─▶ Power Saver ─(Undo)─▶ battery's stop again
//!   ▲              │                     (not remembered)
//!   └── plug in ◀──┴──▶ ac's stop
//! ```
//!
//! # Owns
//!
//! - [`Control`]: the stop now, what each [`Input`] changes, and the [`Effect`]s that follow.
//! - [`Settle`]: a power source has to hold before it counts, so a dock that bounces power doesn't
//!   swing the stop back and forth.
//!
//! # Not here
//!
//! - Doing the effects: `main.rs`'s service. What a stop sets: [`crate::plan`].

use std::time::Duration;
use std::time::Instant;

use crate::memory::Memory;
use crate::stop::Profile;
use crate::stop::Source;
use crate::stop::Stop;

/// How long a power source must hold before the stop follows it. A dock's USB-C power that drops
/// and comes back within this is one change at most.
pub const SETTLE: Duration = Duration::from_secs(3);

/// What happened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input {
    /// The user picked a stop: the slider, the Command menu, the command line.
    Set(Stop),
    /// power-profiles-daemon's profile changed; `held` while a program holds it there (ppd's
    /// `ActiveProfileHolds`: GameMode, a video call).
    Profile { profile: Profile, held: bool },
    /// The power source changed, and held for [`SETTLE`].
    Source(Source),
    /// The battery's charge, in percent.
    Percent(f64),
    /// The low-battery toast's Undo.
    Undo,
}

/// A toast the service shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toast {
    /// The stop followed the power source, a quiet one.
    Followed { source: Source, stop: Stop },
    /// The low-battery drop, with Undo.
    Dropped { to: Stop, from: Stop, percent: u8 },
}

/// What to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Ask power-profiles-daemon for this profile.
    SetProfile(Profile),
    /// Put every row of this stop to work.
    Apply(Stop),
    /// Write the memory down.
    Remember(Memory),
    Toast(Toast),
}

/// The low-battery drop's settings, from the user's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drop {
    /// The level it comes at, in percent.
    pub percent: u8,
    /// Where it goes; `None` turns it off.
    pub to: Option<Stop>,
}

/// The stop the machine is on, and what it remembers.
#[derive(Clone, Debug)]
pub struct Control {
    memory: Memory,
    source: Source,
    stop: Stop,
    percent: Option<f64>,
    drop: Drop,
    /// The profiles asked of power-profiles-daemon whose change hasn't come back yet, oldest
    /// first. Their changes are ours, not an outside one.
    asked: Vec<Profile>,
}

impl Control {
    /// Where the machine starts: on `source`, with power-profiles-daemon on `profile`.
    ///
    /// The stop it was left on, when the source is the same and the profile still fits it, so a
    /// restart or a reboot keeps Super Saver and a drop in progress. A profile that no longer fits
    /// was changed from outside while nothing watched, and the slider follows it. On another
    /// source, that source's remembered stop.
    pub fn start(memory: Memory, source: Source, profile: Option<Profile>, drop: Drop) -> (Control, Vec<Effect>) {
        let (stop, memory) = match memory.last {
            Some((last_source, last_stop)) if last_source == source => match profile {
                Some(profile) if profile != last_stop.profile() => {
                    let stop = Stop::from_profile(profile);
                    let mut memory = memory;
                    memory.choose(source, stop);
                    memory.dropped_from = None;
                    (stop, memory)
                }
                _ => (last_stop, memory),
            },
            _ => {
                let mut memory = memory;
                if source == Source::Ac {
                    memory.dropped_from = None;
                    memory.undone = false;
                }
                (memory.stop_for(source), memory)
            }
        };
        let mut control = Control {
            memory,
            source,
            stop,
            percent: None,
            drop,
            asked: Vec::new(),
        };
        let mut effects = Vec::new();
        if profile != Some(stop.profile()) {
            control.ask(stop.profile(), &mut effects);
        }
        effects.push(Effect::Apply(stop));
        control.memory.last = Some((source, stop));
        effects.push(Effect::Remember(control.memory));
        (control, effects)
    }

    pub fn stop(&self) -> Stop {
        self.stop
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn memory(&self) -> Memory {
        self.memory
    }

    /// The stop the low-battery drop left, while the drop is on.
    pub fn dropped_from(&self) -> Option<Stop> {
        self.memory.dropped_from
    }

    /// Takes the user's new low-battery settings.
    pub fn set_drop(&mut self, drop: Drop) {
        self.drop = drop;
    }

    /// What `input` changes, and what to do about it.
    pub fn handle(&mut self, input: Input) -> Vec<Effect> {
        let mut effects = Vec::new();
        match input {
            Input::Set(stop) => {
                if self.below_drop() {
                    self.memory.undone = true;
                }
                self.memory.dropped_from = None;
                self.memory.choose(self.source, stop);
                self.go(stop, &mut effects);
                effects.push(Effect::Remember(self.memory));
            }
            Input::Profile { profile, held } => self.profile(profile, held, &mut effects),
            Input::Source(source) => {
                if source == self.source {
                    return effects;
                }
                self.source = source;
                if source == Source::Ac {
                    self.memory.dropped_from = None;
                    self.memory.undone = false;
                }
                let stop = self.memory.stop_for(source);
                let moved = stop != self.stop;
                self.go(stop, &mut effects);
                if moved {
                    effects.push(Effect::Toast(Toast::Followed { source, stop }));
                }
                self.check_drop(&mut effects);
                effects.push(Effect::Remember(self.memory));
            }
            Input::Percent(percent) => {
                self.percent = Some(percent);
                if self.check_drop(&mut effects) {
                    effects.push(Effect::Remember(self.memory));
                }
            }
            Input::Undo => {
                if self.memory.dropped_from.take().is_some() {
                    self.memory.undone = true;
                    let stop = self.memory.stop_for(self.source);
                    self.go(stop, &mut effects);
                    effects.push(Effect::Remember(self.memory));
                }
            }
        }
        effects
    }

    /// A profile change from power-profiles-daemon: ours coming back, or one from outside, which
    /// the slider follows. A profile set from outside is the source's choice, the stop named like
    /// it. A program's hold is followed but never remembered, and its end, which ppd marks by going
    /// back to the profile before it, returns to the source's own stop (Super Saver, not Power
    /// Saver).
    fn profile(&mut self, profile: Profile, held: bool, effects: &mut Vec<Effect>) {
        if let Some(at) = self.asked.iter().position(|&asked| asked == profile) {
            self.asked.drain(..=at);
            return;
        }
        self.asked.clear();
        if profile == self.stop.profile() {
            return;
        }
        let remembered = self.memory.stop_for(self.source);
        let stop = if profile == remembered.profile() {
            remembered
        } else {
            Stop::from_profile(profile)
        };
        if !held {
            if self.below_drop() {
                self.memory.undone = true;
            }
            self.memory.dropped_from = None;
            self.memory.choose(self.source, stop);
        }
        self.stop = stop;
        effects.push(Effect::Apply(stop));
        self.memory.last = Some((self.source, stop));
        effects.push(Effect::Remember(self.memory));
    }

    /// Moves to `stop`: power-profiles-daemon first when its profile changes, then the rest.
    fn go(&mut self, stop: Stop, effects: &mut Vec<Effect>) {
        if stop.profile() != self.stop.profile() {
            self.ask(stop.profile(), effects);
        }
        if stop != self.stop {
            self.stop = stop;
            effects.push(Effect::Apply(stop));
        }
        self.memory.last = Some((self.source, stop));
    }

    fn ask(&mut self, profile: Profile, effects: &mut Vec<Effect>) {
        self.asked.push(profile);
        effects.push(Effect::SetProfile(profile));
    }

    /// On battery at or under the drop's level.
    fn below_drop(&self) -> bool {
        self.source == Source::Battery
            && self
                .percent
                .is_some_and(|percent| percent <= f64::from(self.drop.percent))
    }

    /// Drops to the low-battery stop when the battery reached its level, unless the drop is off,
    /// was undone, or the stop is already as low. Says whether it dropped.
    fn check_drop(&mut self, effects: &mut Vec<Effect>) -> bool {
        let Some(to) = self.drop.to else {
            return false;
        };
        if !self.below_drop() || self.memory.undone || self.memory.dropped_from.is_some() || self.stop <= to {
            return false;
        }
        let from = self.stop;
        self.memory.dropped_from = Some(from);
        self.go(to, effects);
        effects.push(Effect::Toast(Toast::Dropped {
            to,
            from,
            percent: self.drop.percent,
        }));
        true
    }
}

/// How long the machine's devices settle before the stop's writes are checked again: a dock's
/// replug or a screen's arrival is a burst of uevents, and power-profiles-daemon puts the GPU back
/// to `auto` on each.
pub const RECHECK: Duration = Duration::from_secs(1);
/// The longest a recheck waits from the first event, however many follow: a device that sends
/// events nonstop can't put it off for ever.
pub const RECHECK_AT_MOST: Duration = Duration::from_secs(5);

/// A check of the stop's writes, after devices came or power-profiles-daemon acted: once the burst
/// of events has been quiet for [`RECHECK`], or [`RECHECK_AT_MOST`] after its first event.
#[derive(Clone, Copy, Debug, Default)]
pub struct Recheck {
    due: Option<Instant>,
    /// The first event of the burst.
    since: Option<Instant>,
}

impl Recheck {
    /// Something changed at `now`.
    pub fn ask(&mut self, now: Instant) {
        let since = *self.since.get_or_insert(now);
        self.due = Some((now + RECHECK).min(since + RECHECK_AT_MOST));
    }

    /// Whether the check is due by `now`, once.
    pub fn due(&mut self, now: Instant) -> bool {
        match self.due {
            Some(due) if now >= due => {
                self.due = None;
                self.since = None;
                true
            }
            _ => false,
        }
    }

    /// How long until it is due, `None` when nothing waits.
    pub fn wait(&self, now: Instant) -> Option<Duration> {
        self.due.map(|due| due.saturating_duration_since(now))
    }
}

/// A power source on its way to counting: it counts once it has held for [`SETTLE`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Settle {
    pending: Option<(Source, Instant)>,
}

impl Settle {
    /// UPower says the source is `source` at `now`; `current` is the one the stop follows.
    pub fn observe(&mut self, source: Source, current: Source, now: Instant) {
        self.pending = match self.pending {
            Some((pending, since)) if pending == source => Some((pending, since)),
            _ if source == current => None,
            _ => Some((source, now)),
        };
    }

    /// The source that has held long enough by `now`, once.
    pub fn due(&mut self, now: Instant) -> Option<Source> {
        let (source, since) = self.pending?;
        if now.duration_since(since) < SETTLE {
            return None;
        }
        self.pending = None;
        Some(source)
    }

    /// How long until [`Settle::due`] has something, `None` when nothing waits.
    pub fn wait(&self, now: Instant) -> Option<Duration> {
        let (_, since) = self.pending?;
        Some(SETTLE.saturating_sub(now.duration_since(since)))
    }
}

#[cfg(test)]
mod tests;
