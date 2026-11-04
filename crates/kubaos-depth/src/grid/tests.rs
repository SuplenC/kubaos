use super::*;

#[test]
fn a_grid_resized_to_its_own_size_is_unchanged() {
    let grid = Grid::from_fn(5, 3, |x, y| (x * 3 + y) as f32);

    assert_eq!(grid.resized(5, 3), grid);
}

#[test]
fn resizing_keeps_values_between_their_neighbours() {
    let grid = Grid::new(2, 1, vec![0.0, 1.0]);

    let wide = grid.resized(8, 1);

    assert!(wide.values().windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(wide.get(0, 0).abs() < 1e-6);
    assert!((wide.get(7, 0) - 1.0).abs() < 1e-6);
}

#[test]
fn a_summed_box_is_the_mean_of_what_it_holds() {
    let grid = Grid::from_fn(4, 4, |x, _| if x < 2 { 1.0 } else { 0.0 });
    let sums = Sums::new(&grid);

    assert!((sums.mean(0.0, 0.0, 2.0, 4.0, 0.0) - 1.0).abs() < 1e-6);
    assert!((sums.mean(0.0, 0.0, 4.0, 4.0, 0.0) - 0.5).abs() < 1e-6);
    assert!((sums.mean(1.0, 0.0, 3.0, 2.0, 0.0) - 0.5).abs() < 1e-6);
}

#[test]
fn a_box_past_the_edge_counts_the_outside_as_told() {
    let sums = Sums::new(&Grid::from_fn(2, 2, |_, _| 1.0));

    assert!((sums.mean(0.0, 0.0, 4.0, 2.0, 0.0) - 0.5).abs() < 1e-6);
    assert!((sums.mean(0.0, 0.0, 4.0, 2.0, 1.0) - 1.0).abs() < 1e-6);
    assert!((sums.mean_inside(-3.0, -3.0, 4.0, 4.0) - 1.0).abs() < 1e-6);
}

#[test]
fn otsu_splits_two_clusters_between_them_with_a_separation_near_one() {
    let values: Vec<f32> = (0..100).map(|i| if i < 60 { 0.1 } else { 0.8 }).collect();

    let (threshold, separation) = otsu(&values);

    assert!(threshold > 0.1 && threshold < 0.8, "{threshold}");
    assert!(separation > 0.99, "{separation}");
}

#[test]
fn otsu_finds_an_even_spread_three_quarters_separated() {
    let values: Vec<f32> = (0..=255).map(|i| i as f32 / 255.0).collect();

    let (_, separation) = otsu(&values);

    assert!((separation - 0.75).abs() < 0.01, "{separation}");
}

#[test]
fn a_box_mean_matches_the_summed_area_tables() {
    let grid = Grid::from_fn(13, 9, |x, y| ((x * 7 + y * 3) % 5) as f32);
    let sums = Sums::new(&grid);

    let means = box_mean(grid.values(), 13, 9, 2);

    for y in 0..9usize {
        for x in 0..13usize {
            let (x0, y0) = (x.saturating_sub(2) as f32, y.saturating_sub(2) as f32);
            let (x1, y1) = ((x + 3).min(13) as f32, (y + 3).min(9) as f32);
            let expected = sums.mean(x0, y0, x1, y1, 0.0);
            assert!((means[y * 13 + x] - expected).abs() < 1e-5, "at {x},{y}");
        }
    }
}
