use crate::Align;
use crate::Axis;
use crate::Fixed;
use crate::Frame;
use crate::Node;
use crate::Rect;
use crate::Role;
use crate::Rule;
use crate::Rules;
use crate::Size;
use crate::check;

fn rules() -> Rules {
    Rules {
        grid: 4.0,
        spacing: vec![4.0, 8.0, 12.0, 16.0, 20.0, 24.0, 32.0, 40.0, 48.0, 64.0],
        container: vec![(0.0, 16.0), (640.0, 24.0), (1024.0, 32.0)],
    }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect { x, y, width, height }
}

fn frame(mode: &str, scale: f32, root: Node) -> Frame {
    Frame {
        page: "test".to_owned(),
        mode: mode.to_owned(),
        scale,
        root,
    }
}

fn with(mut node: Node, children: Vec<Node>) -> Node {
    node.children = children;
    node
}

fn stack(gap: f32, align: Align, bounds: Rect, children: Vec<Node>) -> Node {
    with(
        Node::new(
            Role::Layout {
                axis: Axis::Y,
                gap,
                align,
            },
            bounds,
        ),
        children,
    )
}

fn button(variant: &str, bounds: Rect) -> Node {
    Node::new(
        Role::Component {
            name: "Button".to_owned(),
            variant: variant.to_owned(),
            fixed: Fixed::HEIGHT,
        },
        bounds,
    )
}

fn text(content: &str, natural: Size, bounds: Rect) -> Node {
    Node::new(
        Role::Text {
            content: content.to_owned(),
            natural,
        },
        bounds,
    )
}

fn icon(bounds: Rect) -> Node {
    Node::new(
        Role::Icon {
            name: "search".to_owned(),
        },
        bounds,
    )
}

fn rules_broken(frames: &[Frame]) -> Vec<Rule> {
    check(frames, &rules()).into_iter().map(|f| f.rule).collect()
}

#[test]
fn a_page_on_the_grid_passes_every_rule() {
    let root = stack(
        8.0,
        Align::Start,
        rect(0.0, 0.0, 200.0, 88.0),
        vec![
            button("Filled Md", rect(0.0, 0.0, 120.0, 40.0)),
            button("Filled Md", rect(0.0, 48.0, 96.0, 40.0)),
        ],
    );

    let findings = check(
        &[frame("dark", 1.25, root.clone()), frame("light", 1.25, root)],
        &rules(),
    );

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn an_edge_between_device_pixels_is_off_the_device_grid() {
    let root = Node::new(Role::Box, rect(0.0, 0.0, 100.5, 40.0));

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::DeviceGrid]);
}

#[test]
fn a_whole_device_pixel_at_one_and_a_quarter_is_on_the_device_grid() {
    let root = Node::new(Role::Box, rect(0.8, 0.0, 100.0, 40.0));

    assert_eq!(rules_broken(&[frame("dark", 1.25, root)]), [] as [Rule; 0]);
}

#[test]
fn a_gap_off_the_spacing_scale_is_reported() {
    let root = stack(6.0, Align::Start, rect(0.0, 0.0, 100.0, 40.0), Vec::new());

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::ScaleGrid]);
}

#[test]
fn a_fixed_height_off_the_four_pixel_grid_is_reported() {
    let root = button("Filled Md", rect(0.0, 0.0, 100.0, 38.0));

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::ScaleGrid]);
}

#[test]
fn siblings_further_apart_than_the_gap_are_reported() {
    let root = stack(
        8.0,
        Align::Start,
        rect(0.0, 0.0, 200.0, 92.0),
        vec![
            button("Filled Md", rect(0.0, 0.0, 120.0, 40.0)),
            button("Filled Md", rect(0.0, 52.0, 96.0, 40.0)),
        ],
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::SiblingGap]);
}

#[test]
fn a_child_indented_in_a_start_aligned_stack_is_out_of_line() {
    let root = stack(
        8.0,
        Align::Start,
        rect(0.0, 0.0, 200.0, 88.0),
        vec![
            button("Filled Md", rect(0.0, 0.0, 120.0, 40.0)),
            button("Filled Md", rect(4.0, 48.0, 96.0, 40.0)),
        ],
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::Alignment]);
}

#[test]
fn grid_cells_out_of_their_column_are_reported() {
    let cell = |x: f32, y: f32| Node::new(Role::Box, rect(x, y, 40.0, 40.0));
    let root = with(
        Node::new(
            Role::Grid {
                columns: 2,
                gap: 8.0,
                align: Align::Start,
            },
            rect(0.0, 0.0, 96.0, 88.0),
        ),
        vec![cell(0.0, 0.0), cell(48.0, 0.0), cell(0.0, 48.0), cell(52.0, 48.0)],
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::Alignment]);
}

#[test]
fn two_buttons_of_one_variant_at_different_heights_are_not_identical() {
    let root = stack(
        8.0,
        Align::Start,
        rect(0.0, 0.0, 200.0, 92.0),
        vec![
            button("Filled Md", rect(0.0, 0.0, 120.0, 40.0)),
            button("Filled Md", rect(0.0, 48.0, 96.0, 44.0)),
        ],
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::Identical]);
}

#[test]
fn cards_holding_different_content_are_still_identical() {
    assert_eq!(two_holding("Card", Fixed::HOLDS), [] as [Rule; 0]);
}

#[test]
fn two_toasts_with_their_text_set_differently_are_not_identical() {
    assert_eq!(two_holding("Toast", Fixed::NONE), [Rule::Identical]);
}

/// The rules broken by two `name`s fixing `fixed`, one with a 24 tall line in it and one with 16.
fn two_holding(name: &str, fixed: Fixed) -> Vec<Rule> {
    let holder = |y: f32, content: Node| {
        with(
            Node::new(
                Role::Component {
                    name: name.to_owned(),
                    variant: String::new(),
                    fixed,
                },
                rect(0.0, y, 200.0, 60.0),
            ),
            vec![content],
        )
    };
    let line = |height: f32, y: f32| text("Disk", Size { width: 40.0, height }, rect(16.0, y, 40.0, height));
    let (big, small) = (line(24.0, 16.0), line(16.0, 84.0));
    let root = stack(
        8.0,
        Align::Start,
        rect(0.0, 0.0, 200.0, 128.0),
        vec![holder(0.0, big), holder(68.0, small)],
    );
    rules_broken(&[frame("dark", 1.0, root)])
}

#[test]
fn a_label_lower_in_one_button_than_another_is_not_identical() {
    let labelled = |y: f32, label_y: f32| {
        with(
            button("Filled Md", rect(0.0, y, 120.0, 40.0)),
            vec![text(
                "Go",
                Size {
                    width: 20.0,
                    height: 20.0,
                },
                rect(24.0, y + label_y, 20.0, 20.0),
            )],
        )
    };
    let root = stack(
        8.0,
        Align::Start,
        rect(0.0, 0.0, 200.0, 88.0),
        vec![labelled(0.0, 10.0), labelled(48.0, 12.0)],
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::Identical]);
}

#[test]
fn text_wider_than_its_box_is_clipped() {
    let root = text(
        "Install",
        Size {
            width: 52.3,
            height: 20.0,
        },
        rect(0.0, 0.0, 48.0, 20.0),
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::ClippedText]);
}

#[test]
fn text_sticking_out_of_its_parent_is_clipped() {
    let natural = Size {
        width: 60.0,
        height: 20.0,
    };
    let root = with(
        Node::new(Role::Box, rect(0.0, 0.0, 48.0, 20.0)),
        vec![text("Install", natural, rect(0.0, 0.0, 60.0, 20.0))],
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::ClippedText]);
}

#[test]
fn text_scrolled_past_its_scroll_is_not_clipped_but_past_a_box_inside_it_is() {
    let natural = Size {
        width: 60.0,
        height: 20.0,
    };
    let line = |y: f32| text("Install", natural, rect(0.0, y, 60.0, 20.0));
    let scrolled = with(
        Node::new(Role::Scroll { reach: None }, rect(0.0, 0.0, 100.0, 40.0)),
        vec![with(
            Node::new(Role::Box, rect(0.0, 0.0, 100.0, 80.0)),
            vec![line(0.0), line(60.0)],
        )],
    );
    let boxed = with(
        Node::new(Role::Scroll { reach: None }, rect(0.0, 0.0, 100.0, 40.0)),
        vec![with(Node::new(Role::Box, rect(0.0, 0.0, 48.0, 80.0)), vec![line(0.0)])],
    );
    let too_wide = with(
        Node::new(Role::Scroll { reach: None }, rect(0.0, 0.0, 40.0, 40.0)),
        vec![with(Node::new(Role::Box, rect(0.0, 0.0, 100.0, 80.0)), vec![line(0.0)])],
    );

    assert_eq!(rules_broken(&[frame("dark", 1.0, scrolled)]), [] as [Rule; 0]);
    assert_eq!(rules_broken(&[frame("dark", 1.0, boxed)]), [Rule::ClippedText]);
    assert_eq!(rules_broken(&[frame("dark", 1.0, too_wide)]), [Rule::ClippedText]);
}

#[test]
fn an_icon_a_pixel_off_its_slots_centre_is_reported_and_half_a_pixel_is_not() {
    let slot = |width: f32, icon_x: f32| {
        with(
            Node::new(Role::Box, rect(0.0, 0.0, width, 40.0)),
            vec![icon(rect(icon_x, 10.0, 20.0, 20.0))],
        )
    };

    assert_eq!(
        rules_broken(&[frame("dark", 1.0, slot(40.0, 11.0))]),
        [Rule::IconCentre]
    );
    assert_eq!(rules_broken(&[frame("dark", 1.0, slot(41.0, 10.0))]), [] as [Rule; 0]);
}

#[test]
fn light_and_dark_frames_with_different_boxes_are_reported() {
    let dark = button("Filled Md", rect(0.0, 0.0, 120.0, 40.0));
    let light = button("Filled Md", rect(0.0, 0.0, 124.0, 40.0));

    assert_eq!(
        rules_broken(&[frame("dark", 1.0, dark), frame("light", 1.0, light)]),
        [Rule::ModeInvariant]
    );
}

/// A `Container` `width` wide holding one child inset by `left` and `top`, `content_width` wide.
fn container(width: f32, left: f32, top: f32, content_width: f32) -> Node {
    let child = stack(0.0, Align::Start, rect(left, top, content_width, 40.0), vec![]);
    with(
        Node::new(
            Role::Component {
                name: "Container".to_owned(),
                variant: String::new(),
                fixed: Fixed::HOLDS,
            },
            rect(0.0, 0.0, width, 40.0 + 2.0 * top),
        ),
        vec![child],
    )
}

#[test]
fn a_container_padded_by_its_widths_step_passes() {
    for (width, padding) in [(384.0, 16.0), (800.0, 24.0), (1200.0, 32.0)] {
        let root = container(width, padding, padding, width - 2.0 * padding);

        assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [] as [Rule; 0], "{width}");
    }
}

#[test]
fn a_container_padded_more_than_its_step_is_reported() {
    let root = container(384.0, 24.0, 16.0, 384.0 - 48.0);

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::ContainerPadding]);
}

#[test]
fn content_running_into_a_containers_far_padding_is_reported() {
    let root = container(384.0, 16.0, 16.0, 384.0 - 16.0);

    assert_eq!(rules_broken(&[frame("dark", 1.0, root)]), [Rule::ContainerPadding]);
}

#[test]
fn a_frame_reads_back_from_its_json() {
    let root = stack(
        8.0,
        Align::Centre,
        rect(0.0, 0.0, 200.0, 40.0),
        vec![button("Filled Md", rect(0.0, 0.0, 120.0, 40.0))],
    );
    let written = frame("dark", 1.5, root);

    let json = serde_json::to_string(&written).unwrap();
    let read: Frame = serde_json::from_str(&json).unwrap();

    assert_eq!(read, written);
    assert!(json.contains(r#""role":"layout""#), "{json}");
}

#[test]
fn a_frame_reads_its_marks_back_and_leaves_them_out_when_unset() {
    let mut picture = Node::new(Role::Box, rect(0.0, 0.0, 400.0, 200.0));
    picture.marks.picture = Some(0.25);
    let written = frame(
        "dark",
        1.0,
        with(Node::new(Role::Box, rect(0.0, 0.0, 100.0, 50.0)), vec![picture]),
    );

    let json = serde_json::to_string(&written).unwrap();
    let read: Frame = serde_json::from_str(&json).unwrap();

    assert_eq!(read, written);
    assert_eq!(json.matches("picture").count(), 1, "{json}");
}

/// A room 100×50 holding a picture of a 400×200 layout drawn at a quarter, with `inside` in it.
fn room_with_picture(picture: Option<f32>, inside: Vec<Node>) -> Frame {
    let mut drawn = with(Node::new(Role::Box, rect(0.0, 0.0, 400.0, 200.0)), inside);
    drawn.marks.picture = picture;
    frame(
        "dark",
        1.0,
        with(Node::new(Role::Box, rect(0.0, 0.0, 100.0, 50.0)), vec![drawn]),
    )
}

#[test]
fn a_pictures_text_is_held_to_the_picture_not_the_room_it_is_drawn_in() {
    let natural = Size {
        width: 60.0,
        height: 20.0,
    };
    let words = || text("Unlock", natural, rect(300.0, 160.0, 60.0, 20.0));

    let as_picture = rules_broken(&[room_with_picture(Some(0.25), vec![words()])]);
    let as_layout = rules_broken(&[room_with_picture(None, vec![words()])]);

    assert_eq!(as_picture, [] as [Rule; 0]);
    assert_eq!(as_layout, [Rule::ClippedText]);
}

#[test]
fn a_pictures_own_layout_is_still_held_to_every_rule() {
    let natural = Size {
        width: 60.0,
        height: 20.0,
    };
    let cut = with(
        Node::new(Role::Box, rect(300.0, 160.0, 40.0, 20.0)),
        vec![text("Unlock", natural, rect(300.0, 160.0, 60.0, 20.0))],
    );
    let off_grid = with(
        Node::new(Role::Box, rect(8.0, 8.0, 40.0, 20.0)),
        vec![icon(rect(10.5, 10.0, 16.0, 16.0)), icon(rect(30.0, 10.0, 16.0, 16.0))],
    );

    assert_eq!(
        rules_broken(&[room_with_picture(Some(0.25), vec![cut])]),
        [Rule::ClippedText]
    );
    assert_eq!(
        rules_broken(&[room_with_picture(Some(0.25), vec![off_grid])]),
        [Rule::DeviceGrid]
    );
}

/// A row of two 40 px buttons 8 apart, and a third box placed `over` it or not, at -56.
fn row_with_highlight(over: bool) -> Frame {
    let mut highlight = Node::new(Role::Box, rect(0.0, 0.0, 88.0, 40.0));
    highlight.marks.over = over;
    let row = with(
        Node::new(
            Role::Layout {
                axis: Axis::X,
                gap: 8.0,
                align: Align::Start,
            },
            rect(0.0, 0.0, 88.0, 40.0),
        ),
        vec![
            highlight,
            button("Filled Md", rect(0.0, 0.0, 40.0, 40.0)),
            button("Filled Md", rect(48.0, 0.0, 40.0, 40.0)),
        ],
    );
    frame("dark", 1.0, row)
}

#[test]
fn a_box_placed_over_a_layout_is_not_one_of_its_children() {
    assert_eq!(rules_broken(&[row_with_highlight(true)]), [] as [Rule; 0]);
    assert_eq!(rules_broken(&[row_with_highlight(false)]), [Rule::SiblingGap]);
}

/// A 40 px control holding a bubble 8 under it, the bubble `floats` or not, its words `width` wide
/// in a 60 px box.
fn control_with_bubble(floats: bool, width: f32) -> Frame {
    let natural = Size { width, height: 16.0 };
    let mut bubble = with(
        Node::new(Role::Box, rect(-10.0, 48.0, 60.0, 24.0)),
        vec![text("Search", natural, rect(-6.0, 52.0, width, 16.0))],
    );
    bubble.marks.over = true;
    bubble.marks.floats = floats;
    let control = with(Node::new(Role::Box, rect(100.0, 0.0, 40.0, 40.0)), vec![bubble]);
    frame(
        "dark",
        1.0,
        with(Node::new(Role::Box, rect(0.0, 0.0, 400.0, 200.0)), vec![control]),
    )
}

#[test]
fn a_floating_bubble_is_held_to_its_own_box_not_its_controls() {
    assert_eq!(rules_broken(&[control_with_bubble(true, 52.0)]), [] as [Rule; 0]);
    assert_eq!(rules_broken(&[control_with_bubble(false, 52.0)]), [Rule::ClippedText]);
    assert_eq!(rules_broken(&[control_with_bubble(true, 72.0)]), [Rule::ClippedText]);
}

/// A 200 px scroll holding a 400 px screen with words at `x`, the scroll sideways or not.
fn wide_screen(across: bool, x: f32) -> Frame {
    let natural = Size {
        width: 40.0,
        height: 16.0,
    };
    let screen = with(
        Node::new(Role::Box, rect(0.0, 0.0, 400.0, 40.0)),
        vec![text("61°", natural, rect(x, 0.0, 40.0, 16.0))],
    );
    let mut pane = with(
        Node::new(Role::Scroll { reach: None }, rect(0.0, 0.0, 200.0, 40.0)),
        vec![screen],
    );
    pane.marks.across = across;
    frame("dark", 1.0, pane)
}

#[test]
fn content_past_a_sideways_scroll_is_scrolled_but_past_the_screen_it_is_on_is_clipped() {
    assert_eq!(rules_broken(&[wide_screen(true, 300.0)]), [] as [Rule; 0]);
    assert_eq!(rules_broken(&[wide_screen(false, 300.0)]), [Rule::ClippedText]);
    assert_eq!(rules_broken(&[wide_screen(true, 380.0)]), [Rule::ClippedText]);
}

#[test]
fn a_frame_with_words_the_checker_doesnt_read_still_reads() {
    let json = r#"{"page":"p","mode":"dark","scale":1,"root":{"role":"box","bounds":{"x":0,"y":0,"width":10,"height":10},
        "children":[{"role":"box","unseen":"opacity","bounds":{"x":0,"y":0,"width":4,"height":4}}]}}"#;

    let read: Frame = serde_json::from_str(json).unwrap();

    assert_eq!(read.root.children.len(), 1);
}

/// A 100 px row holding a text that needs `need` across, marked variable or not.
fn cut_words(variable: bool, need: Size) -> Frame {
    let mut words = text("Kuba's Sony WH-1000XM4", need, rect(0.0, 0.0, 100.0, 20.0));
    words.marks.variable = variable;
    frame(
        "dark",
        1.0,
        with(Node::new(Role::Box, rect(0.0, 0.0, 100.0, 20.0)), vec![words]),
    )
}

#[test]
fn words_that_vary_may_be_cut_with_an_ellipsis_and_fixed_words_may_not() {
    let wide = Size {
        width: 180.0,
        height: 20.0,
    };
    let tall = Size {
        width: 80.0,
        height: 40.0,
    };

    assert_eq!(rules_broken(&[cut_words(true, wide)]), [] as [Rule; 0]);
    assert_eq!(rules_broken(&[cut_words(false, wide)]), [Rule::ClippedText]);
    assert_eq!(rules_broken(&[cut_words(true, tall)]), [Rule::ClippedText]);
}

/// A 32×16 icon with a count badge 8 above it, the badge declaring its overhang or not.
fn badged(overhangs: bool) -> Frame {
    let natural = Size {
        width: 8.0,
        height: 16.0,
    };
    let mut badge = with(
        Node::new(
            Role::Component {
                name: "Badge".to_owned(),
                variant: "count".to_owned(),
                fixed: Fixed::HEIGHT,
            },
            rect(20.0, 0.0, 16.0, 16.0),
        ),
        vec![text("3", natural, rect(24.0, 0.0, 8.0, 16.0))],
    );
    badge.marks.overhangs = overhangs;
    badge.marks.over = true;
    let badged = with(
        Node::new(
            Role::Component {
                name: "BadgedIcon".to_owned(),
                variant: "count".to_owned(),
                fixed: Fixed::NONE,
            },
            rect(0.0, 8.0, 32.0, 16.0),
        ),
        vec![badge],
    );
    frame(
        "dark",
        1.0,
        with(Node::new(Role::Box, rect(0.0, 0.0, 64.0, 32.0)), vec![badged]),
    )
}

#[test]
fn a_badge_that_declares_its_overhang_may_hang_past_its_icon() {
    assert_eq!(rules_broken(&[badged(true)]), [] as [Rule; 0]);
    assert_eq!(rules_broken(&[badged(false)]), [Rule::ClippedText]);
}

/// A page's frame in `mode`, with an extra column declared drawn in `only` or undeclared.
fn with_night_column(mode: &str, column: bool, only: Option<&str>) -> Frame {
    let mut children = vec![button("Filled Md", rect(0.0, 0.0, 120.0, 40.0))];
    if column {
        let mut extra = button("Filled Md", rect(0.0, 48.0, 120.0, 40.0));
        extra.marks.only = only.map(str::to_owned);
        children.push(extra);
    }
    frame(
        mode,
        1.0,
        with(Node::new(Role::Box, rect(0.0, 0.0, 200.0, 100.0)), children),
    )
}

#[test]
fn a_sample_declared_for_one_mode_may_be_missing_from_the_other() {
    let declared = [
        with_night_column("dark", true, Some("dark")),
        with_night_column("light", false, None),
    ];
    let undeclared = [
        with_night_column("dark", true, None),
        with_night_column("light", false, None),
    ];
    let stale = [
        with_night_column("dark", true, Some("dark")),
        with_night_column("light", true, Some("dark")),
    ];

    assert_eq!(rules_broken(&declared), [] as [Rule; 0]);
    assert_eq!(rules_broken(&undeclared), [Rule::ModeInvariant]);
    assert_eq!(rules_broken(&stale), [Rule::ModeInvariant]);
}
