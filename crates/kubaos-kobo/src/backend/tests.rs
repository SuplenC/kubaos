use std::path::PathBuf;

use kubaos_core::Event;
use kubaos_core::StepId;

use super::Backend;
use super::Engine;
use super::RunMsg;
use super::demo_probe;
use super::exit_reason;
use crate::model::DiskMode;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kobo-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create the scratch dir");
    dir
}

fn dry_run(log: PathBuf) -> Vec<RunMsg> {
    let engine = Engine { log, dry_run: true };
    let probe = demo_probe();
    let mut form = crate::model::Form::new(&probe);
    form.full_name = "J".into();
    form.password = "a password nobody logs".into();
    // A dry run reads no partition table, so a free-space region can't be checked against one.
    form.disk_mode = Some(DiskMode::Whole);
    let mut out = vec![];

    engine.run(&form.to_config(&probe), &form.to_secrets(), &mut |m| {
        out.push(m)
    });

    out
}

#[test]
fn a_dry_run_streams_the_engine_events_and_exits_once() {
    let dir = scratch("dry-run");

    let out = dry_run(dir.join("install.log"));

    assert!(matches!(
        out.first(),
        Some(RunMsg::Event(Event::Plan { .. }))
    ));
    assert!(out.iter().any(|m| matches!(
        m,
        RunMsg::Event(Event::StepStarted {
            id: StepId::Preflight
        })
    )));
    assert!(matches!(
        out[out.len() - 2],
        RunMsg::Event(Event::Done { .. })
    ));
    assert!(matches!(out.last(), Some(RunMsg::Exited(Ok(())))));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_install_log_has_every_step_and_no_secret() {
    let dir = scratch("log");
    let path = dir.join("install.log");

    dry_run(path.clone());

    let log = std::fs::read_to_string(&path).expect("read the install log");
    assert!(
        log.lines().any(|l| l.ends_with("==> Checking the machine")),
        "{log}"
    );
    assert!(
        log.lines()
            .last()
            .is_some_and(|l| l.contains("installed in ")),
        "{log}"
    );
    assert!(!log.contains("a password nobody logs"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_log_that_cant_be_written_is_a_warning_not_a_failure() {
    let dir = scratch("no-log");

    let out = dry_run(dir.join("missing").join("install.log"));

    assert!(out.iter().any(|m| matches!(m, RunMsg::Event(Event::Log { line, .. }) if line.starts_with("can't write the log"))));
    assert!(matches!(out.last(), Some(RunMsg::Exited(Ok(())))));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_failed_step_is_left_to_its_event_and_a_crash_gets_a_reason() {
    let step = kubaos_core::Error::Step {
        step: StepId::Unpack,
        message: "disk full".into(),
        hint: None,
    };

    let crash = exit_reason(Err(Box::new("index out of bounds")));

    assert_eq!(exit_reason(Ok(Err(step))), Ok(()));
    assert!(crash.is_err_and(|e| e.contains("crashed")));
}
