//! The Summary screen: what is about to happen, block by block, and the confirmation before the
//! disk is erased.

use iced::Length;
use iced::widget;
use kubaos_core::InstallConfig;
use kubaos_core::probe::DiskInfo;
use kubaos_core::probe::Probe;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::component::banner::BannerTone;
use kubaos_ui::component::banner::banner;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::card::card_sized;
use kubaos_ui::component::description::Fact;
use kubaos_ui::component::description::description_list;
use kubaos_ui::component::keycap::keycap;
use kubaos_ui::component::progress::danger_timed_bar;
use kubaos_ui::component::progress::timed_bar;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::row;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::disk::disk_model;
use super::installing::low_battery_note;
use super::wallpapers::wallpapers_summary;
use crate::app::App;
use crate::app::F;
use crate::app::HOLD;
use crate::app::Message;
use crate::app::ids;
use crate::data;
use crate::model;
use crate::model::DiskExt;
use crate::model::DiskMode;
use crate::model::human_bytes;
use crate::theme::jp;
use crate::ui::El;

/// A card on Last chance, as tall as the one beside it.
fn block<'a>(id: &'static str, glyph: Icon, title: &'a str, facts: Vec<Fact<'a>>, look: &Look) -> El<'a> {
    let c = look.colors;
    widget::container(card_sized(
        look,
        stack(look, Space::S12)
            .push(
                row(look, Space::S8)
                    .center()
                    .push(icon(look, glyph, IconSize::S20, c.primary))
                    .push(text(title, Type::Subheading, c.on_surface)),
            )
            .push(description_list(look, facts)),
        Extent::Fill,
    ))
    .id(id)
    .height(Length::Fill)
    .into()
}

pub(super) fn summary<'a>(app: &'a App, look: &Look) -> El<'a> {
    let Some(probe) = app.ready_probe() else {
        return widget::Space::new().into();
    };
    let f = &app.form;
    let cfg = f.to_config(probe);
    let disk = f.selected_disk(probe);
    let whole = f.disk_mode != Some(DiskMode::FreeSpace);

    let lang = summary_language(app, look);
    let acct = summary_account(app, &cfg, look);
    let disk_block = summary_disk(app, probe, disk, whole, look);
    let sys = summary_system(app, probe, &cfg, look);

    let mut top = stack(look, Space::S12).push(header(
        look,
        jp::SUMMARY,
        "Last chance",
        "Nothing's been written yet. Check it over, then hold the button.",
    ));
    // One line, so the warning takes no more of the cards' room than it needs, ending where the
    // cards do, short of their scrollbar's gutter.
    if let Some(note) = low_battery_note(app) {
        top = top.push(
            widget::container(banner(BannerTone::Warning, Icon::Warning, note, "").view(look))
                .padding(Inset::NONE.right(Space::S16).padding()),
        );
    }

    let apart = Space::S20;
    stack(look, apart)
        .height(Extent::Fill)
        .push(top)
        .push(
            scroll(
                look,
                widget::container(
                    grid(look, 2, Space::S16)
                        .uniform_rows()
                        .push(lang)
                        .push(acct)
                        .push(disk_block)
                        .push(sys),
                )
                .padding(Inset::NONE.right(Space::S16).padding()),
            )
            .id(ids::SUMMARY_SCROLL)
            .height(Extent::Fill)
            .room_above(apart)
            .room_below(apart),
        )
        // The cards keep a gutter for their scrollbar; the banner ends where they do.
        .push(widget::container(confirm(app, disk, whole, look)).padding(Inset::NONE.right(Space::S16).padding()))
        .into()
}

fn summary_language<'a>(app: &'a App, look: &Look) -> El<'a> {
    let f = &app.form;
    block(
        ids::SUMMARY_LANGUAGE,
        Icon::Language,
        "Language",
        vec![
            Fact::new("Language", data::locale_name(&f.locale).into_owned()),
            Fact::new(
                "Keyboard",
                f.keymaps
                    .iter()
                    .map(|k| data::keymap_name(k))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Fact::new("Time zone", {
                let (city, region) = data::split_tz(&f.timezone);
                if region.is_empty() {
                    city
                } else {
                    format!("{city} ({region})")
                }
            }),
        ],
        look,
    )
}

fn summary_account<'a>(app: &'a App, cfg: &InstallConfig, look: &Look) -> El<'a> {
    let f = &app.form;
    block(
        ids::SUMMARY_ACCOUNT,
        Icon::Person,
        "You",
        vec![
            Fact::new("Name", f.full_name.clone()),
            Fact::new("Login", format!("{}@{}", cfg.user.username, cfg.host.hostname)),
            match cfg.git.as_ref() {
                Some(g) => Fact::new("Git", g.name.clone()).detail(g.email.clone()),
                None => Fact::new("Git", "Later".to_owned()),
            },
        ],
        look,
    )
}

fn summary_disk<'a>(app: &'a App, probe: &Probe, disk: Option<&DiskInfo>, whole: bool, look: &Look) -> El<'a> {
    let f = &app.form;
    block(
        ids::SUMMARY_DISK,
        Icon::HardDrive,
        "Disk",
        vec![
            match disk {
                Some(d) => {
                    Fact::new("Disk", disk_model(d)).detail(format!("{} · {}", d.path, human_bytes(d.size_bytes)))
                }
                None => Fact::new("Disk", "None picked".to_owned()),
            },
            Fact::new(
                "Mode",
                if whole {
                    "Erase everything".to_owned()
                } else {
                    format!(
                        "Alongside, in {} free",
                        human_bytes(f.free_region(probe).map_or(0, |r| r.end - r.start))
                    )
                },
            ),
            Fact::new(
                "Encryption",
                match (f.encrypt, f.luks_same_as_password) {
                    (false, _) => "Off. Brave.".to_owned(),
                    (true, true) => "LUKS2, account password".to_owned(),
                    (true, false) => "LUKS2, its own passphrase".to_owned(),
                },
            ),
        ],
        look,
    )
}

fn summary_system<'a>(app: &'a App, probe: &Probe, cfg: &InstallConfig, look: &Look) -> El<'a> {
    let m = probe.payload.as_ref();
    block(
        ids::SUMMARY_SYSTEM,
        Icon::DeployedCode,
        "System",
        vec![
            Fact::new("Release", format!("KubaOS {}", app.codename())),
            Fact::new(
                "Image",
                m.map(|m| format!("{} · {}", m.version, m.commit.chars().take(7).collect::<String>()))
                    .unwrap_or_default(),
            )
            .detail(m.map(|m| format!("Linux {}", m.kernel)).unwrap_or_default()),
            Fact::new(
                "Wallpapers",
                wallpapers_summary(cfg.wallpapers.as_ref(), app.wallpaper_sets()),
            ),
            Fact::new(
                "Network",
                app.form
                    .wifi_ssid
                    .clone()
                    .map_or_else(|| "None".to_owned(), |s| format!("Wi-Fi “{s}”")),
            ),
        ],
        look,
    )
}

/// What the install does to the disk, as a banner holding Back and the button to hold, the hold's
/// bar along its foot. Erasing is a danger banner, really red, naming the disk, with no jokes in
/// that sentence (docs/BRAND.md, voice rule 4); going into free space is an info banner.
fn confirm<'a>(app: &'a App, disk: Option<&'a DiskInfo>, whole: bool, look: &Look) -> El<'a> {
    let c = look.colors;
    let held = app
        .hold
        .map_or(0.0, |s| ((app.now - s).as_secs_f32() / HOLD.as_secs_f32()).min(1.0));
    let disk_name = disk
        .map(|d| {
            format!(
                "{} ({}, {})",
                disk_model(d),
                d.path.trim_start_matches("/dev/"),
                model::human_bytes_nb(d.size_bytes)
            )
        })
        .unwrap_or_default();
    let (tone, glyph, kind, message, sub, cta, ink) = if whole {
        let n = disk.map_or(0, |d| d.partitions.len());
        let others = disk
            .and_then(|d| d.other_os())
            .map(|o| format!(", {o} included"))
            .unwrap_or_default();
        (
            BannerTone::Danger,
            Icon::Warning,
            Kind::Danger,
            format!("Last chance to not fuck this up. Everything on {disk_name} gets erased."),
            if n == 0 {
                "There's no undo.".to_string()
            } else {
                format!(
                    "All {n} partition{}{others}. There's no undo.",
                    if n == 1 { "" } else { "s" }
                )
            },
            "Hold to erase and install",
            color::alpha(c.on_error_container, Opacity::SubduedText.value()),
        )
    } else {
        (
            BannerTone::Info,
            Icon::Storage,
            Kind::Filled,
            format!("KubaOS goes into the free space on {disk_name}."),
            format!(
                "{} stays where it is.",
                disk.and_then(|d| d.other_os()).unwrap_or("Everything else")
            ),
            "Hold to install",
            c.on_surface_variant,
        )
    };
    let label = if app.hold.is_some() { "Keep holding…" } else { cta };
    let on_banner = kubaos_ui::component::banner::inside(look, tone);
    let back = button("Back")
        .kind(Kind::Text)
        .icon(Icon::ArrowBack)
        .on_press(Message::Click(F::Back))
        .focused(app.is_focused(F::Back))
        .view(&on_banner);
    let hold_button = stack(look, Space::S8)
        .push(
            button(label)
                .kind(kind)
                .size(ButtonSize::Lg)
                .icon(Icon::Download)
                .on_hold(Message::HoldStart, Message::HoldEnd)
                .focused(app.is_focused(F::Hold))
                .view(look),
        )
        .push(
            row(look, Space::S4)
                .center()
                .push(text("or hold", Type::Caption, ink))
                .push(keycap(look, "Enter")),
        );
    banner(tone, glyph, message, sub)
        .actions([back, hold_button.into()])
        .foot(if tone == BannerTone::Danger {
            danger_timed_bar(look, held)
        } else {
            timed_bar(look, held)
        })
        .view(look)
}
