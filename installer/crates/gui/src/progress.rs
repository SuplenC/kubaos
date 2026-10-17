//! Folds the event stream into what the Installing screen shows.

use std::time::{Duration, Instant};

use dotinstall_core::{Event, Level, StepId, StepInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub info: StepInfo,
    pub state: StepState,
    pub fraction: f32,
    pub started: Option<Instant>,
    pub elapsed: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub level: Level,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Done {
        elapsed: Duration,
    },
    Failed {
        step: Option<StepId>,
        error: String,
        hint: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Install {
    pub started: Instant,
    pub steps: Vec<Step>,
    pub detail: Option<String>,
    pub log: Vec<LogLine>,
    pub outcome: Option<Outcome>,
    /// Smoothed overall fraction actually drawn; eases towards `overall()`.
    pub shown: f32,
}

/// Lines kept for the live log view; the saved log keeps everything.
pub const LOG_VIEW_LINES: usize = 400;

impl Install {
    pub fn new(now: Instant) -> Self {
        Self {
            started: now,
            steps: Vec::new(),
            detail: None,
            log: Vec::new(),
            outcome: None,
            shown: 0.0,
        }
    }

    pub fn apply(&mut self, ev: &Event, now: Instant) {
        match ev {
            Event::Plan { steps } => {
                self.steps = steps
                    .iter()
                    .map(|info| Step {
                        info: info.clone(),
                        state: StepState::Pending,
                        fraction: 0.0,
                        started: None,
                        elapsed: None,
                    })
                    .collect();
            }
            Event::StepStarted { id } => {
                self.detail = None;
                if let Some(s) = self.step_mut(*id) {
                    s.state = StepState::Running;
                    s.started = Some(now);
                }
                self.push(Level::Info, format!("==> {}", id.title()));
            }
            Event::Progress {
                id,
                fraction,
                detail,
            } => {
                if let Some(s) = self.step_mut(*id)
                    && let Some(f) = fraction
                {
                    s.fraction = f.clamp(0.0, 1.0);
                }
                if detail.is_some() {
                    self.detail.clone_from(detail);
                }
            }
            Event::Log { level, line, .. } => self.push(*level, line.clone()),
            Event::StepFinished { id, elapsed_ms } => {
                if let Some(s) = self.step_mut(*id) {
                    s.state = StepState::Done;
                    s.fraction = 1.0;
                    s.elapsed = Some(Duration::from_millis(*elapsed_ms));
                }
                self.detail = None;
            }
            Event::Failed { id, error, hint } => {
                if let Some(s) = self.step_mut(*id) {
                    s.state = StepState::Failed;
                    s.elapsed = s.started.map(|t| now - t);
                }
                self.push(Level::Error, format!("!!! {error}"));
                if let Some(h) = hint {
                    self.push(Level::Info, format!("hint: {h}"));
                }
                self.outcome = Some(Outcome::Failed {
                    step: Some(*id),
                    error: error.clone(),
                    hint: hint.clone(),
                });
            }
            Event::Done { elapsed_ms } => {
                self.outcome = Some(Outcome::Done {
                    elapsed: Duration::from_millis(*elapsed_ms),
                });
            }
        }
    }

    /// The process ended. Fills in a failure when it died without saying why.
    pub fn exited(&mut self, result: Result<(), String>) {
        match (result, &self.outcome) {
            (Err(e), None | Some(Outcome::Done { .. })) => {
                self.push(Level::Error, format!("!!! {e}"));
                let step = self.current().map(|s| s.info.id);
                self.outcome = Some(Outcome::Failed {
                    step,
                    error: e,
                    hint: None,
                });
            }
            (Ok(()), None) => {
                let e = "The installer stopped without finishing.".to_string();
                self.push(Level::Error, format!("!!! {e}"));
                self.outcome = Some(Outcome::Failed {
                    step: self.current().map(|s| s.info.id),
                    error: e,
                    hint: None,
                });
            }
            _ => {}
        }
    }

    /// A line the engine wrote to stderr. In `--events json` mode it
    /// mirrors every event there as human text ("==> Encrypting", "    80%",
    /// "    done in 4.0s"), which the events already show: keep only what
    /// isn't a mirror (crashes, refusals, panics).
    pub fn stderr(&mut self, line: String) {
        const MIRRORS: [&str; 7] = [
            "==> ",
            "    ",
            " !! ",
            " xx ",
            "!!! ",
            "plan: ",
            "installed in ",
        ];
        if line.trim().is_empty() || MIRRORS.iter().any(|m| line.starts_with(m)) {
            return;
        }
        self.push(Level::Debug, line);
    }

    pub fn push(&mut self, level: Level, text: String) {
        self.log.push(LogLine { level, text });
    }

    fn step_mut(&mut self, id: StepId) -> Option<&mut Step> {
        self.steps.iter_mut().find(|s| s.info.id == id)
    }

    pub fn current(&self) -> Option<&Step> {
        self.steps.iter().find(|s| s.state == StepState::Running)
    }

    /// Weighted by `StepInfo.weight`.
    pub fn overall(&self) -> f32 {
        self.overall_at(None)
    }

    /// `overall`, plus a creep through steps that report no fraction
    /// (argon2id takes 10 s and says nothing): their share fills towards
    /// 85% as they run, never to the end, so the bar moves without lying
    /// about a step being done.
    pub fn overall_at(&self, now: Option<Instant>) -> f32 {
        let total: u32 = self.steps.iter().map(|s| s.info.weight).sum();
        if total == 0 {
            return 0.0;
        }
        let done: f32 = self
            .steps
            .iter()
            .map(|s| {
                let f = match (s.state, s.started, now) {
                    (StepState::Running, Some(t), Some(now)) if s.fraction == 0.0 => {
                        let secs = now.saturating_duration_since(t).as_secs_f32();
                        let tau = 0.6 * s.info.weight as f32;
                        0.85 * (1.0 - (-secs / tau.max(0.5)).exp())
                    }
                    _ => s.fraction,
                };
                s.info.weight as f32 * f
            })
            .sum();
        (done / total as f32).clamp(0.0, 1.0)
    }

    /// Eases `shown` towards `overall_at(now)`; true while it is still moving.
    pub fn tick(&mut self, dt: Duration, now: Instant) -> bool {
        // Never backwards: a step that starts reporting its own fraction
        // can land below where the creep had got to.
        let target = self.overall_at(Some(now)).max(self.shown);
        let k = 1.0 - (-dt.as_secs_f32() * 6.0).exp();
        self.shown += (target - self.shown) * k;
        if (target - self.shown).abs() < 0.0005 {
            self.shown = target;
        }
        self.shown != target
    }

    pub fn log_text(&self) -> String {
        let mut s = String::new();
        for l in &self.log {
            let tag = match l.level {
                Level::Debug => "debug",
                Level::Info => "info ",
                Level::Warn => "warn ",
                Level::Error => "error",
            };
            s.push_str(tag);
            s.push(' ');
            s.push_str(&l.text);
            s.push('\n');
        }
        s
    }
}

pub fn fmt_duration(d: Duration) -> String {
    let s = d.as_secs_f32();
    if s < 10.0 {
        format!("{s:.1}s")
    } else if s < 60.0 {
        format!("{s:.0}s")
    } else {
        format!("{}m {:02}s", s as u64 / 60, s as u64 % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stderr_mirrors_are_dropped() {
        let mut i = Install::new(Instant::now());
        for l in [
            "==> Encrypting",
            "    80%  8.0 / 10.0 GB",
            "    done in 4.0s",
            "installed in 57.4s",
            "",
        ] {
            i.stderr(l.into());
        }
        assert!(i.log.is_empty());
        i.stderr("thread 'main' panicked at src/main.rs:1".into());
        i.stderr("dotinstall: can't write the log /var/log/x: denied".into());
        assert_eq!(i.log.len(), 2);
    }

    /// A silent step creeps, never finishes on its own, and the bar never
    /// goes back when the next step reports a smaller fraction.
    #[test]
    fn silent_steps_creep_and_the_bar_never_goes_back() {
        let t0 = Instant::now();
        let mut i = Install::new(t0);
        i.apply(
            &Event::Plan {
                steps: vec![StepId::Encrypt.info(), StepId::Unpack.info()],
            },
            t0,
        );
        i.apply(
            &Event::StepStarted {
                id: StepId::Encrypt,
            },
            t0,
        );
        let share = 5.0 / 65.0;
        let at = |s| t0 + Duration::from_secs(s);
        assert!(i.overall_at(Some(at(3))) > 0.3 * share);
        assert!(i.overall_at(Some(at(600))) <= 0.85 * share + 1e-6);
        for _ in 0..200 {
            i.tick(Duration::from_millis(50), at(10));
        }
        let before = i.shown;
        assert!(before > 0.0);
        i.apply(
            &Event::StepFinished {
                id: StepId::Encrypt,
                elapsed_ms: 10_000,
            },
            at(10),
        );
        i.apply(&Event::StepStarted { id: StepId::Unpack }, at(10));
        i.tick(Duration::from_millis(50), at(10));
        assert!(i.shown >= before);
    }

    #[test]
    fn weighted_progress_and_failure() {
        let now = Instant::now();
        let mut i = Install::new(now);
        i.apply(
            &Event::Plan {
                steps: vec![StepId::Preflight.info(), StepId::Unpack.info()],
            },
            now,
        );
        i.apply(
            &Event::StepStarted {
                id: StepId::Preflight,
            },
            now,
        );
        i.apply(
            &Event::StepFinished {
                id: StepId::Preflight,
                elapsed_ms: 10,
            },
            now,
        );
        assert!((i.overall() - 2.0 / 62.0).abs() < 1e-4);
        i.apply(&Event::StepStarted { id: StepId::Unpack }, now);
        i.apply(
            &Event::Progress {
                id: StepId::Unpack,
                fraction: Some(0.5),
                detail: Some("x".into()),
            },
            now,
        );
        assert!((i.overall() - 32.0 / 62.0).abs() < 1e-4);
        assert_eq!(i.detail.as_deref(), Some("x"));
        i.exited(Err("boom".into()));
        assert!(matches!(
            i.outcome,
            Some(Outcome::Failed {
                step: Some(StepId::Unpack),
                ..
            })
        ));
    }
}
