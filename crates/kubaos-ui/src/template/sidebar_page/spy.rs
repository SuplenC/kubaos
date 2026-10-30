//! Where the page's parts are, and which sub-section the page is scrolled to: the scroll-spy.
//!
//! The page marks its pane, each sub-section and each row ([`mark`]). One operation over the laid
//! out view ([`measure`]) reads where they are in the scroll's content, once per layout; every
//! scroll after that is arithmetic on those numbers.
//!
//! ```text
//!   pane top ─────────── what the person sees starts here
//!     32                 the page's room above its first line, where a click puts a heading
//!   reading line ─────── the last sub-section whose heading is at or above it is current
//! ```

use std::any::Any;
use std::slice;

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
use iced::advanced::widget::operation;

use crate::tokens::Space;

/// The page's room above its first line: a heading scrolled to sits here, as the page's header
/// does at the top.
pub(crate) const LEAD: Space = Space::S32;

/// The room kept between a row the keyboard or the find box brings into view and the pane's edge.
pub(crate) const MARGIN: Space = Space::S24;

/// What a mark stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spot {
    /// The page's pane: the box the person sees the page in.
    Pane,
    /// A row of the sidebar, by its place among the sidebar's rows.
    Entry(usize),
    /// A sub-section, by its place in the section.
    Sub(usize),
    /// A row, by its place in the section's reading order.
    Row(usize),
}

/// Where the page's parts are, in the scroll's content: 0 is the content's top.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Measure {
    /// How far the pane's top is below the scroll's own top: the scroll reaches past the pane.
    pub(crate) reach: f32,
    /// The pane's height.
    pub(crate) pane: f32,
    /// The sidebar's height, how far its rows can scroll, and each row's top and bottom in them.
    pub(crate) sidebar: f32,
    pub(crate) sidebar_max: f32,
    pub(crate) entries: Vec<(f32, f32)>,
    /// How far the content can scroll.
    pub(crate) max: f32,
    /// Each sub-section's top.
    pub(crate) subs: Vec<f32>,
    /// Each row's top and bottom, in reading order.
    pub(crate) rows: Vec<(f32, f32)>,
}

impl Measure {
    /// The offset that puts content `top` at the page's lead, as far as the content scrolls.
    pub fn offset_for(&self, top: f32) -> f32 {
        (top - self.reach - f32::from(LEAD.px())).clamp(0.0, self.max)
    }

    /// The sub-section the page shows at `offset`: the last whose heading has reached the
    /// reading line, or the last of all once the page can scroll no further.
    pub fn current(&self, offset: f32) -> usize {
        if self.subs.is_empty() {
            return 0;
        }
        if self.max > 0.0 && offset >= self.max - 1.0 {
            return self.subs.len() - 1;
        }
        let line = offset + self.reach + f32::from(LEAD.px()) + 1.0;
        self.subs.iter().rposition(|&top| top <= line).unwrap_or(0)
    }

    /// The offset that brings `row` into view with [`MARGIN`] around it, below `covered` at the
    /// pane's top (what floats there), scrolling as little as it can; `None` when it is in view
    /// already.
    pub fn reveal(&self, row: usize, offset: f32, covered: f32) -> Option<f32> {
        let &(top, bottom) = self.rows.get(row)?;
        let margin = f32::from(MARGIN.px());
        let shown = offset + self.reach;
        let above = covered + margin;
        let to = if top - above < shown {
            top - above - self.reach
        } else if bottom + margin > shown + self.pane {
            // A row taller than the pane shows its top.
            (bottom + margin - self.pane - self.reach).min(top - above - self.reach)
        } else {
            return None;
        };
        let to = to.clamp(0.0, self.max);
        ((to - offset).abs() >= 1.0).then_some(to)
    }

    /// The sidebar's offset that shows its row `entry` whole with the sidebar's padding around it,
    /// scrolling as little as it can; `None` when it shows already.
    pub fn reveal_entry(&self, entry: usize, offset: f32, padding: f32) -> Option<f32> {
        let &(top, bottom) = self.entries.get(entry)?;
        let to = if top - padding < offset {
            top - padding
        } else if bottom + padding > offset + self.sidebar {
            bottom + padding - self.sidebar
        } else {
            return None;
        };
        let to = to.clamp(0.0, self.sidebar_max);
        ((to - offset).abs() >= 1.0).then_some(to)
    }

    /// The row a page of the pane away from `row`, down (`forward`) or up: the furthest that shows
    /// whole in one pane with `row`, and always at least the next one.
    pub fn page_from(&self, row: usize, forward: bool) -> Option<usize> {
        let &(top, bottom) = self.rows.get(row)?;
        let last = self.rows.len() - 1;
        Some(if forward {
            let reach = self
                .rows
                .iter()
                .rposition(|&(_, b)| b <= top + self.pane)
                .unwrap_or(row);
            reach.max(row + 1).min(last)
        } else {
            let reach = self
                .rows
                .iter()
                .position(|&(t, _)| t >= bottom - self.pane)
                .unwrap_or(row);
            reach.min(row.saturating_sub(1))
        })
    }
}

/// `content` marked as `spot`, for [`measure`].
pub(crate) fn mark<'a, M: 'a>(spot: Spot, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(Mark {
        spot,
        content: content.into(),
    })
}

/// Reads the marks inside the page's scroll `scroll` and the sidebar's `sidebar` into a
/// [`Measure`].
pub(crate) fn measure(scroll: Id, sidebar: Id) -> impl Operation<Measure> {
    Spy {
        scroll,
        sidebar,
        view: None,
        side: None,
        pane: None,
        subs: Vec::new(),
        rows: Vec::new(),
        entries: Vec::new(),
    }
}

struct Spy {
    scroll: Id,
    sidebar: Id,
    /// Each scroll's box and its content's.
    view: Option<(Rectangle, Rectangle)>,
    side: Option<(Rectangle, Rectangle)>,
    pane: Option<Rectangle>,
    subs: Vec<(usize, Rectangle)>,
    rows: Vec<(usize, Rectangle)>,
    entries: Vec<(usize, Rectangle)>,
}

impl Operation<Measure> for Spy {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Measure>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if id == Some(&self.scroll) {
            self.view = Some((bounds, content_bounds));
        } else if id == Some(&self.sidebar) {
            self.side = Some((bounds, content_bounds));
        }
    }

    fn custom(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        match state.downcast_ref::<Spot>() {
            Some(Spot::Pane) => self.pane = Some(bounds),
            Some(&Spot::Entry(at)) => self.entries.push((at, bounds)),
            Some(&Spot::Sub(at)) => self.subs.push((at, bounds)),
            Some(&Spot::Row(at)) => self.rows.push((at, bounds)),
            None => {}
        }
    }

    fn finish(&self) -> operation::Outcome<Measure> {
        let (Some((scroll, content)), Some(pane)) = (self.view, self.pane) else {
            return operation::Outcome::None;
        };
        // Layout inside a scroll isn't moved by its offset, so the content's top is the reference.
        let in_content = |r: &Rectangle| r.y - content.y;
        let mut subs = self.subs.clone();
        subs.sort_by_key(|&(at, _)| at);
        let mut rows = self.rows.clone();
        rows.sort_by_key(|&(at, _)| at);
        let mut entries = self.entries.clone();
        entries.sort_by_key(|&(at, _)| at);
        let (side, side_content) = self.side.unwrap_or_default();
        operation::Outcome::Some(Measure {
            reach: pane.y - scroll.y,
            pane: pane.height,
            sidebar: side.height,
            sidebar_max: (side_content.height - side.height).max(0.0),
            entries: entries
                .iter()
                .map(|(_, r)| (r.y - side_content.y, r.y - side_content.y + r.height))
                .collect(),
            max: (content.height - scroll.height).max(0.0),
            subs: subs.iter().map(|(_, r)| in_content(r)).collect(),
            rows: rows
                .iter()
                .map(|(_, r)| (in_content(r), in_content(r) + r.height))
                .collect(),
        })
    }
}

/// Lays out and draws as its content, and tells an operation where it is and what it marks.
struct Mark<'a, M> {
    spot: Spot,
    content: Element<'a, M>,
}

impl<M> Widget<M, Theme, Renderer> for Mark<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(node.size(), vec![node])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let mut spot = self.spot;
        operation.custom(None, layout.bounds(), &mut spot);
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], content, renderer, operation);
        }
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
        if let Some(content) = layout.children().next() {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                content,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        layout
            .children()
            .next()
            .map_or_else(mouse::Interaction::default, |content| {
                self.content
                    .as_widget()
                    .mouse_interaction(&tree.children[0], content, cursor, viewport, renderer)
            })
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
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget()
                .draw(&tree.children[0], renderer, theme, style, content, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        let content = layout.children().next()?;
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], content, renderer, viewport, translation)
    }
}

#[cfg(test)]
mod tests;
