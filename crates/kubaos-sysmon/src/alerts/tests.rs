use super::Alerts;
use super::FULL_FOR_MS;
use super::HOT_FOR_MS;
use super::REPEAT_AFTER_MS;
use crate::disks::Disk;
use crate::memory::Memory;
use crate::power::Battery;
use crate::processes::Process;
use crate::sampler::Sample;
use crate::sampler::Top;
use crate::settings::Thresholds;

const GIB: u64 = 1024 * 1024 * 1024;

fn hot(temp: f32) -> Sample {
    let mut sample = Sample {
        top: Some(Top {
            cpu: vec![Process {
                pid: 4242,
                name: "cc1plus".to_owned(),
                cpu: 41.6,
                memory: 0,
            }],
            ..Top::default()
        }),
        ..Sample::default()
    };
    sample.cpu.temp = Some(temp);
    sample
}

#[test]
fn a_hot_cpu_fires_after_thirty_seconds_and_names_the_busiest_process() {
    let mut alerts = Alerts::default();
    let thresholds = Thresholds::default();

    let at_start = alerts.check(&hot(95.0), &thresholds, 0);
    let before = alerts.check(&hot(95.0), &thresholds, HOT_FOR_MS - 1);
    let after = alerts.check(&hot(95.4), &thresholds, HOT_FOR_MS);

    assert_eq!(at_start, [] as [super::Alert; 0]);
    assert_eq!(before, [] as [super::Alert; 0]);
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].title, "CPU at 95 °C for 30 s");
    assert_eq!(after[0].body, "cc1plus uses 42 % of the CPU");
    assert_eq!(after[0].action, "process:4242");
    assert_eq!(after[0].action_label, "Show cc1plus");
}

#[test]
fn cooling_down_in_between_starts_the_wait_again() {
    let mut alerts = Alerts::default();
    let thresholds = Thresholds::default();

    alerts.check(&hot(95.0), &thresholds, 0);
    alerts.check(&hot(60.0), &thresholds, 20_000);
    let fired = alerts.check(&hot(95.0), &thresholds, 35_000);

    assert_eq!(fired, [] as [super::Alert; 0]);
}

#[test]
fn an_alert_fires_once_while_its_condition_holds_and_again_only_after_a_rest() {
    let mut alerts = Alerts::default();
    let thresholds = Thresholds::default();
    alerts.check(&hot(95.0), &thresholds, 0);
    assert_eq!(alerts.check(&hot(95.0), &thresholds, HOT_FOR_MS).len(), 1);

    let still = alerts.check(&hot(95.0), &thresholds, HOT_FOR_MS + 60_000);
    alerts.check(&hot(50.0), &thresholds, HOT_FOR_MS + 61_000);
    alerts.check(&hot(95.0), &thresholds, HOT_FOR_MS + 62_000);
    let too_soon = alerts.check(&hot(95.0), &thresholds, HOT_FOR_MS * 2 + 62_000);
    let rested = alerts.check(&hot(95.0), &thresholds, HOT_FOR_MS + REPEAT_AFTER_MS);

    assert_eq!(still, [] as [super::Alert; 0]);
    assert_eq!(too_soon, [] as [super::Alert; 0]);
    assert_eq!(rested.len(), 1);
}

#[test]
fn memory_almost_full_names_the_name_holding_the_most() {
    let sample = Sample {
        memory: Some(Memory {
            total: 32 * GIB,
            used: 30 * GIB,
            percent: 93.75,
            ..Memory::default()
        }),
        top: Some(Top {
            memory: vec![
                Process {
                    pid: 7,
                    name: "Isolated Web Co".to_owned(),
                    cpu: 0.0,
                    memory: 3 * GIB,
                },
                Process {
                    pid: 9,
                    name: "chrome".to_owned(),
                    cpu: 0.0,
                    memory: 2 * GIB,
                },
            ],
            heaviest: Some(("chrome".to_owned(), 9 * GIB)),
            ..Top::default()
        }),
        ..Sample::default()
    };
    let mut alerts = Alerts::default();

    alerts.check(&sample, &Thresholds::default(), 0);
    let fired = alerts.check(&sample, &Thresholds::default(), FULL_FOR_MS);

    assert_eq!(fired[0].title, "Memory almost full, chrome uses 9.0 GB");
    assert_eq!(fired[0].body, "30 GB of 32 GB used");
    assert_eq!(fired[0].action, "process:9");
}

#[test]
fn a_full_disk_fires_at_once_and_opens_in_files() {
    let sample = Sample {
        disks: vec![Disk {
            point: "/".to_owned(),
            label: "/".to_owned(),
            removable: false,
            total: 512 * GIB,
            used: 490 * GIB,
            free: 22 * GIB,
            percent: 95.7,
        }],
        ..Sample::default()
    };
    let mut alerts = Alerts::default();

    let fired = alerts.check(&sample, &Thresholds::default(), 0);

    assert_eq!(fired[0].id, "disk:/");
    assert_eq!(fired[0].title, "Disk / is 96 % full");
    assert_eq!(fired[0].body, "22 GB free of 512 GB");
    assert_eq!(fired[0].action, "files:/");
}

#[test]
fn a_battery_draining_fast_offers_the_power_saver() {
    let mut sample = hot(50.0);
    sample.battery = Some(Battery {
        percent: 60.0,
        status: "Discharging".to_owned(),
        watts: Some(35.2),
        seconds_left: Some(4200),
        health: None,
    });
    let mut alerts = Alerts::default();

    alerts.check(&sample, &Thresholds::default(), 0);
    let fired = alerts.check(&sample, &Thresholds::default(), 30_000);

    assert_eq!(fired[0].title, "Battery draining fast (35 W)");
    assert_eq!(fired[0].body, "cc1plus uses 42 % of the CPU; 1 h 10 min left");
    assert_eq!(fired[0].action, "power-saver");
}

#[test]
fn charging_at_a_high_rate_is_no_alert() {
    let sample = Sample {
        battery: Some(Battery {
            status: "Charging".to_owned(),
            watts: Some(65.0),
            ..Battery::default()
        }),
        ..Sample::default()
    };
    let mut alerts = Alerts::default();

    alerts.check(&sample, &Thresholds::default(), 0);

    assert_eq!(alerts.check(&sample, &Thresholds::default(), 60_000), [] as [super::Alert; 0]);
}

#[test]
fn processes_are_wanted_while_a_condition_with_a_culprit_holds() {
    let thresholds = Thresholds::default();

    assert!(Alerts::wants_processes(&hot(95.0), &thresholds));
    assert!(!Alerts::wants_processes(&hot(60.0), &thresholds));
}
