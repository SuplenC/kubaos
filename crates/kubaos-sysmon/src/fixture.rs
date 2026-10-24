//! A fake machine for the tests: a folder holding the `/proc` and `/sys` files a sample reads,
//! changed between samples as the kernel would change them.

use std::fs;
use std::path::Path;

/// A laptop's files, under a temporary folder.
pub(crate) struct Machine {
    dir: tempfile::TempDir,
}

impl Machine {
    /// Two cores, 16 GiB, a battery, an AMD GPU, Wi-Fi and one disk, as the first read sees them.
    pub(crate) fn laptop() -> Self {
        let machine = Machine {
            dir: tempfile::tempdir().expect("a temp dir"),
        };
        machine.write("proc/sys/kernel/osrelease", "6.17.0-5-generic\n");
        machine.write("proc/uptime", "3725.42 7000.00\n");
        machine.write("proc/loadavg", "0.50 0.40 0.30 1/500 4242\n");
        machine.stat(1000, 800);
        machine.write(
            "proc/meminfo",
            "MemTotal: 16777216 kB\nMemFree: 1048576 kB\nMemAvailable: 1048576 kB\nCached: 524288 kB\n\
             SwapTotal: 0 kB\nSwapFree: 0 kB\n",
        );
        machine.write(
            "proc/net/route",
            "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n\
             wlan0\t00000000\t0101A8C0\t0003\t0\t0\t600\t00000000\t0\t0\t0\n\
             wlan0\t0001A8C0\t00000000\t0001\t0\t0\t600\t00FFFFFF\t0\t0\t0\n",
        );
        machine.write(
            "proc/net/fib_trie",
            "Main:\n  +-- 192.168.1.0/24 2 0 2\n     |-- 192.168.1.42\n        /32 host LOCAL\n",
        );
        machine.net(0, 0);
        machine.diskstats(0, 0);
        machine.write("sys/block/nvme0n1/size", "1000\n");
        machine.write("proc/self/mounts", "/dev/nvme0n1p2 / ext4 rw 0 0\n");
        machine.write("sys/class/hwmon/hwmon0/name", "k10temp\n");
        machine.write("sys/class/hwmon/hwmon0/temp1_input", "55000\n");
        machine.write("sys/class/hwmon/hwmon0/temp1_label", "Tctl\n");
        machine.write("sys/class/hwmon/hwmon1/name", "tuxedo\n");
        machine.write("sys/class/hwmon/hwmon1/fan1_input", "2100\n");
        machine.write("sys/class/drm/card0/device/vendor", "0x1002\n");
        machine.write("sys/class/drm/card0/device/boot_vga", "1\n");
        machine.write("sys/class/drm/card0/device/gpu_busy_percent", "12\n");
        machine.battery("Discharging", 10_000_000);
        machine.write("sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq", "2000000\n");
        machine.write("sys/devices/system/cpu/cpu1/cpufreq/scaling_cur_freq", "4000000\n");
        machine
    }

    pub(crate) fn root(&self) -> &Path {
        self.dir.path()
    }

    pub(crate) fn write(&self, file: &str, text: &str) {
        let path = self.dir.path().join(file);
        fs::create_dir_all(path.parent().expect("a parent")).expect("make the folder");
        fs::write(path, text).expect("write the file");
    }

    /// `/proc/stat` with `total` ticks, `idle` of them idle, half on each core.
    pub(crate) fn stat(&self, total: u64, idle: u64) {
        let busy = total - idle;
        let core = |n: u64| format!("cpu{n} {} 0 0 {} 0 0 0 0 0 0\n", busy / 2, idle / 2);
        self.write(
            "proc/stat",
            &format!(
                "cpu  {busy} 0 0 {idle} 0 0 0 0 0 0\n{}{}btime 1700000000\n",
                core(0),
                core(1)
            ),
        );
    }

    pub(crate) fn net(&self, received: u64, sent: u64) {
        self.write(
            "proc/net/dev",
            &format!("Inter-| header\n face | header\n  wlan0: {received} 0 0 0 0 0 0 0 {sent} 0 0 0 0 0 0 0\n"),
        );
    }

    pub(crate) fn diskstats(&self, read_sectors: u64, written_sectors: u64) {
        self.write(
            "proc/diskstats",
            &format!(" 259 0 nvme0n1 1 0 {read_sectors} 0 1 0 {written_sectors} 0 0 0 0\n"),
        );
    }

    pub(crate) fn meminfo_percent_used(&self, percent: u64) {
        let total = 16_777_216;
        let available = total * (100 - percent) / 100;
        self.write(
            "proc/meminfo",
            &format!("MemTotal: {total} kB\nMemAvailable: {available} kB\n"),
        );
    }

    pub(crate) fn cpu_temp(&self, celsius: u32) {
        self.write("sys/class/hwmon/hwmon0/temp1_input", &format!("{}\n", celsius * 1000));
    }

    /// The battery, `status`, drawing `microwatts`.
    pub(crate) fn battery(&self, status: &str, microwatts: u64) {
        self.write("sys/class/power_supply/BAT0/type", "Battery\n");
        self.write("sys/class/power_supply/BAT0/status", &format!("{status}\n"));
        self.write("sys/class/power_supply/BAT0/capacity", "80\n");
        self.write("sys/class/power_supply/BAT0/energy_now", "48000000\n");
        self.write("sys/class/power_supply/BAT0/energy_full", "60000000\n");
        self.write("sys/class/power_supply/BAT0/energy_full_design", "64000000\n");
        self.write("sys/class/power_supply/BAT0/power_now", &format!("{microwatts}\n"));
    }

    /// A process `pid` called `name` with `ticks` of CPU and `pages` resident.
    pub(crate) fn process(&self, pid: i32, name: &str, ticks: u64, pages: u64) {
        self.write(
            &format!("proc/{pid}/stat"),
            &format!("{pid} ({name}) S 1 {pid} {pid} 0 -1 0 0 0 0 0 {ticks} 0 0 0 20 0 1 0 100 0 {pages} 0\n"),
        );
    }
}
