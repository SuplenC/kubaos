use std::time::Instant;

use super::Content;
use super::Entry;
use super::Event;
use super::Found;
use super::HomeOutline;
use super::Key;
use super::Measure;
use super::Message;
use super::Outline;
use super::Place;
use super::Region;
use super::RowAt;
use super::SectionOutline;
use super::SidebarPage;
use super::SubOutline;
use crate::icon::Icon;
use crate::motion::Motion;

fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{a} isn't {b}");
}

/// Two sections: the first with three sub-sections of 2, 3 and 1 rows, the second with one of 2.
fn outline() -> Outline {
    Outline {
        home: HomeOutline {
            label: "Home".into(),
            icon: Icon::Search,
            accent: "設定".into(),
            title: "Settings".into(),
            lede: "Everything, one search away".into(),
            placeholder: "Search settings".into(),
        },
        sections: vec![
            SectionOutline::new(Icon::Bolt, "Power", "電源", "Juice, managed")
                .with_sub(SubOutline::new("Battery").well(2))
                .with_sub(SubOutline::new("Sleep").well(2).well(1))
                .with_sub(SubOutline::new("Performance").well(1)),
            SectionOutline::new(Icon::Wifi, "Network", "通信", "Strings attached")
                .with_sub(SubOutline::new("Wi-Fi").well(2)),
        ],
    }
}

/// Rows whose row number in their well is even match anything typed.
struct EvenRows;

impl Content for EvenRows {
    fn matches(&self, at: RowAt, _query: &str) -> bool {
        at.row.is_multiple_of(2)
    }
}

/// No row matches.
struct NoRows;

impl Content for NoRows {
    fn matches(&self, _at: RowAt, _query: &str) -> bool {
        false
    }
}

/// Sub-sections at 100, 400 and 700 in a pane 300 tall, 20 below the scroll's top, the content
/// scrolling 600; rows every 48 from 160.
fn measure() -> Measure {
    Measure {
        reach: 20.0,
        pane: 300.0,
        sidebar: 300.0,
        sidebar_max: 0.0,
        entries: Vec::new(),
        max: 600.0,
        subs: vec![100.0, 400.0, 700.0],
        rows: (0..6)
            .map(|i| (160.0 + 48.0 * i as f32, 204.0 + 48.0 * i as f32))
            .collect(),
    }
}

fn press(page: &mut SidebarPage, key: Key) -> Option<Event> {
    page.press(key, &EvenRows).1
}

fn text(page: &mut SidebarPage, typed: &str) -> Option<Event> {
    press(page, Key::Text(typed.into()))
}

/// On Power, measured, the keyboard in the sidebar.
fn on_power() -> SidebarPage {
    let mut page = SidebarPage::new(outline());
    press(&mut page, Key::Tab { shift: false });
    press(&mut page, Key::Down);
    let _ = page.measured(measure());
    page
}

#[test]
fn it_opens_on_home_with_the_keyboard_in_the_search_field() {
    let page = SidebarPage::new(outline());

    assert_eq!(page.place(), Place::Home);
    assert_eq!(page.region(), Region::Search);
}

#[test]
fn tab_goes_from_search_to_the_sidebar_and_round() {
    let mut page = SidebarPage::new(outline());

    press(&mut page, Key::Tab { shift: false });
    assert_eq!(page.region(), Region::Sidebar);

    press(&mut page, Key::Tab { shift: false });
    assert_eq!(page.region(), Region::Search);
}

#[test]
fn on_a_section_tab_goes_from_the_sidebar_to_the_page_and_shift_tab_back() {
    let mut page = on_power();

    press(&mut page, Key::Tab { shift: false });
    assert_eq!(page.region(), Region::Page);

    press(&mut page, Key::Tab { shift: true });
    assert_eq!(page.region(), Region::Sidebar);
}

#[test]
fn moving_down_the_sidebar_opens_the_section_it_lands_on() {
    let page = on_power();

    assert_eq!(page.place(), Place::Section(0));
    assert_eq!(page.cursor(), Entry::Section(0));
}

#[test]
fn only_the_section_that_shows_lists_its_sub_sections() {
    let page = on_power();

    assert_eq!(
        page.entries(),
        vec![
            Entry::Home,
            Entry::Section(0),
            Entry::Sub(0, 0),
            Entry::Sub(0, 1),
            Entry::Sub(0, 2),
            Entry::Section(1),
        ]
    );
}

#[test]
fn the_neovim_keys_move_in_the_sidebar_as_the_arrows_do_and_wrap_round() {
    let mut page = on_power();

    text(&mut page, "j");
    assert_eq!(page.cursor(), Entry::Sub(0, 0));

    text(&mut page, "k");
    text(&mut page, "k");
    assert_eq!(page.cursor(), Entry::Home);

    press(&mut page, Key::Up);
    assert_eq!(page.cursor(), Entry::Section(1));
}

#[test]
fn a_sub_section_in_the_sidebar_scrolls_its_heading_to_the_pages_lead() {
    let mut page = on_power();

    press(&mut page, Key::Down);
    press(&mut page, Key::Down);

    assert_eq!(page.cursor(), Entry::Sub(0, 1));
    assert_close(page.offset(), measure().offset_for(400.0));
    assert_eq!(page.current_sub(), 1);
}

#[test]
fn a_sub_section_picked_stays_current_where_the_spy_would_pick_the_last() {
    let mut page = on_power();
    let short = Measure {
        max: 340.0,
        ..measure()
    };
    let _ = page.measured(short);
    let _ = page.update(Message::Pick(Entry::Sub(0, 1)), &EvenRows);

    page.scrolled(340.0);
    assert_eq!(page.current_sub(), 1);

    page.scrolled(300.0);
    assert_eq!(page.current_sub(), 0);
}

#[test]
fn the_sidebar_follows_the_page_as_it_scrolls() {
    let mut page = on_power();
    press(&mut page, Key::Down);

    page.scrolled(measure().offset_for(400.0));

    assert_eq!(page.current_sub(), 1);
    assert_eq!(page.cursor(), Entry::Sub(0, 1));
}

#[test]
fn right_goes_into_the_page_at_the_cursors_sub_section_and_left_comes_back() {
    let mut page = on_power();
    press(&mut page, Key::Down);
    press(&mut page, Key::Down);

    press(&mut page, Key::Right { word: false });
    assert_eq!(page.region(), Region::Page);
    assert_eq!(
        page.row(),
        Some(RowAt {
            section: 0,
            sub: 1,
            well: 0,
            row: 0
        })
    );

    press(&mut page, Key::Down);
    press(&mut page, Key::Down);
    text(&mut page, "h");
    assert_eq!(page.region(), Region::Sidebar);
    assert_eq!(page.cursor(), Entry::Sub(0, 1));
}

#[test]
fn rows_wrap_round_but_home_and_end_stop_at_the_ends() {
    let mut page = on_power();
    press(&mut page, Key::Enter { shift: false });

    press(&mut page, Key::Up);
    assert_eq!(
        page.row(),
        Some(RowAt {
            section: 0,
            sub: 2,
            well: 0,
            row: 0
        })
    );

    press(&mut page, Key::Home);
    assert_eq!(
        page.row(),
        Some(RowAt {
            section: 0,
            sub: 0,
            well: 0,
            row: 0
        })
    );

    press(&mut page, Key::End);
    press(&mut page, Key::End);
    assert_eq!(
        page.row(),
        Some(RowAt {
            section: 0,
            sub: 2,
            well: 0,
            row: 0
        })
    );
}

#[test]
fn a_row_below_the_pane_scrolls_into_view_as_the_keyboard_reaches_it() {
    let mut page = on_power();
    press(&mut page, Key::Enter { shift: false });

    for _ in 0..5 {
        press(&mut page, Key::Down);
    }

    assert_close(page.offset(), measure().reveal(5, 0.0, 0.0).unwrap());
}

#[test]
fn enter_and_space_on_a_row_go_to_the_app() {
    let mut page = on_power();
    press(&mut page, Key::Right { word: false });
    let first = RowAt {
        section: 0,
        sub: 0,
        well: 0,
        row: 0,
    };

    assert_eq!(press(&mut page, Key::Enter { shift: false }), Some(Event::Press(first)));
    assert_eq!(press(&mut page, Key::Space), Some(Event::Toggle(first)));
}

#[test]
fn control_page_down_switches_section_from_anywhere_home_included() {
    let mut page = SidebarPage::new(outline());

    press(&mut page, Key::NextSection);
    assert_eq!(page.place(), Place::Section(0));

    press(&mut page, Key::NextSection);
    press(&mut page, Key::NextSection);
    assert_eq!(page.place(), Place::Home);
    assert_eq!(page.region(), Region::Search);

    press(&mut page, Key::PreviousSection);
    assert_eq!(page.place(), Place::Section(1));
}

#[test]
fn changing_section_fades_the_page_in_and_reduced_motion_swaps_it() {
    let page = on_power();
    assert!(page.moving(Instant::now()));

    let mut still = SidebarPage::new(outline()).with_motion(Motion::default().reduced());
    press(&mut still, Key::NextSection);
    assert!(!still.moving(Instant::now()));
}

#[test]
fn settling_ends_every_motion_and_leaves_an_open_find_box_shown() {
    let mut page = on_power();
    press(&mut page, Key::Find);

    page.settle();

    assert!(!page.moving(Instant::now()));
    let (shown, above) = page.find_arrival(Instant::now());
    assert_close(shown, 1.0);
    assert_close(above, 0.0);
}

#[test]
fn typing_in_homes_field_searches_and_enter_opens_the_result() {
    let mut page = SidebarPage::new(outline());

    assert_eq!(text(&mut page, "l"), Some(Event::Query("l".into())));
    page.set_results(3);
    press(&mut page, Key::Down);
    press(&mut page, Key::Down);

    assert_eq!(press(&mut page, Key::Enter { shift: false }), Some(Event::Open(2)));
}

#[test]
fn esc_in_homes_field_clears_it_then_goes_to_the_sidebar() {
    let mut page = SidebarPage::new(outline());
    text(&mut page, "lid");

    assert_eq!(press(&mut page, Key::Esc), Some(Event::Query(String::new())));
    assert_eq!(page.region(), Region::Search);

    press(&mut page, Key::Esc);
    assert_eq!(page.region(), Region::Sidebar);
}

#[test]
fn slash_on_home_goes_to_its_field_rather_than_a_find_box() {
    let mut page = SidebarPage::new(outline());
    press(&mut page, Key::Tab { shift: false });

    text(&mut page, "/");

    assert_eq!(page.region(), Region::Search);
    assert_eq!(page.found(), None);
}

#[test]
fn landing_from_home_opens_the_section_on_the_row_with_the_landed_highlight() {
    let mut page = SidebarPage::new(outline());
    let at = RowAt {
        section: 0,
        sub: 1,
        well: 1,
        row: 0,
    };

    let _ = page.land(at);

    assert_eq!(page.place(), Place::Section(0));
    assert_eq!(page.region(), Region::Page);
    assert_eq!(page.row(), Some(at));
    assert!(page.row_state(at, 4, Instant::now()).landed > 0.9);
}

#[test]
fn find_opens_on_a_page_and_counts_its_matches() {
    let mut page = on_power();
    press(&mut page, Key::Tab { shift: false });

    text(&mut page, "/");
    text(&mut page, "s");

    assert_eq!(page.region(), Region::Find);
    assert_eq!(page.found(), Some((1, 4)));
    assert_eq!(
        page.row(),
        Some(RowAt {
            section: 0,
            sub: 0,
            well: 0,
            row: 0
        })
    );
}

#[test]
fn enter_and_down_go_to_the_next_match_shift_enter_and_up_to_the_previous() {
    let mut page = on_power();
    press(&mut page, Key::Find);
    text(&mut page, "s");

    press(&mut page, Key::Enter { shift: false });
    press(&mut page, Key::Down);
    assert_eq!(page.found(), Some((3, 4)));

    press(&mut page, Key::Enter { shift: true });
    press(&mut page, Key::Up);
    press(&mut page, Key::Up);
    assert_eq!(page.found(), Some((4, 4)));
}

#[test]
fn rows_show_whether_they_match_and_which_is_current() {
    let mut page = on_power();
    press(&mut page, Key::Find);
    text(&mut page, "s");
    let row = |r: usize, well: usize, sub: usize| RowAt {
        section: 0,
        sub,
        well,
        row: r,
    };
    let now = Instant::now();

    assert_eq!(page.row_state(row(0, 0, 0), 0, now).found, Found::Current);
    assert_eq!(page.row_state(row(1, 0, 0), 1, now).found, Found::Miss);
    assert_eq!(page.row_state(row(0, 0, 1), 2, now).found, Found::Match);
    assert_eq!(page.row_state(row(0, 0, 1), 2, now).query, "s");
}

#[test]
fn esc_closes_find_and_puts_the_keyboard_and_the_page_back() {
    let mut page = on_power();
    press(&mut page, Key::Right { word: false });
    press(&mut page, Key::Down);
    let (row, offset) = (page.row(), page.offset());
    press(&mut page, Key::Find);
    text(&mut page, "s");
    press(&mut page, Key::End);
    for _ in 0..3 {
        press(&mut page, Key::Down);
    }

    press(&mut page, Key::Esc);

    assert_eq!(page.found(), None);
    assert_eq!(page.region(), Region::Page);
    assert_eq!(page.row(), row);
    assert_close(page.offset(), offset);
}

#[test]
fn esc_on_find_keeps_a_sub_section_picked_current() {
    let mut page = on_power();
    let _ = page.measured(Measure {
        max: 340.0,
        ..measure()
    });
    let _ = page.update(Message::Pick(Entry::Sub(0, 1)), &EvenRows);
    press(&mut page, Key::Find);
    text(&mut page, "s");
    press(&mut page, Key::Up);

    press(&mut page, Key::Esc);

    assert_eq!(page.current_sub(), 1);
}

#[test]
fn a_press_on_a_home_result_opens_it() {
    let mut page = SidebarPage::new(outline());

    let (_, event) = page.update(Message::Open(2), &EvenRows);

    assert_eq!(event, Some(Event::Open(2)));
}

#[test]
fn with_no_match_enter_searches_every_setting_from_home_with_the_query() {
    let mut page = on_power();
    let _ = page.press(Key::Find, &NoRows);
    let _ = page.press(Key::Text("lid".into()), &NoRows);
    assert_eq!(page.found(), Some((0, 0)));

    let (_, event) = page.press(Key::Enter { shift: false }, &NoRows);

    assert_eq!(event, Some(Event::Query("lid".into())));
    assert_eq!(page.place(), Place::Home);
    assert_eq!(page.region(), Region::Search);
    assert_eq!(page.query(), "lid");
}

#[test]
fn a_press_on_a_row_selects_it_quietly() {
    let mut page = on_power();
    let at = RowAt {
        section: 0,
        sub: 1,
        well: 0,
        row: 1,
    };

    let _ = page.update(Message::Point(at), &EvenRows);

    assert_eq!(page.region(), Region::Page);
    assert_eq!(page.row(), Some(at));
    assert!(!page.keyboard);
}

#[test]
fn page_down_in_the_sidebar_moves_by_the_rows_it_shows_whole() {
    let mut page = on_power();

    press(&mut page, Key::PageDown);

    // 300 tall less its 12 above and below holds five 48 steps.
    assert_eq!(page.cursor(), Entry::Section(1));
}

/// The first row of Power's page laid out at `size`, the find box open over it.
fn first_row_at(size: iced::Size) -> iced::Rectangle {
    use iced::widget;
    use iced_test::selector;
    use iced_test::simulator::Simulator;

    use crate::look::Look;
    use crate::scale::Scale;
    use crate::tokens::Mode;

    let mut page = on_power();
    press(&mut page, Key::Find);
    let look = Look::new(Mode::Dark, Scale::X1);
    let view = super::view(
        &page,
        &look,
        |_| (),
        |_, _| widget::Space::new().into(),
        |at, _, _| {
            let id = if at.sub == 0 && at.row == 0 { "first" } else { "other" };
            widget::container(widget::Space::new().height(44.0))
                .width(iced::Length::Fill)
                .id(id)
                .into()
        },
    );
    let settings = iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    };
    let mut ui = Simulator::with_size(settings, size, view);
    ui.find(selector::id("first")).expect("the first row").bounds()
}

#[test]
fn at_the_smallest_window_the_sidebar_stays_and_the_page_takes_the_small_padding() {
    let first = first_row_at(iced::Size::new(720.0, 480.0));

    // The sidebar's 264, the page's 16, the well's 12.
    assert_close(first.x, 264.0 + 16.0 + 12.0);
    assert_close(first.width, 720.0 - 264.0 - 2.0 * (16.0 + 12.0));
}

#[test]
fn full_screen_the_page_takes_the_large_padding() {
    let first = first_row_at(iced::Size::new(1920.0, 1080.0));

    assert_close(first.x, 264.0 + 32.0 + 12.0);
}
