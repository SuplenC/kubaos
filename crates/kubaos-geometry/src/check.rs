//! The rules, applied to frames.
//!
//! Lengths are compared in device pixels, where the rules are about whole pixels. Two lengths are
//! the same when they differ by under [`EPSILON`] of a device pixel: float error on a page some
//! thousands of pixels tall, never a visible difference.

use std::collections::BTreeMap;

use crate::Align;
use crate::Axis;
use crate::Finding;
use crate::Fixed;
use crate::Frame;
use crate::Node;
use crate::Rect;
use crate::Role;
use crate::Rule;
use crate::Rules;

/// The largest difference, in device pixels, that still counts as none.
const EPSILON: f32 = 1e-2;

/// How far off centre an icon may sit, in device pixels: centring an odd number of spare device
/// pixels leaves half of one on a side.
const HALF_PIXEL: f32 = 0.5;

/// Every broken rule in `frames`, frame by frame, then the rules across frames ([`Rule::Identical`]
/// and [`Rule::ModeInvariant`]).
pub fn check(frames: &[Frame], rules: &Rules) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut instances = Vec::new();
    for frame in frames {
        let mut walk = Walk {
            frame,
            rules,
            findings: &mut findings,
            instances: &mut instances,
        };
        walk.node(&frame.root, &mut Vec::new(), &mut Vec::new());
    }
    identical(&instances, &mut findings);
    mode_invariant(frames, &mut findings);
    findings
}

/// One component, for [`Rule::Identical`].
struct Instance {
    frame: String,
    scale: f32,
    path: String,
    name: String,
    variant: String,
    fixed: Fixed,
    bounds: Rect,
    /// Where its icons and label sit inside it, in device pixels.
    parts: Vec<Part>,
}

/// An icon or the first text of a component, placed relative to the component's box.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Part {
    what: &'static str,
    /// From the component's left edge, when the component fixes its width.
    x: Option<i64>,
    /// From the component's top edge, when the component fixes its height.
    y: Option<i64>,
    width: Option<i64>,
    height: i64,
}

struct Walk<'a> {
    frame: &'a Frame,
    rules: &'a Rules,
    findings: &'a mut Vec<Finding>,
    instances: &'a mut Vec<Instance>,
}

/// A box around the node being checked, as a finding names it.
struct Ancestor {
    label: String,
    bounds: Rect,
    /// Whether its content scrolls in it.
    scrolls: bool,
}

impl Walk<'_> {
    fn node(&mut self, node: &Node, path: &mut Vec<String>, ancestors: &mut Vec<Ancestor>) {
        self.device_grid(node, path);
        self.scale_grid(node, path);
        self.fits(node, path, ancestors);
        match &node.role {
            Role::Layout { axis, gap, align } => self.layout(node, *axis, *gap, *align, path),
            Role::Grid { columns, gap, align } => self.grid(node, *columns, *gap, *align, path),
            Role::Component { name, variant, fixed } => self.instances.push(Instance {
                frame: self.frame.name(),
                scale: self.frame.scale,
                path: path_text(path, &node.role),
                name: name.clone(),
                variant: variant.clone(),
                fixed: *fixed,
                bounds: node.bounds,
                parts: parts(node, *fixed, self.frame.scale),
            }),
            _ => {}
        }
        self.icons_centred(node, path);

        let label = node.role.label();
        ancestors.push(Ancestor {
            label: label.clone(),
            bounds: node.bounds,
            scrolls: node.role == Role::Scroll,
        });
        for (index, child) in node.children.iter().enumerate() {
            let segment = !matches!(node.role, Role::Box);
            if segment {
                path.push(format!("{label}[{index}]"));
            }
            self.node(child, path, ancestors);
            if segment {
                path.pop();
            }
        }
        ancestors.pop();
    }

    fn report(&mut self, rule: Rule, path: &[String], at: &Role, message: String) {
        self.findings.push(Finding {
            frame: self.frame.name(),
            rule,
            path: path_text(path, at),
            message,
        });
    }

    fn device(&self, logical: f32) -> f32 {
        logical * self.frame.scale
    }

    fn whole(&self, logical: f32) -> bool {
        let device = self.device(logical);
        (device - device.round()).abs() < EPSILON
    }

    fn same(&self, a: f32, b: f32) -> bool {
        (self.device(a) - self.device(b)).abs() < EPSILON
    }

    fn on_grid(&self, logical: f32) -> bool {
        let units = logical / self.rules.grid;
        (units - units.round()).abs() * self.rules.grid * self.frame.scale < EPSILON
    }

    /// Every edge on a whole device pixel. A text's far edges are where its letters end, which the
    /// box it sits in rounds, so only where it starts counts. A spacer draws nothing, so it has no
    /// edges to land.
    fn device_grid(&mut self, node: &Node, path: &[String]) {
        if matches!(node.role, Role::Spacer { .. }) {
            return;
        }
        let b = node.bounds;
        let mut edges = vec![("left", b.x), ("top", b.y)];
        if !matches!(node.role, Role::Text { .. }) {
            edges.extend([("right", b.right()), ("bottom", b.bottom())]);
        }
        let off: Vec<String> = edges
            .into_iter()
            .filter(|&(_, v)| !self.whole(v))
            .map(|(edge, v)| format!("{edge} {}", self.device(v)))
            .collect();
        if !off.is_empty() {
            let message = format!("{} has edges between device pixels: {}", b, off.join(", "));
            self.report(Rule::DeviceGrid, path, &node.role, message);
        }
    }

    fn scale_grid(&mut self, node: &Node, path: &[String]) {
        let b = node.bounds;
        let grid = self.rules.grid;
        let message = match &node.role {
            Role::Component { fixed, .. } => {
                let mut off = Vec::new();
                if fixed.width && !self.on_grid(b.width) {
                    off.push(format!("width {}", b.width));
                }
                if fixed.height && !self.on_grid(b.height) {
                    off.push(format!("height {}", b.height));
                }
                (!off.is_empty()).then(|| format!("fixed {} isn't a multiple of {grid}", off.join(" and ")))
            }
            Role::Layout { gap, .. } | Role::Grid { gap, .. } => (*gap != 0.0
                && !self.rules.spacing.iter().any(|&s| self.same(s, *gap)))
            .then(|| format!("gap {gap} isn't on the spacing scale {:?}", self.rules.spacing)),
            Role::Icon { .. } if !self.same(b.width, b.height) => {
                Some(format!("icon box {}×{} isn't square", b.width, b.height))
            }
            Role::Icon { .. } if !self.on_grid(b.width) => {
                Some(format!("icon size {} isn't a multiple of {grid}", b.width))
            }
            Role::Spacer { fill: false } if !self.on_grid(b.width) || !self.on_grid(b.height) => {
                Some(format!("gap {}×{} isn't a multiple of {grid}", b.width, b.height))
            }
            _ => None,
        };
        if let Some(message) = message {
            self.report(Rule::ScaleGrid, path, &node.role, message);
        }
    }

    /// Text that fits its box, and text and icons inside every box around them.
    fn fits(&mut self, node: &Node, path: &[String], ancestors: &[Ancestor]) {
        let b = node.bounds;
        if let Role::Text { natural, .. } = &node.role
            && (self.device(natural.width - b.width) >= EPSILON || self.device(natural.height - b.height) >= EPSILON)
        {
            let message = format!(
                "the text needs {}×{} and its box is {}×{}",
                natural.width, natural.height, b.width, b.height
            );
            self.report(Rule::ClippedText, path, &node.role, message);
            return;
        }
        if !matches!(node.role, Role::Text { .. } | Role::Icon { .. }) {
            return;
        }
        // From the nearest scroll out, the content may run above or below the boxes around it:
        // that is scrolling, not clipping. Scrolls only scroll down, so across it still clips.
        let mut scrolled = false;
        let outside = ancestors.iter().rev().find_map(|ancestor| {
            let a = ancestor.bounds;
            let across = self.device(a.x - b.x) >= EPSILON || self.device(b.right() - a.right()) >= EPSILON;
            let down = self.device(a.y - b.y) >= EPSILON || self.device(b.bottom() - a.bottom()) >= EPSILON;
            scrolled |= ancestor.scrolls;
            let clipped = across || (down && !scrolled);
            clipped.then_some((&ancestor.label, a))
        });
        if let Some((label, a)) = outside {
            let message = format!("{b} sticks out of its {label}, {a}");
            self.report(Rule::ClippedText, path, &node.role, message);
        }
    }

    fn layout(&mut self, node: &Node, axis: Axis, gap: f32, align: Align, path: &[String]) {
        for pair in node.children.windows(2) {
            let (prev, next) = (pair[0].bounds, pair[1].bounds);
            let between = next.start(axis) - prev.end(axis);
            if !self.same(between, gap) {
                let message = format!(
                    "{} and the sibling after it are {between} apart; the layout's gap is {gap}",
                    pair[0].role.label()
                );
                self.report(Rule::SiblingGap, path, &node.role, message);
            }
        }
        let cross = match axis {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        };
        let placed: Vec<&Node> = node
            .children
            .iter()
            .filter(|c| !matches!(c.role, Role::Spacer { .. }))
            .collect();
        self.lined_up(&placed, cross, align, path, &node.role, "the layout");
    }

    fn grid(&mut self, node: &Node, columns: u16, gap: f32, align: Align, path: &[String]) {
        let columns = usize::from(columns.max(1));
        let rows: Vec<&[Node]> = node.children.chunks(columns).collect();
        for column in 0..columns {
            let cells: Vec<&Node> = rows.iter().filter_map(|row| row.get(column)).collect();
            self.lined_up(
                &cells,
                Axis::X,
                Align::Start,
                path,
                &node.role,
                &format!("column {column}"),
            );
        }
        for (index, row) in rows.iter().enumerate() {
            let cells: Vec<&Node> = row.iter().collect();
            self.lined_up(&cells, Axis::Y, align, path, &node.role, &format!("row {index}"));
        }
        for (index, pair) in rows.windows(2).enumerate() {
            let bottom = pair[0].iter().map(|c| c.bounds.bottom()).fold(f32::MIN, f32::max);
            let top = pair[1].iter().map(|c| c.bounds.y).fold(f32::MAX, f32::min);
            if !self.same(top - bottom, gap) {
                let message = format!(
                    "rows {index} and {} are {} apart; the grid's gap is {gap}",
                    index + 1,
                    top - bottom
                );
                self.report(Rule::SiblingGap, path, &node.role, message);
            }
        }
    }

    /// `nodes` sharing a start, centre or end on `axis`, as `align` says. Centres may differ by
    /// half a device pixel, where centring on whole pixels had an odd one to share.
    fn lined_up(&mut self, nodes: &[&Node], axis: Axis, align: Align, path: &[String], role: &Role, what: &str) {
        let Some(first) = nodes.first() else {
            return;
        };
        let at = |n: &Node| match align {
            Align::Start => n.bounds.start(axis),
            Align::Centre => n.bounds.centre(axis),
            Align::End => n.bounds.end(axis),
        };
        let slack = match align {
            Align::Centre => HALF_PIXEL + EPSILON,
            Align::Start | Align::End => EPSILON,
        };
        let edge = match (axis, align) {
            (Axis::X, Align::Start) => "left edge",
            (Axis::X | Axis::Y, Align::Centre) => "centre",
            (Axis::X, Align::End) => "right edge",
            (Axis::Y, Align::Start) => "top",
            (Axis::Y, Align::End) => "bottom",
        };
        for (index, node) in nodes.iter().enumerate().skip(1) {
            if self.device((at(node) - at(first)).abs()) > slack {
                let message = format!(
                    "{}, child {index} of {what}, has its {edge} at {}; the first has it at {}",
                    node.role.label(),
                    at(node),
                    at(first)
                );
                self.report(Rule::Alignment, path, role, message);
            }
        }
    }

    /// An icon alone in its box sits in the middle of it both ways; an icon in a centred layout
    /// sits on the layout's centre line.
    fn icons_centred(&mut self, node: &Node, path: &[String]) {
        let placed: Vec<&Node> = node
            .children
            .iter()
            .filter(|c| !matches!(c.role, Role::Spacer { .. }))
            .collect();
        let axes: &[Axis] = match node.role {
            Role::Box | Role::Component { .. } if placed.len() == 1 => &[Axis::X, Axis::Y],
            Role::Layout {
                axis: Axis::X,
                align: Align::Centre,
                ..
            } => &[Axis::Y],
            Role::Layout {
                axis: Axis::Y,
                align: Align::Centre,
                ..
            } => &[Axis::X],
            _ => &[],
        };
        for icon in placed.iter().filter(|c| matches!(c.role, Role::Icon { .. })) {
            for &axis in axes {
                let off = icon.bounds.centre(axis) - node.bounds.centre(axis);
                if self.device(off.abs()) > HALF_PIXEL + EPSILON {
                    let message = format!(
                        "{} is {off} off the centre of {} {} across {axis:?}",
                        icon.role.label(),
                        node.role.label(),
                        node.bounds
                    );
                    self.report(Rule::IconCentre, path, &node.role, message);
                }
            }
        }
    }
}

/// Whether two frames are at one scale. Scales are the few the tokens list, never computed.
fn same_scale(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

fn path_text(path: &[String], role: &Role) -> String {
    let mut segments = path.to_vec();
    segments.push(role.label());
    segments.join(" > ")
}

/// A component's icons and first text, placed relative to it in device pixels. Nested components
/// are theirs to check.
fn parts(node: &Node, fixed: Fixed, scale: f32) -> Vec<Part> {
    let device = |v: f32| (v * scale).round() as i64;
    let b = node.bounds;
    // Along a measure the component doesn't fix, where a part sits is its content's doing (an icon
    // centred beside two lines or three), so only the fixed measures place parts.
    let x_of = |r: Rect| fixed.width.then(|| device(r.x - b.x));
    let y_of = |r: Rect| fixed.height.then(|| device(r.y - b.y));
    let mut found = Vec::new();
    let mut text_seen = false;
    let mut pending: Vec<&Node> = node.children.iter().rev().collect();
    while let Some(n) = pending.pop() {
        match n.role {
            Role::Component { .. } => continue,
            Role::Icon { .. } => found.push(Part {
                what: "icon",
                x: x_of(n.bounds),
                y: y_of(n.bounds),
                width: Some(device(n.bounds.width)),
                height: device(n.bounds.height),
            }),
            Role::Text { .. } if !text_seen => {
                text_seen = true;
                found.push(Part {
                    what: "text",
                    x: x_of(n.bounds),
                    y: y_of(n.bounds),
                    width: None,
                    height: device(n.bounds.height),
                });
            }
            _ => {}
        }
        pending.extend(n.children.iter().rev());
    }
    found
}

/// Every instance of a component and variant the same: its fixed measures the same at every scale,
/// and its icons and label in the same place at each scale.
fn identical(instances: &[Instance], findings: &mut Vec<Finding>) {
    let mut groups: BTreeMap<(&str, &str), Vec<&Instance>> = BTreeMap::new();
    for instance in instances {
        groups
            .entry((instance.name.as_str(), instance.variant.as_str()))
            .or_default()
            .push(instance);
    }
    for group in groups.values() {
        let first = group[0];
        for instance in &group[1..] {
            let mut differences = Vec::new();
            if instance.fixed != first.fixed {
                differences.push("declares other fixed measures".to_owned());
            }
            if first.fixed.width && (instance.bounds.width - first.bounds.width).abs() * instance.scale >= EPSILON {
                differences.push(format!("is {} wide, not {}", instance.bounds.width, first.bounds.width));
            }
            if first.fixed.height && (instance.bounds.height - first.bounds.height).abs() * instance.scale >= EPSILON {
                differences.push(format!(
                    "is {} tall, not {}",
                    instance.bounds.height, first.bounds.height
                ));
            }
            let reference = group
                .iter()
                .find(|other| same_scale(other.scale, instance.scale))
                .unwrap_or(&first);
            // A card or a page holds whatever the app puts in it, so its parts are the content's.
            if !first.fixed.holds && !same_parts(&instance.parts, &reference.parts) {
                differences.push(format!(
                    "places its parts at {}, where {} has {}",
                    describe(&instance.parts),
                    reference.path,
                    describe(&reference.parts)
                ));
            }
            if !differences.is_empty() {
                findings.push(Finding {
                    frame: instance.frame.clone(),
                    rule: Rule::Identical,
                    path: instance.path.clone(),
                    message: format!(
                        "this {} {}; the first, in {} at {}, doesn't",
                        first.name,
                        differences.join(", "),
                        first.frame,
                        first.path
                    ),
                });
            }
        }
    }
}

/// Parts as a finding shows them, in device pixels: `icon 25×25 at (13, 13), text 30 tall at y 10`.
fn describe(parts: &[Part]) -> String {
    let at = |x: Option<i64>, y: Option<i64>| match (x, y) {
        (Some(x), Some(y)) => format!(" at ({x}, {y})"),
        (Some(x), None) => format!(" at x {x}"),
        (None, Some(y)) => format!(" at y {y}"),
        (None, None) => String::new(),
    };
    let shown: Vec<String> = parts
        .iter()
        .map(|p| match p.width {
            Some(width) => format!("{} {width}×{}{}", p.what, p.height, at(p.x, p.y)),
            None => format!("{} {} tall{}", p.what, p.height, at(p.x, p.y)),
        })
        .collect();
    if shown.is_empty() {
        "none".to_owned()
    } else {
        format!("{} (device px)", shown.join(", "))
    }
}

/// Whether two instances place their parts alike. What an instance shows is its content's to
/// decide (a ticked checkbox has a tick, an empty one none), so labels are compared when both have
/// one, and icons when both have as many.
fn same_parts(a: &[Part], b: &[Part]) -> bool {
    let of = |parts: &[Part], what: &str| parts.iter().filter(|p| p.what == what).cloned().collect::<Vec<_>>();
    let (text_a, text_b) = (of(a, "text"), of(b, "text"));
    let (icons_a, icons_b) = (of(a, "icon"), of(b, "icon"));
    let texts_alike = text_a.is_empty() || text_b.is_empty() || text_a == text_b;
    let icons_alike = icons_a.len() != icons_b.len() || icons_a == icons_b;
    texts_alike && icons_alike
}

/// A frame's boxes in order, each with its role's label. A text's words and an icon's symbol are
/// content, which a mode may change (a contrast ratio, a switch naming the other mode), so they
/// are left out, and with them the width the words take; where the text sits still counts.
fn flatten(node: &Node, into: &mut Vec<(String, Rect)>) {
    let (label, bounds) = match node.role {
        Role::Text { .. } => (
            "Text".to_owned(),
            Rect {
                width: 0.0,
                ..node.bounds
            },
        ),
        Role::Icon { .. } => ("Icon".to_owned(), node.bounds),
        _ => (node.role.label(), node.bounds),
    };
    into.push((label, bounds));
    for child in &node.children {
        flatten(child, into);
    }
}

/// Each page's light frame with the same boxes as its dark one at the same scale: changing modes
/// changes colours, never geometry.
fn mode_invariant(frames: &[Frame], findings: &mut Vec<Finding>) {
    for (index, frame) in frames.iter().enumerate() {
        let Some(other) = frames[..index]
            .iter()
            .find(|o| o.page == frame.page && same_scale(o.scale, frame.scale) && o.mode != frame.mode)
        else {
            continue;
        };
        let (mut a, mut b) = (Vec::new(), Vec::new());
        flatten(&other.root, &mut a);
        flatten(&frame.root, &mut b);
        let first_difference = a.iter().zip(&b).position(|(x, y)| x != y);
        let message = match first_difference {
            Some(at) => format!(
                "box {at} is {} {} here and {} {} in {}",
                b[at].0,
                b[at].1,
                a[at].0,
                a[at].1,
                other.name()
            ),
            None if a.len() != b.len() => {
                format!("{} boxes here and {} in {}", b.len(), a.len(), other.name())
            }
            None => continue,
        };
        findings.push(Finding {
            frame: frame.name(),
            rule: Rule::ModeInvariant,
            path: frame.page.clone(),
            message,
        });
    }
}

#[cfg(test)]
mod tests;
