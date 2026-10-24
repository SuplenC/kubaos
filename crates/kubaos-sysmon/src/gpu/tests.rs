use std::fs;
use std::path::Path;

use super::Gpus;

fn card(root: &Path, name: &str, files: &[(&str, &str)]) {
    let device = root.join(format!("sys/class/drm/{name}/device"));
    for (file, text) in files {
        let path = device.join(file);
        fs::create_dir_all(path.parent().expect("a parent")).expect("make the device");
        fs::write(path, format!("{text}\n")).expect("write a device file");
    }
}

fn amd_igpu(root: &Path) {
    card(
        root,
        "card1",
        &[
            ("vendor", "0x1002"),
            ("boot_vga", "1"),
            ("gpu_busy_percent", "37"),
            ("mem_info_vram_used", "536870912"),
            ("mem_info_vram_total", "2147483648"),
            ("power/runtime_status", "active"),
            ("hwmon/hwmon5/temp1_input", "54000"),
            ("hwmon/hwmon5/freq1_input", "1800000000"),
        ],
    );
}

#[test]
fn an_amd_gpu_shows_its_usage_vram_temperature_and_clock() {
    let root = tempfile::tempdir().expect("a temp dir");
    amd_igpu(root.path());
    // A connector, not a card.
    fs::create_dir_all(root.path().join("sys/class/drm/card1-eDP-1")).expect("make the connector");

    let gpus = Gpus::discover(root.path()).read();

    assert_eq!(gpus.len(), 1);
    let gpu = &gpus[0];
    assert_eq!(gpu.vendor, "amd");
    assert!(!gpu.discrete);
    assert_eq!(gpu.usage, Some(37.0));
    assert_eq!(gpu.vram_used, Some(512 * 1024 * 1024));
    assert_eq!(gpu.vram_total, Some(2048 * 1024 * 1024));
    assert_eq!(gpu.temp, Some(54.0));
    assert_eq!(gpu.mhz, Some(1800));
}

#[test]
fn a_sleeping_discrete_nvidia_gpu_gets_a_row_and_is_not_woken() {
    let root = tempfile::tempdir().expect("a temp dir");
    amd_igpu(root.path());
    card(
        root.path(),
        "card0",
        &[
            ("vendor", "0x10de"),
            ("boot_vga", "0"),
            ("power/runtime_status", "suspended"),
            ("hwmon/hwmon7/temp1_input", "40000"),
        ],
    );

    let gpus = Gpus::discover(root.path()).read();

    let nvidia = gpus.iter().find(|gpu| gpu.vendor == "nvidia").expect("an NVIDIA row");
    assert!(nvidia.discrete);
    assert!(!nvidia.active);
    assert_eq!(nvidia.temp, None);
    assert_eq!(gpus[0].vendor, "amd");
}

#[test]
fn the_clock_falls_back_to_the_starred_dpm_level() {
    let root = tempfile::tempdir().expect("a temp dir");
    card(
        root.path(),
        "card0",
        &[
            ("vendor", "0x1002"),
            ("pp_dpm_sclk", "0: 600Mhz\n1: 1200Mhz *\n2: 2000Mhz"),
        ],
    );

    let gpus = Gpus::discover(root.path()).read();

    assert_eq!(gpus[0].mhz, Some(1200));
    assert_eq!(gpus[0].usage, None);
}

#[test]
fn no_drm_folder_is_no_gpu() {
    let root = tempfile::tempdir().expect("a temp dir");

    assert!(Gpus::discover(root.path()).read().is_empty());
}
