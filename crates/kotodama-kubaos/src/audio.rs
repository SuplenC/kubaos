//! The audio devices `wpctl status` lists: PipeWire's sinks (outputs) and
//! sources (inputs), and which of each is the default.
//!
//! # Owns
//!
//! - Reading the device list out of `wpctl status`.
//!
//! # Not here
//!
//! - Running `wpctl`, and what picking a device does: [`crate::toggles`].

/// Which list of devices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Sinks: speakers, headphones, HDMI.
    Output,
    /// Sources: microphones.
    Input,
}

impl Direction {
    /// The heading of its list in `wpctl status`.
    fn heading(self) -> &'static str {
        match self {
            Direction::Output => "Sinks:",
            Direction::Input => "Sources:",
        }
    }
}

/// One sink or source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    /// PipeWire's object id, which `wpctl set-default` takes.
    pub id: u32,
    pub name: String,
    /// Whether it is the default, which `wpctl status` marks with `*`.
    pub default: bool,
}

/// The devices of `direction` in `status`, the output of `wpctl status`, in
/// its order. Its `Audio` section holds the lists, each under a heading in a
/// tree drawn with box characters:
///
/// ```text
/// Audio
///  ├─ Sinks:
///  │  *   47. Built-in Audio Analog Stereo        [vol: 0.50]
///  │      52. HDMI / DisplayPort                   [vol: 1.00]
///  │
///  ├─ Sources:
/// ```
pub fn devices(status: &str, direction: Direction) -> Vec<Device> {
    let mut in_audio = false;
    let mut in_list = false;
    let mut devices = Vec::new();
    for line in status.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() {
            in_audio = line.trim() == "Audio";
            in_list = false;
            continue;
        }
        let text = line.trim_start_matches([' ', '│', '├', '└', '─']).trim_end();
        if text.ends_with(':') {
            in_list = in_audio && text == direction.heading();
            continue;
        }
        if in_list && let Some(device) = device(text) {
            devices.push(device);
        }
    }
    devices
}

/// One device line, as `*   47. Built-in Audio Analog Stereo   [vol: 0.50]`.
fn device(line: &str) -> Option<Device> {
    let (default, rest) = match line.strip_prefix('*') {
        Some(rest) => (true, rest.trim_start()),
        None => (false, line),
    };
    let (id, rest) = rest.split_once(". ")?;
    let name = match rest.rfind(" [") {
        Some(at) if rest.ends_with(']') => &rest[..at],
        _ => rest,
    };
    Some(Device {
        id: id.parse().ok()?,
        name: name.trim().to_owned(),
        default,
    })
}

#[cfg(test)]
mod tests;
