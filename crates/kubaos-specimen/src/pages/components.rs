//! The components page: every component of the closed set, each kind and size, resting, focused
//! and disabled.

use std::sync::LazyLock;

use iced::Element;
use iced::Length;
use iced::border;
use iced::widget;
use iced::widget::container;
use iced::widget::image;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::component::avatar::TileSize;
use kubaos_ui::component::avatar::badge;
use kubaos_ui::component::avatar::icon_tile;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::Preview;
use kubaos_ui::component::button::button;
use kubaos_ui::component::button::icon_button;
use kubaos_ui::component::calendar::Day;
use kubaos_ui::component::calendar::Month;
use kubaos_ui::component::calendar::Week;
use kubaos_ui::component::calendar::calendar;
use kubaos_ui::component::card::card;
use kubaos_ui::component::card::choice_card;
use kubaos_ui::component::check::Tick;
use kubaos_ui::component::check::checkbox;
use kubaos_ui::component::check::radio;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::dialog::Tone;
use kubaos_ui::component::dialog::dialog;
use kubaos_ui::component::dialog::prompt;
use kubaos_ui::component::dialog::scrim;
use kubaos_ui::component::empty::empty_state;
use kubaos_ui::component::error::Level;
use kubaos_ui::component::error::error_block;
use kubaos_ui::component::error::log;
use kubaos_ui::component::field::Caret;
use kubaos_ui::component::field::State;
use kubaos_ui::component::field::field;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::component::keycap::key_hints;
use kubaos_ui::component::keycap::keycap;
use kubaos_ui::component::list::empty_list;
use kubaos_ui::component::list::list;
use kubaos_ui::component::list::list_row;
use kubaos_ui::component::media::Aspect;
use kubaos_ui::component::media::media;
use kubaos_ui::component::meter::meter;
use kubaos_ui::component::meter::sparkline;
use kubaos_ui::component::picture::NamePlace;
use kubaos_ui::component::picture::picture_choice;
use kubaos_ui::component::progress::bar;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::component::sheet;
use kubaos_ui::component::skeleton::Bone;
use kubaos_ui::component::skeleton::skeleton;
use kubaos_ui::component::slider::Value;
use kubaos_ui::component::slider::slider;
use kubaos_ui::component::slider::slider_held;
use kubaos_ui::component::switch::switch;
use kubaos_ui::component::tabs::Tab;
use kubaos_ui::component::tabs::tabs;
use kubaos_ui::component::tile::tile;
use kubaos_ui::component::toast::toast;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::row;
use kubaos_ui::layout::section;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::snap::centre_x;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;

#[expect(
    clippy::too_many_lines,
    reason = "The page is one section after another, in the order they show; split up, the order would be spread over functions."
)]
pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    stack(look, Space::S48)
        .push(section(
            look,
            "Buttons",
            Some("Filled, Tonal, Text and Danger, in small, regular and large, each resting, hovered, pressed, focused and disabled."),
            buttons(look),
        ))
        .push(section(
            look,
            "Icon buttons",
            Some("Circles as tall as the label buttons of each size. The label is required: it shows in a hint on hover and while focused."),
            icon_buttons(look),
        ))
        .push(section(
            look,
            "Fields",
            Some("As tall as a regular button. The label's row fits a small button, so fields side by side line up."),
            fields(look),
        ))
        .push(section(look, "Switches, checkboxes and radio buttons", None, toggles(look)))
        .push(section(
            look,
            "Sliders",
            Some("A progress bar's track filled up to the thumb. One with stops marks those ahead and snaps to them."),
            sliders(look),
        ))
        .push(section(
            look,
            "Tabs",
            Some("One of a few views or values, picked from small buttons: the picked one tonal. The same row is a setting's segmented control."),
            tab_rows(look),
        ))
        .push(section(
            look,
            "Tiles",
            Some("Quick toggles as tall as a two-line row, filled with the accent while on. The detail arrow stays quiet: no fill, a state layer under the pointer."),
            tiles(look),
        ))
        .push(section(
            look,
            "Icon tiles and badges",
            Some("Every row's icon on one tile, a symbol until a picture loads, so rows read the same size: small and regular, an app's icon, a photo, a critical one. Then badges, a count or a dot over an icon's corner."),
            avatars(look),
        ))
        .push(section(
            look,
            "Skeletons",
            Some("The shape of what is loading, in the placeholder colour, so what arrives moves nothing: a row being scanned, a thumbnail being decoded."),
            skeletons(look),
        ))
        .push(section(
            look,
            "Lists",
            Some("Rows in a well: a surface, 24, with 12 inside, so the rows take 24 − 12 = 12, the control radius. The ring goes round the well; the row shows where the keyboard is."),
            lists(look),
        ))
        .push(section(look, "Chips", None, chips(look)))
        .push(section(
            look,
            "Keycaps and key hints",
            Some("Keys drawn as keys, as tall as a line of body text so they sit inline with it, then what each does."),
            keycaps(look),
        ))
        .push(section(
            look,
            "Cards",
            Some("Picked cards take the accent, so their focus ring is the cold one."),
            cards(look),
        ))
        .push(section(
            look,
            "Sheets",
            Some("A panel growing out of the frame's edge, one surface with it: concave fillets where it meets the edge, the surface radius at its far corners."),
            sheets(look),
        ))
        .push(section(
            look,
            "Calendars",
            Some("A month as six weeks led by their week numbers, today filled and the picked day ringed; the title opens the year."),
            calendars(look),
        ))
        .push(section(
            look,
            "Media",
            Some("Pictures in a box of a fixed aspect, reserved before they load, so nothing moves when they arrive: the placeholder, the picture, then two pictures to pick, picked with its name under it and focused with its name over its foot."),
            media_samples(look),
        ))
        .push(section(
            look,
            "Dialogs",
            Some("A question, the confirmation before something is destroyed, and a prompt for what only you can type."),
            dialogs(look),
        ))
        .push(section(look, "Toasts and progress", None, progress(look)))
        .push(section(
            look,
            "Meters and sparklines",
            Some("Readings at a glance: a reading in parts on a progress bar's track, and the last minute of one as a line."),
            readings(look),
        ))
        .push(section(look, "Empty states", None, empties(look)))
        .push(section(look, "Errors", Some("What broke, what to do next, and the log behind it."), errors(look)))
        .into()
}

/// The states a button is shown in, left to right.
const STATES: [&str; 5] = ["Resting", "Hovered", "Pressed", "Focused", "Disabled"];

fn size_name(size: ButtonSize) -> String {
    let height = match size {
        ButtonSize::Sm => Size::ControlSm,
        ButtonSize::Md => Size::ControlMd,
        ButtonSize::Lg => Size::ControlLg,
    };
    format!("{size:?} · {}", height.px())
}

/// A label button of `kind` and `size` in the `state`th of [`STATES`].
fn button_in<'a>(look: &Look, kind: Kind, size: ButtonSize, state: usize) -> Element<'a, Message> {
    let (label, glyph) = match kind {
        Kind::Filled => ("Continue", Icon::ArrowForward),
        Kind::Tonal => ("Save log", Icon::Save),
        Kind::Text => ("Back", Icon::ArrowBack),
        Kind::Danger => ("Erase disk", Icon::Warning),
    };
    let b = button::<Message>(label).kind(kind).size(size);
    let b = if kind == Kind::Filled {
        b.trailing_icon(glyph)
    } else {
        b.icon(glyph)
    };
    match state {
        0 => b.on_press(Message::Sample),
        1 => b.on_press(Message::Sample).preview(Preview::Hovered),
        2 => b.on_press(Message::Sample).preview(Preview::Pressed),
        3 => b.on_press(Message::Sample).focused(true),
        _ => b,
    }
    .view(look)
}

/// An icon button of `kind` and `size` in the `state`th of [`STATES`].
fn icon_button_in<'a>(look: &Look, kind: Kind, size: ButtonSize, state: usize) -> Element<'a, Message> {
    let (glyph, label) = match kind {
        Kind::Filled => (Icon::ArrowForward, "Continue"),
        Kind::Tonal => (Icon::Settings, "Settings"),
        Kind::Text => (Icon::Close, "Close"),
        Kind::Danger => (Icon::PowerSettingsNew, "Shut down"),
    };
    let b = icon_button::<Message>(glyph, label).kind(kind).size(size);
    match state {
        0 => b.on_press(Message::Sample),
        1 => b.on_press(Message::Sample).preview(Preview::Hovered),
        2 => b.on_press(Message::Sample).preview(Preview::Pressed),
        3 => b.on_press(Message::Sample).focused(true),
        _ => b,
    }
    .view(look)
}

/// The states' names over a table's columns, after the kinds' column.
fn state_heads<'a>(look: &Look) -> Vec<Element<'a, Message>> {
    let c = look.colors;
    std::iter::once(text("", Type::Label, c.on_surface_variant))
        .chain(STATES.map(|state| text(state, Type::Label, c.on_surface_variant)))
        .collect()
}

fn buttons<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut sizes = stack(look, Space::S32);
    for size in ButtonSize::ALL {
        let mut table = split(look, &[1, 2, 2, 2, 2, 2], Space::S16)
            .center()
            .extend(state_heads(look));
        for kind in Kind::ALL {
            table = table.push(text(format!("{kind:?}"), Type::Label, c.on_surface_variant));
            for state in 0..STATES.len() {
                table = table.push(button_in(look, kind, size, state));
            }
        }
        sizes = sizes.push(
            stack(look, Space::S12)
                .push(text(size_name(size), Type::Subheading, c.on_surface))
                .push(table),
        );
    }
    sizes.into()
}

fn icon_buttons<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut sizes = stack(look, Space::S32);
    for size in ButtonSize::ALL {
        // Rows 40 apart, so a focused button's hint has room under it. The columns are the label
        // buttons' above.
        let mut rows = stack(look, Space::S40).push(
            split(look, &[1, 2, 2, 2, 2, 2], Space::S16)
                .center()
                .extend(state_heads(look)),
        );
        for kind in Kind::ALL {
            let mut line = split(look, &[1, 2, 2, 2, 2, 2], Space::S16).center().push(text(
                format!("{kind:?}"),
                Type::Label,
                c.on_surface_variant,
            ));
            for state in 0..STATES.len() {
                line = line.push(icon_button_in(look, kind, size, state));
            }
            rows = rows.push(line);
        }
        sizes = sizes.push(
            stack(look, Space::S12)
                .push(text(size_name(size), Type::Subheading, c.on_surface))
                .push(rows),
        );
    }
    sizes.into()
}

fn fields<'a>(look: &Look) -> Element<'a, Message> {
    let show = button::<Message>("Show")
        .kind(Kind::Text)
        .size(ButtonSize::Sm)
        .icon(Icon::Visibility)
        .on_press(Message::Sample)
        .view(look);
    grid(look, 3, Space::S24)
        .push(labelled(
            look,
            "Full name",
            None,
            field("", "Your name").view(look),
            None,
        ))
        .push(labelled(
            look,
            "Hostname",
            None,
            field("tuxedo", "").icon(Icon::Computer).view(look),
            Some(note(look, "Letters, digits and dashes.", false)),
        ))
        .push(labelled(
            look,
            "Search",
            None,
            field("Shinj", "Search")
                .icon(Icon::Search)
                .caret(Caret::at("Shinj".chars().count()))
                .view(look),
            Some(note(
                look,
                "Focused: the caret, the accent hairline and the ring.",
                false,
            )),
        ))
        .push(labelled(
            look,
            "Password",
            Some(show),
            field("hunter2", "").secret().view(look),
            None,
        ))
        .push(labelled(
            look,
            "Email for git",
            None,
            field("not-an-email", "")
                .icon(Icon::Mail)
                .state(State::Invalid)
                .view(look),
            Some(note(look, "That's not an email. Typo?", true)),
        ))
        .push(labelled(
            look,
            "User name",
            None,
            field("kuba", "").icon(Icon::Lock).state(State::Locked).view(look),
            Some(note(look, "Set when you installed; read-only here.", false)),
        ))
        .into()
}

fn toggles<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let label = |s: &'static str| text(s, Type::Small, c.on_surface_variant);
    let switches = row(look, Space::S32)
        .center()
        .push(
            stack(look, Space::S8)
                .push(switch(look, false, 1.0, Some(Message::Sample), false))
                .push(label("Off")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch(look, true, 1.0, Some(Message::Sample), false))
                .push(label("On")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch(look, true, 0.5, Some(Message::Sample), false))
                .push(label("Halfway")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch(look, true, 1.0, Some(Message::Sample), true))
                .push(label("Focused")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch::<Message>(look, false, 1.0, None, false))
                .push(label("Disabled off")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch::<Message>(look, true, 1.0, None, false))
                .push(label("Disabled on")),
        );
    let checks = row(look, Space::S32)
        .center()
        .push(checkbox(look, "Bluetooth", Tick::Empty, Some(Message::Sample), false))
        .push(checkbox(
            look,
            "Some wallpapers",
            Tick::Partial,
            Some(Message::Sample),
            false,
        ))
        .push(checkbox(
            look,
            "Encrypt the disk",
            Tick::Full,
            Some(Message::Sample),
            false,
        ))
        .push(checkbox(look, "Focused", Tick::Full, Some(Message::Sample), true))
        .push(checkbox::<Message>(look, "Disabled", Tick::Full, None, false));
    let radios = row(look, Space::S32)
        .center()
        .push(radio(look, "Keep Windows", false, Some(Message::Sample), false))
        .push(radio(look, "Erase everything", true, Some(Message::Sample), false))
        .push(radio(look, "Focused", false, Some(Message::Sample), true))
        .push(radio::<Message>(look, "Disabled", true, None, false));
    stack(look, Space::S24).push(switches).push(checks).push(radios).into()
}

fn sliders<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let sample = |value: Value, enabled: bool, focused: bool, label: &'static str| -> Element<'a, Message> {
        let on_change: Option<Box<dyn Fn(Value) -> Message>> =
            enabled.then(|| Box::new(|_| Message::Sample) as Box<dyn Fn(Value) -> Message>);
        stack(look, Space::S8)
            .push(slider(look, value, on_change, focused))
            .push(text(label, Type::Small, c.on_surface_variant))
            .into()
    };
    let held = stack(look, Space::S8)
        .push(slider_held(
            look,
            Value::Continuous(0.62),
            Some(Box::new(|_| Message::Sample) as Box<dyn Fn(Value) -> Message>),
        ))
        .push(text("Pressed: the fill lights up", Type::Small, c.on_surface_variant));
    grid(look, 5, Space::S32)
        .push(sample(Value::Continuous(0.62), true, false, "Volume, 62%"))
        .push(held)
        .push(sample(Value::Stop { index: 2, count: 5 }, true, false, "Five stops"))
        .push(sample(Value::Continuous(0.3), true, true, "Focused"))
        .push(sample(Value::Continuous(0.3), false, false, "Disabled"))
        .into()
}

fn lists<'a>(look: &Look) -> Element<'a, Message> {
    let rows = |look: &Look| -> Vec<Element<'a, Message>> {
        vec![
            list_row("English (US)")
                .chip(ChipKind::Accent, "Default")
                .detail("us")
                .checked(true)
                .on_press(Message::Sample)
                .view(look),
            list_row("Italian")
                .detail("it")
                .checked(false)
                .selected(true)
                .on_press(Message::Sample)
                .view(look),
            list_row("German")
                .detail("de")
                .checked(false)
                .on_press(Message::Sample)
                .view(look),
        ]
    };
    let two_line = vec![
        list_row("Home-5G")
            .subtitle("Secured · strong")
            .icon(Icon::Wifi)
            .selected(true)
            .on_press(Message::Sample)
            .view(look),
        list_row("FRITZ!Box 7590 XL Gästenetz OG")
            .subtitle("Secured · fair")
            .icon(Icon::Wifi)
            .on_press(Message::Sample)
            .view(look),
        list_row("Samsung SSD 990 PRO")
            .subtitle("nvme0n1 · 2 TB · Linux")
            .icon(Icon::HardDrive)
            .detail("2 TB")
            .on_press(Message::Sample)
            .view(look),
    ];
    let power = vec![
        list_row("Reboot")
            .icon(Icon::RestartAlt)
            .on_press(Message::Sample)
            .view(look),
        list_row("Shut down")
            .icon(Icon::PowerSettingsNew)
            .danger()
            .selected(true)
            .on_press(Message::Sample)
            .view(look),
    ];
    grid(look, 3, Space::S24)
        .push(list(look, rows(look), false))
        .push(list(look, two_line, true))
        .push(empty_list(look, "Nothing matches. Typo?"))
        .push(list(look, power, false))
        .into()
}

fn chips<'a>(look: &Look) -> Element<'a, Message> {
    let mut line = row(look, Space::S8).center();
    for kind in ChipKind::ALL {
        let label = match kind {
            ChipKind::Neutral => "2 TB",
            ChipKind::Selected => "English (US)",
            ChipKind::Accent => "Recommended",
            ChipKind::Cold => "Demo",
        };
        line = line.push(chip(look, kind, label, None, None, None::<Message>));
    }
    line = line
        .push(chip(
            look,
            ChipKind::Selected,
            "Default",
            Some(Icon::Check),
            None,
            None::<Message>,
        ))
        .push(chip(
            look,
            ChipKind::Cold,
            "Demo",
            Some(Icon::Bolt),
            None,
            None::<Message>,
        ))
        .push(chip(
            look,
            ChipKind::Neutral,
            "Italian",
            None,
            Some(Message::Sample),
            Some(Message::Sample),
        ))
        .push(chip(
            look,
            ChipKind::Selected,
            "English (US)",
            Some(Icon::Check),
            None,
            Some(Message::Sample),
        ));
    line.into()
}

fn keycaps<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let caps = row(look, Space::S8).center().extend(
        ["Esc", "Tab", "Enter", "↵", "K", "Ctrl", "← →", "+"]
            .into_iter()
            .map(|key| keycap(look, key)),
    );
    let inline = row(look, Space::S4)
        .center()
        .push(text("Press", Type::Body, c.on_surface))
        .push(keycap(look, "Space"))
        .push(text("to pick a layout, or", Type::Body, c.on_surface))
        .push(keycap(look, "Del"))
        .push(text("to drop it.", Type::Body, c.on_surface));
    stack(look, Space::S16)
        .push(caps)
        .push(key_hints(
            look,
            &[("Enter", "continue"), ("Ctrl+K", "actions"), ("Esc", "back")],
        ))
        .push(key_hints(
            look,
            &[("↵", "Open"), ("Tab", "Enter app"), ("Ctrl+K", "Actions")],
        ))
        .push(inline)
        .into()
}

/// The picture the media samples load, in each mode: the accent fading into the cold accent, made
/// once, since a handle made from pixels is uploaded again whenever it is made.
static PICTURES: LazyLock<[image::Handle; 2]> = LazyLock::new(|| [Mode::Dark, Mode::Light].map(gradient));

/// 16 by 10 pixels from `mode`'s accent on the left to its cold accent on the right.
fn gradient(mode: Mode) -> image::Handle {
    let palette = kubaos_ui::tokens::palette(mode);
    let (from, to) = (palette.primary, palette.cold);
    let (width, height) = (16_u32, 10_u32);
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..height {
        for x in 0..width {
            let t = x as f32 / (width - 1) as f32;
            let blend = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
            pixels.extend([blend(from.r, to.r), blend(from.g, to.g), blend(from.b, to.b), u8::MAX]);
        }
    }
    image::Handle::from_rgba(width, height, pixels)
}

fn media_samples<'a>(look: &Look) -> Element<'a, Message> {
    let picture = PICTURES[usize::from(look.mode == Mode::Light)].clone();
    grid(look, 4, Space::S16)
        .push(media(look, None, Aspect::Wallpaper, Radius::Control))
        .push(media(look, Some(picture.clone()), Aspect::Wallpaper, Radius::Control))
        .push(
            picture_choice(Some(picture.clone()), Aspect::Wallpaper, "Shinjuku, night")
                .picked(true)
                .mark(Icon::Check, "Current")
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            picture_choice(Some(picture), Aspect::Wallpaper, "Golden Gai")
                .name(NamePlace::Over)
                .badge(Icon::PlayArrow, "Video")
                .focused(true)
                .on_press(Message::Sample)
                .view(look),
        )
        .into()
}

fn cards<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mode = |title: &'static str, sub: &'static str| -> Element<'a, Message> {
        stack(look, Space::S4)
            .push(text(title, Type::Subheading, c.on_surface))
            .push(text(sub, Type::Small, c.on_surface_variant))
            .into()
    };
    grid(look, 4, Space::S24)
        .push(card(
            look,
            stack(look, Space::S4)
                .push(text("Plain card", Type::Subheading, c.on_surface))
                .push(text(
                    "The well's surface and a soft hairline, 16 inside.",
                    Type::Small,
                    c.on_surface_variant,
                )),
        ))
        .push(choice_card(
            look,
            mode("Erase the disk", "Everything on it goes."),
            true,
            Some(Message::Sample),
            false,
        ))
        .push(choice_card(
            look,
            mode("Install alongside", "Keep Windows, shrink it."),
            false,
            Some(Message::Sample),
            false,
        ))
        .push(choice_card(
            look,
            mode("Focused, picked", "The cold ring."),
            true,
            Some(Message::Sample),
            true,
        ))
        .into()
}

fn dialogs<'a>(look: &Look) -> Element<'a, Message> {
    let normal = dialog(
        look,
        Tone::Normal,
        Icon::PowerSettingsNew,
        "Restart now?",
        "Everything open closes. I'll be back in a few seconds.",
        Some(button("Cancel").kind(Kind::Text).on_press(Message::Sample).view(look)),
        [
            button("Shut down")
                .kind(Kind::Tonal)
                .icon(Icon::PowerSettingsNew)
                .on_press(Message::Sample)
                .view(look),
            button("Restart")
                .icon(Icon::RestartAlt)
                .on_press(Message::Sample)
                .focused(true)
                .view(look),
        ],
    );
    let danger = dialog(
        look,
        Tone::Danger,
        Icon::Warning,
        "Last chance",
        "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased. There's no undo.",
        Some(
            button("Cancel")
                .kind(Kind::Text)
                .on_press(Message::Sample)
                .focused(true)
                .view(look),
        ),
        [button("Erase and install")
            .kind(Kind::Danger)
            .icon(Icon::Warning)
            .on_press(Message::Sample)
            .view(look)],
    );
    let asking = prompt(
        look,
        Icon::AdminPanelSettings,
        "Authentication required",
        "Installing btop needs your password.",
        labelled(
            look,
            "Password for Kuba",
            None,
            field("hunter", "")
                .icon(Icon::Key)
                .secret()
                .state(State::Invalid)
                .view(look),
            Some(note(look, "That didn't work. Fat fingers? Try again.", true)),
        ),
        Some(button("Cancel").kind(Kind::Text).on_press(Message::Sample).view(look)),
        [button("Authenticate").on_press(Message::Sample).view(look)],
    );
    let stage = |d: Element<'a, Message>| -> Element<'a, Message> {
        widget::container(scrim(look, d))
            .height(Length::Fixed(f32::from(Size::Column.px() / 2 + Space::S64.px())))
            .into()
    };
    grid(look, 2, Space::S24)
        .push(stage(normal))
        .push(stage(danger))
        .push(stage(asking))
        .into()
}

fn progress<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let toasts = row(look, Space::S16)
        .center()
        .push(toast(look, "Copied. Evidence secured.", None))
        .push(toast(look, "Replay saved.", Some(Icon::CheckCircle)));
    let bars = stack(look, Space::S16)
        .push(bar(look, 0.0))
        .push(bar(look, 0.42))
        .push(bar(look, 1.0));
    let spinners = row(look, Space::S24)
        .center()
        .extend(
            IconSize::ALL
                .into_iter()
                .take(3)
                .map(|size| spinner(look, size, 0.7, c.primary)),
        )
        .push(text("Poking at the hardware…", Type::Body, c.on_surface_variant));
    let install = stack(look, Space::S12)
        .push(
            row(look, Space::S12)
                .bottom()
                .push(text("Installing", Type::Title, c.on_surface))
                .push(fill())
                .push(text("42%", Type::HeroNumber, c.on_surface)),
        )
        .push(bar(look, 0.42))
        .push(
            row(look, Space::S12)
                .center()
                .push(text("Unpacking the system", Type::BodyStrong, c.on_surface))
                .push(text("1.8 / 4.2 GB", Type::Mono, c.primary))
                .push(fill())
                .push(text("01:42", Type::Mono, c.on_surface_variant)),
        );
    stack(look, Space::S32)
        .push(toasts)
        .push(
            row(look, Space::S48)
                .push(stack(look, Space::S16).width(Extent::Fill).push(bars).push(spinners))
                .push(stack(look, Gap::NONE).width(Extent::Fill).push(install)),
        )
        .into()
}

fn empties<'a>(look: &Look) -> Element<'a, Message> {
    grid(look, 2, Space::S24)
        .push(card(
            look,
            empty_state(
                look,
                Icon::Wifi,
                "No networks",
                "Nothing in range. Plug in a cable, or skip it.",
                Some(button("Skip").kind(Kind::Tonal).on_press(Message::Sample).view(look)),
            ),
        ))
        .push(card(
            look,
            empty_state(look, Icon::Search, "Nothing matches", "Typo? Try fewer letters.", None),
        ))
        .into()
}

fn errors<'a>(look: &Look) -> Element<'a, Message> {
    let lines = [
        (Level::Step, "==> Bootloader"),
        (Level::Info, "Installing for x86_64-efi platform."),
        (Level::Debug, "efibootmgr -c -d /dev/nvme0n1 -p 1 -L ubuntu"),
        (
            Level::Warn,
            "warning: the firmware reports Secure Boot's boot order as locked",
        ),
        (
            Level::Error,
            "grub-install: error: efibootmgr failed to register the boot entry: Operation not permitted.",
        ),
    ];
    stack(look, Space::S16)
        .push(error_block(
            look,
            "efibootmgr failed to register the boot entry: Operation not permitted.",
            "The firmware refused a new boot entry. Turn off Secure Boot's \"lock boot order\" in the setup menu and try again.",
        ))
        .push(error_block(
            look,
            "grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=ubuntu failed with exit status 1.",
            "A message this long drops to body size.",
        ))
        .push(log(look, &lines, Extent::Fixed(Size::Log)))
        .push(
            row(look, Space::S12)
                .center()
                .push(button("Back to the review").kind(Kind::Text).icon(Icon::ArrowBack).on_press(Message::Sample).view(look))
                .push(button("Hide log").kind(Kind::Text).icon(Icon::Terminal).on_press(Message::Sample).view(look))
                .push(fill())
                .push(button("Save log").kind(Kind::Tonal).icon(Icon::Save).on_press(Message::Sample).view(look))
                .push(button("Try again").icon(Icon::RestartAlt).on_press(Message::Sample).focused(true).view(look)),
        )
        .into()
}

fn tab_rows<'a>(look: &Look) -> Element<'a, Message> {
    let views = [
        Tab::new("Wi-Fi"),
        Tab::new("Bluetooth"),
        Tab::new("Output"),
        Tab::new("Keyboard"),
    ];
    let tools = [Tab::new("Timer").icon(Icon::Timer), Tab::new("Alarm").icon(Icon::Alarm)];
    let motion = [Tab::new("Always"), Tab::new("On AC"), Tab::new("Never")];
    row(look, Space::S32)
        .center()
        .push(tabs(&views, 0, |_| Message::Sample).view(look))
        .push(tabs(&tools, 1, |_| Message::Sample).focused(true).view(look))
        .push(tabs(&motion, 1, |_| Message::Sample).view(look))
        .into()
}

fn tiles<'a>(look: &Look) -> Element<'a, Message> {
    grid(look, 4, Space::S16)
        .push(
            tile(Icon::SignalWifi4Bar, "Wi-Fi")
                .subtitle("Shinjuku-5G")
                .checked(true)
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            tile(Icon::Bluetooth, "Bluetooth")
                .subtitle("Off")
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            tile(Icon::Keyboard, "Keyboard")
                .subtitle("US · 1 of 2")
                .checked(true)
                .on_press(Message::Sample)
                .on_detail(Message::Sample)
                .view(look),
        )
        .push(
            tile(Icon::Key, "Proxy")
                .subtitle("Off")
                .on_press(Message::Sample)
                .on_detail(Message::Sample)
                .focused(true)
                .view(look),
        )
        .into()
}

/// A minute of a reading, a sample every two seconds: a wave the QML page draws too.
fn wave(level: f32, swing: f32, phase: f32) -> Vec<f32> {
    (0..30_u8)
        .map(|i| {
            let t = f32::from(i);
            level + swing * (t * 0.4 + phase).sin() + swing * 0.4 * (t * 1.3 + phase).sin()
        })
        .collect()
}

fn readings<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let meters = stack(look, Space::S16)
        .width(Extent::Fill)
        .push(meter(look, &[(0.55, c.primary), (0.15, c.tertiary)]))
        .push(meter(look, &[(0.92, c.warning)]))
        .push(meter(look, &[(0.3, c.secondary)]));
    let graphs = stack(look, Space::S16)
        .width(Extent::Fill)
        .push(sparkline(wave(40.0, 20.0, 0.0)).view(look))
        .push(
            sparkline(wave(1200.0, 600.0, 1.0))
                .second(wave(300.0, 150.0, 2.0))
                .max(0.0)
                .view(look),
        );
    row(look, Space::S48).push(meters).push(graphs).into()
}

fn sheets<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let frame = sheet::surface(look);
    let corner = radius(Radius::Surface);
    // The frame's top edge, which the sheet hangs from.
    let edge = widget::container(widget::Space::new())
        .width(Length::Fill)
        .height(f32::from(Space::S8.px()))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(frame)),
            border: iced::Border {
                radius: border::Radius::default().top(corner),
                ..iced::Border::default()
            },
            ..container::Style::default()
        });
    let body = stack(look, Space::S4)
        .padding(Inset::all(Space::S16))
        .push(text("Control center", Type::Subheading, c.on_surface))
        .push(text(
            "It grows out of the frame's edge.",
            Type::Small,
            c.on_surface_variant,
        ));
    let backdrop = kubaos_ui::color::alpha(c.surface_container_highest, Opacity::Well.value());
    let hanging = widget::container(centre_x(look.scale, sheet::sheet(look, body)))
        .padding(Inset::NONE.top(Space::S8).bottom(Space::S32).padding());
    // The hanging sheet first: a stack is as large as its first layer.
    widget::container(widget::Stack::with_children([hanging.into(), edge.into()]))
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(backdrop)),
            border: iced::Border::default().rounded(corner),
            ..container::Style::default()
        })
        .into()
}

/// The specimen's October 2026, weeks from Monday, as the shots' clock has it: Tuesday the 27th
/// is today, and All Saints' Day falls in the next month's first week.
fn october() -> Month {
    let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(str::to_owned);
    let weeks = (0..6_u8)
        .map(|week| Week {
            number: 40 + u32::from(week),
            days: std::array::from_fn(|weekday| {
                let weekday = u8::try_from(weekday).expect("a week has seven days");
                let index = week * 7 + weekday;
                // Monday the 28th of September starts the grid.
                let (month, day) = match index {
                    0..=2 => (9, 28 + index),
                    3..=33 => (10, index - 2),
                    _ => (11, index - 33),
                };
                Day {
                    date: format!("2026-{month:02}-{day:02}"),
                    day,
                    in_month: month == 10,
                    today: month == 10 && day == 27,
                    weekend: weekday >= 5,
                    holiday: (month == 11 && day == 1).then(|| "All Saints' Day".to_owned()),
                }
            }),
        })
        .collect();
    Month {
        year: 2026,
        month: 10,
        title: "October 2026".to_owned(),
        weekdays: names,
        weeks,
    }
}

static OCTOBER: LazyLock<Month> = LazyLock::new(october);

fn calendars<'a>(look: &Look) -> Element<'a, Message> {
    row(look, Space::S48)
        .push(
            calendar(&OCTOBER)
                .selected("2026-10-30")
                .on_pick(|_| Message::Sample)
                .on_move(|_| Message::Sample)
                .on_title(Message::Sample)
                .view(look),
        )
        .push(
            calendar(&OCTOBER)
                .year(true)
                .on_move(|_| Message::Sample)
                .on_title(Message::Sample)
                .on_month(|_| Message::Sample)
                .view(look),
        )
        .into()
}

/// `glyph` 24 in a small control's box, `mark` over its top right corner.
fn badged<'a>(look: &Look, glyph: Icon, mark: Element<'a, Message>) -> Element<'a, Message> {
    let c = look.colors;
    let side = f32::from(Size::ControlSm.px());
    widget::Stack::with_children([
        widget::container(kubaos_ui::snap::centre(
            look.scale,
            kubaos_ui::icon::icon(look, glyph, IconSize::S24, c.on_surface_variant),
        ))
        .width(side)
        .height(side)
        .into(),
        widget::container(mark)
            .width(side)
            .height(side)
            .align_x(iced::alignment::Horizontal::Right)
            .into(),
    ])
    .into()
}

fn avatars<'a>(look: &Look) -> Element<'a, Message> {
    let picture = PICTURES[usize::from(look.mode == Mode::Light)].clone();
    row(look, Space::S24)
        .center()
        .push(icon_tile(Icon::Terminal, TileSize::Sm).view(look))
        .push(icon_tile(Icon::Terminal, TileSize::Md).view(look))
        .push(
            icon_tile(Icon::Notifications, TileSize::Md)
                .photo(Some(picture))
                .view(look),
        )
        .push(icon_tile(Icon::Warning, TileSize::Md).danger().view(look))
        .push(badge(look, Some(3)))
        .push(badge(look, Some(128)))
        .push(badge(look, None))
        .push(badged(look, Icon::Notifications, badge(look, Some(3))))
        .push(badged(look, Icon::SystemUpdateAlt, badge(look, None)))
        .into()
}

fn skeletons<'a>(look: &Look) -> Element<'a, Message> {
    let scanning = row(look, Space::S12)
        .center()
        .push(skeleton(look, Bone::Circle(IconSize::S24)))
        .push(
            stack(look, Space::S4)
                .width(Extent::Fill)
                .push(skeleton(look, Bone::Line(Type::BodyStrong)))
                .push(skeleton(look, Bone::Line(Type::Small))),
        );
    grid(look, 2, Space::S24)
        .push(scanning)
        .push(skeleton(look, Bone::Block(Size::ControlLg, Radius::Control)))
        .into()
}
