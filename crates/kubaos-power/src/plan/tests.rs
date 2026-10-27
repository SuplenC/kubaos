use super::*;
use crate::settings::PanelMode;

fn plan(stop: Stop) -> Plan {
    Plan::for_stop(stop, &Settings::default())
}

#[test]
fn every_stop_sits_on_its_profile() {
    for stop in Stop::ALL {
        assert_eq!(plan(stop).profile, stop.profile());
    }
}

#[test]
fn boost_is_off_in_both_saver_stops_and_on_above() {
    let boost: Vec<bool> = Stop::ALL.into_iter().map(|stop| plan(stop).boost).collect();

    assert_eq!(boost, [false, false, true, true, true]);
}

#[test]
fn only_super_saver_caps_the_clock() {
    let caps: Vec<Option<u32>> = Stop::ALL.into_iter().map(|stop| plan(stop).cpu_max_mhz).collect();

    assert_eq!(caps, [Some(1600), None, None, None, None]);
}

#[test]
fn the_gpu_runs_low_in_the_savers_and_high_only_at_max_performance() {
    let dpm: Vec<GpuDpm> = Stop::ALL.into_iter().map(|stop| plan(stop).gpu_dpm).collect();

    assert_eq!(
        dpm,
        [GpuDpm::Low, GpuDpm::Low, GpuDpm::Auto, GpuDpm::Auto, GpuDpm::High]
    );
}

#[test]
fn the_panel_drops_to_60_hz_in_super_saver_uses_vrr_up_to_balanced_and_runs_native_above() {
    let panels: Vec<Panel> = Stop::ALL.into_iter().map(|stop| plan(stop).panel).collect();

    assert_eq!(
        panels[0],
        Panel {
            refresh: Some(60),
            vrr: false
        }
    );
    assert_eq!(
        panels[1],
        Panel {
            refresh: None,
            vrr: true
        }
    );
    assert_eq!(
        panels[2],
        Panel {
            refresh: None,
            vrr: true
        }
    );
    assert_eq!(
        panels[3],
        Panel {
            refresh: None,
            vrr: false
        }
    );
    assert_eq!(
        panels[4],
        Panel {
            refresh: None,
            vrr: false
        }
    );
}

#[test]
fn panel_power_saving_is_on_in_the_savers_unless_the_user_turned_it_off() {
    let mut settings = Settings::default();
    let levels = |settings: &Settings| -> Vec<u8> {
        Stop::ALL
            .into_iter()
            .map(|stop| Plan::for_stop(stop, settings).panel_power_saving)
            .collect()
    };

    assert_eq!(levels(&settings), [3, 2, 0, 0, 0]);
    settings.panel_power_saving = false;
    assert_eq!(levels(&settings), [0, 0, 0, 0, 0]);
}

#[test]
fn links_and_radios_save_up_to_balanced_and_stay_awake_above() {
    let rows: Vec<(Aspm, bool, bool)> = Stop::ALL
        .into_iter()
        .map(|stop| {
            let plan = plan(stop);
            (plan.aspm, plan.usb_autosuspend, plan.wifi_power_save)
        })
        .collect();

    assert_eq!(
        rows,
        [
            (Aspm::PowerSuperSave, true, true),
            (Aspm::PowerSave, true, true),
            (Aspm::Default, true, true),
            (Aspm::Performance, true, false),
            (Aspm::Performance, false, false),
        ]
    );
}

#[test]
fn background_work_waits_only_in_the_saver_stops() {
    let quiet: Vec<bool> = Stop::ALL.into_iter().map(|stop| plan(stop).quiet_background).collect();

    assert_eq!(quiet, [true, true, false, false, false]);
}

#[test]
fn with_lavd_on_every_stop_runs_it_in_a_mode_of_its_own() {
    let settings = Settings::parse("scheduler = \"lavd\"\n").expect("parses");

    let modes: Vec<Scheduler> = Stop::ALL
        .into_iter()
        .map(|stop| Plan::for_stop(stop, &settings).scheduler)
        .collect();

    assert_eq!(
        modes,
        [
            Scheduler::Lavd(LavdMode::PowersaveFewestCores),
            Scheduler::Lavd(LavdMode::Powersave),
            Scheduler::Lavd(LavdMode::Autopilot),
            Scheduler::Lavd(LavdMode::Performance),
            Scheduler::Lavd(LavdMode::Performance)
        ]
    );
}

#[test]
fn by_default_every_stop_runs_the_kernels_own_scheduler() {
    assert!(
        Stop::ALL
            .into_iter()
            .all(|stop| plan(stop).scheduler == Scheduler::Kernel)
    );
}

#[test]
fn a_stop_takes_the_users_panel_and_clock_cap() {
    let settings = Settings::parse("[balanced]\npanel = \"native\"\n[super-saver]\ncpu_max_mhz = 0\n").expect("parses");

    assert_eq!(
        Plan::for_stop(Stop::Balanced, &settings).panel,
        Panel::of(PanelMode::Native)
    );
    assert_eq!(Plan::for_stop(Stop::SuperSaver, &settings).cpu_max_mhz, None);
}
