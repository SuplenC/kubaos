//! Sizes and times as alerts and the Command menu write them: `9.1 GB`, `3 h 12 min`. The panel
//! writes them the same way in QML (`config/quickshell/services/Sysmon.qml`).
//!
//! # Owns
//!
//! - Writing bytes, rates and durations for people.

/// `512 MB`, `9.1 GB`, `32 GB`: powers of 1024, with one decimal under ten.
pub fn bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 || value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// `3 h 12 min`, `45 min`, `2 d 4 h`: the two largest units.
pub fn duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = seconds % 86_400 / 3600;
    let minutes = seconds % 3600 / 60;
    if days > 0 {
        format!("{days} d {hours} h")
    } else if hours > 0 {
        format!("{hours} h {minutes} min")
    } else {
        format!("{minutes} min")
    }
}

#[cfg(test)]
mod tests {
    use super::bytes;
    use super::duration;

    #[test]
    fn sizes_take_the_largest_unit_with_a_decimal_under_ten() {
        assert_eq!(bytes(512), "512 B");
        assert_eq!(bytes(512 * 1024 * 1024), "512 MB");
        assert_eq!(bytes(9_771_000_000), "9.1 GB");
        assert_eq!(bytes(32 * 1024 * 1024 * 1024), "32 GB");
    }

    #[test]
    fn durations_take_their_two_largest_units() {
        assert_eq!(duration(45 * 60), "45 min");
        assert_eq!(duration(3 * 3600 + 12 * 60), "3 h 12 min");
        assert_eq!(duration(2 * 86_400 + 4 * 3600 + 59), "2 d 4 h");
    }
}
