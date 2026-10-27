use crate::Fixed;
use crate::Frame;
use crate::Node;
use crate::Rect;
use crate::Role;
use crate::Size;
use crate::parity::compare;

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
