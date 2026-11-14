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
    /// The nearest kept node around it, by its index in the flattened list.
    parent: Option<usize>,
    path: String,
}

/// Every node of `node` but the plain boxes and the scrolls held to their box (each toolkit scrolls
/// a list in its own), in order, each with the nearest kept node around it. A page's scroll is
/// kept, with how far it reaches.
fn flatten<'a>(node: &'a Node, parent: Option<usize>, path: &str, into: &mut Vec<Flat<'a>>) {
    if matches!(node.role, Role::Box | Role::Scroll { reach: None }) {
        for child in &node.children {
            flatten(child, parent, path, into);
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
        parent,
        path: here.clone(),
    });
    let index = into.len() - 1;
    for child in &node.children {
        flatten(child, Some(index), &here, into);
    }
}

/// How two nodes line up when their boxes are compared: the same box; or, inside parts of their
/// own or a component whose size its parent sets, the same place in the part around them, by what
/// that part fixes.
#[derive(Debug, Clone, Copy, Default)]
struct Stretch {
    /// The components' boxes across, the reference's then the other's, when the component doesn't
    /// fix its width and the two differ.
    across: Option<(Span, Span)>,
    /// The same down, for its height.
    down: Option<(Span, Span)>,
}

impl Stretch {
    /// Boxes held to the parts around them, `ours` and `theirs`: their place from those parts'
    /// edges, along both axes.
    fn inside(ours: Rect, theirs: Rect) -> Stretch {
        Stretch {
            across: Some((Span::across(ours), Span::across(theirs))),
            down: Some((Span::down(ours), Span::down(theirs))),
        }
    }
}

/// Every place `other` differs from `reference`, the same page from another toolkit: a node of
/// another role, name, variant, words or symbol, a box moved or resized, a text whose natural size
/// differs, or a node one has and the other hasn't. The two are lined up node by node first, so a
/// node one of them lacks is one difference, where it is, and every node after it is still held
/// to its own counterpart. Each box is held to its place in the part around it, so a part moved is
/// one difference, not one for each box it holds. `names` labels the two toolkits in messages:
/// `("iced", "QML")`.
pub fn compare(reference: &Frame, other: &Frame, names: (&str, &str)) -> Vec<Difference> {
    compare_stretched(reference, other, names, Stretch::default())
}

fn compare_stretched(reference: &Frame, other: &Frame, names: (&str, &str), stretch: Stretch) -> Vec<Difference> {
    let frame = reference.name();
    let scale = reference.scale;
    let (mut ours, mut theirs) = (Vec::new(), Vec::new());
    flatten(&reference.root, None, "", &mut ours);
    flatten(&other.root, None, "", &mut theirs);
    let mut found = Vec::new();
    if (reference.scale - other.scale).abs() > f32::EPSILON || reference.mode != other.mode {
        found.push(Difference {
            frame: frame.clone(),
            path: reference.page.clone(),
            message: format!("{} is compared with {}", frame, other.name()),
        });
        return found;
    }
    let pairs = align(&ours, &theirs);
    let mut ours_to = vec![None; ours.len()];
    let mut theirs_to = vec![None; theirs.len()];
    for &(a, b) in &pairs {
        ours_to[a] = Some(b);
        theirs_to[b] = Some(a);
    }
    let difference = |path: &str, message: String| Difference {
        frame: frame.clone(),
        path: path.to_owned(),
        message,
    };
    for (index, flat) in ours.iter().enumerate() {
        if ours_to[index].is_none() && flat.parent.is_none_or(|p| ours_to[p].is_some()) {
            let message = format!("{} is in {} and not in {}", flat.node.role.label(), names.0, names.1);
            found.push(difference(&flat.path, message));
        }
    }
    for (index, flat) in theirs.iter().enumerate() {
        if theirs_to[index].is_none() && flat.parent.is_none_or(|p| theirs_to[p].is_some()) {
            let message = format!("{} is in {} and not in {}", flat.node.role.label(), names.1, names.0);
            found.push(difference(&flat.path, message));
        }
    }
    for &(i, j) in &pairs {
        let (a, b) = (&ours[i], &theirs[j]);
        let label = a.node.role.label();
        let around = matched_around(&ours, &ours_to, i);
        // A box is held to the part around it, so a part moved or resized is one difference, where
        // it is, and what it holds is held to it wherever it went.
        let within = around
            .and_then(|p| ours_to[p].map(|q| Stretch::inside(ours[p].node.bounds, theirs[q].node.bounds)))
            .unwrap_or(stretch);
        let message = if around != matched_around(&theirs, &theirs_to, j).and_then(|p| theirs_to[p]) {
            Some(format!(
                "{label} sits in another part in {} than in {}",
                names.1, names.0
            ))
        } else if !same_role(&a.node.role, &b.node.role) {
            Some(format!(
                "{label} is {} in {} and {} in {}",
                describe(&b.node.role),
                names.1,
                describe(&a.node.role),
                names.0
            ))
        } else if !same_box(a.node.bounds, b.node.bounds, scale, within) {
            Some(format!(
                "{label} is {} in {} and {} in {}",
                b.node.bounds, names.1, a.node.bounds, names.0
            ))
        } else if let (Role::Text { natural: x, .. }, Role::Text { natural: y, .. }) = (&a.node.role, &b.node.role)
            && ((x.width - y.width).abs() > NATURAL_EPSILON || (x.height - y.height).abs() > NATURAL_EPSILON)
        {
            Some(format!(
                "{label} needs {}×{} in {} and {}×{} in {}",
                y.width, y.height, names.1, x.width, x.height, names.0
            ))
        } else {
            None
        };
        found.extend(message.map(|message| difference(&a.path, message)));
    }
    found
}

/// The nearest node around `index` that has a counterpart on the other side.
fn matched_around(flat: &[Flat<'_>], to: &[Option<usize>], index: usize) -> Option<usize> {
    let mut at = flat[index].parent;
    while let Some(parent) = at {
        if to[parent].is_some() {
            return Some(parent);
        }
        at = flat[parent].parent;
    }
    None
}

/// The most cells the alignment's table may have; past it, the middle where two frames differ is
/// lined up in order, as two pages too unlike to align node by node.
const ALIGNED_CELLS: usize = 16_000_000;

/// The pairs of nodes, one from each side, that are the same node: the run in order whose labels
/// agree (role, name, variant, words, symbol) that pairs the most, a pair of the same role in
/// every detail (a layout's gap, say) counting over one whose label alone agrees. The runs both
/// frames start and end with alike are taken as they are.
fn align(ours: &[Flat<'_>], theirs: &[Flat<'_>]) -> Vec<(usize, usize)> {
    let a: Vec<String> = ours.iter().map(|f| f.node.role.label()).collect();
    let b: Vec<String> = theirs.iter().map(|f| f.node.role.label()).collect();
    let alike = |i: usize, j: usize| a[i] == b[j] && same_role(&ours[i].node.role, &theirs[j].node.role);
    let head = (0..a.len().min(b.len())).take_while(|&i| alike(i, i)).count();
    let tail = (0..(a.len() - head).min(b.len() - head))
        .take_while(|&k| alike(a.len() - 1 - k, b.len() - 1 - k))
        .count();
    let (rows, columns) = (a.len() - head - tail, b.len() - head - tail);
    let mut pairs: Vec<(usize, usize)> = (0..head).map(|i| (i, i)).collect();
    let weight = |i: usize, j: usize| match (a[head + i] == b[head + j], alike(head + i, head + j)) {
        (_, true) => 2,
        (true, false) => 1,
        (false, _) => 0,
    };
    if (rows + 1) * (columns + 1) <= ALIGNED_CELLS {
        pairs.extend(
            heaviest_common(rows, columns, weight)
                .into_iter()
                .map(|(i, j)| (head + i, head + j)),
        );
    } else {
        pairs.extend(
            (0..rows.min(columns))
                .filter(|&i| weight(i, i) > 0)
                .map(|i| (head + i, head + i)),
        );
    }
    pairs.extend((0..tail).map(|k| (a.len() - tail + k, b.len() - tail + k)));
    pairs
}

/// The pairs, in order, of a common subsequence of two sequences `rows` and `columns` long whose
/// pairs weigh the most, `weight` saying what pairing two items weighs (0 where they can't pair).
fn heaviest_common(rows: usize, columns: usize, weight: impl Fn(usize, usize) -> u32) -> Vec<(usize, usize)> {
    let width = columns + 1;
    let mut table = vec![0_u32; (rows + 1) * width];
    for i in (0..rows).rev() {
        for j in (0..columns).rev() {
            let skip = table[(i + 1) * width + j].max(table[i * width + j + 1]);
            let w = weight(i, j);
            table[i * width + j] = if w > 0 {
                skip.max(table[(i + 1) * width + j + 1] + w)
            } else {
                skip
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut pairs = Vec::new();
    while i < rows && j < columns {
        let w = weight(i, j);
        if w > 0 && table[i * width + j] == table[(i + 1) * width + j + 1] + w {
            pairs.push((i, j));
            i += 1;
            j += 1;
        } else if table[(i + 1) * width + j] >= table[i * width + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
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
        let mut twins = theirs
            .iter()
            .filter(|twin| twin.role == component.role && parts(twin) == key)
            .peekable();
        let Some(&first) = twins.peek() else {
            report.unmatched.push(reading(component));
            continue;
        };
        // Where the specimen draws it at the same size, that twin; else the first, held only to
        // what the component fixes along the measures its parent sets.
        let twin = twins
            .find(|twin| {
                same_box(
                    moved_rect(twin.bounds),
                    moved_rect(component.bounds),
                    other.scale,
                    Stretch::default(),
                )
            })
            .unwrap_or(first);
        report.compared += 1;
        let alone = |node: &Node| Frame {
            page: format!("{} {}", other.page, reading(component)),
            mode: other.mode.clone(),
            scale: other.scale,
            root: moved(node, node.bounds.x, node.bounds.y),
        };
        let stretch = stretch_of(component, twin);
        report
            .differences
            .extend(compare_stretched(&alone(twin), &alone(component), names, stretch));
    }
    report
}

/// A box moved to the origin, for comparing sizes alone.
fn moved_rect(bounds: Rect) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        ..bounds
    }
}

/// The axes along which `component` and its `twin` are drawn at different sizes that the
/// component doesn't fix, as both are compared, from their own top left.
fn stretch_of(component: &Node, twin: &Node) -> Stretch {
    let Role::Component { fixed, .. } = component.role else {
        return Stretch::default();
    };
    let (a, b) = (twin.bounds, component.bounds);
    let from_origin = |length: f32| Span { start: 0.0, length };
    Stretch {
        across: (!fixed.width && (a.width - b.width).abs() > f32::EPSILON)
            .then_some((from_origin(a.width), from_origin(b.width))),
        down: (!fixed.height && (a.height - b.height).abs() > f32::EPSILON)
            .then_some((from_origin(a.height), from_origin(b.height))),
    }
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
    flatten(node, None, "", &mut flat);
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
        marks: node.marks.clone(),
        children: node.children.iter().map(|child| moved(child, x, y)).collect(),
    }
}

/// Whether two boxes are the same to a hundredth of a device pixel. Along an axis `stretch` names,
/// the two sit in components of different sizes, so a box is the same when it keeps the same
/// place from either edge, both edges, or the middle: what the component itself fixes.
fn same_box(a: Rect, b: Rect, scale: f32, stretch: Stretch) -> bool {
    let same = |x: f32, y: f32| ((x - y) * scale).abs() < EPSILON;
    let along = |ours: Span, theirs: Span, within: Option<(Span, Span)>| {
        let Some((ours_in, theirs_in)) = within else {
            return same(ours.start, theirs.start) && same(ours.length, theirs.length);
        };
        let from_start = same(ours.start - ours_in.start, theirs.start - theirs_in.start);
        let from_end = same(ours_in.end() - ours.end(), theirs_in.end() - theirs.end());
        let length = same(ours.length, theirs.length);
        let middle = same(ours.middle() - ours_in.middle(), theirs.middle() - theirs_in.middle());
        (from_start && (length || from_end)) || (from_end && length) || (middle && length)
    };
    along(Span::across(a), Span::across(b), stretch.across) && along(Span::down(a), Span::down(b), stretch.down)
}

/// A box along one axis.
#[derive(Debug, Clone, Copy)]
struct Span {
    start: f32,
    length: f32,
}

impl Span {
    fn across(r: Rect) -> Span {
        Span {
            start: r.x,
            length: r.width,
        }
    }

    fn down(r: Rect) -> Span {
        Span {
            start: r.y,
            length: r.height,
        }
    }

    fn end(self) -> f32 {
        self.start + self.length
    }

    fn middle(self) -> f32 {
        self.start + self.length / 2.0
    }
}

#[cfg(test)]
mod tests;
