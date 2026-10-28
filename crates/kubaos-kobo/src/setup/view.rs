//! The setup screen: the window is one panel, the cards on the left, the selected card on the
//! right. Built from `kubaos-ui`'s layout and components, with its tokens.
//!
//! Copy per docs/BRAND.md: attitude around the facts, never in the sentence that matters (the
//! MOK password, the blue screen).

use iced::Background;
use iced::Length;
use iced::widget;
use iced::widget::container;
use kubaos_core::Level;
use kubaos_core::setup::BwMethod;
use kubaos_core::setup::CardId;
use kubaos_core::setup::CardStatus;
use kubaos_core::setup::Facts;
use kubaos_core::setup::MokState;
use kubaos_core::setup::PromptKind;
use kubaos_core::setup::State;
use kubaos_core::setup::finger_name;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::component::banner::BannerTone;
use kubaos_ui::component::banner::banner;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::card::card;
use kubaos_ui::component::check::Tick;
use kubaos_ui::component::check::checkbox;
use kubaos_ui::component::check::radio;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::error::Level as LogLevel;
use kubaos_ui::component::error::error_block;
use kubaos_ui::component::error::log;
use kubaos_ui::component::field::State as FieldState;
use kubaos_ui::component::field::field;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::component::list::scrolling_list;
use kubaos_ui::component::progress::bar;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::component::step::StepState;
use kubaos_ui::component::step::step;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::flow;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::header;
use kubaos_ui::layout::page_clear;
use kubaos_ui::layout::row;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::stack;
use kubaos_ui::motion::Motion;
use kubaos_ui::text::text;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Act;
use super::FLASH;
use super::Outcome;
use super::SF;
use super::Server;
use super::Setup;
use super::method_label;
use crate::app::Message;
use crate::theme::jp;
use crate::ui;
use crate::ui::El;

fn click(f: SF) -> Message {
    Message::Setup(super::Msg::Click(f))
}

pub fn view(s: &Setup) -> El<'_> {
    let look = s.look();
    let mut layers = widget::stack![panel(s, &look)].width(Length::Fill).height(Length::Fill);
    if s.backend.is_demo() {
        layers = layers.push(
            widget::container(chip(&look, ChipKind::Cold, "Demo", Some(Icon::Bolt), None, None))
                .width(Length::Fill)
                .align_right(Length::Fill)
                .padding(Inset::all(Space::S16).padding()),
        );
    }
    if let Some(toast) = ui::toast(&look, s.toast.as_ref(), s.now) {
        layers = layers.push(toast);
    }
    crate::pointer::idle(layers, s.pointer_idle).into()
}

/// The window is the panel. Setup runs as a floating window in Hyprland, which draws the border,
/// the rounding and the shadow, with the desktop right there around it. So no photo and no card
/// inside a card: a solid surface, edge to edge.
fn panel<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let surface = look.colors.surface;
    widget::container(row(look, Gap::NONE).push(left(s, look)).push(detail(s, look)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style::default().background(surface))
        .into()
}

fn card_icon(id: CardId) -> Icon {
    match id {
        CardId::Tuxedo => Icon::Computer,
        CardId::Fingerprint => Icon::Fingerprint,
        CardId::Yubikey => Icon::Key,
        CardId::Bitwarden => Icon::Lock,
        CardId::Git => Icon::Code,
        // The link runs over the Wi-Fi.
        CardId::Phone => Icon::Wifi,
    }
}

/// A card's state as a chip, as its row in the list shows it: to do plain, done tonal with a
/// check, a reboot on the accent.
fn state_chip<'a>(look: &Look, st: State) -> El<'a> {
    let (kind, glyph, word) = match st {
        State::Done => (ChipKind::Accent, Some(Icon::Check), "Done"),
        State::Needed => (ChipKind::Neutral, None, "To do"),
        State::Reboot => (ChipKind::Filled, Some(Icon::RestartAlt), "Reboot"),
        State::Waiting => (ChipKind::Neutral, Some(Icon::Schedule), "Waiting"),
        State::Absent => (ChipKind::Neutral, None, "Not here"),
    };
    chip(look, kind, word, glyph, None, None)
}

fn left<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let c = look.colors;
    let seconds = (s.now - s.epoch).as_secs_f32();
    let cards = left_cards(s, seconds, look);
    let online = left_online(s, look);
    let footer = left_footer(s, look);
    let detecting: El = if s.detecting() && s.status.is_some() {
        row(look, Space::S8)
            .center()
            .push(spinner(look, IconSize::S16, seconds, c.primary))
            .push(text("Checking again…", Type::Small, c.on_surface_variant))
            .into()
    } else {
        widget::Space::new().into()
    };
    let rail = color::alpha(c.surface_container_low, Opacity::Well.value());
    widget::container(page_clear(
        stack(look, Gap::NONE)
            .height(Extent::Fill)
            .push(header(
                look,
                jp::SETUP,
                "First boot",
                "The things the install couldn't do without you or the hardware. Each one checks itself, so done \
                 stays done.",
            ))
            .push(gap(Space::S20))
            .push(cards)
            .push(gap(Space::S12))
            .push(detecting)
            .push(gap(Space::S12))
            .push(online)
            .push(gap(Space::S16))
            .push(footer)
            .push(gap(Space::S16))
            .push(flow(
                look,
                Space::S16,
                [
                    ("↑ ↓", "pick a card"),
                    ("Tab", "next control"),
                    ("Enter", "open, press"),
                    ("Esc", "back to the cards"),
                ]
                .map(|(key, what)| ui::hint(look, key, what)),
            )),
    ))
    .width(side_width(s, look))
    .height(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(rail)),
        ..container::Style::default()
    })
    .into()
}

/// The side panel's width: a third of the window, so the card's page beside it keeps the room, and
/// never wider than a column of text. On a whole device pixel.
fn side_width(s: &Setup, look: &Look) -> f32 {
    let third = s.size.width / 3.0;
    look.scale.snap(third.min(f32::from(Size::Column.px())))
}

/// The cards, or a spinner while the first detection runs. The cards take the room the panel has
/// left and scroll in their well past it: their summaries wrap, so six of them can outgrow it.
fn left_cards<'a>(s: &'a Setup, seconds: f32, look: &Look) -> El<'a> {
    let c = look.colors;
    match &s.status {
        None => widget::container(
            row(look, Space::S12)
                .center()
                .padding(Inset::xy(Space::S12, Space::S12))
                .push(spinner(look, IconSize::S20, seconds, c.primary))
                .push(text(
                    "Checking what this machine has…",
                    Type::Body,
                    c.on_surface_variant,
                )),
        )
        .height(Length::Fill)
        .into(),
        Some(st) => scrolling_list(
            st.cards
                .iter()
                .map(|card| {
                    widget::container(card_row(s, card, look))
                        .id(super::card_item(card.id))
                        .into()
                })
                .collect::<Vec<_>>(),
            s.is_focused(SF::Cards),
        )
        .id(super::CARDS_SCROLL)
        .height(Extent::Fill)
        .view(look),
    }
}

/// Whether the machine is online, once detection has said.
fn left_online<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    match &s.status {
        Some(st) if st.online => chip(look, ChipKind::Neutral, "Online", Some(Icon::Wifi), None, None),
        Some(_) => chip(
            look,
            ChipKind::Neutral,
            "Offline: drivers, fprintd and Bitwarden need the internet",
            Some(Icon::WifiOff),
            None,
            None,
        ),
        None => widget::Space::new().into(),
    }
}

/// The way out, with "Reboot to finish" beside it while a card waits on a reboot.
fn left_footer<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let c = look.colors;
    let reboot = s.reboot_pending();
    // Beside "Reboot to finish" the way out stays quiet.
    let (finish_label, kind) = if reboot {
        ("Done for now", Kind::Text)
    } else if s.settled() {
        ("All set", Kind::Filled)
    } else {
        ("Done for now", Kind::Tonal)
    };
    let finish = button(finish_label)
        .kind(kind)
        .icon(Icon::CheckCircle)
        .on_press(click(SF::Finish))
        .focused(s.is_focused(SF::Finish))
        .view(look);
    if reboot {
        stack(look, Space::S12)
            .push(text(
                s.reboot_nudge().unwrap_or_default(),
                Type::Small,
                c.on_surface_variant,
            ))
            .push(
                row(look, Space::S12)
                    .center()
                    .push(
                        button("Reboot to finish")
                            .icon(Icon::RestartAlt)
                            .on_press(click(SF::RebootAll))
                            .focused(s.is_focused(SF::RebootAll))
                            .view(look),
                    )
                    .push(finish),
            )
            .into()
    } else {
        finish
    }
}

/// A card as a step: to do, done, or waiting on a reboot, each its own look, the reboot on the
/// accent. Its summary wraps, so it is read whole.
fn card_row<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> El<'a> {
    let running = s.run.as_ref().is_some_and(|r| r.card == c.id && r.running());
    let row = step(c.id.title())
        .subtitle(c.summary.clone())
        .icon(card_icon(c.id))
        .selected(c.id == s.sel)
        .on_press_maybe(Some(Message::Setup(super::Msg::Pick(c.id))));
    let row = if running {
        row.words("Running", Some(Icon::ProgressActivity))
    } else {
        match c.state {
            State::Done => row.state(StepState::Done),
            State::Needed => row,
            State::Reboot => row.state(REBOOT),
            State::Waiting => row.words("Waiting", Some(Icon::Schedule)),
            State::Absent => row.words("Not here", None),
        }
    };
    row.view(look)
}

/// A card waiting on a reboot: the action it needs, on the accent.
const REBOOT: StepState = StepState::Action {
    label: "Reboot",
    glyph: Icon::RestartAlt,
};

// -------------------------------------------------------------- detail --

fn detail<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let shown = Motion::default().progress(tokens::Duration::Long, Easing::EmphasizedDecel, s.now - s.changed);
    let look = look.faded(shown);
    // The card's page rises into place as it fades in, a whole device pixel at a time.
    let rise = look.scale.snap((1.0 - shown) * f32::from(Space::S16.px()));
    let inner: El = match s.selected() {
        None => widget::Space::new().into(),
        Some(c) => card_detail(s, c, &look),
    };
    widget::container(page_clear(inner))
        .padding(iced::Padding::ZERO.top(rise))
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .into()
}

fn para<'a>(look: &Look, words: impl Into<String>) -> El<'a> {
    text(words.into(), Type::Body, look.colors.on_surface_variant)
}

/// A warning: something that goes wrong unless it is done right, as a banner with its symbol, its
/// first line and the rest under it.
fn warning<'a>(look: &Look, glyph: Icon, head: &'a str, lines: Vec<String>) -> El<'a> {
    let mut lines = lines.into_iter();
    banner(BannerTone::Warning, glyph, head, lines.next().unwrap_or_default())
        .paragraphs(lines)
        .view(look)
}

/// The serious kind of note: no jokes, a card with its symbol and its first line.
fn notice<'a>(look: &Look, glyph: Icon, head: &'a str, lines: Vec<String>) -> El<'a> {
    let c = look.colors;
    card(
        look,
        stack(look, Space::S8)
            .push(
                row(look, Space::S8)
                    .center()
                    .push(icon(look, glyph, IconSize::S20, c.primary))
                    .push(text(head, Type::Subheading, c.on_surface)),
            )
            .extend(lines.into_iter().map(|line| para(look, line))),
    )
}

fn card_detail<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> El<'a> {
    let colours = look.colors;
    let head = row(look, Space::S16)
        .center()
        .push(icon(
            look,
            card_icon(c.id),
            IconSize::S48,
            if c.state == State::Absent {
                colours.outline
            } else {
                colours.primary
            },
        ))
        .push(
            stack(look, Space::S4)
                .push(
                    row(look, Space::S12)
                        .center()
                        .push(text(c.id.title(), Type::Title, colours.on_surface))
                        .push(state_chip(look, c.state)),
                )
                .push(text(c.summary.as_str(), Type::Body, colours.on_surface_variant)),
        );

    let mut parts = stack(look, Space::S16).padding(Inset::NONE.right(Space::S16));
    // What's happening right now goes first: the touch prompt has to be on screen, not under the
    // explanations.
    if let Some(run) = s.shown_run() {
        parts = parts.push(run_view(s, run, look));
    }
    parts = parts.extend(explain(s, c, look));
    let fields = s.fields();
    if !fields.is_empty() && !s.running() {
        parts = parts.push(form(s, &fields, look));
    }
    if let Some(run) = s.shown_run() {
        parts = parts.push(log_view(s, run, look));
    }
    let apart = Space::S24;
    stack(look, apart)
        .height(Extent::Fill)
        .push(head)
        .push(
            scroll(look, parts)
                .id("setup-detail")
                .height(Extent::Fill)
                .room_above(apart)
                .room_below(apart),
        )
        .push(actions(s, c, look))
        .into()
}

/// What the card is about, for its state. Facts first.
#[expect(
    clippy::too_many_lines,
    reason = "One arm per card and state, each its few sentences: every card's words, read in one place."
)]
fn explain<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> Vec<El<'a>> {
    let mut v: Vec<El> = vec![];
    match (&c.facts, c.state) {
        (_, State::Absent) => v.push(para(
            look,
            match c.id {
                CardId::Tuxedo => "Nothing to install. The TUXEDO drivers only make sense on a TUXEDO.",
                CardId::Fingerprint => {
                    "No reader that fprintd knows. Plugged one in? Check again from a fresh start of setup."
                }
                _ => "Nothing to do here.",
            },
        )),
        (
            Facts::Tuxedo {
                product,
                secure_boot,
                drivers,
                mok,
                ..
            },
            st,
        ) => {
            let machine = if product.is_empty() {
                "This TUXEDO".to_string()
            } else {
                format!("Your {product}")
            };
            if matches!(st, State::Needed | State::Waiting) && !drivers {
                v.push(para(
                    look,
                    format!(
                        "{machine} wants TUXEDO's own kernel drivers for the keyboard backlight, the fans and the \
                         power limits. Setup adds TUXEDO's apt repository and installs tuxedo-drivers. No Control \
                         Center: KubaOS runs power profiles instead, one less app phoning home."
                    ),
                ));
            }
            if s.wants_mok() || matches!(mok, Some(MokState::Pending)) {
                v.push(mok_notice(c.state, look));
            } else if *secure_boot == Some(false) && c.state != State::Done {
                v.push(para(
                    look,
                    "Secure Boot is off, so the drivers load as they are. No key to enroll.",
                ));
            }
            if c.state == State::Reboot && !matches!(mok, Some(MokState::Pending)) {
                v.push(para(look, "They load on the next boot. Reboot when it suits you."));
            }
        }
        (Facts::Fingerprint { reader, fingers, .. }, st) => {
            let reader = reader.clone().unwrap_or_else(|| "The reader".into());
            v.push(para(
                look,
                if st == State::Done {
                    format!(
                        "{reader}. sudo, polkit dialogs and the tty login take your finger first and wait 10 \
                         seconds before they ask for the password. Enrolled: {}.",
                        fingers.iter().map(|f| finger_name(f)).collect::<Vec<_>>().join(", ")
                    )
                } else {
                    format!(
                        "{reader}. Enroll a finger and sudo and password dialogs take it instead of your password. \
                         The password keeps working, so a cut finger is a bad day, not a lockout. You'll touch the \
                         reader eight-ish times; move the finger a little each time."
                    )
                },
            ));
        }
        (
            Facts::Yubikey {
                plugged,
                enrolled,
                pin,
                fingerprints,
                ..
            },
            st,
        ) => {
            v.push(para(
                look,
                "Your YubiKey and the finger on its sensor stand in for the password at sudo, polkit and the lock \
                 screen. The password always keeps working. Enroll a second key as the backup: keys get lost.",
            ));
            if st == State::Needed && (*pin == Some(false) || *fingerprints == Some(false)) {
                v.push(warning(
                    look,
                    Icon::Key,
                    "The key needs its own setup first",
                    vec![
                        "It has no FIDO2 PIN or no fingerprint yet, and it won't enroll without both. That part \
                         talks to the key directly, so it runs in a terminal:"
                            .into(),
                        "ykman fido access change-pin, then ykman fido fingerprints add right-index (and a finger \
                         from the other hand)."
                            .into(),
                        "Come back and press Check again.".into(),
                    ],
                ));
            } else if st == State::Waiting && plugged.is_empty() && *enrolled == 0 {
                v.push(para(look, "Plug it in, then Check again. Or skip it; I'll be here."));
            }
        }
        (
            Facts::Bitwarden {
                desktop,
                configured,
                server,
                email,
                login,
                folders,
                overview,
                agent,
                ..
            },
            st,
        ) => {
            if let (Some(server), true) = (server, s.bw_logged_in()) {
                let who = email.as_deref().unwrap_or("someone");
                v.push(para(
                    look,
                    format!("Logged in to {server} as {who}, the vault {login}."),
                ));
            }
            if st == State::Done {
                v.push(para(
                    look,
                    "SSH public keys and Host blocks, git commit signing, the gh token and every project's env came \
                     from the vault. Changed the vault? Pull again; kubaos bitwarden refresh in a terminal does the \
                     same.",
                ));
            }
            if *configured {
                v.push(keys_overview(folders, overview, agent.as_ref(), look));
            }
            if st != State::Done {
                v.push(para(
                    look,
                    "Log in once and setup pulls what the dotfiles expect from the vault: SSH public keys and Host \
                     blocks, git commit signing, the gh token and your projects' env. The private keys never touch \
                     the disk.",
                ));
                v.push(para(
                    look,
                    "bw can't use the YubiKey Bio for two-step login: use the authenticator app's code. Only the \
                     email and server are saved. The password and the vault session live in memory until this is \
                     done.",
                ));
            }
            if *desktop && (*configured || st == State::Done) {
                v.push(para(
                    look,
                    "Bitwarden desktop starts when you close setup. Last bit by hand: log in there, Settings, turn \
                     on \"Enable SSH agent\". It has no config file for that.",
                ));
            }
        }
        (Facts::Phone { link }, st) => v.extend(phone_explain(s, link, st, look)),
        (Facts::Git { .. }, st) => v.push(para(
            look,
            if st == State::Done {
                "git commits with this name and email. Change them here any time; they live in \
                 ~/.config/git-identity/identity.gitconfig, outside the dotfiles repo."
            } else {
                "The name and email git puts on every commit. The dotfiles don't carry one, so the repo stays \
                 yours to share. Saved to ~/.config/git-identity/identity.gitconfig, which the git config \
                 includes."
            },
        )),
        _ => {}
    }
    v
}

/// The phone card: what the link is, the steps on the phone, the pairing request, and what works
/// each way once paired. iPhone's limits are said plainly: it never sends its clipboard on its own.
fn phone_explain<'a>(s: &Setup, link: &kubaos_core::phone::Status, st: State, look: &Look) -> Vec<El<'a>> {
    let mut v: Vec<El> = vec![];
    let laptop = link.name.clone().unwrap_or_else(|| "this laptop".into());
    if !link.on {
        v.push(para(
            look,
            "Share the clipboard and files between this laptop and your phone (iPhone, iPad or Android) with KDE \
             Connect. It stays off until you set it up: until then nothing on this laptop listens for phones or \
             announces itself on the network.",
        ));
        v.push(notice(
            look,
            Icon::Download,
            "First, on the phone",
            vec![
                "iPhone and iPad: KDE Connect from the App Store.".into(),
                "Android: KDE Connect from Google Play or F-Droid.".into(),
                format!("Same Wi-Fi as the laptop. It will show up there as {laptop}."),
            ],
        ));
        return v;
    }
    if !link.running {
        v.push(para(
            look,
            "The phone link is on, but KDE Connect isn't running. Start it again; if it stops once more, \
             journalctl --user -u kubaos-phone says why.",
        ));
        return v;
    }
    if let Some(asking) = link.request() {
        let mut lines = vec![];
        if let Some(key) = &asking.verification_key {
            lines.push(format!("The phone shows a key too. It must be the same: {key}."));
        }
        lines.push("Accept only a phone you're holding: other KDE Connect devices on this network can ask too.".into());
        v.push(warning(look, Icon::Key, "A phone wants to pair", lines));
    } else if st == State::Waiting {
        v.push(notice(
            look,
            Icon::Wifi,
            "On the phone",
            vec![
                format!("Open KDE Connect. This laptop shows up as {laptop}: tap it, then Request pairing."),
                "The request shows up here within a few seconds. No need to press anything.".into(),
            ],
        ));
        if s.phone_by_address() {
            let address = link.addresses.first().map_or("this laptop's address", String::as_str);
            v.push(notice(
                look,
                Icon::Public,
                "No phone yet?",
                vec![
                    "Some networks, at offices and hotels, keep devices from finding each other. Restarting the app \
                     on the phone helps sometimes."
                        .into(),
                    format!("Or add the laptop by its address in the phone's app (Add devices by IP): {address}."),
                ],
            ));
        }
    }
    let others: Vec<&str> = link.others().map(|d| d.name.as_str()).collect();
    if st != State::Done && !others.is_empty() {
        v.push(para(
            look,
            format!("Also on this network: {}. Not yours? Leave them be.", others.join(", ")),
        ));
    }
    if st == State::Done {
        v.push(notice(
            look,
            Icon::CheckCircle,
            "What works each way",
            vec![
                "Laptop to phone: copy here and it's on the phone's clipboard. Files go from the Command menu's Send \
                 a file to the phone, and Ring the phone finds it."
                    .into(),
                "Phone to laptop on iPhone: open KDE Connect and tap Send clipboard, or share to KDE Connect. iOS \
                 never lets an app read the clipboard in the background, so it is always by hand."
                    .into(),
                "Phone to laptop on Android: the same from the app, its quick tile or its notification.".into(),
                "That's all it can do here: no moving the pointer, typing, running commands or browsing the \
                 laptop's files. KubaOS keeps those KDE Connect plugins off."
                    .into(),
                "Try it: Send a test copy and paste on the phone. Then send something back and press Check what the \
                 phone sent."
                    .into(),
            ],
        ));
    }
    v
}

/// The SSH keys the last pull found, read-only: what each became, and what serves them.
fn keys_overview<'a>(
    folders: &[String],
    overview: &kubaos_core::bitwarden::Overview,
    agent: Option<&kubaos_core::bitwarden::Agent>,
    look: &Look,
) -> El<'a> {
    use kubaos_core::bitwarden::Agent;
    let scope = kubaos_core::bitwarden::scope(folders);
    let mut lines: Vec<String> = overview
        .keys
        .iter()
        .map(|k| {
            let what = match (&k.note, k.block) {
                (_, true) => format!("Host block for {}", k.hosts.join(", ")),
                (Some(why), false) => format!("key only: {why}"),
                (None, false) => "key only".to_string(),
            };
            let signs = if k.signing { "; signs your commits" } else { "" };
            format!("{} ({}): {what}{signs}.", k.name, k.folder)
        })
        .collect();
    if lines.is_empty() {
        lines.push(format!("No SSH keys in {scope}."));
    }
    lines.push(match agent {
        None | Some(Agent::Off) => {
            "The desktop app's SSH agent is off. In Bitwarden desktop: Settings, Enable SSH agent.".into()
        }
        Some(Agent::Locked) => "The agent runs with no keys: unlock Bitwarden desktop.".into(),
        Some(Agent::Serving { keys }) => format!("The agent serves {} key(s).", keys.len()),
        Some(Agent::Dead) => "The agent's socket doesn't answer: restart Bitwarden desktop.".into(),
    });
    lines.push(
        "A key gets a Host block from a text field hosts (comma-separated; hostname, user and port optional), and \
         signs commits with a field signing set to true."
            .into(),
    );
    notice(look, Icon::Key, "SSH keys", lines)
}

/// The vault's folders as checkboxes: none ticked is the whole vault.
fn folder_ticks<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let focused = s.is_focused(SF::Folders) && !s.running();
    let boxes = s.bw_folders().into_iter().enumerate().map(|(i, name)| {
        let on = s.form.folders.iter().any(|f| f == name);
        checkbox(
            look,
            name.to_string(),
            if on { Tick::Full } else { Tick::Empty },
            (!s.running()).then(|| Message::Setup(super::Msg::Folder(name.to_string()))),
            focused && i == s.form.folder_at,
        )
    });
    flow(look, Space::S20, boxes)
}

/// Secure Boot and the blue screen. Dead serious: this is the one that leaves you without drivers
/// if it goes wrong.
fn mok_notice<'a>(st: State, look: &Look) -> El<'a> {
    let steps = "Enroll MOK → Continue → Yes → type the password → Reboot.";
    if st == State::Reboot {
        warning(
            look,
            Icon::RestartAlt,
            "Reboot, then enroll the key on the blue screen",
            vec![
                "Before KubaOS starts, a blue screen called MokManager appears. It waits about 10 seconds; press a \
                 key to stop it."
                    .into(),
                steps.into(),
                "Type the one-time password you picked here. If you miss the screen, the drivers won't load: open \
                 setup again and enroll the key again."
                    .into(),
            ],
        )
    } else {
        warning(
            look,
            Icon::Lock,
            "Secure Boot is on",
            vec![
                "The drivers are built on this machine, so they're signed with a key made here, and the firmware \
                 has to be told to trust it. Pick a one-time password below."
                    .into(),
                format!("After the next reboot a blue screen called MokManager appears before KubaOS starts. {steps}"),
                "You need this password exactly once, on that screen. It assumes a US keyboard, so letters and \
                 digits only."
                    .into(),
            ],
        )
    }
}

/// A field of the card's form.
fn input<'a>(s: &'a Setup, f: SF, placeholder: &'a str, glyph: Option<Icon>, look: &Look) -> El<'a> {
    input_state(s, f, placeholder, glyph, FieldState::Normal, look)
}

/// [`input`] in `state`: invalid shows the error colour, and the reason goes under it. While a card
/// runs, every field is locked.
fn input_state<'a>(
    s: &'a Setup,
    f: SF,
    placeholder: &'a str,
    glyph: Option<Icon>,
    state: FieldState,
    look: &Look,
) -> El<'a> {
    let state = if s.running() { FieldState::Locked } else { state };
    let mut input = field(s.field_value(f), placeholder)
        .state(state)
        .blink_from(s.focus_at)
        .on_pointer(move |pointer| Message::Setup(super::Msg::Pointer(f, pointer)));
    if let Some(glyph) = glyph {
        input = input.icon(glyph);
    }
    if f.is_secret() && !s.show_secrets {
        input = input.secret();
    }
    if s.is_focused(f) {
        input = input.caret(s.caret);
    }
    input.view(look)
}

/// One of a few choices, as radio buttons: Left and Right (or Space) move between them, a click
/// picks one. The picked one has the focus tube while the choice has the keyboard.
fn choice<'a>(s: &'a Setup, f: SF, options: Vec<(&'a str, bool, Message)>, look: &Look) -> El<'a> {
    let focused = s.is_focused(f);
    let radios = options
        .into_iter()
        .map(|(label, on, pick)| radio(look, label, on, (!s.running()).then_some(pick), focused && on));
    flow(look, Space::S20, radios)
}

#[expect(
    clippy::too_many_lines,
    reason = "One arm per field, each its labelled input: every form field of setup, read in one place."
)]
fn form<'a>(s: &'a Setup, fields: &[SF], look: &Look) -> El<'a> {
    let f = &s.form;
    let mut col = stack(look, Space::S16);
    for &sf in fields {
        let el: El = match sf {
            SF::Mok => labelled(
                look,
                "One-time password for the blue screen",
                None,
                input(s, sf, "8 to 16 letters and digits", Some(Icon::Key), look),
                None,
            ),
            SF::MokAgain => labelled(
                look,
                "Same again",
                None,
                input_state(
                    s,
                    sf,
                    "",
                    Some(Icon::Key),
                    if ui::mismatched(&f.mok, &f.mok_again) {
                        FieldState::Invalid
                    } else {
                        FieldState::Normal
                    },
                    look,
                ),
                ui::confirm_note(look, &f.mok, &f.mok_again),
            ),
            SF::Server => labelled(
                look,
                "Server",
                None,
                choice(
                    s,
                    sf,
                    [Server::Com, Server::Eu, Server::Custom]
                        .into_iter()
                        .map(|x| (x.label(), x == f.server, Message::Setup(super::Msg::Server(x))))
                        .collect(),
                    look,
                ),
                None,
            ),
            SF::Url => labelled(
                look,
                "Server address",
                None,
                input(s, sf, "https://vault.example.com", Some(Icon::Public), look),
                None,
            ),
            SF::Email => labelled(
                look,
                "Email",
                None,
                input(s, sf, "you@example.com", Some(Icon::Mail), look),
                None,
            ),
            SF::Folders => labelled(
                look,
                "SSH keys from",
                None,
                folder_ticks(s, look),
                Some(note(
                    look,
                    if f.folders.is_empty() {
                        "The whole vault. Tick folders to take keys from those only; Pull again to apply."
                    } else {
                        "Only the ticked folders. Untick them all for the whole vault; Pull again to apply."
                    },
                    false,
                )),
            ),
            SF::Password => labelled(
                look,
                "Master password",
                None,
                input(s, sf, "", Some(Icon::Lock), look),
                s.bw_logged_in()
                    .then(|| note(look, "Logged in already: this only unlocks the vault.", false)),
            ),
            SF::Method => labelled(
                look,
                "Two-step login",
                None,
                choice(
                    s,
                    sf,
                    [BwMethod::Totp, BwMethod::YubikeyOtp, BwMethod::ApiKey]
                        .into_iter()
                        .map(|m| (method_label(m), m == f.method, Message::Setup(super::Msg::Method(m))))
                        .collect(),
                    look,
                ),
                Some(note(
                    look,
                    match f.method {
                        BwMethod::Totp => "The 6-digit code from your authenticator app.",
                        BwMethod::YubikeyOtp => "A YubiKey 5 types its code when touched. The Bio can't.",
                        BwMethod::ApiKey => {
                            "Web vault: Settings, Security, Keys, View API key. No two-step code; bw keeps a refresh \
                             token."
                        }
                    },
                    false,
                )),
            ),
            SF::Code => labelled(
                look,
                "Two-step code",
                None,
                input(s, sf, "123456", Some(Icon::Timer), look),
                None,
            ),
            SF::ClientId => labelled(
                look,
                "client_id",
                None,
                input(s, sf, "user.…", Some(Icon::Badge), look),
                None,
            ),
            SF::ClientSecret => labelled(
                look,
                "client_secret",
                None,
                input(s, sf, "", Some(Icon::Key), look),
                None,
            ),
            SF::GitName => labelled(
                look,
                "Name",
                None,
                input(s, sf, "Your name", Some(Icon::Person), look),
                None,
            ),
            SF::GitEmail => labelled(
                look,
                "Email",
                None,
                input(s, sf, "you@example.com", Some(Icon::Mail), look),
                Some(note(
                    look,
                    "Public on every commit you push. GitHub's noreply address works too.",
                    false,
                )),
            ),
            _ => continue,
        };
        col = col.push(el);
    }
    if fields.iter().any(|f| f.is_secret()) {
        // The same small toggle as the install screens', under the fields: it comes after them in
        // the Tab order too.
        let (label, glyph) = if s.show_secrets {
            ("Hide", Icon::VisibilityOff)
        } else {
            ("Show", Icon::Visibility)
        };
        col = col.push(
            row(look, Gap::NONE).push(fill()).push(
                button(label)
                    .kind(Kind::Text)
                    .size(kubaos_ui::component::button::ButtonSize::Sm)
                    .icon(glyph)
                    .on_press(click(SF::ShowSecrets))
                    .focused(s.is_focused(SF::ShowSecrets))
                    .view(look),
            ),
        );
    }
    widget::container(col).max_width(f32::from(Size::Column.px())).into()
}

fn prompt_icon(k: PromptKind) -> Icon {
    match k {
        PromptKind::Password => Icon::Lock,
        PromptKind::Finger => Icon::Fingerprint,
        PromptKind::Key => Icon::Key,
        PromptKind::Wait => Icon::Schedule,
    }
}

fn run_view<'a>(s: &'a Setup, run: &'a super::Run, look: &Look) -> El<'a> {
    let c = look.colors;
    let seconds = (s.now - s.epoch).as_secs_f32();
    match (&run.outcome, &run.prompt) {
        (None, prompt) => {
            let (kind, message, count) = prompt.clone().unwrap_or((PromptKind::Wait, "Starting…".into(), None));
            let lead: El = if kind == PromptKind::Wait {
                spinner(look, IconSize::S48, seconds, c.primary)
            } else {
                icon(look, prompt_icon(kind), IconSize::S48, c.primary)
            };
            let mut words = stack(look, Space::S8)
                .width(Extent::Fill)
                .push(text(message, Type::Heading, c.on_surface));
            if let Some((touched, of)) = count {
                words = words.push(
                    stack(look, Space::S4)
                        .push(bar(look, touched as f32 / of.max(1) as f32))
                        .push(text(format!("{touched} of {of}"), Type::Small, c.on_surface_variant)),
                );
            }
            let sub = match kind {
                PromptKind::Password => "polkit's dialog. Your password, or your finger on the enrolled key.",
                PromptKind::Finger => "On the laptop's reader.",
                PromptKind::Key => "On the YubiKey's own sensor, not the laptop's.",
                PromptKind::Wait => "Nothing for you to do. Watch the log if you like that sort of thing.",
            };
            words = words.push(text(sub, Type::Small, c.on_surface_variant));
            card(look, row(look, Space::S20).center().push(lead).push(words))
        }
        (Some(Outcome::Done(summary)), _) => card(
            look,
            row(look, Space::S12)
                .center()
                .push(icon(look, Icon::CheckCircle, IconSize::S24, c.primary))
                .push(
                    stack(look, Space::S4)
                        .push(text("Done.", Type::Subheading, c.on_surface))
                        .push(text(summary.as_str(), Type::Body, c.on_surface_variant)),
                ),
        ),
        (Some(Outcome::Failed { error, hint }), _) => error_block(
            look,
            format!("Fuck. {}.", capital(error)),
            hint.clone().unwrap_or_default(),
        ),
    }
}

/// The run's log, under everything else: the facts and the serious notes come first.
fn log_view<'a>(s: &'a Setup, run: &'a super::Run, look: &Look) -> El<'a> {
    if run.log.is_empty() {
        return widget::Space::new().into();
    }
    let mut col = stack(look, Space::S12).push(
        button(if s.log_open { "Hide the log" } else { "Show the log" })
            .kind(Kind::Text)
            .icon(Icon::Terminal)
            .on_press(click(SF::Log))
            .focused(s.is_focused(SF::Log))
            .view(look),
    );
    if s.log_open {
        let shown: Vec<(LogLevel, &str)> = run
            .log
            .iter()
            .filter(|(level, _)| *level != Level::Debug)
            .map(|(level, line)| {
                let level = match level {
                    Level::Warn => LogLevel::Warn,
                    Level::Error => LogLevel::Error,
                    _ => LogLevel::Debug,
                };
                (level, line.as_str())
            })
            .collect();
        col = col.push(log(look, &shown, Extent::Fixed(Size::Log)));
    }
    col.into()
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

fn actions<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> El<'a> {
    let colours = look.colors;
    let running = s.running();
    let mut r = row(look, Space::S12).center().push(
        button(if c.state.open() { "Skip" } else { "Next" })
            .kind(Kind::Text)
            .trailing_icon(Icon::ArrowForward)
            .on_press(click(SF::Skip))
            .focused(s.is_focused(SF::Skip))
            .view(look),
    );
    r = r.push(fill());
    let blocker = s.blocker().filter(|_| {
        s.primary().is_some_and(|(_, a)| a == Act::Run) || s.secondary().is_some_and(|(_, a)| a == Act::Run)
    });
    if let Some(b) = blocker
        && !running
    {
        let flashing = s.flash.is_some_and(|f| s.now - f < FLASH);
        r = r.push(text(
            b,
            Type::Small,
            if flashing {
                colours.primary
            } else {
                colours.on_surface_variant
            },
        ));
    }
    if s.can_log_out() {
        r = r.push(
            button("Log out")
                .kind(Kind::Text)
                .on_press_maybe((!running).then(|| click(SF::LogOut)))
                .focused(s.is_focused(SF::LogOut))
                .view(look),
        );
    }
    if s.can_turn_off() {
        r = r.push(
            button("Turn the phone link off")
                .kind(Kind::Text)
                .on_press_maybe((!running).then(|| click(SF::TurnOff)))
                .focused(s.is_focused(SF::TurnOff))
                .view(look),
        );
    }
    if let Some((label, act)) = s.secondary() {
        let ok = !(running || act == Act::Run && s.blocker().is_some());
        r = r.push(
            button(label)
                .kind(Kind::Tonal)
                .on_press_maybe(ok.then(|| click(SF::Secondary)))
                .focused(s.is_focused(SF::Secondary))
                .view(look),
        );
    }
    if let Some((label, act)) = s.primary() {
        let ok = !(running || act == Act::Run && s.blocker().is_some());
        r = r.push(
            button(label)
                .on_press_maybe(ok.then(|| click(SF::Primary)))
                .focused(s.is_focused(SF::Primary))
                .view(look),
        );
    }
    r.into()
}
