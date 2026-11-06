//! The critical-battery warning as it reaches a person: a critical toast and a sound, from the
//! session service, which runs whether or not the session is locked.
//!
//! # Owns
//!
//! - [`ring`] and the toast's words.
//!
//! # Not here
//!
//! - When it fires: [`crate::control`]. The level: [`crate::settings`]. The sound's file:
//!   `home/power.nix`.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use crate::settings::UPOWER_ACTION_PERCENT;

/// How long a toast keeps being offered to a shell that isn't up yet. A boot already under the
/// level warns before the shell's notification server has started, and notify-send fails at once
/// without one.
const OFFERS: u32 = 30;
const BETWEEN_OFFERS: Duration = Duration::from_secs(2);

/// The toast's title at `percent`.
pub fn title(percent: u8) -> String {
    format!("Battery at {percent} %")
}

/// The toast's body at `percent`. At UPower's own level the power-off is already on its way, and
/// plugging in within its 20 s still calls it off.
pub fn body(percent: u8) -> String {
    if percent > UPOWER_ACTION_PERCENT {
        format!("The laptop powers off at {UPOWER_ACTION_PERCENT} %. Plug it in now.")
    } else {
        "The laptop powers off within seconds. Plug it in now.".to_owned()
    }
}

/// Plays the sound and shows the toast, each on a thread of its own, so a shell that isn't up
/// delays neither the sound nor the service.
pub fn ring(percent: u8) {
    let (title, body) = (title(percent), body(percent));
    log::warn!("{title}: {body}");
    thread::spawn(play);
    thread::spawn(move || offer(&title, &body));
}

/// The sound, through PipeWire, which the lock screen doesn't stand in front of.
fn play() {
    let Some(sound) = sound() else {
        log::warn!("no HOME or XDG_DATA_HOME to find the critical-battery sound in");
        return;
    };
    let played = Command::new("pw-play")
        .arg(&sound)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if !played.is_ok_and(|status| status.success()) {
        log::warn!("couldn't play {}", sound.display());
    }
}

/// The sound's file, where `home/power.nix` puts it.
fn sound() -> Option<PathBuf> {
    let data = env::var_os("XDG_DATA_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(env::var_os("HOME")?).join(".local/share")))?;
    Some(data.join("kubaos/power/critical.oga"))
}

/// Offers the toast until a notification server takes it. Critical, so Do not disturb lets it up
/// and it stays until dismissed: behind the lock, it is there at the unlock.
fn offer(title: &str, body: &str) {
    for _ in 0..OFFERS {
        let taken = Command::new("notify-send")
            .args(["-a", "Power", "-i", "battery-caution", "-u", "critical", title, body])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if taken {
            return;
        }
        thread::sleep(BETWEEN_OFFERS);
    }
    log::warn!("no notification server took the critical-battery toast");
}

#[cfg(test)]
mod tests;
