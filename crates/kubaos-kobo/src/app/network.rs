//! The Wi-Fi screen's question: whether a network needs a password.

use super::App;

impl App {
    pub fn wifi_secured(&self, ssid: Option<&str>) -> bool {
        let (Some(p), Some(ssid)) = (self.ready_probe(), ssid) else {
            return false;
        };
        p.wifi.iter().any(|w| w.ssid == ssid && w.secured)
    }
}
