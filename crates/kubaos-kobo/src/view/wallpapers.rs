//! The Wallpapers screen: the sets as a grid of cards, and the summary line the review shows.

use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::check::Tick;
use kubaos_ui::component::check::checkbox;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::media::Aspect;
use kubaos_ui::component::picture::NamePlace;
use kubaos_ui::component::picture::picture_choice;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::row;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use super::chrome::FOOTER_GAP;
use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::app::WALLPAPER_COLUMNS;
use crate::app::ids;
use crate::app::wallpaper_item;
use crate::model;
use crate::theme::jp;
use crate::ui::El;

/// `cyberpunk` -> "Cyberpunk", `neo-tokyo` -> "Neo tokyo".
fn set_name(id: &str) -> String {
    let s = id.replace(['-', '_'], " ");
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// `kabukicho-red-gate.jpg` -> "kabukicho red gate", as the shell's picker labels it.
fn wallpaper_label(name: &str) -> String {
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    stem.replace(['-', '_'], " ")
}

/// The Summary's line for `[wallpapers]`: the sets picked whole by name, the others with how many
/// of theirs.
pub fn wallpapers_summary(
    w: Option<&kubaos_core::config::Wallpapers>,
    sets: &[kubaos_core::probe::WallpaperSet],
) -> String {
    let Some(w) = w else {
        return "Every set".into();
    };
    if w.sets.is_empty() && w.files.is_empty() {
        return "Only your own".into();
    }
    let mut parts = Vec::new();
    for set in sets {
        let picked = w
            .files
            .iter()
            .filter(|f| f.split_once('/').is_some_and(|(id, _)| id == set.id))
            .count();
        if w.sets.contains(&set.id) {
            parts.push(set_name(&set.id));
        } else if picked > 0 {
            parts.push(format!("{} ({picked} of {})", set_name(&set.id), set.files.len()));
        }
    }
    parts.join(", ")
}

pub(super) fn wallpapers<'a>(app: &'a App, look: &Look) -> El<'a> {
    let sets = app.wallpaper_sets();
    let mut sections = stack(look, Space::S40).padding(Inset::NONE.right(Space::S16));
    for (s, set) in sets.iter().enumerate() {
        let cards = (0..set.files.len()).map(|i| wallpaper_card(app, s, i, look));
        sections = sections.push(
            stack(look, Space::S12)
                .push(wallpaper_set_header(app, s, set, look))
                .push(grid(look, WALLPAPER_COLUMNS as u16, Space::S16).extend(cards)),
        );
    }
    let apart = Space::S20;
    stack(look, apart)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::WALLPAPERS,
            "Wallpapers",
            "Pick the ones the wallpaper picker shows; a set's name takes all of it. Every one gets installed \
             either way (later: ~/.config/kubaos/desktop/wallpapers/sets). Your own images go in \
             ~/Pictures/wallpapers.",
        ))
        .push(
            scroll(look, sections)
                .id(ids::WALLPAPER_SCROLL)
                .height(Extent::Fill)
                .room_above(apart)
                .room_below(FOOTER_GAP),
        )
        .into()
}

/// A set's header: its checkbox with its name, and a chip with how many of its wallpapers are
/// picked. Space or a click picks the rest of it, or drops it when it's all picked.
fn wallpaper_set_header<'a>(app: &'a App, s: usize, set: &'a kubaos_core::probe::WallpaperSet, look: &Look) -> El<'a> {
    let on = app.form.wallpapers_on(set);
    let tick = Tick::of(on, set.images);
    let count = match (tick, set.images) {
        (Tick::Full | Tick::Empty, 1) => "1 wallpaper".to_string(),
        (Tick::Full, n) => format!("all {n}"),
        (Tick::Empty, n) => format!("{n} wallpapers"),
        (Tick::Partial, n) => format!("{on} of {n}"),
    };
    let kind = match tick {
        Tick::Full => ChipKind::Selected,
        Tick::Partial | Tick::Empty => ChipKind::Neutral,
    };
    let f = F::WallpaperSet(s);
    widget::container(
        row(look, Space::S12)
            .center()
            .push(checkbox(
                look,
                set_name(&set.id),
                tick,
                Some(Message::Click(f)),
                app.is_focused(f),
            ))
            .push(chip(kind, count).view(look)),
    )
    .id(wallpaper_item(f).unwrap_or_default())
    .into()
}

/// A wallpaper to pick: the picture over the whole tile, its name over its foot; picked, the tube
/// and a check. Its box keeps the wallpaper's aspect until the thumbnail is decoded (or for good,
/// when it can't be), so the grid never jumps.
fn wallpaper_card<'a>(app: &'a App, s: usize, i: usize, look: &Look) -> El<'a> {
    let set = &app.wallpaper_sets()[s];
    let file = &set.files[i];
    let f = F::Wallpaper(s, i);
    let mut tile = picture_choice(
        app.thumbs.get(&model::wallpaper_key(set, file)).cloned(),
        Aspect::Wallpaper,
        wallpaper_label(&file.name),
    )
    .name(NamePlace::Over)
    .picked(app.form.wallpaper_on(set, file))
    .mark(Icon::Check, "Picked")
    .focused(app.is_focused(f))
    .on_press(Message::Click(f));
    if file.is_video() {
        tile = tile.badge(Icon::PlayArrow, "Video");
    }
    widget::container(tile.view(look))
        .id(wallpaper_item(f).unwrap_or_default())
        .into()
}
