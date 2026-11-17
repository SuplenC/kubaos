//! The Wallpapers screen's logic: the arrow keys through each set's grid of thumbnails, and
//! decoding the thumbnails off the UI thread.

use std::path::PathBuf;

use iced::Task;
use iced::widget::image;

use super::App;
use super::F;
use super::Message;

impl App {
    /// Decodes the wallpaper thumbnails from `payload_dir` right away (the screenshots).
    #[cfg(test)]
    pub fn load_thumbs_now(&mut self) {
        let sets = self.wallpaper_sets().to_vec();
        for set in &sets {
            self.thumbs.extend(decoded_thumbs(&self.payload_dir, set));
        }
    }

    /// The arrow keys on the Wallpapers screen: through a set's grid of thumbnails, from its
    /// header into it, and past its first or last row to the set above or below.
    pub(super) fn move_in_wallpapers(&mut self, from: F, dir: Dir) -> Task<Message> {
        let Some(to) = self.wallpaper_neighbour(from, dir) else {
            return Task::none();
        };
        self.set_focus(Some(to));
        self.reveal()
    }

    /// Where an arrow key goes from a header or a thumbnail; `None` at the edge.
    pub(crate) fn wallpaper_neighbour(&self, from: F, dir: Dir) -> Option<F> {
        let sets = self.wallpaper_sets();
        let files = |s: usize| sets.get(s).map_or(0, |set| set.files.len());
        let cols = WALLPAPER_COLUMNS;
        // The set above's last row, at its first thumbnail; its header when it has none.
        let above = |s: usize| -> Option<F> {
            let s = s.checked_sub(1)?;
            Some(match files(s) {
                0 => F::WallpaperSet(s),
                n => F::Wallpaper(s, (n - 1) / cols * cols),
            })
        };
        let below = |s: usize| (s + 1 < sets.len()).then_some(F::WallpaperSet(s + 1));
        match (from, dir) {
            (F::WallpaperSet(s), Dir::Up) => above(s),
            (F::WallpaperSet(s), Dir::Down) => match files(s) {
                0 => below(s),
                _ => Some(F::Wallpaper(s, 0)),
            },
            (F::Wallpaper(s, i), Dir::Left) => i.checked_sub(1).map(|j| F::Wallpaper(s, j)),
            (F::Wallpaper(s, i), Dir::Right) => (i + 1 < files(s)).then_some(F::Wallpaper(s, i + 1)),
            (F::Wallpaper(s, i), Dir::Up) => Some(match i.checked_sub(cols) {
                Some(j) => F::Wallpaper(s, j),
                None => F::WallpaperSet(s),
            }),
            (F::Wallpaper(s, i), Dir::Down) => {
                let n = files(s);
                if i + cols < n {
                    Some(F::Wallpaper(s, i + cols))
                } else if i / cols < (n - 1) / cols {
                    // The row below is short: its last thumbnail.
                    Some(F::Wallpaper(s, n - 1))
                } else {
                    below(s)
                }
            }
            _ => None,
        }
    }

    /// Decodes every set's thumbnails, a thread per set, so the screen fills in as they come.
    pub(super) fn load_thumbs(&self) -> Task<Message> {
        let dir = self.payload_dir.clone();
        Task::batch(
            self.wallpaper_sets()
                .iter()
                .cloned()
                .map(|set| Task::perform(load_thumbs(dir.clone(), set), Message::Thumbs)),
        )
    }
}

/// Thumbnails in a row on the Wallpapers screen (`view/wallpapers.rs`, `wallpapers`).
pub const WALLPAPER_COLUMNS: usize = 4;

/// An arrow key's direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// The container id of a header or thumbnail on the Wallpapers screen, which `reveal` scrolls to.
pub fn wallpaper_item(f: F) -> Option<String> {
    match f {
        F::WallpaperSet(s) => Some(format!("wallpaper-set-{s}")),
        F::Wallpaper(s, i) => Some(format!("wallpaper-{s}-{i}")),
        _ => None,
    }
}

/// A set's thumbnails, decoded on a thread of their own (`thumbs::decode`), so a few dozen of them
/// never hold up the UI thread's frames.
async fn load_thumbs(payload: PathBuf, set: kubaos_core::probe::WallpaperSet) -> Vec<(String, image::Handle)> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(decoded_thumbs(&payload, &set));
    });
    rx.await.unwrap_or_default()
}

fn decoded_thumbs(payload: &std::path::Path, set: &kubaos_core::probe::WallpaperSet) -> Vec<(String, image::Handle)> {
    crate::thumbs::decode(payload, std::path::Path::new(crate::thumbs::ORIGINALS), set)
        .into_iter()
        .map(|(key, img)| {
            // Every card's picture is the same shape, so the cards are all one height.
            let img = crate::thumbs::cropped(&img);
            let (w, h) = img.dimensions();
            (key, image::Handle::from_rgba(w, h, img.into_raw()))
        })
        .collect()
}
