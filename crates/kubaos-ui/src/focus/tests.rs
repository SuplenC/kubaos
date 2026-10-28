use std::time::Duration;
use std::time::Instant;

use super::Light;
use super::glow;
use super::rings;
use crate::motion::Motion;
use crate::scale::Scale;
use crate::tokens;

const NEAR: f32 = tokens::FOCUS_NEAR as f32;
const FAR: f32 = tokens::FOCUS_FAR as f32;

#[test]
fn the_glow_starts_at_both_glows_opacities_together() {
    let at_tube = glow(0.0);

    let both = 1.0 - (1.0 - tokens::FOCUS_NEAR_OPACITY) * (1.0 - tokens::FOCUS_FAR_OPACITY);
    assert!((at_tube - both).abs() < 1e-6);
}

#[test]
fn the_glow_fades_out_to_nothing_at_its_reach() {
    let mut last = glow(0.0);
    for i in 1..=80 {
        let d = FAR * i as f32 / 80.0;
        let a = glow(d);
        assert!(a <= last, "the glow brightens at {d}");
        last = a;
    }

    assert!(glow(FAR).abs() < f32::EPSILON);
}

#[test]
fn past_the_near_reach_only_the_far_glow_shows() {
    let a = glow(NEAR);

    let far = tokens::FOCUS_FAR_OPACITY * (1.0 - NEAR / FAR).powi(2);
    assert!((a - far).abs() < 1e-6);
}

#[test]
fn the_glow_is_one_band_a_device_pixel_further_out_to_its_reach() {
    for scale in Scale::SUPPORTED {
        let drawn: Vec<(f32, f32)> = rings(scale).collect();

        let hairline = scale.hairline();
        assert_eq!(drawn.len(), (FAR * scale.factor()).round() as usize, "at {scale:?}");
        for (i, &(out, _)) in drawn.iter().enumerate() {
            assert!((out - hairline * (i + 1) as f32).abs() < 1e-4);
        }
    }
}

#[test]
fn the_bands_laid_over_one_another_make_the_glow() {
    for scale in Scale::SUPPORTED {
        let drawn: Vec<(f32, f32)> = rings(scale).collect();

        let hairline = scale.hairline();
        for i in 0..drawn.len() {
            let clear: f32 = drawn[i..].iter().map(|&(_, opacity)| 1.0 - opacity).product();
            let wanted = glow(hairline * (i as f32 + 0.5));
            assert!((1.0 - clear - wanted).abs() < 1e-5, "at {scale:?}, pixel {i}");
        }
    }
}

#[test]
fn a_control_shown_focused_is_lit_at_once() {
    let now = Instant::now();

    let light = Light::new(true, now);

    assert!((light.value(Motion::default(), now) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn focus_arriving_strikes_the_tube_on() {
    let now = Instant::now();
    let motion = Motion::default();
    let mut light = Light::new(false, now);

    light.to(true, motion, now);

    let step = Duration::from_millis(30);
    for (i, &brightness) in tokens::NEON_STRIKE.iter().enumerate() {
        let at = now + step * i as u32 + step / 2;
        assert!((light.value(motion, at) - brightness).abs() < 1e-6, "step {i}");
        assert!(light.moving(motion, at));
    }
    let done = now + Duration::from_millis(300);
    assert!((light.value(motion, done) - 1.0).abs() < f32::EPSILON);
    assert!(!light.moving(motion, done));
}

#[test]
fn with_the_stutter_off_focus_fades_in() {
    let now = Instant::now();
    let motion = Motion::default().no_stutter();
    let mut light = Light::new(false, now);

    light.to(true, motion, now);

    let early = light.value(motion, now + Duration::from_millis(20));
    let later = light.value(motion, now + Duration::from_millis(100));
    assert!(early > 0.0 && early < later && later < 1.0);
    assert!((light.value(motion, now + Duration::from_millis(200)) - 1.0).abs() < 1e-6);
}

#[test]
fn reduced_motion_fades_focus_in() {
    let now = Instant::now();
    let motion = Motion::default().reduced();
    let mut light = Light::new(false, now);

    light.to(true, motion, now);

    let first = light.value(motion, now + Duration::from_millis(45));
    let second = light.value(motion, now + Duration::from_millis(75));
    assert!(first < second, "a fade only brightens");
}

#[test]
fn focus_leaving_fades_the_tube_out_from_where_it_was() {
    let now = Instant::now();
    let motion = Motion::default();
    let mut light = Light::new(true, now);

    light.to(false, motion, now);

    let half = light.value(motion, now + Duration::from_millis(50));
    assert!(half > 0.0 && half < 1.0);
    assert!(light.value(motion, now + Duration::from_millis(200)).abs() < 1e-6);
    assert!(!light.moving(motion, now + Duration::from_millis(200)));
}

#[test]
fn the_same_focus_again_restarts_nothing() {
    let now = Instant::now();
    let motion = Motion::default();
    let mut light = Light::new(false, now);
    light.to(true, motion, now);
    let later = now + Duration::from_millis(45);
    let before = light.value(motion, later);

    light.to(true, motion, later);

    assert!((light.value(motion, later) - before).abs() < f32::EPSILON);
}
