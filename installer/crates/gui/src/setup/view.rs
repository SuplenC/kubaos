//! The setup screen: the window is one panel, the cards on the left, the selected
//! card on the right. Copy per docs/BRAND.md: attitude around the facts,
//! never in the sentence that matters (the MOK password, the blue screen).

use std::f32::consts::PI;

use dotinstall_core::Level;
use dotinstall_core::setup::{CardId, CardStatus, Facts, MokState, PromptKind, State, finger_name};
use iced::widget::{Space, button, column, container, row, stack, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Theme, Vector};

use super::{Act, Outcome, SF, Setup, method_label};
use crate::app::Message;
use crate::theme::{self, DARK, Icon, Palette, alpha, jp, motion, radius, size, space};
use crate::widgets::{
    FieldLook, Kind, accent, body, btn, chip, field, icon, ring, scroll, spinner, text_field,
    title, well,
};

type El<'a> = Element<'a, Message>;

fn click(f: SF) -> Message {
    Message::Setup(super::Msg::Click(f))
}

pub fn view(s: &Setup) -> El<'_> {
    let mut layers = stack![panel(s)].width(Length::Fill).height(Length::Fill);
    if s.backend.is_demo() {
        layers = layers.push(
            container(chip(
                Some(Icon::Bolt),
                "Demo",
                DARK.tertiary,
                alpha(DARK.tertiary_container, 0.45),
            ))
            .width(Length::Fill)
            .align_right(Length::Fill)
            .padding(14),
        );
    }
    if let Some((msg, at)) = &s.toast {
        let t = (s.now - *at).as_secs_f32();
        let a = (t / 0.2).min(1.0) * ((4.0 - t) / 0.3).clamp(0.0, 1.0);
        let toast = container(
            text(msg.as_str())
                .size(size::MD)
                .color(alpha(DARK.surface, a)),
        )
        .padding(Padding::from([12.0, 18.0]))
        .style(move |_t| container::Style {
            background: Some(Background::Color(alpha(DARK.on_surface, a))),
            border: Border::default().rounded(radius::MD),
            ..container::Style::default()
        });
        layers = layers.push(
            container(toast)
                .center_x(Length::Fill)
                .align_bottom(Length::Fill)
                .padding(28),
        );
    }
    crate::pointer::idle(layers, s.pointer_idle).into()
}

/// The window is the panel. Setup runs as a floating window in Hyprland,
/// which draws the border, the rounding and the shadow, with the desktop
/// right there around it. So no photo and no card inside a card: a solid
/// surface, edge to edge.
fn panel(s: &Setup) -> El<'_> {
    container(row![left(s), detail(s)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_t| container::Style::default().background(DARK.surface))
        .into()
}

fn card_icon(id: CardId) -> Icon {
    match id {
        CardId::Tuxedo => Icon::Computer,
        CardId::Fingerprint => Icon::Badge,
        CardId::Yubikey => Icon::Key,
        CardId::Bitwarden => Icon::Lock,
        CardId::Git => Icon::Code,
    }
}

fn state_chip<'a>(st: State, p: &Palette) -> El<'a> {
    let (ic, word, fg, bg) = match st {
        State::Done => (
            Some(Icon::CheckCircle),
            "Done",
            p.tertiary,
            alpha(p.tertiary_container, 0.35),
        ),
        State::Needed => (None, "To do", p.on_primary, p.primary),
        State::Reboot => (
            Some(Icon::Restart),
            "Reboot",
            p.primary,
            alpha(p.primary_container, 0.35),
        ),
        State::Waiting => (
            Some(Icon::Schedule),
            "Waiting",
            p.on_surface_variant,
            alpha(p.surface_container_highest, 0.8),
        ),
        State::Absent => (
            None,
            "Not here",
            p.outline,
            alpha(p.surface_container_high, 0.5),
        ),
    };
    chip(ic, word, fg, bg)
}

fn hint<'a>(k: &'a str, what: &'a str) -> El<'a> {
    let keycap = container(
        text(k)
            .font(theme::MONO)
            .size(10.5)
            .color(DARK.on_surface_variant),
    )
    .padding(Padding::from([2.0, 6.0]))
    .style(|_t| container::Style {
        background: Some(Background::Color(alpha(
            DARK.surface_container_highest,
            0.7,
        ))),
        border: Border {
            color: alpha(DARK.outline_variant, 0.9),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    });
    row![keycap, text(what).size(size::XS).color(DARK.outline)]
        .spacing(space::SM)
        .align_y(Alignment::Center)
        .into()
}

fn left(s: &Setup) -> El<'_> {
    let p = DARK;
    let list_focused = s.is_focused(SF::Cards);
    let t = (s.now - s.epoch).as_secs_f32();
    let mut rows = column![].spacing(6);
    match &s.status {
        None => {
            rows = rows.push(
                row![
                    spinner(t, 18.0, p.primary),
                    text("Checking what this machine has…")
                        .size(size::MD)
                        .color(p.on_surface_variant)
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center)
                .padding(Padding::from([10.0, 14.0])),
            );
        }
        Some(st) => {
            for c in &st.cards {
                rows = rows.push(card_row(s, c, list_focused));
            }
        }
    }
    let list = container(rows)
        .padding(6)
        .style(move |_t| container::Style {
            border: Border {
                color: if list_focused {
                    p.primary
                } else {
                    Color::TRANSPARENT
                },
                width: 2.0,
                radius: (radius::LG + 4.0).into(),
            },
            ..container::Style::default()
        });

    let online: El = match &s.status {
        Some(st) if st.online => chip(
            Some(Icon::Wifi),
            "Online",
            p.on_surface_variant,
            alpha(p.surface_container_high, 0.65),
        ),
        Some(_) => chip(
            Some(Icon::WifiOff),
            "Offline: drivers, fprintd and Bitwarden need the internet",
            p.on_surface_variant,
            alpha(p.surface_container_high, 0.65),
        ),
        None => Space::new().into(),
    };
    let (finish_label, kind) = if s.settled() {
        ("All set", Kind::Primary)
    } else {
        ("Done for now", Kind::Tonal)
    };
    let finish = btn(
        finish_label,
        Some(Icon::CheckCircle),
        false,
        kind,
        Some(click(SF::Finish)),
        s.is_focused(SF::Finish),
        &p,
    );
    let detecting: El = if s.detecting() && s.status.is_some() {
        row![
            spinner(t, 14.0, p.primary),
            text("Checking again…").size(size::SM).color(p.outline)
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center)
        .into()
    } else {
        Space::new().into()
    };

    container(
        column![
            accent(jp::SETUP, &p).size(15),
            Space::new().height(2),
            title("First boot", &p),
            Space::new().height(space::SM),
            text(
                "The things the installer couldn't do without you or the hardware. \
                 Each one checks itself, so done stays done."
            )
            .size(size::SM)
            .line_height(1.45)
            .color(p.on_surface_variant),
            Space::new().height(space::XL),
            list,
            Space::new().height(space::MD),
            detecting,
            Space::new().height(Length::Fill),
            online,
            Space::new().height(space::LG),
            row![finish].align_y(Alignment::Center),
            Space::new().height(space::LG),
            column![
                row![
                    container(hint("↑ ↓", "pick a card")).width(Length::FillPortion(1)),
                    container(hint("Tab", "next control")).width(Length::FillPortion(1)),
                ],
                row![
                    container(hint("Enter", "open, press")).width(Length::FillPortion(1)),
                    container(hint("Esc", "back to the cards")).width(Length::FillPortion(1)),
                ],
            ]
            .spacing(6),
        ]
        .height(Length::Fill),
    )
    .width(400)
    .height(Length::Fill)
    .padding(Padding {
        top: 34.0,
        right: 22.0,
        bottom: 24.0,
        left: 26.0,
    })
    .style(|_t| container::Style {
        background: Some(Background::Color(alpha(
            DARK.surface_container_lowest,
            0.42,
        ))),
        ..container::Style::default()
    })
    .into()
}

fn card_row<'a>(s: &'a Setup, c: &'a CardStatus, list_focused: bool) -> El<'a> {
    let p = DARK;
    let sel = c.id == s.sel;
    let running = s
        .run
        .as_ref()
        .is_some_and(|r| r.card == c.id && r.running());
    let dim = c.state == State::Absent;
    let t = (s.now - s.epoch).as_secs_f32();
    let fg = if dim { p.outline } else { p.on_surface };
    let lead: El = if running {
        spinner(t, 22.0, p.primary)
    } else {
        icon(
            card_icon(c.id),
            22.0,
            if dim { p.outline } else { p.primary },
        )
        .into()
    };
    let content = row![
        container(lead).center_x(28),
        column![
            row![
                text(c.id.title())
                    .font(theme::SANS_SEMIBOLD)
                    .size(size::MD)
                    .color(fg),
                Space::new().width(Length::Fill),
                state_chip(c.state, &p),
            ]
            .align_y(Alignment::Center),
            text(c.summary.as_str())
                .size(size::SM)
                .line_height(1.35)
                .color(if dim { p.outline } else { p.on_surface_variant }),
        ]
        .spacing(4)
        .width(Length::Fill),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Start);
    let pp = p;
    button(content)
        .width(Length::Fill)
        .padding(Padding::from([12.0, 12.0]))
        .on_press(Message::Setup(super::Msg::Pick(c.id)))
        .style(move |_t: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered);
            let bg = if sel {
                Some(Background::Color(alpha(
                    pp.secondary_container,
                    if list_focused { 0.85 } else { 0.55 },
                )))
            } else if hover {
                Some(Background::Color(alpha(pp.on_surface, theme::STATE_HOVER)))
            } else {
                None
            };
            button::Style {
                background: bg,
                text_color: pp.on_surface,
                border: Border::default().rounded(radius::LG),
                shadow: Shadow::default(),
                snap: true,
            }
        })
        .into()
}

// -------------------------------------------------------------- detail --

fn detail(s: &Setup) -> El<'_> {
    let f = {
        let t = (s.now - s.changed).as_secs_f32() / motion::LARGE.as_secs_f32();
        motion::bezier(motion::EMPHASIZED_DECEL, t)
    };
    let p = DARK.faded(f);
    let dy = (1.0 - f) * 18.0;
    let inner: El = match s.selected() {
        None => Space::new().into(),
        Some(c) => card_detail(s, c, &p),
    };
    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: (34.0 + dy).max(0.0),
            right: 40.0,
            bottom: 26.0,
            left: 40.0,
        })
        .clip(true)
        .into()
}

fn para<'a>(s: impl text::IntoFragment<'a>, p: &Palette) -> El<'a> {
    body(s, p).width(Length::Fill).into()
}

/// The serious kind of note: no jokes, outlined, an icon.
fn notice<'a>(ic: Icon, head: &'a str, lines: Vec<String>, p: &Palette) -> El<'a> {
    let pp = *p;
    let mut c = column![
        row![
            icon(ic, 20.0, p.primary),
            text(head)
                .font(theme::SANS_SEMIBOLD)
                .size(size::MD)
                .color(p.on_surface)
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center)
    ]
    .spacing(6);
    for l in lines {
        c = c.push(
            text(l)
                .size(size::MD)
                .line_height(1.45)
                .color(p.on_surface_variant)
                .width(Length::Fill),
        );
    }
    container(c)
        .width(Length::Fill)
        .padding(Padding::from([14.0, 16.0]))
        .style(move |_t| container::Style {
            background: Some(Background::Color(alpha(pp.primary_container, 0.12))),
            border: Border {
                color: alpha(pp.primary, 0.45),
                width: 1.0,
                radius: radius::LG.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn card_detail<'a>(s: &'a Setup, c: &'a CardStatus, p: &Palette) -> El<'a> {
    let pp = *p;
    let badge = container(icon(
        card_icon(c.id),
        30.0,
        if c.state == State::Absent {
            p.outline
        } else {
            p.primary
        },
    ))
    .center(56)
    .style(move |_t| container::Style {
        background: Some(Background::Color(alpha(pp.primary_container, 0.18))),
        border: Border::default().rounded(radius::FULL),
        ..container::Style::default()
    });
    let head = row![
        badge,
        column![
            row![title(c.id.title(), p), state_chip(c.state, p)]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            text(c.summary.as_str())
                .size(size::MD)
                .color(p.on_surface_variant)
        ]
        .spacing(4)
    ]
    .spacing(space::LG)
    .align_y(Alignment::Center);

    let mut col = column![head, Space::new().height(space::XL)].spacing(0);
    // What's happening right now goes first: the touch prompt has to be
    // on screen, not under the explanations.
    if let Some(run) = s.shown_run() {
        col = col.push(run_view(s, run, p));
        col = col.push(Space::new().height(space::XL));
    }
    for e in explain(s, c, p) {
        col = col.push(e).push(Space::new().height(space::MD));
    }
    let fields = s.fields();
    if !fields.is_empty() && !s.running() {
        col = col.push(Space::new().height(space::SM));
        col = col.push(form(s, &fields, p));
        col = col.push(Space::new().height(space::LG));
    }
    if let Some(run) = s.shown_run() {
        col = col.push(log_view(s, run, p));
    }
    let scrolled = scroll("setup-detail", col.width(Length::Fill), p).height(Length::Fill);
    column![scrolled, Space::new().height(space::LG), actions(s, c, p)]
        .height(Length::Fill)
        .into()
}

/// What the card is about, for its state. Facts first.
fn explain<'a>(s: &'a Setup, c: &'a CardStatus, p: &Palette) -> Vec<El<'a>> {
    let mut v: Vec<El> = vec![];
    match (&c.facts, c.state) {
        (_, State::Absent) => v.push(para(
            match c.id {
                CardId::Tuxedo => "Nothing to install. The TUXEDO drivers only make sense on a TUXEDO.",
                CardId::Fingerprint => {
                    "No reader that fprintd knows. Plugged one in? Check again from a fresh start of setup."
                }
                _ => "Nothing to do here.",
            },
            p,
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
            match st {
                State::Needed | State::Waiting if !drivers => {
                    v.push(para(
                        format!(
                            "{machine} wants TUXEDO's own kernel drivers for the keyboard backlight, the fans \
                             and the power limits. Setup adds TUXEDO's apt repository and installs tuxedo-drivers. \
                             No Control Center: KubaOS runs power profiles instead, one less app phoning home."
                        ),
                        p,
                    ));
                }
                _ => {}
            }
            if s.wants_mok() || matches!(mok, Some(MokState::Pending)) {
                v.push(mok_notice(c.state, p));
            } else if *secure_boot == Some(false) && c.state != State::Done {
                v.push(para(
                    "Secure Boot is off, so the drivers load as they are. No key to enroll.",
                    p,
                ));
            }
            if c.state == State::Reboot && !matches!(mok, Some(MokState::Pending)) {
                v.push(para("They load on the next boot. Reboot when it suits you.", p));
            }
        }
        (Facts::Fingerprint { reader, fingers, .. }, st) => {
            let reader = reader.clone().unwrap_or_else(|| "The reader".into());
            match st {
                State::Done => v.push(para(
                    format!(
                        "{reader}. sudo, polkit dialogs and the tty login take your finger first and \
                         wait 10 seconds before they ask for the password. Enrolled: {}.",
                        fingers.iter().map(|f| finger_name(f)).collect::<Vec<_>>().join(", ")
                    ),
                    p,
                )),
                _ => v.push(para(
                    format!(
                        "{reader}. Enroll a finger and sudo and password dialogs take it instead of your \
                         password. The password keeps working, so a cut finger is a bad day, not a lockout. \
                         You'll touch the reader eight-ish times; move the finger a little each time."
                    ),
                    p,
                )),
            }
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
                "Your YubiKey and the finger on its sensor stand in for the password at sudo, polkit and the \
                 lock screen. The password always keeps working. Enroll a second key as the backup: keys get lost.",
                p,
            ));
            if st == State::Needed && (*pin == Some(false) || *fingerprints == Some(false)) {
                v.push(notice(
                    Icon::Key,
                    "The key needs its own setup first",
                    vec![
                        "It has no FIDO2 PIN or no fingerprint yet, and it won't enroll without both. \
                         That part talks to the key directly, so it runs in a terminal:"
                            .into(),
                        "ykman fido access change-pin, then ykman fido fingerprints add right-index (and a finger \
                         from the other hand)."
                            .into(),
                        "Come back and press Check again.".into(),
                    ],
                    p,
                ));
            } else if st == State::Waiting && plugged.is_empty() && *enrolled == 0 {
                v.push(para("Plug it in, then Check again. Or skip it; it's here whenever.", p));
            }
        }
        (
            Facts::Bitwarden {
                desktop,
                configured,
                ..
            },
            st,
        ) => {
            if st == State::Done {
                v.push(para(
                    "SSH public keys and Host blocks, git commit signing, the gh token and every project's env \
                     came from the vault (dot-bw-setup). Changed the vault? Pull again.",
                    p,
                ));
            } else {
                v.push(para(
                    "Log in once and setup pulls what the dotfiles expect from the vault: SSH public keys and Host \
                     blocks, git commit signing, the gh token and your projects' env. The private keys never touch \
                     the disk.",
                    p,
                ));
                v.push(para(
                    "bw can't use the YubiKey Bio for two-step login: use the authenticator app's code. Only the \
                     email and server are saved. The password and the vault session live in memory until this is done.",
                    p,
                ));
            }
            if *desktop && (*configured || st == State::Done) {
                v.push(para(
                    "Bitwarden desktop starts when you close setup. Last bit by hand: log in there, Settings, turn on \"Enable SSH agent\". It has no config file for that.",
                    p,
                ));
            }
        }
        (Facts::Git { .. }, st) => v.push(para(
            if st == State::Done {
                "git commits with this name and email. Change them here any time; they live in \
                 ~/.config/git-identity/identity.gitconfig, outside the dotfiles repo."
            } else {
                "The name and email git puts on every commit. The dotfiles don't carry one, so the \
                 repo stays yours to share. Saved to ~/.config/git-identity/identity.gitconfig, which \
                 the git config includes."
            },
            p,
        )),
        _ => {}
    }
    v
}

/// Secure Boot and the blue screen. Dead serious: this is the one that
/// leaves you without drivers if it goes wrong.
fn mok_notice<'a>(st: State, p: &Palette) -> El<'a> {
    let steps = "Enroll MOK → Continue → Yes → type the password → Reboot.";
    if st == State::Reboot {
        notice(
            Icon::Restart,
            "Reboot, then enroll the key on the blue screen",
            vec![
                "Before KubaOS starts, a blue screen called MokManager appears. It waits about 10 seconds; press a key \
                 to stop it."
                    .into(),
                steps.into(),
                "Type the one-time password you picked here. If you miss the screen, the drivers won't load: \
                 open setup again and enroll the key again."
                    .into(),
            ],
            p,
        )
    } else {
        notice(
            Icon::Lock,
            "Secure Boot is on",
            vec![
                "The drivers are built on this machine, so they're signed with a key made here, and the firmware \
                 has to be told to trust it. Pick a one-time password below."
                    .into(),
                format!(
                    "After the next reboot a blue screen called MokManager appears before KubaOS starts. {steps}"
                ),
                "You need this password exactly once, on that screen. It assumes a US keyboard, so letters and \
                 digits only."
                    .into(),
            ],
            p,
        )
    }
}

/// Our text field. One place, so the widget's API is wrapped once.
fn input<'a>(s: &'a Setup, f: SF, placeholder: &'a str, ic: Option<Icon>, p: &Palette) -> El<'a> {
    input_err(s, f, placeholder, ic, false, p)
}

/// `input`, red when `error` (the reason goes under it).
fn input_err<'a>(
    s: &'a Setup,
    f: SF,
    placeholder: &'a str,
    ic: Option<Icon>,
    error: bool,
    p: &Palette,
) -> El<'a> {
    let focused = s.is_focused(f) && !s.running();
    text_field(
        FieldLook {
            value: s.field_value(f),
            placeholder,
            secret: f.is_secret() && !s.show_secrets,
            caret: focused.then_some(s.caret),
            blink_from: s.focus_at,
            locked: s.running(),
            error,
            icon: ic,
        },
        (!s.running()).then_some(move |ptr| Message::Setup(super::Msg::Pointer(f, ptr))),
        p,
    )
}

/// A choice that Left/Right (or Space) cycles.
fn choice<'a>(s: &'a Setup, f: SF, options: Vec<(&'a str, bool)>, p: &Palette) -> El<'a> {
    let pp = *p;
    let mut r = row![].spacing(4);
    for (label, on) in options {
        r = r.push(
            container(
                text(label)
                    .font(theme::SANS_MEDIUM)
                    .size(size::SM)
                    .color(if on {
                        pp.on_secondary_container
                    } else {
                        pp.on_surface_variant
                    }),
            )
            .padding(Padding::from([7.0, 14.0]))
            .style(move |_t| container::Style {
                background: on.then_some(Background::Color(pp.secondary_container)),
                border: Border::default().rounded(radius::FULL),
                ..container::Style::default()
            }),
        );
    }
    let b = button(r)
        .padding(3)
        .on_press_maybe((!s.running()).then_some(click(f)))
        .style(move |_t: &Theme, _st| button::Style {
            background: Some(Background::Color(alpha(pp.surface_container_high, 0.9))),
            text_color: pp.on_surface,
            border: Border {
                color: alpha(pp.outline_variant, 0.9),
                width: 1.0,
                radius: radius::FULL.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        });
    ring(b, s.is_focused(f), radius::FULL, p)
}

fn form<'a>(s: &'a Setup, fields: &[SF], p: &Palette) -> El<'a> {
    let mut col = column![].spacing(space::LG).width(Length::Fill);
    let f = &s.form;
    for &sf in fields {
        let el: El = match sf {
            SF::Mok => field(
                "One-time password for the blue screen",
                input(s, sf, "8 to 16 letters and digits", Some(Icon::Key), p),
                None,
                p,
            ),
            SF::MokAgain => field(
                "Same again",
                input_err(
                    s,
                    sf,
                    "",
                    Some(Icon::Key),
                    crate::widgets::mismatched(&f.mok, &f.mok_again),
                    p,
                ),
                crate::widgets::confirm_note(&f.mok, &f.mok_again, p),
                p,
            ),
            SF::Server => field(
                "Server",
                choice(
                    s,
                    sf,
                    [super::Server::Com, super::Server::Eu, super::Server::Custom]
                        .iter()
                        .map(|x| (x.label(), *x == f.server))
                        .collect(),
                    p,
                ),
                None,
                p,
            ),
            SF::Url => field(
                "Server address",
                input(s, sf, "https://vault.example.com", Some(Icon::Public), p),
                None,
                p,
            ),
            SF::Email => field("Email", input(s, sf, "you@example.com", Some(Icon::Mail), p), None, p),
            SF::Password => field(
                "Master password",
                input(s, sf, "", Some(Icon::Lock), p),
                s.bw_logged_in().then(|| {
                    text("Logged in already: this only unlocks the vault.")
                        .size(size::SM)
                        .color(p.on_surface_variant)
                        .into()
                }),
                p,
            ),
            SF::Method => field(
                "Two-step login",
                choice(
                    s,
                    sf,
                    [
                        dotinstall_core::setup::BwMethod::Totp,
                        dotinstall_core::setup::BwMethod::YubikeyOtp,
                        dotinstall_core::setup::BwMethod::ApiKey,
                    ]
                    .iter()
                    .map(|m| (method_label(*m), *m == f.method))
                    .collect(),
                    p,
                ),
                Some(
                    text(match f.method {
                        dotinstall_core::setup::BwMethod::Totp => "The 6-digit code from your authenticator app.",
                        dotinstall_core::setup::BwMethod::YubikeyOtp => {
                            "A YubiKey 5 types its code when touched. The Bio can't."
                        }
                        dotinstall_core::setup::BwMethod::ApiKey => {
                            "Web vault: Settings, Security, Keys, View API key. No two-step code; bw keeps a refresh token."
                        }
                    })
                    .size(size::SM)
                    .color(p.on_surface_variant)
                    .into(),
                ),
                p,
            ),
            SF::Code => field(
                "Two-step code",
                input(s, sf, "123456", Some(Icon::Timer), p),
                None,
                p,
            ),
            SF::ClientId => field("client_id", input(s, sf, "user.…", Some(Icon::Badge), p), None, p),
            SF::ClientSecret => field("client_secret", input(s, sf, "", Some(Icon::Key), p), None, p),
            SF::GitName => field("Name", input(s, sf, "Your name", Some(Icon::Person), p), None, p),
            SF::GitEmail => field(
                "Email",
                input(s, sf, "you@example.com", Some(Icon::Mail), p),
                Some(
                    text("Public on every commit you push. GitHub's noreply address works too.")
                        .size(size::SM)
                        .color(p.on_surface_variant)
                        .into(),
                ),
                p,
            ),
            _ => continue,
        };
        col = col.push(el);
    }
    let secret = fields.iter().any(|f| f.is_secret());
    if secret {
        // The same small toggle as the installer's, under the fields (it
        // comes after them in the Tab order too).
        col = col.push(
            row![
                Space::new().width(Length::Fill),
                crate::widgets::show_toggle(
                    s.show_secrets,
                    click(SF::ShowSecrets),
                    s.is_focused(SF::ShowSecrets),
                    p,
                )
            ]
            .width(Length::Fill),
        );
    }
    container(col).max_width(560).into()
}

fn prompt_icon(k: PromptKind) -> Icon {
    match k {
        PromptKind::Password => Icon::Lock,
        PromptKind::Finger => Icon::Badge,
        PromptKind::Key => Icon::Key,
        PromptKind::Wait => Icon::Schedule,
    }
}

fn run_view<'a>(s: &'a Setup, run: &'a super::Run, p: &Palette) -> El<'a> {
    let pp = *p;
    let t = (s.now - s.epoch).as_secs_f32();
    let mut col = column![].spacing(space::MD).width(Length::Fill);
    match (&run.outcome, &run.prompt) {
        (None, prompt) => {
            let (kind, msg, count) =
                prompt
                    .clone()
                    .unwrap_or((PromptKind::Wait, "Starting…".into(), None));
            // A soft pulse around whatever wants a touch.
            let since = (s.now - run.prompt_at).as_secs_f32();
            let pulse = 0.5 + 0.5 * (since * PI * 1.4).sin();
            let lead: El = if kind == PromptKind::Wait {
                spinner(t, 40.0, p.primary)
            } else {
                container(icon(prompt_icon(kind), 34.0, p.on_primary))
                    .center(64)
                    .style(move |_t| container::Style {
                        background: Some(Background::Color(pp.primary)),
                        border: Border::default().rounded(radius::FULL),
                        shadow: Shadow {
                            color: alpha(pp.primary, 0.25 + 0.35 * pulse),
                            offset: Vector::ZERO,
                            blur_radius: 18.0 + 14.0 * pulse,
                        },
                        ..container::Style::default()
                    })
                    .into()
            };
            let mut words = column![
                text(msg)
                    .font(theme::SANS_SEMIBOLD)
                    .size(20)
                    .color(p.on_surface)
                    .width(Length::Fill)
            ]
            .spacing(space::SM);
            if let Some((k, n)) = count {
                let mut dots = row![].spacing(6);
                for i in 0..n {
                    let on = i < k;
                    dots = dots.push(container(Space::new()).width(14).height(14).style(
                        move |_t| container::Style {
                            background: Some(Background::Color(if on {
                                pp.primary
                            } else {
                                alpha(pp.outline, 0.35)
                            })),
                            border: Border::default().rounded(radius::FULL),
                            ..container::Style::default()
                        },
                    ));
                }
                words = words.push(
                    row![
                        dots,
                        text(format!("{k} of {n}"))
                            .size(size::SM)
                            .color(p.on_surface_variant)
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                );
            }
            let sub = match kind {
                PromptKind::Password => {
                    "polkit's dialog. Your password, or your finger on the enrolled key."
                }
                PromptKind::Finger => "On the laptop's reader.",
                PromptKind::Key => "On the YubiKey's own sensor, not the laptop's.",
                PromptKind::Wait => {
                    "Nothing for you to do. Watch the log if you like that sort of thing."
                }
            };
            words = words.push(text(sub).size(size::SM).color(p.on_surface_variant));
            col = col.push(
                container(
                    row![container(lead).center_x(72), words]
                        .spacing(space::XL)
                        .align_y(Alignment::Center),
                )
                .padding(Padding::from([18.0, 20.0]))
                .width(Length::Fill)
                .style(well(pp)),
            );
        }
        (Some(Outcome::Done(summary)), _) => {
            col = col.push(
                container(
                    row![
                        icon(Icon::CheckCircle, 26.0, p.tertiary),
                        column![
                            text("Done.")
                                .font(theme::SANS_SEMIBOLD)
                                .size(18)
                                .color(p.on_surface),
                            text(summary.as_str())
                                .size(size::MD)
                                .color(p.on_surface_variant)
                        ]
                        .spacing(2)
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([14.0, 18.0]))
                .width(Length::Fill)
                .style(well(pp)),
            );
        }
        (Some(Outcome::Failed { error, hint }), _) => {
            let mut words = column![
                text(format!("Well, shit. {}.", capital(error)))
                    .font(theme::SANS_SEMIBOLD)
                    .size(16)
                    .color(p.on_error_container)
            ]
            .spacing(4);
            if let Some(h) = hint {
                words = words.push(text(h.as_str()).size(size::MD).color(p.on_error_container));
            }
            col = col.push(
                container(
                    row![icon(Icon::Error, 24.0, p.error), words.width(Length::Fill)]
                        .spacing(space::MD)
                        .align_y(Alignment::Center),
                )
                .padding(Padding::from([14.0, 18.0]))
                .width(Length::Fill)
                .style(move |_t| container::Style {
                    background: Some(Background::Color(alpha(pp.error_container, 0.45))),
                    border: Border {
                        color: alpha(pp.error, 0.6),
                        width: 1.0,
                        radius: radius::LG.into(),
                    },
                    ..container::Style::default()
                }),
            );
        }
    }
    col.into()
}

/// The run's log, under everything else: the facts and the serious notes
/// come first.
fn log_view<'a>(s: &'a Setup, run: &'a super::Run, p: &Palette) -> El<'a> {
    let pp = *p;
    let mut col = column![].spacing(space::MD).width(Length::Fill);
    if !run.log.is_empty() {
        col = col.push(btn(
            if s.log_open {
                "Hide the log"
            } else {
                "Show the log"
            },
            Some(Icon::Terminal),
            false,
            Kind::Text,
            Some(click(SF::Log)),
            s.is_focused(SF::Log),
            p,
        ));
        if s.log_open {
            let mut lines = column![].spacing(2);
            let shown: Vec<&(Level, String)> =
                run.log.iter().filter(|(l, _)| *l != Level::Debug).collect();
            for (lvl, l) in shown.iter().rev().take(14).rev() {
                let c = match lvl {
                    Level::Warn => p.primary,
                    Level::Error => p.error,
                    _ => p.on_surface_variant,
                };
                lines = lines.push(text(l.as_str()).font(theme::MONO).size(12).color(c));
            }
            col = col.push(
                container(lines)
                    .padding(Padding::from([12.0, 14.0]))
                    .width(Length::Fill)
                    .style(well(pp)),
            );
        }
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

fn actions<'a>(s: &'a Setup, c: &'a CardStatus, p: &Palette) -> El<'a> {
    let running = s.running();
    let mut r = row![].spacing(space::MD).align_y(Alignment::Center);
    let next_label = if c.state.open() { "Skip" } else { "Next" };
    r = r.push(btn(
        next_label,
        Some(Icon::ArrowForward),
        true,
        Kind::Text,
        Some(click(SF::Skip)),
        s.is_focused(SF::Skip),
        p,
    ));
    r = r.push(Space::new().width(Length::Fill));
    let blocker = s.blocker().filter(|_| {
        s.primary().is_some_and(|(_, a)| a == Act::Run)
            || s.secondary().is_some_and(|(_, a)| a == Act::Run)
    });
    if let Some(b) = blocker
        && !running
    {
        let flashing = s
            .flash
            .is_some_and(|f| s.now - f < std::time::Duration::from_millis(900));
        r = r.push(text(b).size(size::SM).color(if flashing {
            p.primary
        } else {
            p.on_surface_variant
        }));
    }
    if let Some((l, a)) = s.secondary() {
        let ok = !(running || a == Act::Run && s.blocker().is_some());
        r = r.push(btn(
            l,
            None,
            false,
            Kind::Tonal,
            ok.then(|| click(SF::Secondary)),
            s.is_focused(SF::Secondary),
            p,
        ));
    }
    if let Some((l, a)) = s.primary() {
        let ok = !(running || a == Act::Run && s.blocker().is_some());
        r = r.push(btn(
            l,
            None,
            false,
            Kind::Primary,
            ok.then(|| click(SF::Primary)),
            s.is_focused(SF::Primary),
            p,
        ));
    }
    r.into()
}
