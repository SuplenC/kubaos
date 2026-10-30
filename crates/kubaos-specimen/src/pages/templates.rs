//! The templates page: `SidebarPage` filled with stand-in sections and rows, live in the window and
//! in its states for the screenshots.
//!
//! # Owns
//!
//! - The stand-in settings ([`outline`], their rows and values), the live sidebar page and its
//!   answers ([`Demo`]), and the page's frames: Home, a section, the find box with matches and with
//!   none, and the smallest window.
//!
//! # Not here
//!
//! - The template: `kubaos_ui::template::sidebar_page`.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use iced::Border;
use iced::Element;
use iced::Length;
use iced::Task;
use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::list::list;
use kubaos_ui::component::list::list_row;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::section;
use kubaos_ui::layout::stack;
use kubaos_ui::template::sidebar_page;
use kubaos_ui::template::sidebar_page::Content;
use kubaos_ui::template::sidebar_page::Entry;
use kubaos_ui::template::sidebar_page::Event;
use kubaos_ui::template::sidebar_page::HomeOutline;
use kubaos_ui::template::sidebar_page::HomeView;
use kubaos_ui::template::sidebar_page::Key;
use kubaos_ui::template::sidebar_page::Outline;
use kubaos_ui::template::sidebar_page::RowAt;
use kubaos_ui::template::sidebar_page::RowState;
use kubaos_ui::template::sidebar_page::SectionOutline;
use kubaos_ui::template::sidebar_page::SidebarPage;
use kubaos_ui::template::sidebar_page::SubOutline;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;

/// A stand-in setting: its name and its value.
type Setting = (&'static str, &'static str);

/// The stand-in settings, section by section, sub-section by sub-section, well by well.
const SETTINGS: &[&[&[&[Setting]]]] = &[
    &[
        &[&[("Light or dark", "Dark"), ("Switch at sunset", "On")]],
        &[&[("Wallpaper", "Dunes"), ("Motion", "On AC")]],
        &[
            &[("Accent", "From the wallpaper"), ("Cold accent", "Teal")],
            &[("Fine-tune roles", "Off")],
        ],
    ],
    &[
        &[&[("Arrangement", "2 screens"), ("Scale", "1.5×")]],
        &[&[("Automatic brightness", "On"), ("Panel power saving", "Off")]],
    ],
    &[
        &[&[("Charge limit", "80%"), ("Battery saver", "At 20%")]],
        &[
            &[("Turn screens off after", "5 min"), ("When the lid closes", "Sleep")],
            &[("Keep awake", "Off")],
        ],
        &[&[("Power profile", "Balanced"), ("Fans", "Quiet")]],
    ],
    &[
        &[&[("English (US)", "us"), ("Polish", "pl"), ("Japanese", "ja")]],
        &[&[("Command menu", "Super+Space"), ("Close a window", "Super+Q")]],
    ],
    &[
        &[&[("Wi-Fi", "Home-5G"), ("Ask to join", "On")]],
        &[&[("Proxy", "Off"), ("Local network", "Let through")]],
    ],
];

/// The stand-in sections and their sub-sections, shaped as [`SETTINGS`] is.
fn outline() -> Outline {
    let section = |icon, title, accent, lede, subs: &[&str], index: usize| {
        let mut outline = SectionOutline::new(icon, title, accent, lede);
        for (sub, name) in subs.iter().enumerate() {
            let mut shape = SubOutline::new(*name);
            for well in SETTINGS[index][sub] {
                shape = shape.well(well.len());
            }
            outline = outline.with_sub(shape);
        }
        outline
    };
    Outline {
        home: HomeOutline {
            label: "Home".into(),
            icon: Icon::ManageSearch,
            accent: "設定".into(),
            title: "Settings".into(),
            lede: "Everything about this machine, one search away.".into(),
            placeholder: "Search every setting".into(),
        },
        sections: vec![
            section(
                Icon::Wallpaper,
                "Appearance",
                "外観",
                "Looking good.",
                &["Mode", "Wallpaper", "Colour"],
                0,
            ),
            section(
                Icon::DesktopWindows,
                "Displays",
                "画面",
                "Screen time.",
                &["Screens", "Brightness"],
                1,
            ),
            section(
                Icon::Bolt,
                "Power",
                "電源",
                "Juice, managed.",
                &["Battery", "Sleep and lock", "Performance"],
                2,
            ),
            section(
                Icon::Keyboard,
                "Keyboard",
                "鍵盤",
                "Type cast.",
                &["Layouts", "Shortcuts"],
                3,
            ),
            section(
                Icon::Wifi,
                "Network",
                "通信",
                "Strings attached.",
                &["Wi-Fi", "Proxy"],
                4,
            ),
        ],
    }
}

fn setting(at: RowAt) -> Setting {
    SETTINGS[at.section][at.sub][at.well][at.row]
}

/// The stand-in's matching: a setting whose name holds what is typed, in any case. An app answers
/// with its own search.
struct Names;

impl Content for Names {
    fn matches(&self, at: RowAt, query: &str) -> bool {
        setting(at).0.to_lowercase().contains(&query.to_lowercase())
    }
}

/// The live sidebar page, and the settings turned on and off in it.
#[derive(Debug)]
pub struct Demo {
    pub page: SidebarPage,
    /// Home's results for what its field holds.
    results: Vec<RowAt>,
    /// Settings pressed with Enter or Space, which show a check.
    toggled: BTreeSet<RowAt>,
}

impl Default for Demo {
    fn default() -> Self {
        Demo {
            page: SidebarPage::new(outline()),
            results: Vec::new(),
            toggled: BTreeSet::new(),
        }
    }
}

impl Demo {
    pub fn update(&mut self, message: sidebar_page::Message) -> Task<Message> {
        let (task, event) = self.page.update(message, &Names);
        let task = task.map(Message::Template);
        match event {
            Some(Event::Query(query)) => {
                self.search(&query);
                task
            }
            Some(Event::Open(result)) => match self.results.get(result) {
                Some(&at) => task.chain(self.page.land(at).map(Message::Template)),
                None => task,
            },
            Some(Event::Press(at) | Event::Toggle(at)) => {
                if !self.toggled.remove(&at) {
                    self.toggled.insert(at);
                }
                task
            }
            None => task,
        }
    }

    fn search(&mut self, query: &str) {
        self.results = if query.is_empty() {
            Vec::new()
        } else {
            (0..SETTINGS.len())
                .flat_map(|section| self.page.outline().rows(section).collect::<Vec<_>>())
                .filter(|&at| Names.matches(at, query))
                .collect()
        };
        self.page.set_results(self.results.len());
    }

    pub fn view<'a>(&'a self, look: &Look) -> Element<'a, Message> {
        sidebar_page::view(
            &self.page,
            look,
            Message::Template,
            |look, home| self.results(look, home),
            |at, state, look| self.row(at, state, look),
        )
    }

    /// Home's results: the sections with nothing typed, the settings found otherwise.
    fn results<'a>(&'a self, look: &Look, home: HomeView<'a>) -> Element<'a, Message> {
        let outline = self.page.outline();
        if home.query.is_empty() {
            let rows = outline.sections.iter().enumerate().map(|(index, section)| {
                list_row(section.title.as_str())
                    .icon(section.icon)
                    .tile()
                    .on_press(Message::Template(sidebar_page::Message::Pick(Entry::Section(index))))
                    .view(look)
            });
            return list(look, rows, false);
        }
        let rows = self.results.iter().enumerate().map(|(index, &at)| {
            let (name, value) = setting(at);
            let path = format!(
                "{} › {}",
                outline.sections[at.section].title, outline.sections[at.section].subs[at.sub].title
            );
            list_row(name)
                .subtitle(path)
                .detail(value)
                .selected(home.result == Some(index))
                .view(look)
        });
        list(look, rows, false)
    }

    fn row<'a>(&'a self, at: RowAt, state: RowState<'a>, look: &Look) -> Element<'a, Message> {
        let (name, value) = setting(at);
        list_row(name)
            .detail(value)
            .checked(self.toggled.contains(&at))
            .selected(state.selected)
            .on_press(Message::Template(sidebar_page::Message::Point(at)))
            .view(look)
    }
}

/// The template in its states, each at its window's size: what the screenshots and the geometry
/// check see. Built once, since the frames borrow them.
static FRAMES: LazyLock<[Frame; 5]> = LazyLock::new(|| {
    let on_power = [Key::NextSection, Key::NextSection, Key::NextSection];
    let found = pressed(&[&on_power, &[Key::Find, Key::Text("o".into()), Key::Down]]);
    let missing = pressed(&[&on_power, &[Key::Find, Key::Text("hdmi".into())]]);
    [
        Frame {
            title: "Home",
            lede: "The window opens here, the keyboard in the search field.",
            demo: pressed(&[]),
            size: (Size::AppWidth, Size::AppHeight),
        },
        Frame {
            title: "A section",
            lede: "Power, its first sub-section current.",
            demo: pressed(&[&on_power]),
            size: (Size::AppWidth, Size::AppHeight),
        },
        Frame {
            title: "Find, with matches",
            lede: "\"o\" typed, on its second match.",
            demo: found,
            size: (Size::AppWidth, Size::AppHeight),
        },
        Frame {
            title: "Find, with none",
            lede: "Enter searches every setting from Home.",
            demo: missing,
            size: (Size::AppWidth, Size::AppHeight),
        },
        Frame {
            title: "The smallest window",
            lede: "720 × 480: the whole sidebar beside a page padded 16, the keyboard in it.",
            demo: pressed(&[&[Key::NextSection, Key::Tab { shift: false }]]),
            size: (Size::AppMinWidth, Size::AppMinHeight),
        },
    ]
});

/// A state of the template, and the window it is shown in.
#[derive(Debug)]
struct Frame {
    title: &'static str,
    lede: &'static str,
    demo: Demo,
    size: (Size, Size),
}

pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    stack(look, Space::S48)
        .extend(FRAMES.iter().map(|frame| {
            let (width, height) = frame.size;
            section(
                look,
                frame.title,
                Some(frame.lede),
                framed(look, &frame.demo, width, height),
            )
        }))
        .into()
}

/// A fresh demo after `keys`, in order.
fn pressed(keys: &[&[Key]]) -> Demo {
    let mut demo = Demo::default();
    for key in keys.iter().flat_map(|keys| keys.iter()) {
        let (_, event) = demo.page.press(key.clone(), &Names);
        if let Some(Event::Query(query)) = event {
            demo.search(&query);
        }
    }
    demo.page.settle();
    demo
}

/// `demo` in a window `width` × `height`, edged with the soft hairline.
fn framed<'a>(look: &Look, demo: &'static Demo, width: Size, height: Size) -> Element<'a, Message> {
    let edge = look.outline_soft();
    let hairline = look.hairline();
    widget::container(demo.view(look))
        .width(f32::from(width.px()))
        .height(f32::from(height.px()))
        .clip(true)
        .style(move |_theme| widget::container::Style {
            border: Border {
                color: edge,
                width: hairline,
                ..Border::default()
            },
            ..widget::container::Style::default()
        })
        .into()
}

/// The live template, as large as the window lets it be: resize the window to see it reflow.
pub fn live<'a>(demo: &'a Demo, look: &Look) -> Element<'a, Message> {
    widget::container(demo.view(look))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
