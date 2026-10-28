use super::*;

fn at(ms: u64) -> Sequence {
    at_with(ms, Motion::default())
}

fn at_with(ms: u64, motion: Motion) -> Sequence {
    let arrived = Instant::now();
    Sequence::new(arrived, arrived + Duration::from_millis(ms), motion)
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

#[test]
fn reduce_motion_shows_every_part_in_place_from_the_first_frame() {
    let seq = at_with(0, Motion::default().reduced());

    assert!(!seq.moving());
    for i in 0..3 {
        assert_eq!(seq.word(i), Reveal { shown: 1.0, below: 0.0 });
    }
    assert_eq!(seq.lede(), Reveal { shown: 1.0, below: 0.0 });
    assert_eq!(seq.actions(), Reveal { shown: 1.0, below: 0.0 });
    assert_eq!(seq.hints(), Reveal { shown: 1.0, below: 0.0 });
    for k in 0..mark::COUNT {
        assert!(close(seq.drawn(k), 1.0) && close(seq.lit(k), 1.0), "stroke {k}");
    }
    assert!(seq.stamp().is_some_and(|s| s.landed));
    assert!(close(seq.flare(), 0.0) && close(seq.halo(), 0.0));
    assert_eq!(seq.jolt(), Vector::ZERO);
}

#[test]
fn reduce_motion_holds_the_hum_still() {
    let reduced = Motion::default().reduced();

    let hums: Vec<f32> = [0, 900, 3000, 5200].map(|ms| at_with(ms, reduced).hum()).to_vec();

    assert!(hums.iter().all(|h| close(*h, hums[0])), "{hums:?}");
}

#[test]
fn nothing_shows_but_the_unlit_glass_at_the_start() {
    let seq = at(0);

    assert!(seq.moving());
    assert!(close(seq.drawn(0), 0.0) && close(seq.lit(0), 0.0));
    assert!(close(seq.word(0).shown, 0.0) && close(seq.lede().shown, 0.0));
    assert!(seq.stamp().is_none());
}

#[test]
fn everything_settles_by_3_6_s_and_only_the_hum_goes_on() {
    let seq = at(3600);

    assert!(!seq.moving());
    assert_eq!(seq.word(2), Reveal { shown: 1.0, below: 0.0 });
    assert!(close(seq.hints().shown, 1.0));
    assert!(seq.stamp().is_some_and(|s| s.landed));
    assert!(close(seq.flare(), 0.0) && close(seq.halo(), 0.0));
    assert_eq!(seq.jolt(), Vector::ZERO);
    assert!((HUM_LOW..=HUM_HIGH).contains(&seq.hum()));
}

#[test]
fn the_strokes_are_written_in_order_and_each_strikes_as_it_ends() {
    let seq = at(300 + 220 + 100);

    assert!(seq.drawn(0) > seq.drawn(1));
    assert!(close(seq.drawn(2), 0.0));
    assert!(seq.lit(1) <= 0.0, "stroke 1 strikes once its writing ends");
}

#[test]
fn every_tube_is_lit_when_the_hum_starts() {
    let seq = at(2000);

    for k in 0..mark::COUNT {
        assert!(close(seq.drawn(k), 1.0) && close(seq.lit(k), 1.0), "stroke {k}");
    }
}

#[test]
fn the_hum_starts_at_its_top_without_a_step() {
    let start = at(0).hum_at();
    let ms = u64::try_from(start.as_millis()).unwrap();

    assert!(close(at(ms - 1).hum(), HUM_HIGH));
    assert!((at(ms + 1).hum() - HUM_HIGH).abs() < 0.001);
}

#[test]
fn the_hum_stays_between_its_bottom_and_its_top() {
    for ms in (2000..8000).step_by(37) {
        let h = at(ms).hum();
        assert!((HUM_LOW - 1e-4..=HUM_HIGH + 1e-4).contains(&h), "{ms} ms: {h}");
    }
}

#[test]
fn the_hum_never_asks_for_frames_faster_than_the_display_nor_waits_long() {
    for levels in [HUM_LEVELS, HUM_LEVELS_SOFTWARE] {
        for ms in (4000..9000).step_by(13) {
            let next = at(ms).next_hum(levels);
            assert!(next >= FRAME && next <= HUM_WAIT, "{ms} ms: {next:?}");
        }
    }
}

/// How many times the hum redraws in one breath, stepping `levels` at a time.
fn redraws_per_breath(levels: f32) -> u32 {
    let mut ms = 4000.0_f64;
    let mut redraws = 0;
    while ms < 4000.0 + HUM_PERIOD.as_secs_f64() * 1000.0 {
        ms += at(ms as u64).next_hum(levels).as_secs_f64() * 1000.0;
        redraws += 1;
    }
    redraws
}

#[test]
fn the_hum_redraws_about_thirty_times_a_second_on_the_gpu_and_ten_in_software() {
    let (gpu, software) = (redraws_per_breath(HUM_LEVELS), redraws_per_breath(HUM_LEVELS_SOFTWARE));

    assert!((60..=84).contains(&gpu), "{gpu} redraws in one breath on the GPU");
    assert!(
        (20..=30).contains(&software),
        "{software} redraws in one breath in software"
    );
}

#[test]
fn the_hanko_lands_full_size_at_its_lean() {
    let coming = at(2650).stamp().unwrap();
    let landed = at(2600 + 300).stamp().unwrap();

    assert!(!coming.landed && coming.scale > 1.0);
    assert!(landed.landed && close(landed.scale, 1.0) && close(landed.lean.0, art::LEAN.0));
}

#[test]
fn the_mark_flares_white_hot_on_impact_and_is_back_within_micro() {
    assert!(close(at(2779).flare(), 0.0));
    assert!(at(2780).flare() > 0.99);
    assert!(close(at(2780 + 100).flare(), 0.0));
    assert!(at(2780 + 400).halo() > 0.0);
    assert!(close(at(2780 + 800).halo(), 0.0));
}

#[test]
fn the_jolt_ends_where_it_started() {
    assert_eq!(at(2779).jolt(), Vector::ZERO);
    assert_ne!(at(2830).jolt(), Vector::ZERO);
    assert_eq!(at(3000).jolt(), Vector::ZERO);
}

#[test]
fn a_stroke_is_drawn_part_of_the_way_and_not_at_all_before_it_starts() {
    let stroke = &mark::strokes()[1];

    assert!(stroke.path(0.0).is_none());
    assert!(stroke.path(0.5).is_some());
    assert!(stroke.length() > stroke.width());
}

#[test]
fn the_mark_has_the_logos_six_strokes_the_dakuten_cold() {
    let tones: Vec<mark::Tone> = mark::strokes().iter().map(|s| s.tone).collect();

    assert_eq!(tones.len(), mark::COUNT);
    assert_eq!(&tones[4..], &[mark::Tone::Cold, mark::Tone::Cold]);
    assert!(tones[..4].iter().all(|t| *t == mark::Tone::Warm));
}

fn glow(arrived: Instant, motion: Motion) -> Glow {
    Glow {
        arrived,
        now: arrived,
        motion,
        colors: color::colors(kubaos_ui::Mode::Dark),
        art: None,
    }
}

/// What the glow asks for when a frame `ms` after the screen arrived is drawn.
fn redraw_after(ms: u64, motion: Motion) -> Option<window::RedrawRequest> {
    let arrived = Instant::now();
    let at = arrived + Duration::from_millis(ms);
    let event = canvas::Event::Window(window::Event::RedrawRequested(at));

    let action = canvas::Program::update(
        &glow(arrived, motion),
        &mut GlowState::default(),
        &event,
        Rectangle::default(),
        mouse::Cursor::Unavailable,
    );

    action.map(|action| action.into_inner().1)
}

#[test]
fn the_app_draws_every_frame_until_the_done_screen_settles_and_then_none() {
    let arrived = Instant::now();

    assert!(moving(arrived, arrived + Duration::from_secs(3), Motion::default()));
    assert!(!moving(
        arrived,
        arrived + Duration::from_millis(3600),
        Motion::default()
    ));
    assert!(!moving(arrived, arrived, Motion::default().reduced()));
}

#[test]
fn the_settled_glow_asks_for_its_next_frame_itself_and_only_then() {
    assert_eq!(redraw_after(1000, Motion::default()), None);
    assert_eq!(redraw_after(9000, Motion::default().reduced()), None);
    let Some(window::RedrawRequest::At(next)) = redraw_after(9000, Motion::default()) else {
        panic!("the settled hum asks for a frame at a time");
    };
    assert!(next > Instant::now());
}
