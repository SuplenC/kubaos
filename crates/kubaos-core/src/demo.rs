//! A fake install: the real event sequence with made-up timings, so Kōbō
//! can be built and screenshotted anywhere (macOS included).

use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::{Event, Level, StepId};

#[derive(Debug, Clone)]
pub struct DemoOptions {
    /// 1.0 is roughly a 40 s install; 10.0 is 4 s.
    pub speed: f32,
    pub fail_at: Option<StepId>,
}

impl Default for DemoOptions {
    fn default() -> Self {
        Self {
            speed: 1.0,
            fail_at: None,
        }
    }
}

pub fn run(opts: &DemoOptions, emit: &mut dyn FnMut(Event)) -> bool {
    let started = Instant::now();
    let steps: Vec<StepId> = StepId::ALL.to_vec();
    emit(Event::Plan {
        steps: steps.iter().map(|s| s.info()).collect(),
    });

    for id in steps {
        let step_started = Instant::now();
        emit(Event::StepStarted { id });
        let ticks = 10;
        let total = Duration::from_millis(u64::from(id.weight()) * 600);
        for i in 1..=ticks {
            sleep(total.div_f32(ticks as f32 * opts.speed.max(0.01)));
            emit(Event::Progress {
                id,
                fraction: Some(i as f32 / ticks as f32),
                detail: (id == StepId::Unpack)
                    .then(|| format!("{:.1} / 9.4 GB", 9.4 * i as f32 / ticks as f32)),
            });
            if i % 3 == 0 {
                emit(Event::Log {
                    id,
                    level: Level::Info,
                    line: format!("{}: tick {i}", id.title()),
                });
            }
            if opts.fail_at == Some(id) && i == ticks / 2 {
                emit(Event::Failed {
                    id,
                    error: "demo failure".into(),
                    hint: Some("This is --fail-at in the demo.".into()),
                });
                return false;
            }
        }
        emit(Event::StepFinished {
            id,
            elapsed_ms: step_started.elapsed().as_millis() as u64,
        });
    }
    emit(Event::Done {
        elapsed_ms: started.elapsed().as_millis() as u64,
    });
    true
}
