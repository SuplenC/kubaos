//! How a sidebar page is drawn: the sidebar with its gliding bar, the page with its fade and rise,
//! Home's column, and the find box floating over the page.

use std::slice;
use std::time::Instant;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Renderer;
use iced::Shadow;
use iced::Size as IcedSize;
use iced::Theme;
use iced::Vector;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Renderer as _;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;
use iced::widget;

use super::Entry;
use super::Message;
use super::Place;
use super::Region;
use super::RowAt;
use super::RowState;
use super::SidebarPage;
use super::spy::Spot;
use super::spy::mark;
use crate::color;
use crate::component::field::field;
use crate::component::keycap::key_hints;
use crate::component::layered;
use crate::component::list::list;
use crate::component::menu::popover;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::focus::rings;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::container;
use crate::layout::divider;
use crate::layout::fill;
use crate::layout::header;
use crate::layout::page_clear;
use crate::layout::row;
use crate::layout::section;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::motion::Motion;
use crate::motion::Tween;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::scroll::scroll;
use crate::snap::centre_x;
use crate::text::line;
use crate::text::text;
use crate::tokens::Duration;
use crate::tokens::Easing;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// The sidebar's padding: its rows take the control radius, as a well's do.
pub(super) const SIDEBAR_INSET: Space = Space::S12;
/// The gap between the sidebar's rows.
pub(super) const ROW_GAP: Space = Space::S4;
/// A section's row, and Home's.
pub(super) const SECTION_ROW: Size = Size::Row;
/// A sub-section's row: smaller, as its words are.
const SUB_ROW: Size = Size::ControlSm;
/// A section's row's words, and the symbol beside them.
const SECTION_TYPE: Type = Type::Body;
/// A sub-section's row's words, which start where the section's do.
const SUB_TYPE: Type = Type::Small;
/// The page's parts: the header, then each sub-section.
const PAGE_GAP: Space = Space::S40;
/// The wells of one sub-section.
const WELL_GAP: Space = Space::S16;
/// Home's column: its header, its field, its results.
const HOME_GAP: Space = Space::S24;

/// What Home's results are drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HomeView<'a> {
    /// What is typed in the field.
    pub query: &'a str,
    /// The result the keyboard is on.
    pub result: Option<usize>,
}

/// `state` drawn with `look`: the sidebar beside the page. The template's own messages go through
/// `on`; `home` draws Home's results under its field and `row` each row of a section's wells, as
/// [`RowState`] asks.
pub fn view<'a, M: Clone + 'a>(
    state: &'a SidebarPage,
    look: &Look,
    on: fn(Message) -> M,
    home: impl FnOnce(&Look, HomeView<'a>) -> Element<'a, M>,
    row_view: impl Fn(RowAt, RowState<'a>, &Look) -> Element<'a, M>,
) -> Element<'a, M> {
    let now = Instant::now();
    let (shown, below) = state.arrival(now);
    let faded = if shown < 1.0 { look.faded(shown) } else { *look };
    let content = match state.place {
        Place::Home => home_page(state, &faded, on, home),
        Place::Section(section) => section_page(state, section, &faded, now, row_view),
    };
    let scrolled: Element<'a, M> = scroll(look, page_clear(content))
        .id(state.id.clone())
        .height(Extent::Fill)
        .on_scroll(move |viewport| on(Message::Scrolled(viewport)))
        .into();
    let surface = look.colors.surface;
    let pane = widget::container(mark(Spot::Pane, shift(below, scrolled)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| widget::container::Style::default().background(surface));
    let mut layers = vec![pane.into()];
    if let Some(found) = find_box(state, look, on, now) {
        layers.push(found);
    }
    let pane: Element<'a, M> = widget::Stack::with_children(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    let whole: Element<'a, M> = row(look, Gap::NONE)
        .height(Extent::Fill)
        .push(sidebar(state, look).map(on))
        .push(pane)
        .into();
    probe::component("SidebarPage", ["", "", ""], Fixed::HOLDS, whole)
}

/// The sidebar band: Home over a hairline, then the sections, the one showing with its
/// sub-sections, and the bar beside the current one. It scrolls on its own when it runs past the
/// window.
fn sidebar<'a>(state: &'a SidebarPage, look: &Look) -> Element<'a, Message> {
    let outline = &state.outline;
    let ring = |entry: Entry| state.region == Region::Sidebar && state.keyboard && state.cursor == entry;
    let mut entry = 0;
    let mut marked = |row: Element<'a, Message>| {
        let row = mark(Spot::Entry(entry), row);
        entry += 1;
        row
    };
    let mut rows = stack(look, ROW_GAP).width(Extent::Fill).push(marked(sidebar_row(
        look,
        Some(outline.home.icon),
        &outline.home.label,
        state.place == Place::Home,
        ring(Entry::Home),
        Entry::Home,
    )));
    rows = rows.push(
        stack(look, Gap::NONE)
            .width(Extent::Fill)
            .padding(Inset::y(Space::S4))
            .push(divider(look)),
    );
    // The bar's row, counted as the stack's children are: Home and the hairline first.
    let mut bar = None;
    let mut children = 2;
    for (index, section) in outline.sections.iter().enumerate() {
        let open = state.place == Place::Section(index);
        rows = rows.push(marked(sidebar_row(
            look,
            Some(section.icon),
            &section.title,
            open,
            ring(Entry::Section(index)),
            Entry::Section(index),
        )));
        children += 1;
        if !open {
            continue;
        }
        for (sub, shape) in section.subs.iter().enumerate() {
            let current = sub == state.spy;
            if current {
                bar = Some(children);
            }
            rows = rows.push(marked(sidebar_row(
                look,
                None,
                &shape.title,
                current,
                ring(Entry::Sub(index, sub)),
                Entry::Sub(index, sub),
            )));
            children += 1;
        }
    }
    let key = match state.place {
        Place::Home => 0,
        Place::Section(section) => section + 1,
    };
    let glide = Element::new(Glide {
        content: rows.into(),
        target: bar,
        key,
        color: look.colors.primary,
        scale: look.scale,
        motion: look.motion,
    });
    let scrolled = crate::scroll::flush(
        look,
        widget::container(glide).padding(Inset::all(SIDEBAR_INSET).padding()),
    )
    .id(state.sidebar_id.clone())
    .on_scroll(Message::SidebarScrolled)
    .height(Extent::Fill);
    let band = look.colors.surface_container_low;
    widget::container(scrolled)
        .width(f32::from(Size::Sidebar.px()))
        .height(Length::Fill)
        .style(move |_theme| widget::container::Style::default().background(band))
        .into()
}

/// A row of the sidebar: a section's (with its symbol) or a sub-section's, its words where a
/// section's start. The current one is filled, a section's symbol in the accent; the keyboard's
/// lights its edge.
fn sidebar_row<'a>(
    look: &Look,
    glyph: Option<Icon>,
    title: &'a str,
    current: bool,
    focused: bool,
    entry: Entry,
) -> Element<'a, Message> {
    let c = look.colors;
    let is_sub = glyph.is_none();
    let (height, style) = if is_sub {
        (SUB_ROW, SUB_TYPE)
    } else {
        (SECTION_ROW, SECTION_TYPE)
    };
    let words = if current { c.on_surface } else { c.on_surface_variant };
    let column = SECTION_TYPE.icon();
    let mark: Element<'a, Message> = match glyph {
        Some(glyph) => icon(
            look,
            glyph,
            column,
            if current { c.primary } else { c.on_surface_variant },
        ),
        None => widget::Space::new()
            .width(f32::from(column.px()))
            .height(f32::from(column.px()))
            .into(),
    };
    let content = row(look, Space::S12)
        .center()
        .width(Extent::Fill)
        .height(Extent::Fill)
        .push(mark)
        .push(line(title, style, words));
    let fill = (current && !is_sub).then_some(c.surface_container_high);
    let pressable = widget::button(content)
        .width(Length::Fill)
        .height(f32::from(height.px()))
        .padding(Inset::x(Space::S12).padding())
        .on_press(Message::Pick(entry))
        .style(move |_theme: &Theme, _status| widget::button::Style {
            background: fill.map(Background::Color),
            text_color: words,
            border: Border::default().rounded(radius(Radius::Control)),
            shadow: Shadow::default(),
            snap: true,
        });
    let layer = layered(pressable, Radius::Control, c.on_surface, true, None);
    let view = focus_ring(look, layer, Radius::Control, focused, RingColor::Accent);
    let variant = if is_sub { "sub" } else { "section" };
    probe::component("SidebarRow", [variant, "", ""], Fixed::HEIGHT, view)
}

/// Home: a centred column of the header, the search field and the app's results.
fn home_page<'a, M: Clone + 'a>(
    state: &'a SidebarPage,
    look: &Look,
    on: fn(Message) -> M,
    results: impl FnOnce(&Look, HomeView<'a>) -> Element<'a, M>,
) -> Element<'a, M> {
    let home = &state.outline.home;
    let mut search = field(&state.search.text, &home.placeholder)
        .icon(Icon::Search)
        .on_pointer(move |pointer| on(Message::SearchPointer(pointer)));
    if state.region == Region::Search {
        search = search.caret(state.search.caret);
    }
    let column = stack(look, HOME_GAP)
        .width(Extent::Fill)
        .push(header(
            look,
            home.accent.as_str(),
            home.title.as_str(),
            home.lede.as_str(),
        ))
        .push(search.view(look))
        .push(results(
            look,
            HomeView {
                query: &state.search.text,
                result: state.result,
            },
        ));
    centre_x(
        look.scale,
        widget::container(column)
            .width(Length::Fill)
            .max_width(f32::from(Size::Column.px())),
    )
}

/// A section's page: its header, then each sub-section's heading over its wells of rows.
fn section_page<'a, M: Clone + 'a>(
    state: &'a SidebarPage,
    index: usize,
    look: &Look,
    now: Instant,
    row_view: impl Fn(RowAt, RowState<'a>, &Look) -> Element<'a, M>,
) -> Element<'a, M> {
    let outline = &state.outline.sections[index];
    let mut page = stack(look, PAGE_GAP).width(Extent::Fill).push(header(
        look,
        outline.accent.as_str(),
        outline.title.as_str(),
        outline.lede.as_str(),
    ));
    let mut flat = 0;
    for (sub, shape) in outline.subs.iter().enumerate() {
        let mut wells = stack(look, WELL_GAP).width(Extent::Fill);
        for (well, &count) in shape.wells.iter().enumerate() {
            let mut rows = Vec::with_capacity(count);
            let mut lit = false;
            for row in 0..count {
                let at = RowAt {
                    section: index,
                    sub,
                    well,
                    row,
                };
                let row_state = state.row_state(at, flat, now);
                lit |= row_state.selected && state.keyboard;
                rows.push(mark(Spot::Row(flat), row_view(at, row_state, look)));
                flat += 1;
            }
            wells = wells.push(list(look, rows, lit));
        }
        page = page.push(mark(Spot::Sub(sub), section(look, &shape.title, None, wells)));
    }
    page.into()
}

/// The find box over the page's top-right corner, while it is open or on its way out: the field
/// and "n of m", and with no match, Enter to search every setting from Home.
fn find_box<'a, M: Clone + 'a>(
    state: &'a SidebarPage,
    look: &Look,
    on: fn(Message) -> M,
    now: Instant,
) -> Option<Element<'a, M>> {
    let find = state.find.as_ref()?;
    let (shown, above) = state.find_arrival(now);
    if shown <= 0.0 && !find.open {
        return None;
    }
    let look = &if shown < 1.0 { look.faded(shown) } else { *look };
    let c = look.colors;
    let mut entry = field(&find.query.text, FIND_PLACEHOLDER)
        .icon(Icon::Search)
        .on_pointer(move |pointer| on(Message::FindPointer(pointer)));
    if state.region == Region::Find && find.open {
        entry = entry.caret(find.query.caret);
    }
    let mut top = row(look, Space::S12)
        .center()
        .width(Extent::Fill)
        .push(entry.view(look));
    if !find.query.is_empty() {
        let (at, of) = if find.matches.is_empty() {
            (0, 0)
        } else {
            (find.current + 1, find.matches.len())
        };
        top = top.push(text(format!("{at} of {of}"), Type::Mono, c.on_surface_variant));
    }
    let mut body = stack(look, Space::S12).width(Extent::Fill).push(top);
    if find.matches.is_empty() && !find.query.is_empty() {
        body = body.push(key_hints(look, &[("Enter", "Search every setting from Home")]));
    }
    let floating = popover(look, Extent::Fixed(Size::Find), body);
    let corner = container(
        row(look, Gap::NONE)
            .width(Extent::Fill)
            .padding(Inset::NONE.top(FIND_TOP))
            .push(fill())
            .push(floating),
    );
    Some(shift(-above, corner))
}

/// How far below the pane's top the find box's field reaches: its room above, the popover's 12 on
/// either side of the field, and the field.
pub(super) fn find_reach() -> f32 {
    f32::from(FIND_TOP.px() + 2 * Space::S12.px() + Size::ControlMd.px())
}

/// The find box's room above it, a small control's band.
const FIND_TOP: Space = Space::S16;

/// The find box's placeholder.
const FIND_PLACEHOLDER: &str = "Find on this page";

/// `content` drawn `by` below its place, taking no layout: a page rising in, a box dropping in.
///
/// HAZARD: it wraps its content at rest too. Unwrapped at 0, the tree would change shape as the
/// motion ends, and iced would rebuild the content's state: a page's scroll back at its top.
fn shift<'a, M: 'a>(by: f32, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(Shift {
        by,
        content: content.into(),
    })
}

struct Shift<'a, M> {
    by: f32,
    content: Element<'a, M>,
}

impl<M> Widget<M, Theme, Renderer> for Shift<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(slice::from_ref(&self.content));
    }

    fn size(&self) -> IcedSize<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(node.size(), vec![node])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
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
        let by = self.offset();
        if let Some(content) = layout.children().next() {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                content,
                cursor - by,
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
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    content,
                    cursor - self.offset(),
                    viewport,
                    renderer,
                )
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
        let Some(content) = layout.children().next() else {
            return;
        };
        renderer.with_translation(self.offset(), |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                content,
                cursor - self.offset(),
                viewport,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, Renderer>> {
        let content = layout.children().next()?;
        let by = self.offset();
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], content, renderer, viewport, translation + by)
    }
}

impl<M> Shift<'_, M> {
    fn offset(&self) -> Vector {
        Vector::new(0.0, self.by)
    }
}

/// The sidebar's rows with the bar beside the current sub-section: a short pill in the accent with
/// the neon glow, which glides from row to row over [`Duration::Medium`] as the page scrolls and
/// on a press. A section changing puts it in place at once; so does reduced motion.
struct Glide<'a, M> {
    content: Element<'a, M>,
    /// Which of the rows' stack's children the bar is beside.
    target: Option<usize>,
    /// The section the rows list: the bar glides only within one.
    key: usize,
    color: Color,
    scale: Scale,
    motion: Motion,
}

/// Where the bar is going and where it is.
struct Bar {
    key: usize,
    y: Option<Tween>,
}

impl<M> Glide<'_, M> {
    /// The box of the row the bar is beside, in the drawn layout.
    fn target(&self, layout: Layout<'_>) -> Option<Rectangle> {
        let rows = layout.children().next()?;
        rows.children().nth(self.target?).map(|row| row.bounds())
    }
}

impl<M> Widget<M, Theme, Renderer> for Glide<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Bar>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Bar { key: self.key, y: None })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(slice::from_ref(&self.content));
    }

    fn size(&self) -> IcedSize<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = node.size();
        // The rows' stack is the content itself, laid out at the glide's origin.
        let target = self
            .target
            .and_then(|index| node.children().get(index))
            .map(layout::Node::bounds);
        let bar = tree.state.downcast_mut::<Bar>();
        match target {
            Some(row) => {
                let y = row.center_y();
                let instant = bar.key != self.key || bar.y.is_none() || self.motion.is_reduced();
                if instant {
                    bar.y = Some(Tween::new(y, Duration::Medium, Easing::Standard));
                } else if let Some(tween) = bar.y.as_mut() {
                    tween.to(y, Instant::now());
                }
            }
            None => bar.y = None,
        }
        bar.key = self.key;
        layout::Node::with_children(size, vec![node])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
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
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event
            && tree
                .state
                .downcast_ref::<Bar>()
                .y
                .is_some_and(|tween| tween.moving(*now))
        {
            shell.request_redraw();
        }
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
        let Some(content) = layout.children().next() else {
            return;
        };
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, content, cursor, viewport);
        let (Some(tween), Some(row)) = (tree.state.downcast_ref::<Bar>().y, self.target(layout)) else {
            return;
        };
        let column = f32::from(SECTION_TYPE.icon().px());
        let width = self.scale.thin();
        let height = f32::from(SUB_TYPE.icon().px());
        let x = row.x + f32::from(Space::S12.px()) + self.scale.snap((column - width) / 2.0);
        let y = self
            .scale
            .snap(layout.bounds().y + tween.value(Instant::now()) - height / 2.0);
        let bar = Rectangle::new(iced::Point::new(x, y), IcedSize::new(width, height));
        for (out, opacity) in rings(self.scale) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bar.expand(out),
                    border: Border {
                        color: color::alpha(self.color, opacity),
                        width: out,
                        radius: radius(Radius::Full).into(),
                    },
                    snap: true,
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }
        renderer.fill_quad(
            renderer::Quad {
                bounds: bar,
                border: Border::default().rounded(radius(Radius::Full)),
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(self.color),
        );
    }
}
