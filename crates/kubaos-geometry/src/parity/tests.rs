use crate::Fixed;
use crate::Frame;
use crate::Node;
use crate::Reach;
use crate::Rect;
use crate::Role;
use crate::Size;
use crate::parity::compare;
use crate::parity::components;

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect { x, y, width, height }
}

fn button(bounds: Rect, label_width: f32) -> Node {
    let mut node = Node::new(
        Role::Component {
            name: "Button".to_owned(),
            variant: "Filled Md".to_owned(),
            fixed: Fixed::HEIGHT,
        },
        bounds,
    );
    node.children = vec![Node::new(
        Role::Text {
            content: "Continue".to_owned(),
            natural: Size {
                width: label_width,
                height: 24.0,
            },
        },
        rect(bounds.x + 24.0, bounds.y + 8.0, 53.0, 24.0),
    )];
    node
}

fn frame(root: Node) -> Frame {
    Frame {
        page: "components".to_owned(),
        mode: "dark".to_owned(),
        scale: 1.25,
        root,
    }
}

fn boxed(node: Node) -> Node {
    let mut wrapper = Node::new(Role::Box, node.bounds);
    wrapper.children = vec![node];
    wrapper
}

#[test]
fn the_same_boxes_with_other_plain_boxes_around_them_are_at_parity() {
    let iced = frame(button(rect(0.0, 0.0, 101.6, 40.0), 52.956));
    let qml = frame(boxed(button(rect(0.0, 0.0, 101.6, 40.0), 52.953)));

    assert_eq!(compare(&iced, &qml, ("iced", "QML")), []);
}

#[test]
fn a_box_a_device_pixel_wider_is_a_difference() {
    let iced = frame(button(rect(0.0, 0.0, 101.6, 40.0), 52.956));
    let qml = frame(button(rect(0.0, 0.0, 102.4, 40.0), 52.956));

    let found = compare(&iced, &qml, ("iced", "QML"));

    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message,
        "Button(Filled Md) is 102.4×40 at (0, 0) in QML and 101.6×40 at (0, 0) in iced"
    );
}

#[test]
fn a_text_that_needs_more_room_is_a_difference_inside_the_same_box() {
    let iced = frame(button(rect(0.0, 0.0, 101.6, 40.0), 52.956));
    let qml = frame(button(rect(0.0, 0.0, 101.6, 40.0), 53.2));

    let found = compare(&iced, &qml, ("iced", "QML"));

    assert_eq!(found.len(), 1);
    assert!(found[0].message.starts_with("Text \"Continue\" needs 53.2×24 in QML"));
    assert_eq!(found[0].path, "Button(Filled Md) > Text \"Continue\"");
}

#[test]
fn a_missing_node_is_counted() {
    let iced = frame(button(rect(0.0, 0.0, 101.6, 40.0), 52.956));
    let mut short = button(rect(0.0, 0.0, 101.6, 40.0), 52.956);
    short.children.clear();
    let qml = frame(short);

    let found = compare(&iced, &qml, ("iced", "QML"));

    assert_eq!(
        found.last().map(|d| d.message.as_str()),
        Some("iced has 2 nodes and QML has 1")
    );
}

const GLOW: Reach = Reach {
    above: 20.0,
    below: 20.0,
    left: 20.0,
};

/// A page's scroll reaching `reach`, a button at its content's top left; in iced, the scroll it
/// reaches with is held to the larger box `held`, around the button.
fn page_scroll(reach: Reach, held: Option<Rect>) -> Node {
    let content = button(rect(20.0, 20.0, 101.6, 40.0), 52.956);
    let inner = match held {
        Some(bounds) => {
            let mut inner = Node::new(Role::Scroll { reach: None }, bounds);
            inner.children = vec![content];
            inner
        }
        None => content,
    };
    let mut scroll = Node::new(Role::Scroll { reach: Some(reach) }, rect(20.0, 20.0, 300.0, 120.0));
    scroll.children = vec![inner];
    scroll
}

#[test]
fn a_pages_scroll_that_reaches_as_far_with_its_content_as_far_in_is_at_parity() {
    let iced = frame(page_scroll(GLOW, Some(rect(0.0, 0.0, 324.0, 160.0))));
    let qml = frame(page_scroll(GLOW, None));

    assert_eq!(compare(&iced, &qml, ("iced", "QML")), []);
}

#[test]
fn a_pages_scroll_that_reaches_less_far_is_a_difference() {
    let iced = frame(page_scroll(GLOW, None));
    let qml = frame(page_scroll(Reach { above: 4.0, ..GLOW }, None));

    let found = compare(&iced, &qml, ("iced", "QML"));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].path, "Scroll");
}

fn page_of(page: &str, children: Vec<Node>) -> Frame {
    let mut root = Node::new(Role::Box, rect(0.0, 0.0, 1280.0, 2000.0));
    root.children = children;
    Frame {
        page: page.to_owned(),
        ..frame(root)
    }
}

#[test]
fn a_component_matches_its_twin_wherever_each_page_puts_it() {
    let iced = page_of("components", vec![button(rect(152.0, 640.0, 101.6, 40.0), 52.956)]);
    let mockup = page_of("button", vec![button(rect(127.2, 96.8, 101.6, 40.0), 52.953)]);

    let report = components(&[iced], &mockup, ("iced", "mockup"));

    assert_eq!(report.compared, 1);
    assert_eq!(report.unmatched, Vec::<String>::new());
    assert_eq!(report.differences, []);
}

#[test]
fn a_component_s_part_moved_inside_it_is_a_difference() {
    let iced = page_of("components", vec![button(rect(152.0, 640.0, 101.6, 40.0), 52.956)]);
    let mut moved = button(rect(0.0, 0.0, 101.6, 40.0), 52.956);
    moved.children[0].bounds.y += 0.8;
    let mockup = page_of("button", vec![moved]);

    let report = components(&[iced], &mockup, ("iced", "mockup"));

    assert_eq!(report.compared, 1);
    assert_eq!(report.differences.len(), 1, "{:?}", report.differences);
    assert!(report.differences[0].message.contains("in mockup"));
}

#[test]
fn a_component_with_no_twin_of_its_name_variant_and_words_is_unmatched() {
    let iced = page_of("components", vec![button(rect(152.0, 640.0, 101.6, 40.0), 52.956)]);
    let mut longer = button(rect(0.0, 0.0, 180.0, 40.0), 120.0);
    if let Role::Text { content, .. } = &mut longer.children[0].role {
        *content = "Continue to the disks".to_owned();
    }
    let mockup = page_of("button", vec![longer]);

    let report = components(&[iced], &mockup, ("iced", "mockup"));

    assert_eq!(report.compared, 0);
    assert_eq!(
        report.unmatched,
        ["Button(Filled Md) reading \"Continue to the disks\""]
    );
}

#[test]
fn only_a_twin_of_the_same_scale_and_mode_counts() {
    let iced = page_of("components", vec![button(rect(152.0, 640.0, 101.6, 40.0), 52.956)]);
    let mut mockup = page_of("button", vec![button(rect(0.0, 0.0, 101.6, 40.0), 52.956)]);
    mockup.scale = 2.0;

    let report = components(&[iced], &mockup, ("iced", "mockup"));

    assert_eq!(report.compared, 0);
    assert_eq!(report.unmatched.len(), 1);
}
