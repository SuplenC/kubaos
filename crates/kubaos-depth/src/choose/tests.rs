use super::*;

/// A 3:2 picture the size of the shipped wallpapers, and the masks' size over it.
const WIDTH: f32 = 3840.0;
const HEIGHT: f32 = 2560.0;
const MASK: (usize, usize) = (384, 256);

/// A depth map over the picture from `depth` given shares of the picture.
fn depth(depth: impl Fn(f32, f32) -> f32) -> Grid {
    Grid::from_fn(MASK.0, MASK.1, |x, y| {
        depth((x as f32 + 0.5) / MASK.0 as f32, (y as f32 + 0.5) / MASK.1 as f32)
    })
}

/// A skyline: the sky at the far plane above 0.55, a city from 0.3 to 1 below it.
fn skyline(x: f32, y: f32) -> f32 {
    if y < 0.55 {
        0.0
    } else {
        0.3 + 0.7 * (y - 0.55) / 0.45 + 0.05 * (x * 40.0).sin().abs()
    }
}

#[test]
fn a_clean_sky_is_kept_with_the_city_in_front() {
    let candidate = sky(&depth(skyline)).expect("a sky");

    let scored = candidate_score(&candidate, WIDTH, HEIGHT);

    assert!(candidate.separation > 0.99, "{}", candidate.separation);
    assert!(scored.score >= SCORE_MIN, "{scored:?}");
    assert!(candidate.mask.get(10, 10) < 0.01);
    assert!(candidate.mask.get(10, 250) > 0.99);
}

#[test]
fn a_street_with_no_sky_gives_no_sky() {
    let street = depth(|_, y| 0.1 + 0.9 * y);

    assert!(sky(&street).is_none());
}

#[test]
fn a_sky_hazed_into_the_city_is_not_kept() {
    // The far plane above, and a haze just past it as far down as the city: the sky's edge is
    // anybody's guess.
    let hazy = depth(|x, y| {
        if y < 0.4 {
            0.0
        } else if y < 0.6 {
            0.02 + 0.01 * x
        } else {
            skyline(x, y)
        }
    });

    let candidate = sky(&hazy).expect("a sky, however hazy");

    assert!(candidate.separation < SEPARATION_MIN, "{}", candidate.separation);
    assert!(candidate_score(&candidate, WIDTH, HEIGHT).score < f32::EPSILON);
}

#[test]
fn a_sky_not_along_the_top_is_no_sky() {
    let low = depth(|_, y| if (0.5..0.7).contains(&y) { 0.0 } else { 0.6 });

    assert!(sky(&low).is_none());
}

#[test]
fn a_clean_tower_is_kept() {
    let tower = depth(|x, y| if (0.6..0.66).contains(&x) && y > 0.15 { 1.0 } else { 0.0 });

    let candidate = object(&tower).expect("an object");
    let scored = candidate_score(&candidate, WIDTH, HEIGHT);

    assert!(scored.score >= SCORE_MIN, "{scored:?}");
    assert_eq!(choose(&[scored]).0, Some(scored));
}

#[test]
fn an_unsure_object_is_not_kept() {
    // BiRefNet half sure of a large blob: its chances spread from 0.3 to 0.7.
    let unsure = depth(|x, y| {
        if (0.3..0.7).contains(&x) && y > 0.5 {
            0.3 + 0.4 * x
        } else {
            0.3 * y
        }
    });

    let scored = object(&unsure).map(|c| candidate_score(&c, WIDTH, HEIGHT));

    assert!(scored.is_none_or(|s| s.score < SCORE_MIN), "{scored:?}");
}

#[test]
fn no_object_is_no_candidate() {
    assert!(object(&depth(|_, _| 0.0)).is_none());
}

#[test]
fn an_object_filling_the_picture_is_no_candidate() {
    assert!(object(&depth(|_, y| if y > 0.2 { 1.0 } else { 0.0 })).is_none());
}

#[test]
fn the_better_score_is_kept() {
    let sky = Scored {
        method: Method::Sky,
        score: 0.8,
        placed: Some(placed()),
    };
    let object = Scored {
        method: Method::Object,
        score: 0.9,
        placed: Some(placed()),
    };

    assert_eq!(choose(&[sky, object]).0, Some(object));
}

#[test]
fn nothing_is_kept_under_the_least_score_but_the_best_is_told() {
    let weak = Scored {
        method: Method::Sky,
        score: SCORE_MIN - 0.01,
        placed: Some(placed()),
    };

    assert_eq!(choose(&[weak]), (None, Some(weak)));
}

fn placed() -> Placed {
    Placed {
        place: clock::Place { x: 0.5, y: 0.5 },
        cover: 0.2,
    }
}
