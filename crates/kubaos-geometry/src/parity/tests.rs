use crate::Fixed;
use crate::Frame;
use crate::Node;
use crate::Reach;
use crate::Rect;
use crate::Role;
use crate::Size;
use crate::parity::Ahead;
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
fn a_missing_node_is_one_difference_where_it_is() {
    let iced = frame(button(rect(0.0, 0.0, 101.6, 40.0), 52.956));
    let mut short = button(rect(0.0, 0.0, 101.6, 40.0), 52.956);
    short.children.clear();
    let qml = frame(short);

    let found = compare(&iced, &qml, ("iced", "QML"));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].message, "Text \"Continue\" is in iced and not in QML");
    assert_eq!(found[0].path, "Button(Filled Md) > Text \"Continue\"");
}

/// A row of `count` buttons, the first wrapped in an extra row when `nested`.
fn buttons(count: usize, nested: bool) -> Node {
    let mut row = Node::new(
        Role::Layout {
            axis: crate::Axis::X,
            gap: 8.0,
            align: crate::Align::Start,
        },
        rect(0.0, 0.0, 1000.0, 40.0),
    );
    row.children = (0..count)
        .map(|i| button(rect(i as f32 * 110.0, 0.0, 101.6, 40.0), 52.956))
        .collect();
    if nested {
        let mut extra = Node::new(
            Role::Layout {
                axis: crate::Axis::X,
                gap: 0.0,
                align: crate::Align::Start,
            },
            rect(0.0, 0.0, 101.6, 40.0),
        );
        extra.children = vec![row.children.remove(0)];
        row.children.insert(0, extra);
    }
    row
}

#[test]
fn a_level_one_side_lacks_is_one_difference_not_one_for_every_node_after_it() {
    let iced = frame(buttons(20, true));
    let qml = frame(buttons(20, false));

    let found = compare(&iced, &qml, ("iced", "QML"));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].message, "Row is in iced and not in QML");
}

#[test]
fn a_node_moved_into_another_part_is_a_difference() {
    let iced = frame(buttons(2, false));
    let mut qml = buttons(2, false);
    let label = qml.children[1].children.remove(0);
    qml.children.push(label);

    let found = compare(&iced, &frame(qml), ("iced", "QML"));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        found[0].message,
        "Text \"Continue\" sits in another part in QML than in iced"
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

/// A bar `width` wide, its fill 40 from its start and its value at its end, as a `ProgressBar` the
/// parent sets the width of.
fn bar(x: f32, width: f32, value_from_end: f32) -> Node {
    let mut node = Node::new(
        Role::Component {
            name: "ProgressBar".to_owned(),
            variant: String::new(),
            fixed: Fixed::HEIGHT,
        },
        rect(x, 0.0, width, 24.0),
    );
    let text = |content: &str, bounds: Rect| {
        Node::new(
            Role::Text {
                content: content.to_owned(),
                natural: Size {
                    width: bounds.width,
                    height: 16.0,
                },
            },
            bounds,
        )
    };
    node.children = vec![
        text("Copying", rect(x, 0.0, 52.0, 16.0)),
        text("40%", rect(x + width - value_from_end - 24.0, 0.0, 24.0, 16.0)),
        Node::new(Role::Box, rect(x, 20.0, width, 4.0)),
    ];
    node
}

#[test]
fn a_component_its_parent_sizes_is_held_to_what_it_fixes_at_another_width() {
    let iced = page_of("components", vec![bar(0.0, 584.0, 0.0)]);
    let narrower = page_of("progress-bar", vec![bar(100.0, 560.0, 0.0)]);
    let value_moved = page_of("progress-bar", vec![bar(100.0, 560.0, 4.0)]);

    let at_width = components(std::slice::from_ref(&iced), &narrower, ("iced", "mockup"));
    let moved = components(&[iced], &value_moved, ("iced", "mockup"));

    assert_eq!(at_width.differences, []);
    assert_eq!(moved.differences.len(), 1, "{:?}", moved.differences);
}

#[test]
fn a_twin_drawn_at_the_same_size_is_preferred() {
    let iced = page_of("components", vec![bar(0.0, 584.0, 4.0), bar(0.0, 560.0, 0.0)]);
    let mockup = page_of("progress-bar", vec![bar(100.0, 560.0, 0.0)]);

    let report = components(&[iced], &mockup, ("iced", "mockup"));

    assert_eq!(report.compared, 1);
    assert_eq!(report.differences, []);
}

#[test]
fn a_layout_level_one_side_lacks_pairs_the_level_alike_in_every_detail() {
    let mut iced = buttons(1, true);
    iced.children[0] = {
        let mut inner = buttons(1, false);
        inner.bounds = rect(0.0, 0.0, 101.6, 40.0);
        let mut outer = Node::new(
            Role::Layout {
                axis: crate::Axis::X,
                gap: 0.0,
                align: crate::Align::Start,
            },
            rect(0.0, 0.0, 101.6, 40.0),
        );
        outer.children = vec![inner];
        outer
    };
    let mut qml = buttons(1, false);
    qml.children[0] = {
        let mut inner = buttons(1, false);
        inner.bounds = rect(0.0, 0.0, 101.6, 40.0);
        inner
    };

    let found = compare(&frame(iced), &frame(qml), ("iced", "QML"));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].message, "Row is in iced and not in QML");
}

#[test]
fn a_part_moved_is_one_difference_not_one_for_every_box_it_holds() {
    let iced = frame(buttons(5, false));
    let mut qml = buttons(5, false);
    qml.bounds.x += 8.0;
    for button in &mut qml.children {
        button.bounds.x += 8.0;
        for part in &mut button.children {
            part.bounds.x += 8.0;
        }
    }

    let found = compare(&iced, &frame(qml), ("iced", "QML"));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].path, "Row");
}

#[test]
fn a_box_moved_inside_a_part_that_stayed_is_still_a_difference() {
    let iced = frame(buttons(5, false));
    let mut qml = buttons(5, false);
    qml.children[2].children[0].bounds.x += 0.8;

    let found = compare(&iced, &frame(qml), ("iced", "QML"));

    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].path.ends_with("Text \"Continue\""), "{found:?}");
}

#[test]
fn a_component_ahead_of_its_twin_is_listed_and_not_counted() {
    let iced = page_of("components", vec![button(rect(152.0, 640.0, 101.6, 40.0), 52.956)]);
    let mut ahead = button(rect(0.0, 0.0, 101.6, 40.0), 52.956);
    ahead.children[0].bounds.x += 4.0;
    ahead.marks.ahead = Some("button.md: the label 4 further in".to_owned());
    let mockup = page_of("button", vec![ahead]);

    let report = components(&[iced], &mockup, ("iced", "mockup"));

    assert_eq!(report.differences, []);
    assert_eq!(
        report.ahead,
        [Ahead {
            component: "Button(Filled Md) reading \"Continue\"".to_owned(),
            reason: "button.md: the label 4 further in".to_owned(),
            differences: 1,
        }]
    );
}

#[test]
fn a_component_declared_ahead_that_matches_its_twin_fails() {
    let iced = page_of("components", vec![button(rect(152.0, 640.0, 101.6, 40.0), 52.956)]);
    let mut caught_up = button(rect(0.0, 0.0, 101.6, 40.0), 52.956);
    caught_up.marks.ahead = Some("button.md: the label 4 further in".to_owned());
    let mockup = page_of("button", vec![caught_up]);

    let report = components(&[iced], &mockup, ("iced", "mockup"));

    assert_eq!(report.ahead, []);
    assert_eq!(report.differences.len(), 1);
    assert!(
        report.differences[0].message.contains("the library caught up"),
        "{:?}",
        report.differences
    );
}
