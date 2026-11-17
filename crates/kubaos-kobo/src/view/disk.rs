//! The Disk screen: the disks, how the picked one is laid out before and after, the install modes
//! and the encryption settings.

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Length;
use iced::Shadow;
use iced::Vector;
use iced::widget;
use iced::widget::container;
use kubaos_core::probe::DiskInfo;
use kubaos_core::probe::Probe;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::component::card::card;
use kubaos_ui::component::card::choice_card;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::list::list;
use kubaos_ui::component::list::list_row;
use kubaos_ui::component::switch::switch;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::flow;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::row;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Shadow as ShadowToken;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::account::show_toggle;
use super::chrome::FOOTER_GAP;
use super::input;
use super::input_state;
use super::invalid;
use crate::app::App;
use crate::app::F;
use crate::app::List;
use crate::app::Message;
use crate::app::ids;
use crate::model;
use crate::model::DiskExt;
use crate::model::DiskMode;
use crate::model::human_bytes;
use crate::theme::jp;
use crate::ui;
use crate::ui::El;

/// Some disks have no model string (virtio, SD cards): name them by kind.
pub(super) fn disk_model(d: &DiskInfo) -> &str {
    let m = d.model.trim();
    let dev = d.path.trim_start_matches("/dev/");
    match m {
        "" if dev.starts_with("vd") || dev.starts_with("xvd") => "Virtual disk",
        "" if dev.starts_with("mmcblk") => "SD card",
        "" if dev.starts_with("nvme") => "NVMe disk",
        "" => "Unnamed disk",
        m => m,
    }
}

/// A partition's colour in the layout bar.
fn part_color(hint: Option<&str>, i: usize, look: &Look) -> Color {
    let c = look.colors;
    match hint {
        Some(h) if h.contains("Windows") => c.cold_container,
        Some(h) if h.contains("EFI") => c.secondary_container,
        _ => [c.surface_bright, c.surface_container_highest][i % 2],
    }
}

fn disk_row<'a>(d: &'a DiskInfo, sel: bool, small: bool, look: &Look) -> El<'a> {
    let mut row = list_row(disk_model(d).to_string())
        .subtitle(format!("{} · {}", d.path, human_bytes(d.size_bytes)))
        .icon(Icon::HardDrive);
    if let Some(os) = d.other_os() {
        row = row.chip(ChipKind::Neutral, os.to_string());
    }
    if small {
        row = row.chip(ChipKind::Neutral, "Too small");
    }
    if d.removable {
        row = row.chip(ChipKind::Neutral, "USB");
    }
    row.selected(sel)
        .checked(sel)
        .on_press(Message::Pick(List::Disk, d.path.clone()))
        .view(look)
}

/// The disk after the install: kept partitions, then ours, each as wide as its share of the disk
/// (a sliver at least, so a small one still shows), with a legend under it.
fn layout_bar<'a>(d: &'a DiskInfo, mode: DiskMode, encrypt: bool, look: &Look) -> El<'a> {
    let c = look.colors;
    let total = d.size_bytes.max(1) as f32;
    // Shares in thousandths of the disk, a hundredth and a half at least.
    let share = |bytes: u64| ((bytes as f32 / total) * 1000.0).max(14.0) as u16;
    let ours = format!("KubaOS{}", if encrypt { " · encrypted" } else { "" });
    let mut parts: Vec<(Color, u16, bool)> = vec![];
    let mut legend: Vec<El<'a>> = vec![];
    let legend_item = |colour: Color, name: String, bytes: u64| -> El<'a> {
        row(look, Space::S8)
            .center()
            .push(
                widget::container(widget::Space::new())
                    .width(f32::from(Size::RadioDot.px()))
                    .height(f32::from(Size::RadioDot.px()))
                    .style(move |_theme| {
                        container::Style::default()
                            .background(colour)
                            .border(Border::default().rounded(radius(Radius::Full)))
                    }),
            )
            .push(text(name, Type::Small, c.on_surface))
            .push(text(human_bytes(bytes), Type::Mono, c.on_surface_variant))
            .into()
    };
    match mode {
        DiskMode::Whole => {
            parts.push((c.primary, 1000, true));
            legend.push(legend_item(c.primary, ours, d.size_bytes));
        }
        DiskMode::FreeSpace => {
            let free = d.largest_free();
            let mut items: Vec<(u64, Option<usize>)> = vec![];
            // Partitions in order, with our region slotted after the last partition that ends
            // before it: good enough for a picture.
            let mut placed = false;
            let mut offset = 0u64;
            for (i, part) in d.partitions.iter().enumerate() {
                if let (Some(r), false) = (free, placed)
                    && offset >= r.start
                {
                    items.push((r.end - r.start, None));
                    placed = true;
                }
                offset += part.size_bytes;
                items.push((part.size_bytes, Some(i)));
            }
            if let (Some(r), false) = (free, placed) {
                items.push((r.end - r.start, None));
            }
            for (bytes, which) in items {
                if let Some(i) = which {
                    let part = &d.partitions[i];
                    let colour = part_color(part.os_hint.as_deref(), i, look);
                    parts.push((colour, share(bytes), false));
                    if bytes as f32 / total > 0.05 || part.os_hint.as_deref().is_some_and(|h| h.contains("EFI")) {
                        let name = part
                            .label
                            .clone()
                            .or(part.os_hint.clone())
                            .unwrap_or_else(|| part.path.rsplit('/').next().unwrap_or("").to_string());
                        legend.push(legend_item(colour, name, bytes));
                    }
                } else {
                    parts.push((c.primary, share(bytes), true));
                    legend.push(legend_item(c.primary, ours.clone(), bytes));
                }
            }
        }
    }
    let weights: Vec<u16> = parts.iter().map(|&(_, w, _)| w).collect();
    let glow = color::alpha(c.primary, ShadowToken::Glow.opacity());
    let segments = parts.into_iter().map(|(colour, _, ours)| -> El<'a> {
        widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(f32::from(Size::ControlSm.px()))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(colour)),
                border: Border::default().rounded(radius(Radius::Small)),
                shadow: if ours {
                    Shadow {
                        color: glow,
                        offset: Vector::new(0.0, f32::from(ShadowToken::Glow.y())),
                        blur_radius: f32::from(ShadowToken::Glow.blur()),
                    }
                } else {
                    Shadow::default()
                },
                ..container::Style::default()
            })
            .into()
    });
    stack(look, Space::S12)
        .push(split(look, &weights, Space::S4).extend(segments))
        .push(flow(look, Space::S16, legend))
        .into()
}

/// One of the two ways to use the disk: its symbol and name, and what it means for this disk.
fn mode_card<'a>(
    app: &'a App,
    glyph: Icon,
    title: &'a str,
    sub: String,
    mode: DiskMode,
    on_pick: Option<Message>,
    look: &Look,
) -> El<'a> {
    let c = look.colors;
    let picked = app.form.disk_mode.unwrap_or(DiskMode::Whole) == mode;
    let dim = |colour: Color| {
        if on_pick.is_some() {
            colour
        } else {
            color::alpha(colour, Opacity::DisabledContent.value())
        }
    };
    let (main, muted) = if picked {
        (
            c.on_secondary_container,
            color::alpha(c.on_secondary_container, Opacity::SubduedText.value()),
        )
    } else {
        (c.on_surface, c.on_surface_variant)
    };
    let content = stack(look, Space::S4)
        .push(
            row(look, Space::S8)
                .center()
                .push(icon(look, glyph, IconSize::S20, dim(main)))
                .push(text(title, Type::Subheading, dim(main))),
        )
        .push(text(sub, Type::Small, dim(muted)));
    choice_card(look, content, picked, on_pick, picked && app.is_focused(F::DiskMode))
}

/// A setting with a switch: its symbol, its name and what it does, the switch at the end.
fn setting<'a>(app: &'a App, glyph: Icon, title: &'a str, sub: &'a str, on: bool, f: F, look: &Look) -> El<'a> {
    let c = look.colors;
    row(look, Space::S16)
        .center()
        .push(icon(look, glyph, IconSize::S24, c.primary))
        .push(
            stack(look, Space::S4)
                .width(Extent::Fill)
                .push(text(title, Type::BodyStrong, c.on_surface))
                .push(text(sub, Type::Small, c.on_surface_variant)),
        )
        .push(switch(
            look,
            on,
            app.switch_progress(f),
            Some(Message::Click(f)),
            app.is_focused(f),
        ))
        .into()
}

pub(super) fn disk<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let Some(probe) = app.ready_probe() else {
        return widget::Space::new().into();
    };
    let f = &app.form;
    let rows: Vec<El> = probe
        .disks
        .iter()
        .filter(|d| d.is_target())
        .map(|d| disk_row(d, f.disk.as_deref() == Some(&d.path), !d.fits(probe), look))
        .collect();
    let disks = widget::container(list(look, rows, app.is_focused(F::DiskList))).id(ids::REVEAL_DISKS);

    let mut sections = stack(look, Space::S16)
        .padding(Inset::NONE.right(Space::S16))
        .push(disks);
    if let Some(d) = f.selected_disk(probe) {
        let mode = f.disk_mode.unwrap_or(DiskMode::Whole);
        sections = sections.push(disk_modes(app, d, probe, look)).push(
            stack(look, Space::S8)
                .push(text("After the install", Type::Label, c.on_surface_variant))
                .push(layout_bar(d, mode, f.encrypt, look)),
        );
    }
    sections = sections.push(disk_encryption(app, look));
    // The title stays put; only what's under it scrolls (src/reveal.rs).
    let apart = Space::S20;
    stack(look, apart)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::DISK,
            "Where it goes",
            "Pick a disk, then decide whether whatever's on it gets to live.",
        ))
        .push(
            scroll(look, sections)
                .id(ids::DISK_SCROLL)
                .height(Extent::Fill)
                .room_above(apart)
                .room_below(FOOTER_GAP),
        )
        .into()
}

/// The two ways to use the selected disk: beside what's on it, or all of it.
fn disk_modes<'a>(app: &'a App, d: &DiskInfo, probe: &Probe, look: &Look) -> El<'a> {
    let free = d.largest_free().map_or(0, |r| r.end - r.start);
    let others = d.other_os().map(str::to_string);
    let free_sub = if d.fits_dual_boot(probe) {
        format!(
            "Keep {} and use the {} of free space.",
            others.as_deref().unwrap_or("everything"),
            human_bytes(free)
        )
    } else if free > 0 {
        format!("Only {} free. Shrink something first.", human_bytes(free))
    } else {
        "No free space on this disk.".to_string()
    };
    let whole_sub = match d.partitions.len() {
        _ if !d.fits(probe) => format!("Too small: KubaOS needs {}.", human_bytes(model::needed(probe))),
        0 => "The disk is empty anyway.".to_string(),
        n => format!(
            "Wipes all {n} partition{}{}.",
            if n == 1 { "" } else { "s" },
            others.map(|o| format!(", {o} included")).unwrap_or_default()
        ),
    };
    let modes = grid(look, 2, Space::S16)
        .push(mode_card(
            app,
            Icon::Storage,
            "Install alongside",
            free_sub,
            DiskMode::FreeSpace,
            d.fits_dual_boot(probe)
                .then_some(Message::DiskModePick(DiskMode::FreeSpace)),
            look,
        ))
        .push(mode_card(
            app,
            Icon::Warning,
            "Erase the whole disk",
            whole_sub,
            DiskMode::Whole,
            Some(Message::DiskModePick(DiskMode::Whole)),
            look,
        ));
    widget::container(modes).id(ids::REVEAL_MODE).into()
}

fn disk_encryption<'a>(app: &'a App, look: &Look) -> El<'a> {
    let f = &app.form;
    let mut encryption = stack(look, Space::S16).push(
        widget::container(setting(
            app,
            if f.encrypt { Icon::Lock } else { Icon::LockOpen },
            "Encrypt the disk",
            "LUKS2. Lose the laptop, keep your secrets. You type the passphrase once at boot.",
            f.encrypt,
            F::Encrypt,
            look,
        ))
        .id(ids::REVEAL_ENCRYPT),
    );
    if f.encrypt {
        encryption = encryption.push(
            widget::container(setting(
                app,
                Icon::Key,
                "Use my account password",
                "One password for boot and login.",
                f.luks_same_as_password,
                F::LuksSame,
                look,
            ))
            .id(ids::REVEAL_LUKS_SAME),
        );
        if !f.luks_same_as_password {
            let mismatch = ui::mismatched(&f.luks, &f.luks_confirm);
            encryption = encryption.push(
                widget::container(
                    grid(look, 2, Space::S16)
                        .push(labelled(
                            look,
                            "Encryption passphrase",
                            None,
                            stack(look, Space::S8)
                                .push(input(app, F::Luks, "", None, look))
                                .push(ui::strength(look, &f.luks)),
                            None,
                        ))
                        .push(labelled(
                            look,
                            "Confirm the passphrase",
                            Some(show_toggle(app, look)),
                            input_state(app, F::LuksConfirm, "", None, invalid(mismatch), look),
                            ui::confirm_note(look, &f.luks, &f.luks_confirm),
                        )),
                )
                .id(ids::REVEAL_LUKS),
            );
        }
    }
    card(look, encryption)
}
