//! Parity across toolkits: the same page laid out by two toolkits must be the same boxes.
//!
//! `kubaos-ui` and `KubaOS.Components` implement one spec, so a page drawn with each is held to
//! one frame: every node that says what it is, in order, with the same role, name, variant, words
//! and symbol, and the same box to a hundredth of a device pixel. A text's natural size counts
//! too, to a hundredth of a logical pixel, since a fraction there is a device pixel somewhere
//! else. Plain boxes are left out: each toolkit wraps content in containers of its own, and they
//! claim nothing.
//!
//! # Owns
//!
//! - [`compare`] and its [`Difference`].
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

/// Every node of `node` but the plain boxes, in order, each with its depth among the kept ones.
fn flatten<'a>(node: &'a Node, depth: usize, path: &str, into: &mut Vec<Flat<'a>>) {
    if node.role == Role::Box {
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

/// A role as a difference names it: its label, and a layout's or grid's gap and alignment.
fn describe(role: &Role) -> String {
    match role {
        Role::Layout { gap, align, .. } => format!("{} gap {gap} {align:?}", role.label()),
        Role::Grid { columns, gap, align } => format!("{} of {columns} gap {gap} {align:?}", role.label()),
        Role::Component { fixed, .. } => format!("{} fixing {fixed:?}", role.label()),
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

/// Whether two boxes are the same to a hundredth of a device pixel.
fn same_box(a: Rect, b: Rect, scale: f32) -> bool {
    [(a.x, b.x), (a.y, b.y), (a.width, b.width), (a.height, b.height)]
        .iter()
        .all(|(x, y)| ((x - y) * scale).abs() < EPSILON)
}

#[cfg(test)]
mod tests;
