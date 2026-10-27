use std::fs;
use std::process::Command;

use super::*;
use crate::fake::FakeRoot;
use crate::machine::LAVD_ENV;
use crate::settings::Settings;
use crate::stop::Stop;

/// The HX 370's order for the fewest cores: the Zen 5c cores, each with its second thread, then
/// the Zen 5 ones.
const HX370_FEWEST: &str = "4,16,5,17,6,18,7,19,8,20,9,21,10,22,11,23,0,12,1,13,2,14,3,15";

fn lavd_on() -> Settings {
    Settings::parse("scheduler = \"lavd\"\n").expect("parses")
}

fn plan(stop: Stop, settings: &Settings) -> Plan {
    Plan::for_stop(stop, settings)
}

fn args(writes: &[Write]) -> Vec<String> {
    writes.iter().map(Write::arg).collect()
}

/// What the root helper left for the unit, as it would after `scheduler=asked`.
fn asked_before(root: &FakeRoot, asked: &str) {
    root.write(
        LAVD_ENV,
        &format!("KUBAOS_SCHEDULER={asked}\nLAVD_ARGS={}", command_line(asked)),
    );
}

#[test]
fn the_fewest_cores_fill_the_zen_5c_cores_first_each_cores_threads_together() {
    let root = FakeRoot::laptop();

    let order = fewest_cores(&root.machine().cpus()).expect("an order");

    let order: Vec<String> = order.iter().map(u32::to_string).collect();
    assert_eq!(order.join(","), HX370_FEWEST);
}

#[test]
fn cores_all_alike_or_saying_nothing_have_no_order_of_their_own() {
    let alike: Vec<Cpu> = (0..4)
        .map(|id| Cpu {
            id,
            highest_perf: Some(166),
            siblings: vec![id],
        })
        .collect();
    let silent: Vec<Cpu> = alike
        .iter()
        .map(|cpu| Cpu {
            highest_perf: None,
            ..cpu.clone()
        })
        .collect();

    assert_eq!(fewest_cores(&alike), None);
    assert_eq!(fewest_cores(&silent), None);
}

#[test]
fn an_offline_cpu_is_left_out_of_the_order() {
    let root = FakeRoot::laptop();
    root.write("sys/devices/system/cpu/cpu16/online", "0");

    let order = fewest_cores(&root.machine().cpus()).expect("an order");

    assert_eq!(&order[..3], &[4, 5, 17]);
    assert!(!order.contains(&16));
}

#[test]
fn each_mode_asks_its_flag_and_super_saver_its_order_too() {
    let root = FakeRoot::laptop();
    let machine = root.machine();

    assert_eq!(
        asked(LavdMode::PowersaveFewestCores, &machine),
        format!("powersave:{HX370_FEWEST}")
    );
    assert_eq!(asked(LavdMode::Powersave, &machine), "powersave");
    assert_eq!(asked(LavdMode::Autopilot, &machine), "autopilot");
    assert_eq!(asked(LavdMode::Performance, &machine), "performance");
}

#[test]
fn an_asked_value_comes_to_lavds_command_line() {
    assert_eq!(command_line("powersave:4,16"), "--powersave --cpu-pref-order 4,16");
    assert_eq!(command_line("autopilot"), "--autopilot");
}

#[test]
fn with_lavd_off_and_never_on_nothing_is_asked() {
    let root = FakeRoot::laptop();

    let writes = writes(&plan(Stop::Balanced, &Settings::default()), &root.machine(), false);

    assert_eq!(writes, Vec::<Write>::new());
}

#[test]
fn turning_lavd_off_stops_it_while_its_mode_is_still_there() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    asked_before(&root, "autopilot");

    let writes = writes(&plan(Stop::Balanced, &Settings::default()), &root.machine(), false);

    assert_eq!(args(&writes), ["scheduler=off"]);
}

#[test]
fn lavd_on_starts_it_in_the_stops_mode() {
    let root = FakeRoot::laptop();
    root.install_lavd();

    let writes = writes(&plan(Stop::Balanced, &lavd_on()), &root.machine(), false);

    assert_eq!(args(&writes), ["scheduler=autopilot"]);
}

#[test]
fn a_running_lavd_in_the_stops_mode_is_left_alone() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    asked_before(&root, "performance");
    root.sched_ext_running("lavd_1.1.2_x86_64_unknown_linux_gnu");

    let writes = writes(&plan(Stop::MaxPerformance, &lavd_on()), &root.machine(), false);

    assert_eq!(writes, Vec::<Write>::new());
}

#[test]
fn a_new_stop_restarts_lavd_in_its_mode() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    asked_before(&root, "autopilot");
    root.sched_ext_running("lavd_1.1.2");

    let writes = writes(&plan(Stop::SuperSaver, &lavd_on()), &root.machine(), false);

    assert_eq!(args(&writes), [format!("scheduler=powersave:{HX370_FEWEST}")]);
}

#[test]
fn turning_lavd_on_again_resets_it_before_it_asks() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    asked_before(&root, "powersave");

    let writes = writes(&plan(Stop::PowerSaver, &lavd_on()), &root.machine(), true);

    assert_eq!(args(&writes), ["scheduler=reset", "scheduler=powersave"]);
}

#[test]
fn without_sched_ext_or_scx_lavd_nothing_is_asked() {
    let without_lavd = FakeRoot::laptop();
    let without_sched_ext = FakeRoot::laptop();
    without_sched_ext.install_lavd();
    without_sched_ext.remove("sys/kernel/sched_ext");

    assert_eq!(
        writes(&plan(Stop::Balanced, &lavd_on()), &without_lavd.machine(), false),
        Vec::<Write>::new()
    );
    assert_eq!(
        writes(&plan(Stop::Balanced, &lavd_on()), &without_sched_ext.machine(), false),
        Vec::<Write>::new()
    );
}

#[test]
fn systemctl_shows_lines_read_into_the_units_state() {
    let state = UnitState::parse(
        "LoadState=loaded\nActiveState=activating\nSubState=auto-restart\nResult=exit-code\nNRestarts=1\n",
    );

    assert_eq!(
        state,
        UnitState {
            load: "loaded".to_owned(),
            active: "activating".to_owned(),
            sub: "auto-restart".to_owned(),
            result: "exit-code".to_owned(),
            restarts: 1,
        }
    );
    assert_eq!(state.words(), "activating (auto-restart), 1 restart, last exit-code");
}

fn unit(active: &str, sub: &str, result: &str) -> UnitState {
    UnitState {
        load: "loaded".to_owned(),
        active: active.to_owned(),
        sub: sub.to_owned(),
        result: result.to_owned(),
        restarts: 0,
    }
}

#[test]
fn a_crash_into_a_restart_is_a_fallback_once() {
    let running = unit("active", "running", "success");
    let restarting = unit("activating", "auto-restart", "exit-code");

    assert_eq!(
        fallback(Some(&running), &restarting),
        Some(Fallback::Restarting {
            result: "exit-code".to_owned()
        })
    );
    assert_eq!(fallback(Some(&restarting), &restarting), None);
    assert_eq!(fallback(Some(&restarting), &running), None);
}

#[test]
fn giving_up_is_a_fallback_and_so_is_finding_it_given_up() {
    let restarting = unit("activating", "auto-restart", "exit-code");
    let failed = unit("failed", "failed", "start-limit-hit");

    assert_eq!(fallback(Some(&restarting), &failed), Some(Fallback::GaveUp));
    assert_eq!(fallback(None, &failed), Some(Fallback::GaveUp));
    assert_eq!(fallback(Some(&failed), &failed), None);
}

#[test]
fn a_stop_asked_for_is_no_fallback() {
    let running = unit("active", "running", "success");
    let stopping = unit("deactivating", "stop-sigterm", "success");
    let stopped = unit("inactive", "dead", "success");

    assert_eq!(fallback(Some(&running), &stopping), None);
    assert_eq!(fallback(Some(&stopping), &stopped), None);
    assert_eq!(fallback(None, &stopped), None);
}

#[test]
fn the_last_error_is_lavds_exit_and_a_clean_stop_isnt_one() {
    let log = "scx_lavd scheduler starts running.\n\
               Error: EXIT: runtime error (scx_lavd[1234] failed to dispatch)\n\
               scx_lavd scheduler starts running.\n\
               EXIT: unregistered from user space\n";

    assert_eq!(
        last_exit(log).as_deref(),
        Some("EXIT: runtime error (scx_lavd[1234] failed to dispatch)")
    );
    assert_eq!(last_exit("EXIT: unregistered from user space\n"), None);
}

#[test]
fn lavd_off_reports_the_kernels_own_scheduler_and_agrees() {
    let root = FakeRoot::laptop();

    let report = report(&plan(Stop::Balanced, &Settings::default()), &root.machine(), None, None);

    assert_eq!(report.running, "Linux's own (EEVDF)");
    assert!(report.agrees);
    assert!(report.why.contains("LAVD is off"));
}

#[test]
fn lavd_in_the_stops_mode_says_so() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    asked_before(&root, "autopilot");
    root.sched_ext_running("lavd_1.1.2");
    let running = unit("active", "running", "success");

    let report = report(&plan(Stop::Balanced, &lavd_on()), &root.machine(), Some(&running), None);

    assert_eq!(report.running, "LAVD (--autopilot)");
    assert!(report.agrees);
    assert_eq!(report.why, "LAVD is on, in Balanced's mode: autopilot.");
}

#[test]
fn a_lavd_given_up_on_says_why_and_how_to_try_again() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    asked_before(&root, "autopilot");
    let failed = unit("failed", "failed", "start-limit-hit");

    let report = report(
        &plan(Stop::Balanced, &lavd_on()),
        &root.machine(),
        Some(&failed),
        Some("EXIT: runtime error (stall)"),
    );

    assert_eq!(report.running, "Linux's own (EEVDF)");
    assert!(!report.agrees);
    assert!(report.why.contains("(EXIT: runtime error (stall))"), "{}", report.why);
    assert!(report.why.contains("\"CPU scheduler: LAVD on\" tries it again"));
}

#[test]
fn a_machine_without_sched_ext_or_scx_lavd_says_which() {
    let without_lavd = FakeRoot::laptop();
    let without_sched_ext = FakeRoot::laptop();
    without_sched_ext.remove("sys/kernel/sched_ext");

    let missing = report(&plan(Stop::Balanced, &lavd_on()), &without_lavd.machine(), None, None);
    let kernel = report(
        &plan(Stop::Balanced, &lavd_on()),
        &without_sched_ext.machine(),
        None,
        None,
    );

    assert!(missing.why.contains("`kubaos update` installs it"));
    assert!(kernel.why.contains("no sched_ext"));
}

#[test]
fn the_status_row_shows_why_when_lavd_isnt_what_runs() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    let settings = lavd_on();
    let restarting = unit("activating", "auto-restart", "exit-code");
    let plan = plan(Stop::SuperSaver, &settings);

    let row = row(&plan, &report(&plan, &root.machine(), Some(&restarting), None));

    assert_eq!(row.wanted, "LAVD, powersave on the fewest cores");
    assert_eq!(row.agrees, Some(false));
    assert!(
        row.now
            .as_deref()
            .is_some_and(|now| now.starts_with("Linux's own (EEVDF): LAVD crashed"))
    );
}

// ----- The root helper's `scheduler=` ---------------------------------------------------------

/// A `systemctl` of the tests' own on the helper's PATH: it keeps the unit's state as files
/// under the fake root (`active`, `enabled`, `failed`, and `auto-restart` for the wait after a
/// crash, its `SubState`) and logs every call.
const FAKE_SYSTEMCTL: &str = r#"#!/usr/bin/env bash
state="$KUBAOS_POWER_ROOT/fake-systemd"
mkdir -p "$state"
echo "$*" >>"$state/log"
case "$1" in
  is-active | is-enabled | is-failed) [[ -e $state/${1#is-} ]] ;;
  enable) touch "$state/enabled" ;;
  disable) rm -f "$state/enabled"; [[ " $* " == *" --now "* ]] && rm -f "$state/active"; true ;;
  show) if [[ -e $state/auto-restart ]]; then echo auto-restart; else echo dead; fi ;;
  start | restart) rm -f "$state/auto-restart"; touch "$state/active" ;;
  reset-failed) rm -f "$state/failed" ;;
  *) exit 1 ;;
esac
"#;

/// Runs the root helper on `root` with the fake `systemctl`: whether it succeeded, and its stdout.
fn helper(root: &FakeRoot, args: &[&str]) -> (bool, String) {
    let bin = root.path("fake-bin");
    fs::create_dir_all(&bin).expect("mkdir");
    let systemctl = bin.join("systemctl");
    fs::write(&systemctl, FAKE_SYSTEMCTL).expect("write the fake systemctl");
    let mut permissions = fs::metadata(&systemctl).expect("stat").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&systemctl, permissions).expect("chmod");
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap_or_default());
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../system/files/power/kubaos-power-write"
    );
    let output = Command::new("bash")
        .arg(script)
        .args(args)
        .env("KUBAOS_POWER_ROOT", root.dir.path())
        .env("PATH", path)
        .output()
        .expect("run the helper");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

/// The `systemctl` calls that changed something, the questions (`is-…`, `show`) left out.
fn changes(root: &FakeRoot) -> Vec<String> {
    fs::read_to_string(root.path("fake-systemd/log"))
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.starts_with("is-") && !line.starts_with("show "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_helper_writes_lavds_command_line_and_starts_it_enabled_for_the_next_boot() {
    let root = FakeRoot::laptop();
    root.install_lavd();

    let (ok, _) = helper(&root, &["scheduler=powersave:4,16,5,17"]);

    assert!(ok);
    let env = root.read(LAVD_ENV);
    assert!(env.contains("\nKUBAOS_SCHEDULER=powersave:4,16,5,17\n"), "{env}");
    assert!(
        env.ends_with("\nLAVD_ARGS=--powersave --cpu-pref-order 4,16,5,17"),
        "{env}"
    );
    assert_eq!(
        changes(&root),
        [
            "enable --quiet kubaos-scx.service",
            "reset-failed kubaos-scx.service",
            "start kubaos-scx.service"
        ]
    );
    assert_eq!(root.machine().lavd_asked().as_deref(), Some("powersave:4,16,5,17"));
}

#[test]
fn the_helper_restarts_lavd_for_a_new_mode_and_leaves_the_same_one_alone() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    assert!(helper(&root, &["scheduler=autopilot"]).0);
    fs::remove_file(root.path("fake-systemd/log")).expect("clear the log");

    assert!(helper(&root, &["scheduler=autopilot"]).0);
    assert_eq!(changes(&root), Vec::<String>::new());
    assert!(helper(&root, &["scheduler=performance"]).0);

    assert_eq!(
        changes(&root),
        ["reset-failed kubaos-scx.service", "restart kubaos-scx.service"]
    );
}

#[test]
fn the_helper_leaves_a_lavd_waiting_to_restart_alone_unless_its_mode_changed() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    assert!(helper(&root, &["scheduler=autopilot"]).0);
    fs::remove_file(root.path("fake-systemd/active")).expect("crash");
    root.write("fake-systemd/auto-restart", "");
    fs::remove_file(root.path("fake-systemd/log")).expect("clear the log");

    assert!(helper(&root, &["scheduler=autopilot"]).0);
    assert_eq!(changes(&root), Vec::<String>::new());
    assert!(helper(&root, &["scheduler=performance"]).0);

    assert_eq!(
        changes(&root),
        ["reset-failed kubaos-scx.service", "start kubaos-scx.service"]
    );
}

#[test]
fn the_helper_never_starts_a_lavd_systemd_gave_up_on_until_it_is_reset() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    root.write("fake-systemd/failed", "");
    root.write("fake-systemd/enabled", "");

    let (ok, stdout) = helper(&root, &["scheduler=performance"]);

    assert!(!ok);
    assert_eq!(stdout.trim(), "failed scheduler=performance");
    assert_eq!(changes(&root), Vec::<String>::new());
    assert_eq!(root.machine().lavd_asked().as_deref(), Some("performance"));

    assert!(helper(&root, &["scheduler=reset", "scheduler=performance"]).0);
    assert!(changes(&root).contains(&"start kubaos-scx.service".to_owned()));
}

#[test]
fn the_helper_turns_lavd_off_stopped_disabled_and_its_mode_gone() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    assert!(helper(&root, &["scheduler=autopilot"]).0);
    fs::remove_file(root.path("fake-systemd/log")).expect("clear the log");

    assert!(helper(&root, &["scheduler=off"]).0);

    assert!(!root.path(LAVD_ENV).exists());
    assert_eq!(changes(&root), ["disable --now --quiet kubaos-scx.service"]);
    assert_eq!(root.machine().lavd_asked(), None);
}

#[test]
fn the_helper_refuses_what_isnt_a_mode_and_starts_nothing() {
    let root = FakeRoot::laptop();
    root.install_lavd();

    for asked in [
        "scheduler=turbo",
        "scheduler=performance:0,12",
        "scheduler=powersave:",
        "scheduler=powersave:4,99",
        "scheduler=powersave:4,16,4",
        "scheduler=powersave:4;reboot",
        "scheduler=--powersave",
    ] {
        let (ok, stdout) = helper(&root, &[asked]);

        assert!(!ok, "{asked}");
        assert_eq!(stdout.trim(), format!("failed {asked}"));
    }
    assert_eq!(changes(&root), Vec::<String>::new());
    assert!(!root.path(LAVD_ENV).exists());
}

#[test]
fn the_helper_says_when_the_kernel_or_scx_lavd_is_missing() {
    let without_lavd = FakeRoot::laptop();
    let without_sched_ext = FakeRoot::laptop();
    without_sched_ext.install_lavd();
    without_sched_ext.remove("sys/kernel/sched_ext");

    assert!(!helper(&without_lavd, &["scheduler=autopilot"]).0);
    assert!(!helper(&without_sched_ext, &["scheduler=autopilot"]).0);
    assert_eq!(changes(&without_lavd), Vec::<String>::new());
    assert_eq!(changes(&without_sched_ext), Vec::<String>::new());
}
