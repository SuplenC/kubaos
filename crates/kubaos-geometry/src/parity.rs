//! Parity across toolkits: the same page laid out by two toolkits must be the same boxes.
//!
//! `kubaos-ui` and `KubaOS.Components` implement one spec, so a page drawn with each is held to
//! one frame: every node that says what it is, in order, with the same role, name, variant, words
//! and symbol, and the same box to a hundredth of a device pixel. A text's natural size counts
//! too, to a hundredth of a logical pixel, since a fraction there is a device pixel somewhere
//! else. Plain boxes are left out: each toolkit wraps content in containers of its own, and they
//! claim nothing.
//!
//! A page that shows components on a layout of its own (a mockup's) is held to a toolkit's pages
//! component by component instead: each of its components to a twin of the same name, variant and
//! words, wherever each page puts it ([`components`]).
//!
//! # Owns
//!
//! - [`compare`], [`components`] and their [`Difference`].
//!
//! # Not here
//!
//! - Laying either toolkit's page out: `kubaos-specimen` and the QML specimen in `qml/specimen/`.

use std::fmt;

use crate::Frame;
use crate::Node;
use crate::Rect;
use crate::Role;

/// The largest difference in a box, in device pixels, that still counts as none.
const EPSILON: f32 = 1e-2;

/// The largest difference in a text's natural size, in logical pixels, that still counts as none:
/// two toolkits sum a line's advances in their own precision.
const NATURAL_EPSILON: f32 = 1e-2;

/// How many differences of one frame are listed; the count says how many there are.
pub const SHOWN: usize = 10;

/// A node where two frames of one page part ways.
#[derive(Debug, Clone, PartialEq)]
pub struct Difference {
    /// The reference frame's name ([`Frame::name`]).
    pub frame: String,
    /// Where the node is in the reference frame: its ancestors' roles, outermost first.
    pub path: String,
    pub message: String,
}

impl fmt::Display for Difference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} parity: {}\n    at {}", self.frame, self.message, self.path)
    }
}

/// A node as parity compares it, with where it sits.
struct Flat<'a> {
    node: &'a Node,
    depth: usize,
    path: String,
}

/// Every node of `node` but the plain boxes and the scrolls held to their box (each toolkit scrolls
/// a list in its own), in order, each with its depth among the kept ones. A page's scroll is kept,
/// with how far it reaches.
fn flatten<'a>(node: &'a Node, depth: usize, path: &str, into: &mut Vec<Flat<'a>>) {
    if matches!(node.role, Role::Box | Role::Scroll { reach: None }) {
        for child in &node.children {
            flatten(child, depth, path, into);
        }
        return;
    }
    let here = if path.is_empty() {
        node.role.label()
    } else {
        format!("{path} > {}", node.role.label())
    };
    into.push(Flat {
        node,
        depth,
        path: here.clone(),
    });
    for child in &node.children {
        flatten(child, depth + 1, &here, into);
    }
}

/// Every place `other` differs from `reference`, the same page from another toolkit: a node of
/// another role, name, variant, words or symbol, a box moved or resized, a text whose natural size
/// differs, or nodes one has and the other hasn't. `names` labels the two toolkits in messages:
/// `("iced", "QML")`.
pub fn compare(reference: &Frame, other: &Frame, names: (&str, &str)) -> Vec<Difference> {
    let frame = reference.name();
    let scale = reference.scale;
    let (mut ours, mut theirs) = (Vec::new(), Vec::new());
    flatten(&reference.root, 0, "", &mut ours);
    flatten(&other.root, 0, "", &mut theirs);
    let mut found = Vec::new();
    if (reference.scale - other.scale).abs() > f32::EPSILON || reference.mode != other.mode {
        found.push(Difference {
            frame: frame.clone(),
            path: reference.page.clone(),
            message: format!("{} is compared with {}", frame, other.name()),
        });
        return found;
    }
    for (a, b) in ours.iter().zip(&theirs) {
        let mut parts = Vec::new();
        if a.depth != b.depth || a.node.role.label() != b.node.role.label() || !same_role(&a.node.role, &b.node.role) {
            parts.push(format!(
                "{} is {} in {} and {} in {}",
                a.node.role.label(),
                describe(&b.node.role),
                names.1,
                describe(&a.node.role),
                names.0
            ));
        } else if !same_box(a.node.bounds, b.node.bounds, scale) {
            parts.push(format!(
                "{} is {} in {} and {} in {}",
                a.node.role.label(),
                b.node.bounds,
                names.1,
                a.node.bounds,
                names.0
            ));
        } else if let (Role::Text { natural: x, .. }, Role::Text { natural: y, .. }) = (&a.node.role, &b.node.role)
            && ((x.width - y.width).abs() > NATURAL_EPSILON || (x.height - y.height).abs() > NATURAL_EPSILON)
        {
            parts.push(format!(
                "{} needs {}×{} in {} and {}×{} in {}",
                a.node.role.label(),
                y.width,
                y.height,
                names.1,
                x.width,
                x.height,
                names.0
            ));
        }
        found.extend(parts.into_iter().map(|message| Difference {
            frame: frame.clone(),
            path: a.path.clone(),
            message,
        }));
    }
    if ours.len() != theirs.len() {
        found.push(Difference {
            frame: frame.clone(),
            path: reference.page.clone(),
            message: format!(
                "{} has {} nodes and {} has {}",
                names.0,
                ours.len(),
                names.1,
                theirs.len()
            ),
        });
    }
    found
}

/// A role as a difference names it: its label, a layout's or grid's gap and alignment, and how far
/// a page's scroll reaches.
fn describe(role: &Role) -> String {
    match role {
        Role::Layout { gap, align, .. } => format!("{} gap {gap} {align:?}", role.label()),
        Role::Grid { columns, gap, align } => format!("{} of {columns} gap {gap} {align:?}", role.label()),
        Role::Component { fixed, .. } => format!("{} fixing {fixed:?}", role.label()),
        Role::Scroll { reach: Some(reach) } => format!(
            "{} reaching {} above, {} below and {} left",
            role.label(),
            reach.above,
            reach.below,
            reach.left
        ),
        _ => role.label(),
    }
}

/// Whether two roles are the same thing, but for a text's natural size, which is compared on its
/// own.
fn same_role(a: &Role, b: &Role) -> bool {
    match (a, b) {
        (Role::Text { content: x, .. }, Role::Text { content: y, .. }) => x == y,
        (
            Role::Layout {
                axis: ax,
                gap: ag,
                align: aa,
            },
            Role::Layout {
                axis: bx,
                gap: bg,
                align: ba,
            },
        ) => ax == bx && (ag - bg).abs() < f32::EPSILON && aa == ba,
        (
            Role::Grid {
                columns: ac,
                gap: ag,
                align: aa,
            },
            Role::Grid {
                columns: bc,
                gap: bg,
                align: ba,
            },
        ) => ac == bc && (ag - bg).abs() < f32::EPSILON && aa == ba,
        _ => a == b,
    }
}

/// What [`components`] found.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Components {
    /// How many components had a twin to be held to.
    pub compared: usize,
    /// The components with no twin, as `Button(Filled Md) reading "Continue to the disks"`.
    pub unmatched: Vec<String>,
    /// Where a component and its twin part ways.
    pub differences: Vec<Difference>,
}

/// Every component of `other`, outermost only, held to its twin in `references` of the same mode
/// and scale: the first component there with its name, variant and the same parts (texts with
/// their words, icons with their symbols) in the same order. A state (hovered, focused, disabled)
/// is the same variant and never moves a box, so the twin may be in another state. Each is compared
/// as [`compare`] compares a page, its boxes measured from the component's own top left, so where
/// each page puts it doesn't count. `names` labels the two sides in messages: `("iced", "mockup")`.
pub fn components(references: &[Frame], other: &Frame, names: (&str, &str)) -> Components {
    let mut ours = Vec::new();
    outermost(&other.root, &mut ours);
    let mut theirs = Vec::new();
    for reference in references
        .iter()
        .filter(|r| r.mode == other.mode && (r.scale - other.scale).abs() < f32::EPSILON)
    {
        every_component(&reference.root, &mut theirs);
    }
    let mut report = Components::default();
    for component in ours {
        let key = parts(component);
        let Some(twin) = theirs
            .iter()
            .find(|twin| twin.role == component.role && parts(twin) == key)
        else {
            report.unmatched.push(reading(component));
            continue;
        };
        report.compared += 1;
        let alone = |node: &Node| Frame {
            page: format!("{} {}", other.page, reading(component)),
            mode: other.mode.clone(),
            scale: other.scale,
            root: moved(node, node.bounds.x, node.bounds.y),
        };
        report
            .differences
            .extend(compare(&alone(twin), &alone(component), names));
    }
    report
}

/// The outermost components under `node`.
fn outermost<'a>(node: &'a Node, into: &mut Vec<&'a Node>) {
    if matches!(node.role, Role::Component { .. }) {
        into.push(node);
        return;
    }
    for child in &node.children {
        outermost(child, into);
    }
}

/// Every component under `node`, nested ones too.
fn every_component<'a>(node: &'a Node, into: &mut Vec<&'a Node>) {
    if matches!(node.role, Role::Component { .. }) {
        into.push(node);
    }
    for child in &node.children {
        every_component(child, into);
    }
}

/// The labels of what a component holds that says what it is, in order: its texts with their
/// words, its icons with their symbols, its nested components.
fn parts(node: &Node) -> Vec<String> {
    let mut flat = Vec::new();
    flatten(node, 0, "", &mut flat);
    flat.iter()
        .skip(1)
        .filter(|f| {
            matches!(
                f.node.role,
                Role::Text { .. } | Role::Icon { .. } | Role::Component { .. }
            )
        })
        .map(|f| f.node.role.label())
        .collect()
}

/// A component as a finding names it: its label and its words.
fn reading(node: &Node) -> String {
    let mut words = Vec::new();
    texts(node, &mut words);
    format!("{} reading {:?}", node.role.label(), words.join(" "))
}

fn texts<'a>(node: &'a Node, into: &mut Vec<&'a str>) {
    if let Role::Text { content, .. } = &node.role {
        into.push(content);
    }
    for child in &node.children {
        texts(child, into);
    }
}

/// `node` and everything in it moved by `-x`, `-y`.
fn moved(node: &Node, x: f32, y: f32) -> Node {
    Node {
        role: node.role.clone(),
        bounds: Rect {
            x: node.bounds.x - x,
            y: node.bounds.y - y,
            ..node.bounds
        },
        children: node.children.iter().map(|child| moved(child, x, y)).collect(),
    }
}

/// Whether two boxes are the same to a hundredth of a device pixel.
fn same_box(a: Rect, b: Rect, scale: f32) -> bool {
    [(a.x, b.x), (a.y, b.y), (a.width, b.width), (a.height, b.height)]
        .iter()
        .all(|(x, y)| ((x - y) * scale).abs() < EPSILON)
}

#[cfg(test)]
mod tests;
