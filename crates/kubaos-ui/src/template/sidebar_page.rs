//! `SidebarPage`: an app's window as a sidebar of sections beside a scrolling page, with Home, a
//! page that is only search, at the top.
//!
//! The template owns where the person is and how they move: which section shows and which of its
//! sub-sections the page is scrolled to (the scroll-spy), where the keyboard is, the keys, the find
//! box and its matches, and the motion between pages. The app owns what is on the pages: it
//! describes them ([`Outline`]), draws each row when asked ([`RowState`] says how the template
//! needs it shown), answers whether a row matches what is typed in the find box ([`Content`]), and
//! runs Home's search ([`Event::Query`]).
//!
//! ```text
//!   ┌ sidebar 264 ┬ page ──────────────────────────────┐
//!   │ Home        │ accent, title, lede      [find box]│
//!   │ ─────────── │ Sub-section                        │
//!   │ Section     │ ┌ well ──────────────────────────┐ │
//!   │ ▌ Sub       │ │ row                            │ │
//!   │   Sub       │ └────────────────────────────────┘ │
//!   │ Section     │ Sub-section …                      │
//!   └─────────────┴────────────────────────────────────┘
//! ```
//!
//! # Owns
//!
//! - [`SidebarPage`], its [`Message`]s and [`Event`]s, and its keys ([`Key`], [`translate`]).
//! - Its view: [`view`].
//!
//! # Not here
//!
//! - The rows, Home's search and its results: the app's.

mod keys;
mod outline;
mod query;
mod spy;
mod view;

use std::time::Instant;

use iced::Subscription;
use iced::Task;
use iced::advanced::widget::Id;
use iced::keyboard;
use iced::widget::operation::AbsoluteOffset;
use iced::widget::scrollable;
use iced::window;

pub use self::keys::Key;
pub use self::keys::translate;
pub use self::outline::HomeOutline;
pub use self::outline::Outline;
pub use self::outline::RowAt;
pub use self::outline::SectionOutline;
pub use self::outline::SubOutline;
use self::query::Query;
pub use self::spy::Measure;
pub use self::view::HomeView;
pub use self::view::view;
use crate::component::field::Pointer;
use crate::motion::Motion;
use crate::tokens::Duration;
use crate::tokens::Easing;

/// What the app answers for the find box: whether a row matches what is typed. The row lights the
/// letters itself, from [`RowState::query`].
pub trait Content {
    fn matches(&self, at: RowAt, query: &str) -> bool;
}

/// Where the keyboard is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// Home's search field.
    Search,
    Sidebar,
    /// The page's rows.
    Page,
    /// The find box's field.
    Find,
}

/// What the page shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Home,
    Section(usize),
}

/// A row of the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Home,
    Section(usize),
    /// A sub-section of the section that shows, the only one whose sub-sections are listed.
    Sub(usize, usize),
}

/// How the find box shows a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    /// No find box, or nothing typed in it.
    Off,
    /// It matches: its letters are lit.
    Match,
    /// It doesn't: it dims.
    Miss,
    /// The match the find box is on: the landed highlight.
    Current,
}

/// How the template needs a row shown. The row draws it as its own look says.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowState<'a> {
    /// The keyboard is on this row of the page.
    pub selected: bool,
    pub found: Found,
    /// What the find box holds, for the row to light the letters it matches.
    pub query: &'a str,
    /// The landed highlight's strength, 1 as a row is landed on from Home's search, fading to 0.
    pub landed: f32,
}

/// Something the app does in answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Home's field holds this now: search again, then give the template the result count
    /// ([`SidebarPage::set_results`]).
    Query(String),
    /// Enter on Home's result: go there, with [`SidebarPage::land`].
    Open(usize),
    /// Enter on a row: open it, or press its control.
    Press(RowAt),
    /// Space on a row: toggle its switch, checkbox or radio.
    Toggle(RowAt),
}

#[derive(Debug, Clone)]
pub enum Message {
    Key(keyboard::Event),
    /// A press on a sidebar row.
    Pick(Entry),
    /// A press on a row of the page: the app sends it from its row.
    Point(RowAt),
    SearchPointer(Pointer),
    FindPointer(Pointer),
    Scrolled(scrollable::Viewport),
    Measured(Measure),
    Resized,
    Frame(Instant),
}

/// The find box, open or on its way out.
#[derive(Debug, Clone)]
struct Find {
    query: Query,
    matches: Vec<RowAt>,
    current: usize,
    /// Where the keyboard and the page were as it opened, which Esc puts back.
    region: Region,
    offset: f32,
    row: usize,
    open: bool,
    /// When it last arrived or started to go.
    at: Instant,
}

/// Where the page should scroll once it is measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Goal {
    Sub(usize),
    Row(usize),
}

/// A sidebar page's state: the app keeps one and passes it its [`Message`]s.
#[derive(Debug, Clone)]
pub struct SidebarPage {
    outline: Outline,
    motion: Motion,
    place: Place,
    region: Region,
    /// The last move was the keyboard's, so focus shows; a press selects quietly.
    keyboard: bool,
    cursor: Entry,
    /// The sub-section the page is scrolled to: the one the sidebar's bar is beside.
    spy: usize,
    /// A sub-section picked in the sidebar, current while the page stays where it was scrolled to
    /// for it, though the spy alone would pick another (a short last one).
    pin: Option<(f32, usize)>,
    /// The page's row the keyboard is on, in its section's reading order.
    row: usize,
    search: Query,
    results: usize,
    result: Option<usize>,
    find: Option<Find>,
    id: Id,
    offset: f32,
    measure: Option<Measure>,
    goal: Option<Goal>,
    /// When the section last changed: the page fades and rises in.
    changed: Option<Instant>,
    landed: Option<(RowAt, Instant)>,
}

impl SidebarPage {
    /// Open on Home, the keyboard in its search field.
    #[must_use]
    pub fn new(outline: Outline) -> SidebarPage {
        SidebarPage {
            outline,
            motion: Motion::default(),
            place: Place::Home,
            region: Region::Search,
            keyboard: true,
            cursor: Entry::Home,
            spy: 0,
            pin: None,
            row: 0,
            search: Query::default(),
            results: 0,
            result: None,
            find: None,
            id: Id::unique(),
            offset: 0.0,
            measure: None,
            goal: None,
            changed: None,
            landed: None,
        }
    }

    /// Moving as `motion` says; reduced, a section changes with a plain swap.
    #[must_use]
    pub fn with_motion(self, motion: Motion) -> Self {
        SidebarPage { motion, ..self }
    }

    pub fn set_motion(&mut self, motion: Motion) {
        self.motion = motion;
    }

    pub fn outline(&self) -> &Outline {
        &self.outline
    }

    pub fn place(&self) -> Place {
        self.place
    }

    pub fn region(&self) -> Region {
        self.region
    }

    pub fn cursor(&self) -> Entry {
        self.cursor
    }

    /// The sub-section the page is scrolled to.
    pub fn current_sub(&self) -> usize {
        self.spy
    }

    /// The page's row the keyboard is on.
    pub fn row(&self) -> Option<RowAt> {
        match self.place {
            Place::Section(section) => self.outline.row_at(section, self.row),
            Place::Home => None,
        }
    }

    /// Where the page is scrolled to.
    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// What Home's field holds.
    pub fn query(&self) -> &str {
        &self.search.text
    }

    /// The result of Home's search the keyboard is on.
    pub fn result(&self) -> Option<usize> {
        self.result
    }

    /// The find box's match the page is on and how many there are, counting from 1, while it is
    /// open: `(0, 0)` with none.
    pub fn found(&self) -> Option<(usize, usize)> {
        let find = self.find.as_ref().filter(|find| find.open)?;
        Some(if find.matches.is_empty() {
            (0, 0)
        } else {
            (find.current + 1, find.matches.len())
        })
    }

    /// What the find box holds while it is open.
    pub fn find_query(&self) -> Option<&str> {
        self.find
            .as_ref()
            .filter(|find| find.open)
            .map(|find| find.query.text.as_str())
    }

    /// Home's search found `count` results: the keyboard goes to the first.
    pub fn set_results(&mut self, count: usize) {
        self.results = count;
        self.result = (count > 0).then_some(0);
    }

    /// Goes to `at` from Home's search: its section, scrolled to the row, the keyboard on it and
    /// the landed highlight on it.
    pub fn land(&mut self, at: RowAt) -> Task<Message> {
        let Some(index) = self.outline.index_of(at) else {
            return Task::none();
        };
        let mut task = if self.place == Place::Section(at.section) {
            Task::none()
        } else {
            self.open_section(at.section)
        };
        self.region = Region::Page;
        self.keyboard = true;
        self.row = index;
        self.cursor = Entry::Sub(at.section, at.sub);
        self.landed = Some((at, Instant::now()));
        task = task.chain(self.reveal_or_wait(index));
        task
    }

    /// Measures the page, once it has been laid out: the app calls it as the window opens.
    pub fn measure(&self) -> Task<Message> {
        iced::advanced::widget::operate(spy::measure(self.id.clone())).map(Message::Measured)
    }

    /// The keys, the window's size and, while anything moves, its frames.
    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            keyboard::listen().map(Message::Key),
            window::resize_events().map(|_| Message::Resized),
        ];
        if self.moving(Instant::now()) {
            subscriptions.push(window::frames().map(Message::Frame));
        }
        Subscription::batch(subscriptions)
    }

    /// Ends every motion where it ends: what a still of it (a screenshot, a test) shows.
    pub fn settle(&mut self) {
        self.changed = None;
        self.landed = None;
        if let Some(find) = self.find.as_mut() {
            if find.open {
                find.at = Instant::now()
                    .checked_sub(self.motion.duration(Duration::Short))
                    .unwrap_or(find.at);
            } else {
                self.find = None;
            }
        }
    }

    /// Whether anything is still moving at `now`.
    pub fn moving(&self, now: Instant) -> bool {
        let within =
            |at: Instant, duration: Duration| now.saturating_duration_since(at) < self.motion.duration(duration);
        self.changed.is_some_and(|at| within(at, Duration::Short))
            || self.find.as_ref().is_some_and(|find| within(find.at, Duration::Short))
            || self.landed.is_some_and(|(_, at)| within(at, Duration::Slow))
    }

    pub fn update(&mut self, message: Message, content: &impl Content) -> (Task<Message>, Option<Event>) {
        match message {
            Message::Key(event) => match translate(&event) {
                Some(key) => self.press(key, content),
                None => (Task::none(), None),
            },
            Message::Pick(entry) => {
                self.keyboard = false;
                let task = self.pick(entry);
                if entry == Entry::Home {
                    self.region = Region::Search;
                } else {
                    self.region = Region::Sidebar;
                }
                (task, None)
            }
            Message::Point(at) => {
                if let Some(index) = self.outline.index_of(at)
                    && self.place == Place::Section(at.section)
                {
                    self.region = Region::Page;
                    self.keyboard = false;
                    self.row = index;
                }
                (Task::none(), None)
            }
            Message::SearchPointer(pointer) => {
                self.region = Region::Search;
                self.search.point(pointer);
                (Task::none(), None)
            }
            Message::FindPointer(pointer) => {
                if let Some(find) = self.find.as_mut().filter(|find| find.open) {
                    find.query.point(pointer);
                    self.region = Region::Find;
                }
                (Task::none(), None)
            }
            Message::Scrolled(viewport) => {
                self.scrolled(viewport.absolute_offset().y);
                (Task::none(), None)
            }
            Message::Measured(measure) => (self.measured(measure), None),
            Message::Resized => (self.measure(), None),
            Message::Frame(now) => {
                if let Some(find) = &self.find
                    && !find.open
                    && !self.moving(now)
                {
                    self.find = None;
                }
                (Task::none(), None)
            }
        }
    }

    /// `key`, wherever the keyboard is: what a key event does once read, and what a scripted tour
    /// presses.
    pub fn press(&mut self, key: Key, content: &impl Content) -> (Task<Message>, Option<Event>) {
        match key {
            Key::PreviousSection => return (self.step_place(false), None),
            Key::NextSection => return (self.step_place(true), None),
            Key::Tab { shift } => {
                self.keyboard = true;
                return (self.tab(shift), None);
            }
            _ => {}
        }
        self.keyboard = true;
        match self.region {
            Region::Search => self.search_key(key),
            Region::Find => self.find_key(key, content),
            Region::Sidebar => (self.sidebar_key(&key), None),
            Region::Page => self.page_key(&key),
        }
    }

    fn search_key(&mut self, key: Key) -> (Task<Message>, Option<Event>) {
        let before = self.search.text.clone();
        match key {
            Key::Up => self.result = step(self.result, self.results, false),
            Key::Down => self.result = step(self.result, self.results, true),
            Key::Enter { .. } => return (Task::none(), self.result.map(Event::Open)),
            Key::Esc if self.search.is_empty() => {
                self.region = Region::Sidebar;
                self.cursor = Entry::Home;
            }
            Key::Find => self.search.select_all(),
            Key::Esc | Key::Clear => self.search.clear(),
            key => edit(&mut self.search, key),
        }
        let event = (self.search.text != before).then(|| Event::Query(self.search.text.clone()));
        (Task::none(), event)
    }

    fn find_key(&mut self, key: Key, content: &impl Content) -> (Task<Message>, Option<Event>) {
        let Some(find) = self.find.as_mut() else {
            self.region = Region::Sidebar;
            return (Task::none(), None);
        };
        let before = find.query.text.clone();
        match key {
            Key::Esc => return (self.close_find(), None),
            Key::Enter { shift: false } if find.matches.is_empty() && !find.query.is_empty() => {
                let query = find.query.text.clone();
                return (self.home_with(&query), Some(Event::Query(query)));
            }
            Key::Down | Key::Enter { shift: false } => return (self.step_match(true), None),
            Key::Up | Key::Enter { shift: true } => return (self.step_match(false), None),
            Key::Find => find.query.select_all(),
            Key::Clear => find.query.clear(),
            key => edit(&mut find.query, key),
        }
        if find.query.text == before {
            return (Task::none(), None);
        }
        (self.requery(content), None)
    }

    fn sidebar_key(&mut self, key: &Key) -> Task<Message> {
        let entries = self.entries();
        let at = entries.iter().position(|&e| e == self.cursor).unwrap_or(0);
        let last = entries.len() - 1;
        let to = match key {
            Key::Up => Some(if at == 0 { last } else { at - 1 }),
            Key::Down => Some(if at == last { 0 } else { at + 1 }),
            Key::Text(t) if t == "k" => Some(if at == 0 { last } else { at - 1 }),
            Key::Text(t) if t == "j" => Some(if at == last { 0 } else { at + 1 }),
            Key::Home => Some(0),
            Key::End => Some(last),
            Key::PageUp => Some(at.saturating_sub(self.sidebar_page())),
            Key::PageDown => Some((at + self.sidebar_page()).min(last)),
            Key::Right { .. } | Key::Enter { .. } => return self.enter_page(),
            Key::Text(t) if t == "l" => return self.enter_page(),
            Key::Find => return self.open_find(),
            Key::Text(t) if t == "/" => return self.open_find(),
            _ => None,
        };
        match to {
            Some(to) => self.pick(entries[to]),
            None => Task::none(),
        }
    }

    fn page_key(&mut self, key: &Key) -> (Task<Message>, Option<Event>) {
        let Place::Section(section) = self.place else {
            self.region = Region::Search;
            return (Task::none(), None);
        };
        let count = self.outline.row_count(section);
        let at = self.row.min(count.saturating_sub(1));
        let paged = |forward| {
            self.measure
                .as_ref()
                .and_then(|m| m.page_from(at, forward))
                .unwrap_or(if forward {
                    (at + 1).min(count.saturating_sub(1))
                } else {
                    at.saturating_sub(1)
                })
        };
        let to = match key {
            Key::Up => step(Some(at), count, false),
            Key::Down => step(Some(at), count, true),
            Key::Text(t) if t == "k" => step(Some(at), count, false),
            Key::Text(t) if t == "j" => step(Some(at), count, true),
            Key::Home => Some(0),
            Key::End => count.checked_sub(1),
            Key::PageUp => Some(paged(false)),
            Key::PageDown => Some(paged(true)),
            Key::Left { .. } | Key::Esc => return (self.back_to_sidebar(), None),
            Key::Text(t) if t == "h" => return (self.back_to_sidebar(), None),
            Key::Enter { shift: false } => return (Task::none(), self.row().map(Event::Press)),
            Key::Space => return (Task::none(), self.row().map(Event::Toggle)),
            Key::Find => return (self.open_find(), None),
            Key::Text(t) if t == "/" => return (self.open_find(), None),
            _ => None,
        };
        let Some(to) = to else {
            return (Task::none(), None);
        };
        self.row = to;
        (self.reveal_or_wait(to), None)
    }

    /// How many of the sidebar's rows show whole: what Page Up and Page Down move by.
    fn sidebar_page(&self) -> usize {
        let pitch = f32::from(view::SECTION_ROW.px() + view::ROW_GAP.px());
        let inset = 2.0 * f32::from(view::SIDEBAR_INSET.px());
        self.measure
            .as_ref()
            .map_or(1, |m| (((m.sidebar - inset) / pitch).floor() as usize).max(1))
    }

    /// The regions Tab goes round, in reading order.
    fn regions(&self) -> Vec<Region> {
        match self.place {
            Place::Home => vec![Region::Search, Region::Sidebar],
            Place::Section(_) if self.find.as_ref().is_some_and(|find| find.open) => {
                vec![Region::Find, Region::Sidebar, Region::Page]
            }
            Place::Section(_) => vec![Region::Sidebar, Region::Page],
        }
    }

    fn tab(&mut self, back: bool) -> Task<Message> {
        let regions = self.regions();
        let at = regions.iter().position(|&r| r == self.region).unwrap_or(0);
        let n = regions.len();
        self.region = regions[if back { (at + n - 1) % n } else { (at + 1) % n }];
        if self.region == Region::Page {
            return self.reveal_or_wait(self.row);
        }
        Task::none()
    }

    /// The sidebar's rows, top to bottom: Home, then each section, the one showing with its
    /// sub-sections under it.
    fn entries(&self) -> Vec<Entry> {
        let mut entries = vec![Entry::Home];
        for section in 0..self.outline.sections.len() {
            entries.push(Entry::Section(section));
            if self.place == Place::Section(section) {
                entries.extend((0..self.outline.sub_count(section)).map(|sub| Entry::Sub(section, sub)));
            }
        }
        entries
    }

    /// Shows what `entry` names, the sidebar's cursor on it.
    fn pick(&mut self, entry: Entry) -> Task<Message> {
        self.cursor = entry;
        match entry {
            Entry::Home => {
                self.place = Place::Home;
                self.find = None;
                Task::none()
            }
            Entry::Section(section) if self.place == Place::Section(section) => self.scroll_pinned(0.0, 0),
            Entry::Section(section) => self.open_section(section),
            Entry::Sub(section, sub) => {
                let mut task = Task::none();
                if self.place != Place::Section(section) {
                    task = self.open_section(section);
                    self.cursor = entry;
                }
                let offset = self
                    .measure
                    .as_ref()
                    .and_then(|m| m.subs.get(sub).map(|&top| m.offset_for(top)));
                if let Some(offset) = offset {
                    return task.chain(self.scroll_pinned(offset, sub));
                }
                self.goal = Some(Goal::Sub(sub));
                self.spy = sub;
                task
            }
        }
    }

    /// Shows `section` from its top: the page fades and rises in, unless motion is reduced.
    fn open_section(&mut self, section: usize) -> Task<Message> {
        self.place = Place::Section(section);
        self.cursor = Entry::Section(section);
        self.spy = 0;
        self.pin = None;
        self.row = 0;
        self.find = None;
        self.measure = None;
        self.goal = None;
        self.offset = 0.0;
        self.changed = (!self.motion.is_reduced()).then(Instant::now);
        if self.region == Region::Search || self.region == Region::Find {
            self.region = Region::Sidebar;
        }
        self.scroll_task(0.0).chain(self.measure())
    }

    /// The place before or after this one, Home first, wrapping round.
    fn step_place(&mut self, forward: bool) -> Task<Message> {
        self.keyboard = true;
        let n = self.outline.sections.len() + 1;
        let at = match self.place {
            Place::Home => 0,
            Place::Section(section) => section + 1,
        };
        let to = if forward { (at + 1) % n } else { (at + n - 1) % n };
        if to == 0 {
            self.place = Place::Home;
            self.cursor = Entry::Home;
            self.find = None;
            self.region = Region::Search;
            return Task::none();
        }
        let region = self.region;
        let task = self.open_section(to - 1);
        if region == Region::Page {
            self.region = Region::Page;
        }
        task
    }

    fn enter_page(&mut self) -> Task<Message> {
        match (self.place, self.cursor) {
            (Place::Home, _) | (_, Entry::Home) => {
                self.region = Region::Search;
                Task::none()
            }
            (Place::Section(section), cursor) => {
                let sub = match cursor {
                    Entry::Sub(_, sub) => sub,
                    _ => self.spy,
                };
                self.region = Region::Page;
                self.row = self.outline.first_of_sub(section, sub).unwrap_or(0);
                self.reveal_or_wait(self.row)
            }
        }
    }

    fn back_to_sidebar(&mut self) -> Task<Message> {
        self.region = Region::Sidebar;
        if let Some(at) = self.row() {
            self.cursor = Entry::Sub(at.section, at.sub);
        }
        Task::none()
    }

    fn open_find(&mut self) -> Task<Message> {
        if self.place == Place::Home {
            self.region = Region::Search;
            return Task::none();
        }
        match self.find.as_mut() {
            Some(find) if find.open => {
                find.query.select_all();
            }
            _ => {
                self.find = Some(Find {
                    query: Query::default(),
                    matches: Vec::new(),
                    current: 0,
                    region: self.region,
                    offset: self.offset,
                    row: self.row,
                    open: true,
                    at: Instant::now(),
                });
            }
        }
        self.region = Region::Find;
        Task::none()
    }

    /// Closes the find box and puts the page and the keyboard back where they were as it opened.
    fn close_find(&mut self) -> Task<Message> {
        let Some(find) = self.find.as_mut() else {
            return Task::none();
        };
        find.open = false;
        find.at = Instant::now();
        let (region, offset, row) = (find.region, find.offset, find.row);
        self.region = if region == Region::Find {
            Region::Sidebar
        } else {
            region
        };
        self.row = row;
        self.scroll_free(offset)
    }

    /// Home, its field holding `query`, the find box gone.
    fn home_with(&mut self, query: &str) -> Task<Message> {
        self.find = None;
        self.place = Place::Home;
        self.cursor = Entry::Home;
        self.region = Region::Search;
        self.search = Query::new(query);
        Task::none()
    }

    fn requery(&mut self, content: &impl Content) -> Task<Message> {
        let Place::Section(section) = self.place else {
            return Task::none();
        };
        let Some(find) = self.find.as_mut() else {
            return Task::none();
        };
        find.current = 0;
        find.matches = if find.query.is_empty() {
            Vec::new()
        } else {
            self.outline
                .rows(section)
                .filter(|&at| content.matches(at, &find.query.text))
                .collect()
        };
        self.show_match()
    }

    fn step_match(&mut self, forward: bool) -> Task<Message> {
        let Some(find) = self.find.as_mut() else {
            return Task::none();
        };
        let n = find.matches.len();
        if n == 0 {
            return Task::none();
        }
        find.current = if forward {
            (find.current + 1) % n
        } else {
            (find.current + n - 1) % n
        };
        self.show_match()
    }

    /// Scrolls the find box's current match into view, the page's row on it.
    fn show_match(&mut self) -> Task<Message> {
        let Some(at) = self.find.as_ref().and_then(|find| find.matches.get(find.current)) else {
            return Task::none();
        };
        let Some(index) = self.outline.index_of(*at) else {
            return Task::none();
        };
        self.row = index;
        self.reveal_or_wait(index)
    }

    /// Brings row `index` into view, now or once the page is measured.
    fn reveal_or_wait(&mut self, index: usize) -> Task<Message> {
        let Some(measure) = &self.measure else {
            self.goal = Some(Goal::Row(index));
            return Task::none();
        };
        match measure.reveal(index, self.offset) {
            Some(offset) => self.scroll_free(offset),
            None => Task::none(),
        }
    }

    /// Scrolls to `offset`, the spy following.
    fn scroll_free(&mut self, offset: f32) -> Task<Message> {
        self.pin = None;
        self.offset = offset;
        self.spy();
        self.scroll_task(offset)
    }

    /// Scrolls to `offset` for sub-section `sub`, which stays current while the page stays there.
    fn scroll_pinned(&mut self, offset: f32, sub: usize) -> Task<Message> {
        self.pin = Some((offset, sub));
        self.offset = offset;
        self.spy = sub;
        self.scroll_task(offset)
    }

    fn scroll_task(&self, offset: f32) -> Task<Message> {
        iced::widget::operation::scroll_to(
            self.id.clone(),
            AbsoluteOffset {
                x: None,
                y: Some(offset),
            },
        )
    }

    fn scrolled(&mut self, offset: f32) {
        self.offset = offset;
        if self.pin.is_some_and(|(at, _)| (at - offset).abs() > 1.0) {
            self.pin = None;
        }
        self.spy();
    }

    fn measured(&mut self, measure: Measure) -> Task<Message> {
        self.offset = self.offset.clamp(0.0, measure.max);
        self.measure = Some(measure);
        match self.goal.take() {
            Some(Goal::Sub(sub)) => {
                let offset = self
                    .measure
                    .as_ref()
                    .and_then(|m| m.subs.get(sub).map(|&top| m.offset_for(top)))
                    .unwrap_or(0.0);
                self.scroll_pinned(offset, sub)
            }
            Some(Goal::Row(row)) => self.reveal_or_wait(row),
            None => {
                self.spy();
                Task::none()
            }
        }
    }

    /// The current sub-section from where the page is scrolled to, and the sidebar's cursor with
    /// it while it is on a sub-section.
    fn spy(&mut self) {
        self.spy = match (self.pin, &self.measure) {
            (Some((_, sub)), _) => sub,
            (None, Some(measure)) => measure.current(self.offset),
            (None, None) => self.spy,
        };
        if let Entry::Sub(section, _) = self.cursor {
            self.cursor = Entry::Sub(section, self.spy);
        }
    }

    /// The page's fade and rise at `now`: its opacity, and how far below its place it is.
    fn arrival(&self, now: Instant) -> (f32, f32) {
        let Some(at) = self.changed else {
            return (1.0, 0.0);
        };
        let t = self.motion.progress(
            Duration::Short,
            Easing::EmphasizedDecel,
            now.saturating_duration_since(at),
        );
        (t, (1.0 - t) * f32::from(RISE.px()))
    }

    /// The find box's fade and drop at `now`: its opacity, and how far above its place it is.
    /// Reduced motion keeps the fade and drops nothing.
    fn find_arrival(&self, now: Instant) -> (f32, f32) {
        let Some(find) = &self.find else {
            return (0.0, 0.0);
        };
        let t = self.motion.progress(
            Duration::Short,
            Easing::Standard,
            now.saturating_duration_since(find.at),
        );
        let shown = if find.open { t } else { 1.0 - t };
        let drop = if self.motion.is_reduced() {
            0.0
        } else {
            (1.0 - shown) * f32::from(RISE.px())
        };
        (shown, drop)
    }

    /// How the template needs row `at` shown at `now`.
    fn row_state(&self, at: RowAt, index: usize, now: Instant) -> RowState<'_> {
        let find = self.find.as_ref().filter(|find| find.open && !find.query.is_empty());
        let found = match find {
            None => Found::Off,
            Some(find) if find.matches.get(find.current) == Some(&at) => Found::Current,
            Some(find) if find.matches.contains(&at) => Found::Match,
            Some(_) => Found::Miss,
        };
        let landed = match self.landed {
            Some((row, since)) if row == at => {
                1.0 - self
                    .motion
                    .progress(Duration::Slow, Easing::Standard, now.saturating_duration_since(since))
            }
            _ => 0.0,
        };
        RowState {
            selected: self.region == Region::Page && self.row == index,
            found,
            query: find.map_or("", |find| find.query.text.as_str()),
            landed,
        }
    }
}

/// How far a page rises as it arrives, and the find box drops.
const RISE: crate::tokens::Space = crate::tokens::Space::S8;

/// The next (`forward`) or previous place of `count`, wrapping round; none of none.
fn step(at: Option<usize>, count: usize, forward: bool) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match at {
        None => 0,
        Some(at) if forward => (at + 1) % count,
        Some(at) => (at + count - 1) % count,
    })
}

/// `key` as an edit of `query`, when it is one.
fn edit(query: &mut Query, key: Key) {
    match key {
        Key::Text(text) => query.insert(&text),
        Key::Space => query.insert(" "),
        Key::Backspace { word } => query.backspace(word),
        Key::Delete { word } => query.delete(word),
        Key::Left { word } => query.left(word),
        Key::Right { word } => query.right(word),
        Key::Home => query.home(),
        Key::End => query.end(),
        Key::SelectAll => query.select_all(),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
