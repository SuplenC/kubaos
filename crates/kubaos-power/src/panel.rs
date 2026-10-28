//! The laptop panel's refresh rate and VRR, which Hyprland sets: the service writes what the stop
//! wants to a file, and `config/hypr/monitors.lua` turns it into the panel's monitor rule, both
//! when the service asks (`hyprctl eval 'dot.power_panel()'`) and at every config reload.
//!
//! VRR on or off is one atomic commit without a modeset, so it never shows. A refresh rate is a
//! new mode, and a new mode is a modeset that blanks the panel for a moment, unless amdgpu keeps
//! the timing and stretches only the vertical blank (`amdgpu.freesync_video=1`,
//! `system/files/modprobe.d/kubaos-amdgpu.conf`). Without that the panel keeps its native rate
//! and saves through VRR instead, and `status` says why.
//!
//! # Owns
//!
//! - [`Target`]: what the panel gets for a [`Panel`] on this machine, and the file's text.
//!
//! # Not here
//!
//! - The monitor rule, which keeps the panel's position, scale and transform as they are, and never
//!   touches another screen: `config/hypr/monitors.lua`.

use std::path::Path;
use std::path::PathBuf;

use crate::plan::Panel;

/// The file in `$XDG_RUNTIME_DIR` that `config/hypr/monitors.lua` reads.
pub const FILE: &str = "kubaos-power/panel";
/// What the service asks Hyprland to run once the file is written.
pub const EVAL: &str = "dot.power_panel()";

/// What the panel gets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    /// The refresh rate in Hz, `None` for the native one.
    pub refresh: Option<u32>,
    pub vrr: bool,
    /// The stop's refresh rate was left for VRR, since it would have blanked the panel.
    pub held_back: bool,
}

impl Target {
    /// What `panel` comes to where switching the refresh rate is `seamless` or not.
    pub fn of(panel: Panel, seamless: bool) -> Target {
        match panel.refresh {
            Some(_) if !seamless => Target {
                refresh: None,
                vrr: true,
                held_back: true,
            },
            refresh => Target {
                refresh,
                vrr: panel.vrr,
                held_back: false,
            },
        }
    }

    /// The file's text: `refresh=60` or `refresh=native`, then `vrr=0` or `vrr=1`.
    pub fn text(self) -> String {
        let refresh = self.refresh.map_or_else(|| "native".to_owned(), |hz| hz.to_string());
        format!("refresh={refresh}\nvrr={}\n", u8::from(self.vrr))
    }

    /// The words for `status`: "60 Hz", "native rate with VRR".
    pub fn words(self) -> String {
        let rate = self
            .refresh
            .map_or_else(|| "native rate".to_owned(), |hz| format!("{hz} Hz"));
        if self.vrr { format!("{rate} with VRR") } else { rate }
    }
}

/// The file under the runtime dir `runtime`.
pub fn path(runtime: &Path) -> PathBuf {
    runtime.join(FILE)
}

#[cfg(test)]
mod tests;
