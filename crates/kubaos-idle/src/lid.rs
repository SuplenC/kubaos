//! What closing the lid does, done in the session rather than by logind, without root.
//!
//! logind's file (`system/files/logind.conf.d/lid.conf`) suspends on battery and ignores the lid
//! plugged in or docked. While the settings say the same and no Keep awake holds the lid, nothing
//! here runs and logind does it. When they say anything else, or Keep awake with the lid closed is
//! on, the session holds a `handle-lid-switch` inhibitor (`kubaos-idle lid hold` under
//! `systemd-inhibit`, the user unit [`UNIT`]), so logind leaves the lid alone, and Hyprland's lid
//! bind runs `kubaos-idle lid closed`, which does what the settings say, or nothing while a Keep
//! awake holds the lid. A plain Keep awake is none of this: it holds off the idle timers only, and
//! the lid does what the settings say. The holder exits with the Hyprland it was started in, and
//! once neither needs it with the lid open, so logind is always the fallback outside the session.
//!
//! logind also looks at a closed lid again while it stays closed, so a laptop closed plugged in
//! and then unplugged suspends. The holder does the same with [`Recheck`], which is also how a lid
//! closed when a lid-holding Keep awake ends gets its action: the holder stays while the lid is
//! closed.
//!
//! ```text
//! Lid held, lid closed ─▶ nothing ─Keep awake ends─▶ the setting, once closed HOLDOFF awake
//! ```
//!
//! # Owns
//!
//! - The decision, whether a holder is needed or may end, and the recheck.
//!
//! # Not here
//!
//! - Holding the inhibitor, the bind and running the action: `main.rs`, `config/hypr/keybinds.lua`.

use std::time::Duration;

use crate::settings::DEFAULT_LID_ON_AC;
use crate::settings::DEFAULT_LID_ON_BATTERY;
use crate::settings::LidAction;
use crate::settings::Power;
use crate::settings::Settings;

/// The holder's transient systemd user unit; one runs at a time, and `kubaos-idle lid closed`
/// acts only while it does.
pub const UNIT: &str = "kubaos-lid-hold";

/// How long a lid stays closed, awake, before a recheck suspends: logind's `HoldoffTimeoutSec`,
/// which also keeps a laptop that wakes with the lid closed awake that long first.
pub const HOLDOFF: Duration = Duration::from_secs(30);

/// How often the holder looks at the lid, the power and the session.
pub const TICK: Duration = Duration::from_secs(5);

/// What closing the lid does now: nothing when docked or `lid_held` by a Keep awake, else the
/// setting for the power in use.
pub fn action(settings: &Settings, docked: bool, power: Power, lid_held: bool) -> LidAction {
    if docked || lid_held {
        LidAction::Nothing
    } else {
        settings.on(power).lid
    }
}

/// Whether the session must hold the lid: a Keep awake holds it, or the settings aren't what
/// logind's file does.
pub fn needs_holder(settings: &Settings, lid_held: bool) -> bool {
    lid_held || settings.battery.lid != DEFAULT_LID_ON_BATTERY || settings.ac.lid != DEFAULT_LID_ON_AC
}

/// Whether a running holder may end: nothing needs it, and the lid is open. A lid that stays
/// closed keeps it, so its recheck gives the lid its action once the Keep awake holding it ends.
pub fn holder_ends(settings: &Settings, lid_held: bool, closed: bool) -> bool {
    !closed && !needs_holder(settings, lid_held)
}

/// The holder's recheck of a lid that stays closed, fed every [`TICK`] with the time awake.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recheck {
    /// When the lid was first seen closed, or the last suspend it asked for, whichever is later.
    since: Option<Duration>,
}

impl Recheck {
    /// Whether to suspend now, `now` being the time awake (a monotonic clock, which stands still
    /// while suspended). Only a suspend is rechecked: a lock, like logind's, is for the moment the
    /// lid closes.
    pub fn tick(&mut self, now: Duration, closed: bool, action: LidAction) -> bool {
        if !closed {
            self.since = None;
            return false;
        }
        let since = *self.since.get_or_insert(now);
        if action == LidAction::Suspend && now.saturating_sub(since) >= HOLDOFF {
            self.since = Some(now);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests;
