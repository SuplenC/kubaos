use std::fs::File;
use std::io::Write;
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
    switch_time: Duration,
}

impl Host for FakeHost {
    fn start_worker(&self, job: &JobId) -> Result<(), HostError> {
        if self.refuse_to_start {
            return Err(HostError::exited("systemd-run", Some(1)));
        }
        self.started.lock().unwrap().push(job.clone());
        Ok(())
    }

    fn switch(&self, mut log: File) -> Result<(), HostError> {
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
        host.switch_command(),
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
