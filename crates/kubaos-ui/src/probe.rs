//! Probes: what each box of a view is, for the geometry check (`kubaos-geometry`).
//!
//! iced reports a laid-out view's boxes through an [`Operation`], but only as containers and text:
//! it can't say that a container is a button, that a row's gap is 8, or how wide a text wants to
//! be. Every layout primitive, component, text and icon of this crate wraps itself in a probe that
//! adds that. A probe lays out and draws exactly as its content, one call deeper, and allocates
//! nothing until an operation walks the tree.
//!
//! [`Collect`] walks a view's operations and builds the [`kubaos_geometry::Node`] tree the check
//! reads. A widget iced reports without a probe (a toolkit's own container) shows as a box that
//! claims nothing.
//!
//! # Owns
//!
//! - [`probe`] and [`component`], the wrappers, with their [`Tag`]s, and [`Collect`], the tree.
//!
//! # Not here
//!
//! - The rules: `kubaos-geometry`. Rendering pages to collect from: `kubaos-specimen`.

use std::any::Any;

use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::Vector;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Id;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;
use iced::advanced::widget::operation::Scrollable;
use iced::advanced::widget::tree;
pub use kubaos_geometry::Align;
pub use kubaos_geometry::Axis;
pub use kubaos_geometry::Fixed;
use kubaos_geometry::Node;
use kubaos_geometry::Rect;
pub use kubaos_geometry::Role;

use crate::text::LineState;

/// What a probe says its content is. Built with every view, so it holds no allocation; the
/// [`Role`] is made from it only when an operation asks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tag {
    /// A row or stack of the layout.
    Layout { axis: Axis, gap: f32, align: Align },
    /// A grid or split of `columns` columns.
    Grid { columns: u16, gap: f32, align: Align },
    /// A component of the spec: its shared name, its variant's words, and its own measures.
    Component {
        name: &'static str,
        variant: [&'static str; 3],
        fixed: Fixed,
    },
    /// Text in a type style; its content and the size it needs are read when probed.
    Text,
    /// A symbol.
    Icon(&'static str),
    /// Empty room: a gap, or a fill.
    Spacer { fill: bool },
}

/// `content`, reported to the geometry check as `tag` says.
pub fn probe<'a, M: 'a>(tag: Tag, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(Probe {
        content: content.into(),
        tag,
    })
}

/// A component of the spec, by its shared name, the words of its variant (`["Filled", "Md", ""]`)
/// and the measures that are its own.
pub fn component<'a, M: 'a>(
    name: &'static str,
    variant: [&'static str; 3],
    fixed: Fixed,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    probe(Tag::Component { name, variant, fixed }, content)
}

struct Probe<'a, M> {
    content: Element<'a, M>,
    tag: Tag,
}

impl Tag {
    fn role(self, tree: &Tree, bounds: Rectangle) -> Role {
        match self {
            Tag::Layout { axis, gap, align } => Role::Layout { axis, gap, align },
            Tag::Grid { columns, gap, align } => Role::Grid { columns, gap, align },
            Tag::Component { name, variant, fixed } => Role::Component {
                name: name.to_owned(),
                variant: variant
                    .iter()
                    .filter(|w| !w.is_empty())
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" "),
                fixed,
            },
            Tag::Text => Role::Text {
                content: String::new(),
                natural: natural(tree, bounds),
            },
            Tag::Icon(name) => Role::Icon { name: name.to_owned() },
            Tag::Spacer { fill } => Role::Spacer { fill },
        }
    }
}

impl<M> Widget<M, Theme, Renderer> for Probe<'_, M> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let mut role = self.tag.role(tree, layout.bounds());
        operation.custom(None, layout.bounds(), &mut role);
        let content = &mut self.content;
        operation.traverse(&mut |operation| {
            content.as_widget_mut().operate(tree, layout, renderer, operation);
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

type Paragraph = <Renderer as iced::advanced::text::Renderer>::Paragraph;

/// The size a text widget's shaped paragraph needs, from its state; its box's when it has none.
fn natural(tree: &Tree, bounds: Rectangle) -> kubaos_geometry::Size {
    let needed = if tree.tag == tree::Tag::of::<LineState>() {
        tree.state.downcast_ref::<LineState>().min_bounds()
    } else {
        tree.state
            .downcast_ref::<iced::widget::text::State<Paragraph>>()
            .min_bounds()
    };
    let (width, height) = if needed.width > 0.0 || needed.height > 0.0 {
        (needed.width, needed.height)
    } else {
        (bounds.width, bounds.height)
    };
    kubaos_geometry::Size { width, height }
}

/// Builds the geometry tree of a view from its operations: run it with
/// `UserInterface::operate`, then take the tree with [`Collect::into_tree`].
#[derive(Debug)]
pub struct Collect {
    nodes: Vec<Collected>,
    scopes: Vec<Scope>,
}

#[derive(Debug)]
struct Collected {
    role: Role,
    bounds: Rectangle,
    children: Vec<usize>,
    /// A probe whose content's own report has been folded into it.
    folded: bool,
    probe: bool,
}

/// Where reports land: under `parent`, after `last`.
#[derive(Debug, Clone, Copy)]
struct Scope {
    parent: usize,
    last: Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Last {
    None,
    Node(usize),
    /// The report was the probe's own content, so what it traverses belongs to the probe.
    Folded,
}

impl Default for Collect {
    fn default() -> Self {
        Collect {
            nodes: vec![Collected {
                role: Role::Box,
                bounds: Rectangle::default(),
                children: Vec::new(),
                folded: true,
                probe: false,
            }],
            scopes: vec![Scope {
                parent: 0,
                last: Last::None,
            }],
        }
    }
}

impl Collect {
    #[must_use]
    pub fn new() -> Collect {
        Collect::default()
    }

    /// The view's tree: its one root, or a box around its roots.
    pub fn into_tree(self) -> Option<Node> {
        let roots = &self.nodes[0].children;
        match roots.as_slice() {
            [] => None,
            [root] => Some(self.build(*root)),
            _ => {
                let children: Vec<Node> = roots.iter().map(|&i| self.build(i)).collect();
                let union = children
                    .iter()
                    .map(|c| rectangle(c.bounds))
                    .reduce(|a, b| a.union(&b))
                    .unwrap_or_default();
                Some(Node {
                    role: Role::Box,
                    bounds: rect(union),
                    children,
                })
            }
        }
    }

    /// The node at `index`, with the boxes that add nothing taken out: a plain box around one
    /// child with its bounds is that child, and a plain box as the only child of a node with its
    /// bounds gives the node its children. Slots and iced's own containers report such boxes
    /// around most content.
    fn build(&self, index: usize) -> Node {
        let node = &self.nodes[index];
        let mut children: Vec<Node> = node.children.iter().map(|&c| self.build(c)).collect();
        while let [only] = children.as_slice()
            && only.role == Role::Box
            && only.bounds == rect(node.bounds)
        {
            children = children.pop().map(|only| only.children).unwrap_or_default();
        }
        if node.role == Role::Box
            && let [only] = children.as_slice()
            && only.bounds == rect(node.bounds)
            && let Some(only) = children.pop()
        {
            return only;
        }
        Node {
            role: node.role.clone(),
            bounds: rect(node.bounds),
            children,
        }
    }

    fn scope(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("the root scope is never popped")
    }

    /// Records a report, or folds it into the probe it is the content of: the first report under a
    /// probe with the probe's own bounds is the wrapped widget itself.
    fn report(&mut self, role: Role, bounds: Rectangle, probe: bool) {
        let scope = *self.scope();
        let parent = &mut self.nodes[scope.parent];
        let is_content = parent.probe && !parent.folded && parent.children.is_empty() && parent.bounds == bounds;
        if is_content && !probe {
            parent.folded = true;
            // A text probe learns its content from the text widget it wraps.
            if let (Role::Text { content, .. }, Role::Text { content: read, .. }) = (&mut parent.role, role) {
                *content = read;
            }
            self.scope().last = Last::Folded;
            return;
        }
        if is_content {
            parent.folded = true;
        }
        let index = self.nodes.len();
        self.nodes.push(Collected {
            role,
            bounds,
            children: Vec::new(),
            folded: false,
            probe,
        });
        self.nodes[scope.parent].children.push(index);
        self.scope().last = Last::Node(index);
    }
}

impl Operation for Collect {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        let scope = *self.scope();
        let parent = match scope.last {
            Last::Node(index) => index,
            Last::Folded | Last::None => scope.parent,
        };
        self.scopes.push(Scope {
            parent,
            last: Last::None,
        });
        operate(self);
        self.scopes.pop();
    }

    fn container(&mut self, _id: Option<&Id>, bounds: Rectangle) {
        self.report(Role::Box, bounds, false);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        self.report(Role::Scroll, bounds, false);
    }

    fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
        let role = Role::Text {
            content: text.to_owned(),
            natural: kubaos_geometry::Size {
                width: bounds.width,
                height: bounds.height,
            },
        };
        self.report(role, bounds, false);
    }

    fn custom(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        if let Some(role) = state.downcast_ref::<Role>() {
            self.report(role.clone(), bounds, true);
        }
    }
}

fn rect(r: Rectangle) -> Rect {
    Rect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

fn rectangle(r: Rect) -> Rectangle {
    Rectangle {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

#[cfg(test)]
mod tests;
