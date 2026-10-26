//! hypridle's listeners, from the settings: the file `config/hypr/hypridle.conf` sources at
//! `~/.local/state/kubaos/idle/hypridle.conf`, before the user's own `hypridle.conf`.
//!
//! A timer with the same wait on battery and plugged in is one listener. With two waits, each
//! source gets a listener of its own whose `condition_cmd` asks `dot-hypr-on-ac`, and
//! `condition_retry` asks again every minute while still idle. So a power change while idle takes
//! effect within a minute: unplugged past the battery's wait, the battery's listener fires at its
//! next retry; plugged in, it keeps waiting and the plugged-in listener fires at its own time.
//!
//! # Owns
//!
//! - The listeners' text and where the file goes.
//!
//! # Not here
//!
//! - hypridle's `general` block and the sources: `config/hypr/hypridle.conf`. Restarting hypridle:
//!   `scripts/dot-session start-idle`, which `kubaos-idle` dispatches through Hyprland.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::settings::Settings;
use crate::settings::Timer;
use crate::settings::Wait;
use crate::words;

/// The file under the home. `config/hypr/hypridle.conf` names it literally, since hyprlang has no
/// variables for `source`, so it stays under `~/.local/state` whatever `XDG_STATE_HOME` says.
pub const FILE: &str = ".local/state/kubaos/idle/hypridle.conf";

/// How often a listener whose power source isn't the one in use asks again while idle, in
/// seconds: how late a power change while idle can be.
pub const CONDITION_RETRY: u32 = 60;

/// The file for the home `home`.
pub fn path(home: &Path) -> PathBuf {
    home.join(FILE)
}

/// The listeners' file for `settings`.
pub fn config(settings: &Settings) -> String {
    let mut out = String::from(
        "# Written by kubaos-idle from ~/.config/kubaos/desktop/idle/idle.toml at every change.\n\
         # Edits here are lost: change the times there, or from the Command menu.\n",
    );
    for timer in Timer::ALL {
        let _ = writeln!(out, "#\n# {}.", words::timer(settings, timer));
    }
    for timer in Timer::ALL {
        let (battery, ac) = settings.waits(timer);
        if battery == ac {
            listener(&mut out, timer, battery, None);
        } else {
            listener(&mut out, timer, battery, Some("! dot-hypr-on-ac"));
            listener(&mut out, timer, ac, Some("dot-hypr-on-ac"));
        }
    }
    out
}

/// One listener for `timer` after `wait`, none for never; `condition` is the shell test that lets
/// it fire.
fn listener(out: &mut String, timer: Timer, wait: Wait, condition: Option<&str>) {
    if wait.is_never() {
        return;
    }
    let _ = write!(out, "\nlistener {{\n    timeout = {}\n", wait.secs());
    if let Some(condition) = condition {
        let _ = write!(
            out,
            "    condition_cmd = {condition}\n    condition_retry = {CONDITION_RETRY}\n"
        );
    }
    let (on_timeout, on_resume) = match timer {
        Timer::Lock => ("loginctl lock-session", None),
        Timer::ScreensOff => (
            r#"hyprctl dispatch 'hl.dsp.dpms({ action = "disable" })'"#,
            Some(r#"hyprctl dispatch 'hl.dsp.dpms({ action = "enable" })'"#),
        ),
        Timer::Suspend => ("systemctl suspend", None),
    };
    let _ = writeln!(out, "    on-timeout = {on_timeout}");
    if let Some(on_resume) = on_resume {
        let _ = writeln!(out, "    on-resume = {on_resume}");
    }
    out.push_str("}\n");
}

/// Writes the listeners for `settings` to `path`, through a temporary file and a rename.
///
/// # Errors
///
/// When the folder can't be made or the file written.
pub fn write(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("conf.new");
    fs::write(&tmp, config(settings))?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests;
