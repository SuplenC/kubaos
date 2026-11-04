use image::Rgb;

use super::*;

/// A purple sky over a dark city whose roofline is at row `roof`.
fn skyline(width: u32, height: u32, roof: u32) -> RgbImage {
    RgbImage::from_fn(width, height, |_, y| {
        if y < roof {
            Rgb([120, 80, 200])
        } else {
            Rgb([30, 25, 50])
        }
    })
}

#[test]
fn the_sky_is_keyed_out_at_the_pictures_own_roofline() {
    // 1920 wide, so the band the sky is keyed in reaches 10 rows either side of the coarse edge.
    let picture = skyline(1920, 300, 150);
    // A coarse edge 8 rows low and soft over 12.
    let coarse = Grid::from_fn(1920, 300, |_, y| ((y as f32 - 158.0) / 12.0 + 0.5).clamp(0.0, 1.0));

    let keyed = key_sky(&picture, &coarse);

    assert_close(keyed.get(200, 149), 0.0);
    assert_close(keyed.get(200, 150), 1.0);
    assert_close(keyed.get(200, 10), 0.0);
    assert_close(keyed.get(200, 290), 1.0);
}

#[test]
fn a_city_coloured_like_the_sky_keeps_the_coarse_edge() {
    let picture = RgbImage::from_pixel(200, 200, Rgb([120, 80, 200]));
    let coarse = Grid::from_fn(200, 200, |_, y| if y < 100 { 0.0 } else { 1.0 });

    assert_eq!(key_sky(&picture, &coarse), coarse);
}

#[test]
fn sam_is_pointed_inside_and_outside_the_object() {
    let region = Grid::from_fn(100, 100, |x, y| if (40..60).contains(&x) && y > 20 { 1.0 } else { 0.0 });

    let prompt = prompt_for(Method::Object, &region);

    let inside = |x: f32, y: f32| region.get((x * 100.0) as usize, (y * 100.0) as usize) > 0.5;
    for &(x, y, label) in &prompt.points {
        match label {
            1.0 => assert!(inside(x, y), "({x}, {y}) is outside"),
            0.0 => assert!(!inside(x, y), "({x}, {y}) is inside"),
            _ => {}
        }
    }
    assert!(prompt.points.iter().any(|p| is(p.2, 1.0)));
    assert!(prompt.points.iter().any(|p| is(p.2, 0.0)));
    assert!(prompt.points.contains(&(0.4, 0.21, 2.0)));
    assert!(prompt.points.contains(&(0.6, 1.0, 3.0)));
}

#[test]
fn the_sky_is_pointed_at_without_a_box() {
    let sky = Grid::from_fn(100, 100, |_, y| if y < 40 { 1.0 } else { 0.0 });

    let prompt = prompt_for(Method::Sky, &sky);

    assert!(prompt.points.iter().all(|p| is(p.2, 0.0) || is(p.2, 1.0)));
}

#[test]
fn sams_edge_is_taken_only_along_the_coarse_one() {
    // 1024 wide, so the band SAM's edge is taken in reaches 8 rows either side of the coarse edge.
    let coarse = Grid::from_fn(1024, 100, |_, y| if y < 50 { 0.0 } else { 1.0 });
    // SAM's region of the sky ends 3 rows higher, and claims a far patch of city as sky too.
    let traced = Grid::from_fn(1024, 100, |x, y| if y < 47 || (x < 10 && y > 90) { 1.0 } else { 0.0 });

    let merged = merge(Method::Sky, &coarse, &traced);

    assert_close(merged.get(100, 48), 1.0);
    assert_close(merged.get(100, 46), 0.0);
    assert_close(merged.get(5, 95), 1.0);
}

#[test]
fn iou_is_the_shared_share() {
    let a = Grid::new(4, 1, vec![1.0, 1.0, 0.0, 0.0]);
    let b = Grid::new(4, 1, vec![0.0, 1.0, 1.0, 0.0]);

    assert!((iou(&a, &b) - 1.0 / 3.0).abs() < 1e-6);
}

fn assert_close(value: f32, expected: f32) {
    assert!((value - expected).abs() < 1e-6, "{value} isn't {expected}");
}

fn is(value: f32, expected: f32) -> bool {
    (value - expected).abs() < 1e-6
}
