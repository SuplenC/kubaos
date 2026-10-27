use std::time::Duration as Elapsed;
use std::time::Instant;

use super::move_to;
use crate::motion::Tween;
use crate::tokens::Duration;
use crate::tokens::Easing;

#[test]
fn a_following_bar_eases_to_the_work_and_a_timed_one_is_there_at_once() {
    let now = Instant::now();
    let soon = now + Elapsed::from_millis(50);
    let mut following = Tween::new(0.0, Duration::Short, Easing::Linear);
    let mut timed = Tween::new(0.0, Duration::Short, Easing::Linear);

    move_to(&mut following, 0.8, true, now);
    move_to(&mut timed, 0.8, false, now);

    assert!(following.value(soon) < 0.8);
    assert!((timed.value(soon) - 0.8).abs() < 1e-6);
}
