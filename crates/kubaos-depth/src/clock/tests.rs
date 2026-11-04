use super::*;

/// A 3:2 picture the size of the shipped wallpapers.
const WIDTH: f32 = 3840.0;
const HEIGHT: f32 = 2560.0;

/// A mask over the picture at a tenth of its size, from `inside` given shares of the picture.
fn mask(inside: impl Fn(f32, f32) -> bool) -> (Sums, usize) {
    let (width, height) = (384, 256);
    let grid = Grid::from_fn(width, height, |x, y| {
        if inside((x as f32 + 0.5) / width as f32, (y as f32 + 0.5) / height as f32) {
            1.0
        } else {
            0.0
        }
    });
    (Sums::new(&grid), width)
}

use crate::grid::Grid;

#[test]
fn the_digits_box_is_the_clock_rest_size_on_the_lock_page_screen() {
    let (boxes, fits) = boxes(Place { x: 0.5, y: 0.5 }, WIDTH, HEIGHT, PRIMARY);

    // The picture covers 1536 by 1024 on the 1536 by 960 screen: a tenth of a picture pixel is
    // 0.04 of a logical one.
    let scale = PRIMARY.width / WIDTH;
    assert!(fits);
    assert!(((boxes.digits.x1 - boxes.digits.x0) * scale - DIGITS_WIDTH * DIGITS_SIZE).abs() < 0.01);
    assert!(((boxes.digits.y1 - boxes.digits.y0) * scale - CAP_HEIGHT * DIGITS_SIZE).abs() < 0.01);
    assert!((boxes.digits.y1 - HEIGHT / 2.0).abs() < 0.01);
}

#[test]
fn the_kicker_stands_above_the_digits_with_its_halo() {
    let (boxes, _) = boxes(Place { x: 0.5, y: 0.5 }, WIDTH, HEIGHT, PRIMARY);

    let scale = PRIMARY.width / WIDTH;
    assert!(boxes.kicker.y1 < boxes.digits.y0);
    assert!(((boxes.digits.y0 - boxes.kicker.y1) * scale - (KICKER_GAP - HALO)).abs() < 0.01);
    assert!(((boxes.kicker.x1 - boxes.kicker.x0) * scale - (KICKER_WIDTH + 2.0 * HALO)).abs() < 0.01);
}

#[test]
fn a_clock_past_the_screen_margin_does_not_fit() {
    let (_, fits) = boxes(Place { x: 0.1, y: 0.5 }, WIDTH, HEIGHT, PRIMARY);

    assert!(!fits);
}

#[test]
fn a_skyline_puts_the_clock_with_its_feet_behind_the_city() {
    let (sums, mask_width) = mask(|_, y| y > 0.55);

    let placed = place_for(&sums, mask_width, WIDTH, HEIGHT).expect("a place over a skyline");

    assert!((COVER_MIN..=COVER_MAX).contains(&placed.cover), "{placed:?}");
    assert!(placed.place.y > 0.55, "{placed:?}");
    assert!((placed.place.x - 0.5).abs() < 0.01, "{placed:?}");
}

#[test]
fn the_kicker_stays_clear_of_the_cut_out_on_every_screen() {
    let (sums, mask_width) = mask(|x, y| (0.62..0.64).contains(&x) && y > 0.2);

    let placed = place_for(&sums, mask_width, WIDTH, HEIGHT).expect("a place beside a spire");

    for screen in SCREENS {
        let (boxes, fits) = boxes(placed.place, WIDTH, HEIGHT, screen);
        let kicker = boxes.kicker;
        assert!(fits);
        let beside = kicker.x1 < 0.62 * WIDTH || kicker.x0 > 0.64 * WIDTH;
        assert!(beside || kicker.y1 < 0.2 * HEIGHT, "{placed:?} on {screen:?}");
    }
}

#[test]
fn a_mask_over_the_whole_picture_leaves_the_clock_no_place() {
    let (sums, mask_width) = mask(|_, _| true);

    assert_eq!(place_for(&sums, mask_width, WIDTH, HEIGHT), None);
}

#[test]
fn an_empty_mask_leaves_the_clock_no_place() {
    let (sums, mask_width) = mask(|_, _| false);

    assert_eq!(place_for(&sums, mask_width, WIDTH, HEIGHT), None);
}
