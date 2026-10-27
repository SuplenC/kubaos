use std::time;

use super::Motion;
use super::Tween;
use super::ease;
use crate::tokens::Duration;
use crate::tokens::Easing;

const EASINGS: [Easing; 6] = [
    Easing::Standard,
    Easing::StandardDecel,
    Easing::StandardAccel,
    Easing::EmphasizedDecel,
    Easing::EmphasizedAccel,
    Easing::Linear,
];

#[test]
fn every_curve_starts_at_zero_and_ends_at_one() {
    for easing in EASINGS {
        assert!(ease(easing, 0.0).abs() < 1e-6, "{easing:?}");
        assert!((ease(easing, 1.0) - 1.0).abs() < 1e-6, "{easing:?}");
    }
}

#[test]
fn every_curve_only_moves_forward() {
    for easing in EASINGS {
        let mut last = 0.0;
        for i in 0..=200 {
            let y = ease(easing, i as f32 / 200.0);
            assert!(y + 1e-4 >= last, "{easing:?} at {i}");
            last = y;
        }
    }
}

#[test]
fn linear_is_the_identity() {
    for i in 0..=10 {
        let x = i as f32 / 10.0;
        assert!((ease(Easing::Linear, x) - x).abs() < 1e-4);
    }
}

#[test]
fn arriving_curves_lead_the_clock_and_leaving_ones_lag_it() {
    assert!(ease(Easing::EmphasizedDecel, 0.25) > 0.6);
    assert!(ease(Easing::StandardDecel, 0.25) > 0.25);
    assert!(ease(Easing::StandardAccel, 0.25) < 0.25);
    assert!(ease(Easing::EmphasizedAccel, 0.25) < 0.25);
}

#[test]
fn speed_divides_every_duration() {
    let twice = Motion::default().speed(2.0);

    assert_eq!(twice.duration(Duration::Medium), time::Duration::from_millis(150));
}

#[test]
fn speed_stays_in_the_settings_range() {
    let fast = Motion::default().speed(10.0);

    assert_eq!(fast.duration(Duration::Long), time::Duration::from_millis(250));
}

#[test]
fn reduced_motion_makes_long_moves_short() {
    let reduced = Motion::default().reduced();

    assert_eq!(reduced.duration(Duration::Scene), time::Duration::from_millis(200));
    assert_eq!(reduced.duration(Duration::Micro), time::Duration::from_millis(100));
}

#[test]
fn an_instant_animation_is_already_done() {
    let progress = Motion::default().progress(Duration::Instant, Easing::Standard, time::Duration::ZERO);

    assert!((progress - 1.0).abs() < f32::EPSILON);
}

#[test]
fn durations_grow_from_instant_to_scene() {
    let ms = [
        Duration::Instant,
        Duration::Micro,
        Duration::Short,
        Duration::Medium,
        Duration::Long,
        Duration::Slow,
        Duration::Scene,
    ]
    .map(Duration::ms);

    assert_eq!(ms, [0, 100, 200, 300, 500, 800, 1200]);
}

#[test]
fn a_tween_moves_from_where_it_is_to_each_new_value() {
    let start = std::time::Instant::now();
    let mut tween = Tween::new(0.0, Duration::Short, Easing::Linear);
    tween.to(1.0, start);
    let half = start + time::Duration::from_millis(100);

    assert!((tween.value(half) - 0.5).abs() < 1e-3);
    assert!(tween.moving(half));
    tween.to(0.0, half);
    assert!((tween.value(half) - 0.5).abs() < 1e-3);
    let done = half + time::Duration::from_millis(200);
    assert!(tween.value(done).abs() < 1e-6);
    assert!(!tween.moving(done));
}

#[test]
fn the_same_target_again_restarts_nothing() {
    let start = std::time::Instant::now();
    let mut tween = Tween::new(0.0, Duration::Short, Easing::Linear);
    tween.to(1.0, start);
    let later = start + time::Duration::from_millis(100);

    tween.to(1.0, later);

    assert!((tween.value(later) - 0.5).abs() < 1e-3);
}
