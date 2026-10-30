use super::Measure;

fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{a} isn't {b}");
}

/// A page whose pane is 400 tall and 20 below the scroll's top, with sub-sections at 100, 500 and
/// 900, and a row every 48 from 160 on.
fn page() -> Measure {
    Measure {
        reach: 20.0,
        pane: 400.0,
        sidebar: 400.0,
        sidebar_max: 0.0,
        entries: Vec::new(),
        max: 800.0,
        subs: vec![100.0, 500.0, 900.0],
        rows: (0..20)
            .map(|i| (160.0 + 48.0 * i as f32, 204.0 + 48.0 * i as f32))
            .collect(),
    }
}

#[test]
fn the_first_sub_section_is_current_at_the_top() {
    assert_eq!(page().current(0.0), 0);
}

#[test]
fn a_sub_section_turns_current_as_its_heading_reaches_the_reading_line() {
    let page = page();
    let lands = page.offset_for(500.0);

    assert_eq!(page.current(lands - 2.0), 0);
    assert_eq!(page.current(lands), 1);
}

#[test]
fn the_last_sub_section_is_current_once_the_page_can_scroll_no_further() {
    let page = page();

    assert_eq!(page.current(page.max), 2);
}

#[test]
fn a_heading_scrolled_to_sits_at_the_pages_lead() {
    assert_close(page().offset_for(500.0), 500.0 - 20.0 - 32.0);
}

#[test]
fn a_heading_near_the_end_scrolls_only_as_far_as_the_page_goes() {
    assert_close(page().offset_for(5000.0), 800.0);
}

#[test]
fn a_row_in_view_doesnt_move_the_page() {
    assert_eq!(page().reveal(0, 0.0, 0.0), None);
}

#[test]
fn a_row_below_the_pane_scrolls_up_just_enough_to_show_it() {
    let page = page();
    let (_, bottom) = page.rows[10];

    let to = page.reveal(10, 0.0, 0.0).unwrap();

    assert_close(to + page.reach + page.pane, bottom + 24.0);
}

#[test]
fn a_row_above_the_pane_scrolls_down_to_show_it() {
    let page = page();
    let (top, _) = page.rows[2];

    let to = page.reveal(2, 600.0, 0.0).unwrap();

    assert_close(to + page.reach, top - 24.0);
}

#[test]
fn a_page_down_reaches_the_last_row_a_pane_shows_whole() {
    let page = page();

    assert_eq!(page.page_from(0, true), Some(7));
    assert_eq!(page.page_from(7, false), Some(0));
    assert_eq!(page.page_from(19, true), Some(19));
}

#[test]
fn a_row_scrolled_down_to_clears_what_floats_over_the_panes_top() {
    let page = page();
    let (top, _) = page.rows[2];

    let to = page.reveal(2, 600.0, 80.0).unwrap();

    assert_close(to + page.reach, top - 80.0 - 24.0);
}

#[test]
fn the_sidebar_scrolls_to_show_the_keyboards_row_with_its_padding() {
    let sidebar = Measure {
        sidebar: 200.0,
        sidebar_max: 300.0,
        entries: (0..10)
            .map(|i| (12.0 + 48.0 * i as f32, 56.0 + 48.0 * i as f32))
            .collect(),
        ..page()
    };

    assert_eq!(sidebar.reveal_entry(1, 0.0, 12.0), None);
    assert_close(sidebar.reveal_entry(5, 0.0, 12.0).unwrap(), 296.0 + 12.0 - 200.0);
    assert_close(sidebar.reveal_entry(0, 100.0, 12.0).unwrap(), 0.0);
}
