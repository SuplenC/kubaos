//! KubaOS's geometry check: the boxes of a rendered page, from any toolkit, held to the design
//! language's grid (`docs/DESIGN.md`, "Checks").
//!
//! A toolkit lays a page out at a scale and describes it as a [`Frame`]: a tree of [`Node`]s, each a
//! box in logical pixels and the [`Role`] it plays (a row of the layout, a component, a text, an
//! icon). [`check`] reads only that tree, so iced (through `kubaos-ui`'s probes) and QML (through a
//! dump of its item tree as JSON) are judged by the same rules:
//!
//! ```text
//! kubaos-ui probes ──┐
//!                    ├─▶ Frame (JSON) ─▶ check ─▶ Finding, one per broken rule and box
//! QML item dump ─────┘
//! ```
//!
//! # Owns
//!
//! - The frame's shape ([`Frame`], [`Node`], [`Role`]), the rules ([`Rule`], [`Rules`]) and
//!   checking frames against them ([`check`]).
//!
//! # Not here
//!
//! - Laying a page out and describing it: `kubaos-ui`'s `probe` module for iced, and
//!   `kubaos-specimen check` for the specimen's pages.

#![forbid(unsafe_code)]

mod check;

use std::fmt;

use serde::Deserialize;
use serde::Serialize;

pub use check::check;

/// One page laid out at one scale and mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// The page's name; frames of one page in two modes must have the same boxes.
    pub page: String,
    /// `dark` or `light`.
    pub mode: String,
    /// Device pixels per logical pixel.
    pub scale: f32,
    pub root: Node,
}

impl Frame {
    /// The frame's name in a finding: `components-dark-1.25x`.
    pub fn name(&self) -> String {
        format!("{}-{}-{}x", self.page, self.mode, self.scale)
    }
}

/// A box on the page, in logical pixels from the page's top left.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(flatten)]
    pub role: Role,
    pub bounds: Rect,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
}

impl Node {
    /// A node with no children yet.
    #[must_use]
    pub fn new(role: Role, bounds: Rect) -> Node {
        Node {
            role,
            bounds,
            children: Vec::new(),
        }
    }
}

/// What a box is, as much as the toolkit knows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum Role {
    /// A box that claims nothing: a toolkit's own container. Its edges are checked, its children's
    /// placement is not.
    Box,
    /// Children one after another along `axis`, `gap` apart, lined up across it by `align`: the
    /// spec's Row and Stack, QML's `RowLayout` and `ColumnLayout`.
    Layout { axis: Axis, gap: f32, align: Align },
    /// Children in `columns` columns, row by row, `gap` apart both ways: the spec's Grid and Split.
    Grid { columns: u16, gap: f32, align: Align },
    /// A component of the spec, by its shared name (`Button`, `IconButton`, `ListRow`) and variant
    /// (`Filled Md`). Every instance of one name and variant has the same `fixed` measures.
    Component { name: String, variant: String, fixed: Fixed },
    /// Text whose laid-out size is `natural`; it is clipped when that doesn't fit the box.
    Text { content: String, natural: Size },
    /// A symbol, by name, in a square box of its size.
    Icon { name: String },
    /// Empty room: a gap, or a fill that pushes what follows to the far end.
    Spacer { fill: bool },
}

impl Role {
    /// The role as a path segment in a finding.
    fn label(&self) -> String {
        match self {
            Role::Box => "Box".to_owned(),
            Role::Layout { axis: Axis::X, .. } => "Row".to_owned(),
            Role::Layout { axis: Axis::Y, .. } => "Stack".to_owned(),
            Role::Grid { .. } => "Grid".to_owned(),
            Role::Component { name, variant, .. } if variant.is_empty() => name.clone(),
            Role::Component { name, variant, .. } => format!("{name}({variant})"),
            Role::Text { content, .. } => format!("Text {:?}", ellipsis(content)),
            Role::Icon { name } => format!("Icon {name}"),
            Role::Spacer { fill: true } => "Fill".to_owned(),
            Role::Spacer { fill: false } => "Gap".to_owned(),
        }
    }
}

/// The first few words of a text, for a finding.
fn ellipsis(content: &str) -> String {
    const SHOWN: usize = 24;
    if content.chars().count() <= SHOWN {
        content.to_owned()
    } else {
        format!("{}…", content.chars().take(SHOWN).collect::<String>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    X,
    Y,
}

/// How children line up across a layout's axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Start,
    Centre,
    End,
}

/// Which of a component's measures are its own, the same in every instance, rather than its
/// content's or its parent's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fixed {
    pub width: bool,
    pub height: bool,
}

impl Fixed {
    pub const BOTH: Fixed = Fixed {
        width: true,
        height: true,
    };
    pub const HEIGHT: Fixed = Fixed {
        width: false,
        height: true,
    };
    pub const NONE: Fixed = Fixed {
        width: false,
        height: false,
    };
}

/// A rectangle in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn right(self) -> f32 {
        self.x + self.width
    }

    fn bottom(self) -> f32 {
        self.y + self.height
    }

    fn start(self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }

    fn end(self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.right(),
            Axis::Y => self.bottom(),
        }
    }

    fn length(self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.width,
            Axis::Y => self.height,
        }
    }

    fn centre(self, axis: Axis) -> f32 {
        self.start(axis) + self.length(axis) / 2.0
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{} at ({}, {})", self.width, self.height, self.x, self.y)
    }
}

/// A width and height in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// The design language's measures the boxes are held to: `design/tokens.toml`'s, passed in by the
/// caller so this crate reads no file.
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    /// Every fixed size, gap and icon is a multiple of this, in logical pixels.
    pub grid: f32,
    /// The gaps a layout may use, besides none.
    pub spacing: Vec<f32>,
}

/// A rule of the geometry check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// Every edge on a whole device pixel.
    DeviceGrid,
    /// Fixed sizes, gaps and icons on the 4 px grid and the spacing scale.
    ScaleGrid,
    /// Siblings exactly their layout's gap apart.
    SiblingGap,
    /// Siblings lined up across their layout, and grid cells in their columns and rows.
    Alignment,
    /// Every instance of a component and variant the same size, with its icons and label in the same
    /// place.
    Identical,
    /// Text that fits its box, and boxes that stay inside their parents.
    ClippedText,
    /// An icon centred in its slot, to the nearest device pixel.
    IconCentre,
    /// A page's light and dark frames with the same boxes.
    ModeInvariant,
}

impl Rule {
    pub const ALL: [Rule; 8] = [
        Rule::DeviceGrid,
        Rule::ScaleGrid,
        Rule::SiblingGap,
        Rule::Alignment,
        Rule::Identical,
        Rule::ClippedText,
        Rule::IconCentre,
        Rule::ModeInvariant,
    ];

    /// The rule's name in findings and docs.
    pub fn name(self) -> &'static str {
        match self {
            Rule::DeviceGrid => "device-grid",
            Rule::ScaleGrid => "scale-grid",
            Rule::SiblingGap => "sibling-gap",
            Rule::Alignment => "alignment",
            Rule::Identical => "identical",
            Rule::ClippedText => "clipped-text",
            Rule::IconCentre => "icon-centre",
            Rule::ModeInvariant => "mode-invariant",
        }
    }
}

/// A box that breaks a rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// The frame's name ([`Frame::name`]).
    pub frame: String,
    pub rule: Rule,
    /// Where the box is: its ancestors' roles, outermost first, with each one's place among its
    /// siblings.
    pub path: String,
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {}\n    at {}",
            self.frame,
            self.rule.name(),
            self.message,
            self.path
        )
    }
}
