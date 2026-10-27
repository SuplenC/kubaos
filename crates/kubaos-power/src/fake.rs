//! A machine of the tests' own: a folder laid out as `/sys` and `/proc` are, the reference
//! laptop's knobs in it, each of which a test can take away.

use std::fs;

use tempfile::TempDir;

use crate::machine::Machine;

/// A fake `/` in a temporary folder.
pub struct FakeRoot {
    pub dir: TempDir,
}

impl FakeRoot {
    /// An empty machine: no knob at all.
    pub fn empty() -> FakeRoot {
        FakeRoot {
            dir: tempfile::tempdir().expect("a temp dir"),
        }
    }

    /// The reference laptop as it stood on 2026-10-27, on battery in Power Saver: two CPU
    /// policies, amdgpu's card1 with eDP-1, ASPM at `default`, a webcam, a Logitech receiver, an
    /// Ethernet adapter, a hub and a security key, Wi-Fi, TUXEDO's charging profile and a battery
    /// drawing 16.8 W.
    pub fn laptop() -> FakeRoot {
        let root = FakeRoot::empty();
        for policy in ["policy0", "policy1", "policy10"] {
            let dir = format!("sys/devices/system/cpu/cpufreq/{policy}");
            root.write(&format!("{dir}/energy_performance_preference"), "power");
            root.write(&format!("{dir}/boost"), "0");
            root.write(&format!("{dir}/scaling_max_freq"), "2000000");
            root.write(&format!("{dir}/cpuinfo_max_freq"), "2000000");
        }
        root.write("sys/firmware/acpi/platform_profile", "low-power");
        root.write("sys/class/drm/card1/device/power_dpm_force_performance_level", "auto");
        root.write("sys/class/drm/card1-eDP-1/status", "connected");
        root.write("sys/class/drm/card1-eDP-1/amdgpu/panel_power_savings", "0");
        root.write("sys/class/drm/card1-HDMI-A-1/status", "connected");
        root.dir_at("sys/class/drm/renderD128");
        root.write(
            "sys/module/pcie_aspm/parameters/policy",
            "[default] performance powersave powersupersave",
        );
        root.usb("1-1", "2b7e", "c906", &["0e", "0e"], "auto", "FHD WebCam");
        root.usb("7-1", "2109", "2822", &["09"], "auto", "USB2.0 Hub");
        root.usb("7-1.1.3", "046d", "c547", &["03", "03"], "on", "USB Receiver");
        root.usb("8-1.1.2", "0bda", "8153", &["ff"], "on", "USB 10/100/1000 LAN");
        root.usb("3-2", "1050", "0407", &["0b"], "on", "YubiKey FIDO+CCID");
        root.dir_at("sys/class/net/wlp99s0/wireless");
        root.dir_at("sys/class/net/eno1");
        root.write(
            "sys/devices/platform/tuxedo_keyboard/charging_profile/charging_profile",
            "high_capacity",
        );
        root.write("sys/class/power_supply/AC0/type", "Mains");
        root.write("sys/class/power_supply/AC0/online", "0");
        root.write("sys/class/power_supply/BAT0/type", "Battery");
        root.write("sys/class/power_supply/BAT0/status", "Discharging");
        root.write("sys/class/power_supply/BAT0/capacity", "64");
        root.write("sys/class/power_supply/BAT0/current_now", "1050000");
        root.write("sys/class/power_supply/BAT0/voltage_now", "16000000");
        root.write("sys/class/power_supply/hidpp_battery_5/type", "Battery");
        root.write("sys/class/power_supply/hidpp_battery_5/scope", "Device");
        root.write("sys/module/amdgpu/parameters/freesync_video", "1");
        root
    }

    pub fn machine(&self) -> Machine {
        Machine::at(self.dir.path())
    }

    pub fn path(&self, relative: &str) -> std::path::PathBuf {
        self.dir.path().join(relative)
    }

    pub fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(path, format!("{text}\n")).expect("write a fake file");
    }

    pub fn remove(&self, relative: &str) {
        let path = self.path(relative);
        if path.is_dir() {
            fs::remove_dir_all(path).expect("remove a fake folder");
        } else {
            fs::remove_file(path).expect("remove a fake file");
        }
    }

    pub fn dir_at(&self, relative: &str) {
        fs::create_dir_all(self.path(relative)).expect("mkdir");
    }

    /// A USB device with one interface a class.
    pub fn usb(&self, name: &str, vendor: &str, product: &str, classes: &[&str], control: &str, title: &str) {
        let dir = format!("sys/bus/usb/devices/{name}");
        self.write(&format!("{dir}/idVendor"), vendor);
        self.write(&format!("{dir}/idProduct"), product);
        self.write(&format!("{dir}/power/control"), control);
        self.write(&format!("{dir}/product"), title);
        for (number, class) in classes.iter().enumerate() {
            self.write(&format!("{dir}/{name}:1.{number}/bInterfaceClass"), class);
            // The kernel lists each interface beside its device too.
            self.dir_at(&format!("sys/bus/usb/devices/{name}:1.{number}"));
        }
    }

    /// A fake file's text, trimmed.
    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative))
            .expect("read a fake file")
            .trim()
            .to_owned()
    }
}
