use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use super::Host;
use super::HostError;
use super::JobId;
use super::JobState;
use super::Queue;
use super::QueueError;
use super::Target;
use crate::held::CurrentHome;
use crate::held::Evaluated;
use crate::held::Found;
use crate::held::HeldFile;
use crate::held::Hold;
use crate::held::Holds;
use crate::preview::Checkout;
use crate::preview::PreviewFile;

/// A host that records what the queue asks of it instead of running systemd and home-manager.
#[derive(Default)]
struct FakeHost {
    started: Mutex<Vec<JobId>>,
    in_flight: AtomicUsize,
    most_at_once: AtomicUsize,
    switches: AtomicUsize,
    /// The switches that fail, counted from 1.
    failing: Vec<usize>,
    refuse_to_start: bool,
    /// Every worker unit is gone, as when its program failed to start.
    units_dead: bool,
    /// The units asked about.
    checked: Mutex<Vec<String>>,
    switch_time: Duration,
    /// The checkout each switch built KubaOS from, `None` for the release.
    switched_from: Mutex<Vec<Option<PathBuf>>>,
    /// The commit every checkout is at.
    commit: Mutex<String>,
    /// Paths git can't describe.
    not_checkouts: Vec<PathBuf>,
    /// The listed packages each evaluation finds, or `None` when they don't evaluate.
    listed: Option<Vec<Evaluated>>,
    /// The checkout each evaluation of the listed packages took KubaOS from, `None` for the
    /// release.
    listed_from: Mutex<Vec<Option<PathBuf>>>,
    /// The outputs that never build.
    unbuildable: Vec<String>,
    /// The installables of each build.
    built: Mutex<Vec<Vec<String>>>,
    /// The listed packages the home has.
    current: CurrentHome,
}

impl Host for FakeHost {
    fn worker_unit(&self, job: &JobId) -> String {
        format!("fake-{job}")
    }

    fn start_worker(&self, job: &JobId) -> Result<(), HostError> {
        if self.refuse_to_start {
            return Err(HostError::exited("systemd-run", Some(1)));
        }
        self.started.lock().unwrap().push(job.clone());
        Ok(())
    }

    fn worker_alive(&self, unit: &str) -> Result<bool, HostError> {
        self.checked.lock().unwrap().push(unit.to_owned());
        Ok(!self.units_dead)
    }

    fn switch(&self, mut log: File, from: Option<&Path>) -> Result<(), HostError> {
        self.switched_from.lock().unwrap().push(from.map(Path::to_path_buf));
        let n = self.switches.fetch_add(1, Ordering::SeqCst) + 1;
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.most_at_once.fetch_max(now, Ordering::SeqCst);
        writeln!(log, "building switch {n}").unwrap();
        thread::sleep(self.switch_time);
        writeln!(log, "activating").unwrap();
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        if self.failing.contains(&n) {
            return Err(HostError::exited("home-manager", Some(1)));
        }
        Ok(())
    }

    fn describe(&self, path: &Path) -> Result<Checkout, HostError> {
        if self.not_checkouts.iter().any(|p| p == path) {
            return Err(HostError::exited("git", Some(128)));
        }
        let commit = self.commit.lock().unwrap().clone();
        Ok(Checkout {
            root: path.to_path_buf(),
            branch: Some(format!("feature/{}", path.file_name().unwrap().to_string_lossy())),
            commit,
            dirty: false,
        })
    }

    fn evaluate(&self, _path: &Path) -> Result<(), HostError> {
        Ok(())
    }

    fn listed(&self, _log: &File, from: Option<&Path>) -> Result<Vec<Evaluated>, HostError> {
        self.listed_from.lock().unwrap().push(from.map(Path::to_path_buf));
        self.listed.clone().ok_or_else(|| HostError::exited("nix", Some(1)))
    }

    fn build(
        &self,
        mut log: &File,
        installables: &[String],
        outputs: &[String],
    ) -> Result<BTreeSet<String>, HostError> {
        self.built.lock().unwrap().push(installables.to_vec());
        writeln!(log, "building the listed packages").unwrap();
        Ok(outputs
            .iter()
            .filter(|output| self.unbuildable.contains(output))
            .cloned()
            .collect())
    }

    fn current_home(&self) -> Result<CurrentHome, HostError> {
        Ok(self.current.clone())
    }
}

#[test]
fn a_submitted_job_is_queued_and_its_worker_started() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = FakeHost::default();

    let id = queue.submit(&host).unwrap();

    assert_eq!(queue.job(&id).unwrap().state, JobState::Queued);
    assert_eq!(*host.started.lock().unwrap(), [id]);
}

#[test]
fn a_worker_that_cannot_start_fails_its_job() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = FakeHost {
        refuse_to_start: true,
        ..FakeHost::default()
    };

    let err = queue.submit(&host).unwrap_err();

    let QueueError::Start { job, .. } = &err else {
        panic!("{err:?}")
    };
    let job = queue.job(job).unwrap();
    assert_eq!(job.state, JobState::Failed);
    assert!(job.error.is_some());
}

#[test]
fn the_worker_runs_every_queued_job_oldest_first() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let ids = [
        queue.enqueue().unwrap(),
        queue.enqueue().unwrap(),
        queue.enqueue().unwrap(),
    ];
    let host = FakeHost::default();

    let ran = queue.work(&host).unwrap();

    assert_eq!(ran, ids);
    for id in &ids {
        let job = queue.job(id).unwrap();
        assert_eq!(job.state, JobState::Succeeded);
        assert!(job.started_at.is_some() && job.finished_at.is_some());
    }
}

#[test]
fn a_worker_with_nothing_queued_does_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = FakeHost::default();

    assert!(queue.work(&host).unwrap().is_empty());
    assert_eq!(host.switches.load(Ordering::SeqCst), 0);
}

#[test]
fn two_workers_never_switch_at_the_same_time() {
    let tmp = tempfile::tempdir().unwrap();
    let host = Arc::new(FakeHost {
        switch_time: Duration::from_millis(60),
        ..FakeHost::default()
    });
    let start = Arc::new(Barrier::new(4));

    let workers: Vec<_> = (0..4)
        .map(|_| {
            let queue = Queue::new(tmp.path());
            let host = Arc::clone(&host);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                let id = queue.enqueue().unwrap();
                start.wait();
                queue.work(&*host).unwrap();
                id
            })
        })
        .collect();
    let ids: Vec<JobId> = workers.into_iter().map(|w| w.join().unwrap()).collect();

    assert_eq!(host.most_at_once.load(Ordering::SeqCst), 1);
    assert_eq!(host.switches.load(Ordering::SeqCst), 4, "each job switches once");
    let queue = Queue::new(tmp.path());
    let mut jobs: Vec<_> = ids.iter().map(|id| queue.job(id).unwrap()).collect();
    jobs.sort_by_key(|j| j.started_at);
    for pair in jobs.windows(2) {
        assert_eq!(pair[0].state, JobState::Succeeded);
        assert!(pair[0].finished_at <= pair[1].started_at, "{pair:?}");
    }
}

#[test]
fn a_failed_switch_fails_its_job_and_the_next_job_still_runs() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let first = queue.enqueue().unwrap();
    let second = queue.enqueue().unwrap();
    let host = FakeHost {
        failing: vec![1],
        ..FakeHost::default()
    };

    queue.work(&host).unwrap();

    let failed = queue.job(&first).unwrap();
    assert_eq!(failed.state, JobState::Failed);
    assert!(failed.error.unwrap().contains("home-manager"));
    assert_eq!(queue.job(&second).unwrap().state, JobState::Succeeded);
}

#[test]
fn a_job_left_running_by_a_dead_worker_is_marked_interrupted() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let id = queue.enqueue().unwrap();
    let mut job = queue.job(&id).unwrap();
    job.state = JobState::Running;
    queue.write_job(&job).unwrap();
    let host = FakeHost::default();

    let ran = queue.work(&host).unwrap();

    assert!(ran.is_empty());
    let job = queue.job(&id).unwrap();
    assert_eq!(job.state, JobState::Failed);
    assert!(job.error.unwrap().contains("interrupted"));
}

#[test]
fn the_switch_output_goes_to_the_jobs_log() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let id = queue.enqueue().unwrap();

    queue.work(&FakeHost::default()).unwrap();

    let log = std::fs::read_to_string(queue.log_path(&id)).unwrap();
    assert!(log.contains("building switch 1\nactivating\n"), "{log}");
}

#[test]
fn following_a_job_streams_its_log_until_it_ends() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let id = queue.enqueue().unwrap();
    let worker = {
        let queue = Queue::new(tmp.path());
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            queue
                .work(&FakeHost {
                    switch_time: Duration::from_millis(50),
                    ..FakeHost::default()
                })
                .unwrap();
        })
    };
    let mut seen = String::new();

    let job = queue
        .follow(&id, Duration::from_millis(10), |chunk| seen.push_str(chunk))
        .unwrap();
    worker.join().unwrap();

    assert_eq!(job.state, JobState::Succeeded);
    assert!(seen.contains("building switch 1\nactivating\n"), "{seen}");
}

#[test]
fn jobs_lists_every_job_oldest_first() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let a = queue.enqueue().unwrap();
    let b = queue.enqueue().unwrap();

    let jobs = queue.jobs().unwrap();

    let ids: Vec<_> = jobs.into_iter().map(|j| j.id).collect();
    assert_eq!(ids, [a, b]);
}

#[test]
fn a_job_id_is_checked_before_it_names_a_file() {
    assert!(JobId::parse("00000001761136000000000000-0000042-0001").is_ok());
    assert!(JobId::parse("../../etc/passwd").is_err());
    assert!(JobId::parse("").is_err());
}

#[test]
fn an_unknown_job_is_its_own_error() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());

    let err = queue.job(&JobId::parse("1-2-3").unwrap()).unwrap_err();

    assert!(matches!(err, QueueError::NoSuchJob(_)), "{err:?}");
}

#[test]
fn the_systemd_host_runs_the_worker_in_a_transient_user_unit() {
    let host = super::Systemd::new("/usr/bin/kubaos", "/home/kuba/.config/kubaos/home");
    let id = JobId::parse("7-8-9").unwrap();

    let argv = host.worker_command(&id, &[("PATH", "/bin".into())]);

    assert_eq!(
        argv,
        [
            "systemd-run",
            "--user",
            "--unit=kubaos-switch-7-8-9",
            "--description=KubaOS switch 7-8-9",
            "--collect",
            "--quiet",
            "--setenv=PATH=/bin",
            "/usr/bin/kubaos",
            "job",
            "work",
        ]
    );
    assert_eq!(
        host.switch_command(None),
        [
            "home-manager",
            "switch",
            "-b",
            "hm-backup",
            "--flake",
            "/home/kuba/.config/kubaos/home"
        ]
    );
}

fn dead_units() -> FakeHost {
    FakeHost {
        units_dead: true,
        ..FakeHost::default()
    }
}

#[test]
fn a_submitted_job_names_its_worker_unit() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());

    let id = queue.submit(&FakeHost::default()).unwrap();

    assert_eq!(queue.job(&id).unwrap().unit, Some(format!("fake-{id}")));
}

#[test]
fn a_queued_job_whose_worker_unit_died_is_failed() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = dead_units();
    let id = queue.submit(&host).unwrap();

    let reaped = queue.reap(&host).unwrap();

    assert_eq!(reaped, std::slice::from_ref(&id));
    let job = queue.job(&id).unwrap();
    assert_eq!(job.state, JobState::Failed);
    let error = job.error.unwrap();
    assert!(error.contains(&format!("fake-{id}")), "{error}");
    assert!(job.finished_at.is_some());
}

#[test]
fn a_queued_job_whose_worker_unit_is_alive_stays_queued() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = FakeHost::default();
    let id = queue.submit(&host).unwrap();

    let reaped = queue.reap(&host).unwrap();

    assert!(reaped.is_empty());
    assert_eq!(queue.job(&id).unwrap().state, JobState::Queued);
}

#[test]
fn a_queued_job_stays_queued_while_another_worker_holds_the_lock() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = dead_units();
    let id = queue.submit(&host).unwrap();
    let lock = queue.lock().unwrap();

    let reaped = queue.reap(&host).unwrap();
    drop(lock);

    assert!(reaped.is_empty(), "the worker holding the lock runs every queued job");
    assert_eq!(queue.job(&id).unwrap().state, JobState::Queued);
}

#[test]
fn a_job_run_in_the_calling_process_is_never_reaped() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = dead_units();
    let id = queue.enqueue().unwrap();

    queue.reap(&host).unwrap();

    assert_eq!(queue.job(&id).unwrap().state, JobState::Queued);
    assert!(host.checked.lock().unwrap().is_empty());
}

#[test]
fn following_a_job_whose_worker_unit_died_ends_with_it_failed() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    let host = dead_units();
    let id = queue.submit(&host).unwrap();

    let job = queue
        .follow_worker(&id, Duration::from_millis(10), &host, |_| {})
        .unwrap();

    assert_eq!(job.state, JobState::Failed);
}

#[test]
fn a_job_file_from_before_worker_units_were_recorded_still_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = Queue::new(tmp.path());
    std::fs::write(
        tmp.path().join("1-2-3.json"),
        r#"{"id":"1-2-3","state":"queued","queued_at":1,"started_at":null,"finished_at":null,"error":null}"#,
    )
    .unwrap();

    let job = queue.job(&JobId::parse("1-2-3").unwrap()).unwrap();

    assert_eq!(job.unit, None);
}

#[test]
fn the_systemd_host_asks_systemd_for_the_units_state() {
    let host = super::Systemd::new("/usr/bin/kubaos", "/f");

    assert_eq!(host.worker_unit(&JobId::parse("7-8-9").unwrap()), "kubaos-switch-7-8-9");
    assert_eq!(
        super::Systemd::unit_state_command("kubaos-switch-7-8-9"),
        [
            "systemctl",
            "--user",
            "show",
            "--property=ActiveState",
            "--value",
            "kubaos-switch-7-8-9"
        ]
    );
    for alive in ["active", "activating", "deactivating", "reloading"] {
        assert!(super::is_alive_state(alive), "{alive}");
    }
    for dead in ["inactive", "failed", ""] {
        assert!(!super::is_alive_state(dead), "{dead:?}");
    }
}

/// A queue in `dir` that records previews in `dir/preview.json`.
fn previewing(dir: &Path) -> (Queue, PreviewFile) {
    let file = PreviewFile::new(dir.join("preview.json"));
    (Queue::new(dir.join("jobs")).previews(file.clone()), file)
}

fn checkout(path: &str) -> Target {
    Target::Checkout { path: path.into() }
}

#[test]
fn a_job_to_a_checkout_overrides_kubaos_and_records_the_preview() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost::default();
    *host.commit.lock().unwrap() = "1a2b3c4d5e".to_owned();

    let id = queue.enqueue_to(checkout("/w/live-preview")).unwrap();
    queue.work(&host).unwrap();

    assert_eq!(queue.job(&id).unwrap().target, checkout("/w/live-preview"));
    assert_eq!(
        *host.switched_from.lock().unwrap(),
        [Some(PathBuf::from("/w/live-preview"))]
    );
    let preview = file.read().unwrap().expect("a preview recorded");
    assert_eq!(preview.path, Path::new("/w/live-preview"));
    assert_eq!(preview.branch.as_deref(), Some("feature/live-preview"));
    assert_eq!(preview.commit, "1a2b3c4d5e");
    assert!(preview.since > 0);
}

#[test]
fn a_job_to_the_release_switches_the_flake_as_locked_and_clears_the_preview() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost::default();
    queue.enqueue_to(checkout("/w/a")).unwrap();
    queue.work(&host).unwrap();

    queue.enqueue_to(Target::Release).unwrap();
    queue.work(&host).unwrap();

    assert_eq!(host.switched_from.lock().unwrap()[1], None);
    assert_eq!(file.read().unwrap(), None);
    assert_eq!(std::fs::read_to_string(file.path()).unwrap(), "");
}

#[test]
fn a_switch_of_the_home_as_it_stands_keeps_the_preview_and_its_start() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost::default();
    let folder = tmp.path().join("a");
    std::fs::create_dir(&folder).unwrap();
    *host.commit.lock().unwrap() = "aaaaaaa".to_owned();
    queue.enqueue_to(Target::Checkout { path: folder.clone() }).unwrap();
    queue.work(&host).unwrap();
    let first = file.read().unwrap().unwrap();
    *host.commit.lock().unwrap() = "bbbbbbb".to_owned();

    queue.enqueue().unwrap();
    queue.work(&host).unwrap();

    assert_eq!(host.switched_from.lock().unwrap()[1], Some(folder));
    let kept = file.read().unwrap().unwrap();
    assert_eq!(kept.commit, "bbbbbbb");
    assert_eq!(kept.since, first.since);
}

#[test]
fn a_switch_of_the_home_as_it_stands_whose_checkout_is_gone_says_how_to_go_back() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost::default();
    queue.enqueue_to(checkout("/w/gone")).unwrap();
    queue.work(&host).unwrap();

    let id = queue.enqueue().unwrap();
    queue.work(&host).unwrap();

    let job = queue.job(&id).unwrap();
    assert_eq!(job.state, JobState::Failed);
    assert!(job.error.unwrap().contains("kubaos switch --release"));
    assert_eq!(host.switches.load(Ordering::SeqCst), 1);
    assert!(file.read().unwrap().is_some());
}

#[test]
fn a_record_that_does_not_read_never_blocks_the_release_or_a_new_preview() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost::default();
    std::fs::write(file.path(), "{ not a record").unwrap();

    let stuck = queue.enqueue().unwrap();
    let release = queue.enqueue_to(Target::Release).unwrap();
    queue.work(&host).unwrap();

    let stuck = queue.job(&stuck).unwrap();
    assert_eq!(stuck.state, JobState::Failed);
    assert!(stuck.error.unwrap().contains("kubaos switch --release"));
    assert_eq!(queue.job(&release).unwrap().state, JobState::Succeeded);
    assert_eq!(std::fs::read_to_string(file.path()).unwrap(), "");

    std::fs::write(file.path(), "{ not a record").unwrap();
    let preview = queue.enqueue_to(checkout("/w/b")).unwrap();
    queue.work(&host).unwrap();

    assert_eq!(queue.job(&preview).unwrap().state, JobState::Succeeded);
    assert_eq!(file.read().unwrap().unwrap().path, Path::new("/w/b"));
}

#[test]
fn a_switch_of_the_home_as_it_stands_without_a_preview_is_the_release() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost::default();

    queue.enqueue().unwrap();
    queue.work(&host).unwrap();

    assert_eq!(*host.switched_from.lock().unwrap(), [None]);
    assert_eq!(file.read().unwrap(), None);
}

#[test]
fn a_second_checkout_replaces_the_first_preview() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost::default();
    queue.enqueue_to(checkout("/w/a")).unwrap();
    queue.work(&host).unwrap();

    queue.enqueue_to(checkout("/w/b")).unwrap();
    queue.work(&host).unwrap();

    assert_eq!(file.read().unwrap().unwrap().path, Path::new("/w/b"));
}

#[test]
fn a_failed_preview_switch_leaves_the_record_as_it_was() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost {
        failing: vec![2],
        ..FakeHost::default()
    };
    queue.enqueue_to(checkout("/w/a")).unwrap();
    queue.work(&host).unwrap();

    let id = queue.enqueue_to(checkout("/w/b")).unwrap();
    queue.work(&host).unwrap();

    assert_eq!(queue.job(&id).unwrap().state, JobState::Failed);
    assert_eq!(file.read().unwrap().unwrap().path, Path::new("/w/a"));
}

#[test]
fn a_checkout_that_cannot_be_read_fails_its_job_before_switching() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = previewing(tmp.path());
    let host = FakeHost {
        not_checkouts: vec![PathBuf::from("/w/gone")],
        ..FakeHost::default()
    };

    let id = queue.enqueue_to(checkout("/w/gone")).unwrap();
    queue.work(&host).unwrap();

    let job = queue.job(&id).unwrap();
    assert_eq!(job.state, JobState::Failed);
    assert!(job.error.unwrap().contains("/w/gone"));
    assert_eq!(host.switches.load(Ordering::SeqCst), 0);
    assert_eq!(file.read().unwrap(), None);
}

#[test]
fn a_job_file_without_a_target_reads_as_the_home_as_it_stands() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("1-2-3.json"),
        r#"{"id":"1-2-3","state":"queued","queued_at":1,"started_at":null,"finished_at":null,"error":null}"#,
    )
    .unwrap();
    let queue = Queue::new(tmp.path());

    let job = queue.job(&JobId::parse("1-2-3").unwrap()).unwrap();

    assert_eq!(job.target, Target::Current);
    assert!(job.held.is_empty());
}

#[test]
fn the_systemd_host_overrides_kubaos_without_writing_the_lock_file() {
    let host = super::Systemd::new("/usr/bin/kubaos", "/h/.config/kubaos/home");

    assert_eq!(
        host.switch_command(Some(Path::new("/h/Projects/kubaos"))),
        [
            "home-manager",
            "switch",
            "-b",
            "hm-backup",
            "--flake",
            "/h/.config/kubaos/home",
            "--override-input",
            "kubaos",
            "git+file:///h/Projects/kubaos",
            "--no-write-lock-file",
        ]
    );
    assert_eq!(
        host.evaluate_command(Path::new("/h/Projects/kubaos")),
        [
            "nix",
            "eval",
            "--json",
            "/h/.config/kubaos/home#homeConfigurations",
            "--override-input",
            "kubaos",
            "git+file:///h/Projects/kubaos",
            "--no-write-lock-file",
            "--apply",
            "configs: map (config: config.activationPackage.drvPath) (builtins.attrValues configs)",
        ]
    );
    assert_eq!(
        super::Systemd::worktrees_command(Path::new("/h/Projects/kubaos")),
        ["git", "-C", "/h/Projects/kubaos", "worktree", "list", "--porcelain"]
    );
}

const DISCORD_OLD: &str = "/nix/store/84k7g556fj4ib11413jna62l3hgfzyna-discord-1.0.159";

/// The derivation the fake evaluates `name` to, at `version`.
fn drv(name: &str, version: &str) -> String {
    format!("/nix/store/{name}-{version}.drv")
}

/// The output the fake builds of `name`.
fn output(name: &str) -> String {
    format!("/nix/store/{name}-out")
}

fn listed(name: &str, version: &str) -> Evaluated {
    Evaluated {
        name: name.to_owned(),
        found: Some(Found {
            drv: drv(name, version),
            outputs: BTreeMap::from([("out".to_owned(), output(name))]),
            pname: name.to_owned(),
        }),
    }
}

/// A queue in `dir/jobs` that holds packages in `dir/home/kubaos.d`, whose `packages.toml` lists
/// `packages`.
fn holding(dir: &Path, packages: &[&str]) -> (Queue, HeldFile) {
    let state = crate::state::StateDir::new(dir.join("home/kubaos.d"));
    std::fs::create_dir_all(state.root()).unwrap();
    let list: Vec<String> = packages.iter().map(|p| format!("\"{p}\"")).collect();
    std::fs::write(state.packages_file(), format!("packages = [{}]\n", list.join(", "))).unwrap();
    let file = HeldFile::new(&state, dir.join("kubaos.d.lock"));
    (Queue::new(dir.join("jobs")).holding(file.clone()), file)
}

/// The home as a generation records Discord in it.
fn discord_installed() -> CurrentHome {
    CurrentHome::Recorded(BTreeMap::from([("discord".to_owned(), DISCORD_OLD.to_owned())]))
}

/// Discord held since its derivation at `version` failed.
fn discord_held(version: &str) -> Holds {
    let hold = Hold {
        path: DISCORD_OLD.to_owned(),
        failed: Some(drv("discord", version)),
    };
    Holds::from([("discord".to_owned(), hold)])
}

/// A host on which Discord's build fails, as it does while its download is out of reach.
fn discord_unbuildable(version: &str) -> FakeHost {
    FakeHost {
        listed: Some(vec![listed("discord", version), listed("jq", "1.7")]),
        unbuildable: vec![output("discord")],
        current: discord_installed(),
        ..FakeHost::default()
    }
}

fn log_of(queue: &Queue, id: &JobId) -> String {
    std::fs::read_to_string(queue.log_path(id)).unwrap()
}

#[test]
fn a_listed_package_that_fails_to_build_keeps_the_version_the_home_has() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = holding(tmp.path(), &["discord", "jq"]);
    let id = queue.enqueue().unwrap();
    let host = discord_unbuildable("1");

    queue.work(&host).unwrap();

    let job = queue.job(&id).unwrap();
    assert_eq!(job.state, JobState::Succeeded);
    assert_eq!(job.held, ["discord"]);
    assert_eq!(file.read().unwrap(), discord_held("1"));
    let log = log_of(&queue, &id);
    assert!(
        log.contains(&format!(
            "kubaos: discord stays at its current version: it couldn't be built (kept at {DISCORD_OLD})"
        )),
        "{log}"
    );
    assert!(
        log.find("building the listed packages") < log.find("building switch 1"),
        "the pre-build comes before the switch: {log}"
    );
}

#[test]
fn a_held_package_whose_build_is_unchanged_is_not_built_again_and_stays_held() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = holding(tmp.path(), &["discord", "jq"]);
    file.write(&discord_held("1")).unwrap();
    let id = queue.enqueue().unwrap();
    let host = discord_unbuildable("1");

    queue.work(&host).unwrap();

    assert_eq!(*host.built.lock().unwrap(), [vec![format!("{}^out", drv("jq", "1.7"))]]);
    let job = queue.job(&id).unwrap();
    assert_eq!(job.held, ["discord"]);
    assert_eq!(file.read().unwrap(), discord_held("1"));
    let log = log_of(&queue, &id);
    assert!(log.contains("kubaos: not building discord again"), "{log}");
    assert!(
        log.contains("kubaos: discord stays at its current version: it couldn't be built"),
        "every switch it stays held names it: {log}"
    );
}

#[test]
fn a_held_package_whose_derivation_changed_is_built_again() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = holding(tmp.path(), &["discord", "jq"]);
    file.write(&discord_held("1")).unwrap();
    queue.enqueue().unwrap();
    let host = discord_unbuildable("2");

    queue.work(&host).unwrap();

    let built = host.built.lock().unwrap();
    assert!(built[0].contains(&format!("{}^out", drv("discord", "2"))), "{built:?}");
    assert_eq!(file.read().unwrap(), discord_held("2"));
}

#[test]
fn an_updates_switch_builds_every_held_package_again() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = holding(tmp.path(), &["discord", "jq"]);
    let queue = queue.retrying_held();
    file.write(&discord_held("1")).unwrap();
    let id = queue.enqueue_to(Target::Release).unwrap();
    let host = discord_unbuildable("1");

    queue.work(&host).unwrap();

    let built = host.built.lock().unwrap();
    assert!(built[0].contains(&format!("{}^out", drv("discord", "1"))), "{built:?}");
    assert_eq!(queue.job(&id).unwrap().retry_held, crate::held::Retry::All);
    assert_eq!(file.read().unwrap(), discord_held("1"));
}

#[test]
fn a_failed_package_the_home_does_not_have_is_not_held_and_the_switch_goes_on() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = holding(tmp.path(), &["discord"]);
    let id = queue.enqueue().unwrap();
    let host = FakeHost {
        current: CurrentHome::Nothing,
        failing: vec![1],
        ..discord_unbuildable("1")
    };

    queue.work(&host).unwrap();

    let job = queue.job(&id).unwrap();
    assert_eq!(job.state, JobState::Failed);
    assert!(job.held.is_empty());
    assert!(!file.state().held_file().exists());
    assert_eq!(host.switches.load(Ordering::SeqCst), 1);
    assert!(
        log_of(&queue, &id).contains("kubaos: discord couldn't be built, and your home has no version of it to keep")
    );
}

#[test]
fn a_held_package_that_builds_again_loses_its_hold_and_the_file_goes_with_it() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = holding(tmp.path(), &["discord"]);
    file.write(&discord_held("1")).unwrap();
    let id = queue.enqueue().unwrap();
    let host = FakeHost {
        unbuildable: Vec::new(),
        ..discord_unbuildable("2")
    };

    queue.work(&host).unwrap();

    assert!(queue.job(&id).unwrap().held.is_empty());
    assert!(!file.state().held_file().exists());
    assert!(log_of(&queue, &id).contains("kubaos: discord builds again: it is no longer held"));
}

#[test]
fn when_the_listed_packages_do_not_evaluate_the_holds_stay_and_the_switch_runs() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, file) = holding(tmp.path(), &["discord"]);
    file.write(&discord_held("1")).unwrap();
    let id = queue.enqueue().unwrap();
    let host = FakeHost::default();

    queue.work(&host).unwrap();

    let job = queue.job(&id).unwrap();
    assert_eq!(job.state, JobState::Succeeded);
    assert_eq!(job.held, ["discord"]);
    assert_eq!(file.read().unwrap(), discord_held("1"));
    assert_eq!(host.switches.load(Ordering::SeqCst), 1);
}

#[test]
fn a_home_that_lists_and_holds_nothing_builds_nothing_before_its_switch() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, _file) = holding(tmp.path(), &[]);
    queue.enqueue().unwrap();
    let host = FakeHost::default();

    queue.work(&host).unwrap();

    assert!(host.listed_from.lock().unwrap().is_empty());
    assert_eq!(host.switches.load(Ordering::SeqCst), 1);
}

#[test]
fn the_pre_build_builds_kubaos_from_the_checkout_the_switch_does() {
    let tmp = tempfile::tempdir().unwrap();
    let (queue, _file) = holding(tmp.path(), &["jq"]);
    let path = tmp.path().join("kubaos");
    std::fs::create_dir_all(&path).unwrap();
    queue.enqueue_to(Target::Checkout { path: path.clone() }).unwrap();
    queue.enqueue_to(Target::Release).unwrap();
    let host = FakeHost {
        listed: Some(vec![listed("jq", "1.7")]),
        ..FakeHost::default()
    };

    queue.work(&host).unwrap();

    assert_eq!(*host.listed_from.lock().unwrap(), [Some(path), None]);
}

#[test]
fn the_systemd_host_evaluates_builds_and_checks_the_listed_packages() {
    let host = super::Systemd::new("/usr/bin/kubaos", "/h/.config/kubaos/home");

    let listed = host.listed_command(Some(Path::new("/h/Projects/kubaos")));

    assert_eq!(
        listed[..8],
        [
            "nix",
            "eval",
            "--json",
            "/h/.config/kubaos/home#homeConfigurations",
            "--override-input",
            "kubaos",
            "git+file:///h/Projects/kubaos",
            "--no-write-lock-file",
        ]
    );
    assert_eq!(listed[8], "--apply");
    assert!(listed[9].to_string_lossy().contains("kubaos.listedPackages"));
    assert_eq!(host.listed_command(None).len(), 6);
    assert_eq!(
        super::Systemd::build_command(&["/nix/store/b-discord.drv^out".to_owned()]),
        [
            "nix",
            "build",
            "--no-link",
            "--keep-going",
            "/nix/store/b-discord.drv^out"
        ]
    );
    assert_eq!(
        super::Systemd::missing_command(&["/nix/store/g-discord".to_owned()]),
        [
            "nix-store",
            "--check-validity",
            "--print-invalid",
            "/nix/store/g-discord"
        ]
    );
    assert_eq!(
        super::Systemd::references_command(Path::new("/h/.local/state/home-manager/gcroots/current-home")),
        [
            "nix-store",
            "--query",
            "--references",
            "/h/.local/state/home-manager/gcroots/current-home/home-path"
        ]
    );
}

#[test]
fn the_systemd_host_reads_what_the_home_has_from_its_generations_record() {
    let tmp = tempfile::tempdir().unwrap();
    let generation = tmp.path().join("generation");
    let record = generation.join(crate::state::GENERATION_RECORD);
    std::fs::create_dir_all(record.parent().unwrap()).unwrap();
    std::fs::write(&record, format!(r#"{{"discord":"{DISCORD_OLD}"}}"#)).unwrap();
    let host = super::Systemd::new("/usr/bin/kubaos", "/h/.config/kubaos/home");

    let without = host.current_home().unwrap();
    let with = host.generation(&generation).current_home().unwrap();

    assert_eq!(without, CurrentHome::Nothing);
    assert_eq!(with, discord_installed());
}
