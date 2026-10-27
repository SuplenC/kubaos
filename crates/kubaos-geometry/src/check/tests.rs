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
        Node::new(Role::Scroll, rect(0.0, 0.0, 100.0, 40.0)),
        vec![with(
            Node::new(Role::Box, rect(0.0, 0.0, 100.0, 80.0)),
            vec![line(0.0), line(60.0)],
        )],
    );
    let boxed = with(
        Node::new(Role::Scroll, rect(0.0, 0.0, 100.0, 40.0)),
        vec![with(Node::new(Role::Box, rect(0.0, 0.0, 48.0, 80.0)), vec![line(0.0)])],
    );
    let too_wide = with(
        Node::new(Role::Scroll, rect(0.0, 0.0, 40.0, 40.0)),
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
