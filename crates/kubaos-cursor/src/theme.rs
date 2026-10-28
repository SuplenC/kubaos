//! The generated theme directory, `~/.local/share/icons/KubaOS-Bibata/`, and how it changes:
//!
//! ```text
//! KubaOS-Bibata/
//!   index.theme     Inherits=Bibata-Modern-Classic, for every cursor this theme lacks
//!   .stamp          what the theme was made from (`stamp`)
//!   manifest.hl     the hyprcursor theme, absent for the classic style
//!   hyprcursors/    one <name>.hlc per cursor
//!   cursors/        the XCursor theme, written after the rest; its .stamp adds the size
//! ```
//!
//! Every change is built beside the theme under a dot name and swapped in with one rename, so a
//! half-written theme is never the live one. With no `cursors/` yet, XCursor clients get
//! Bibata-Modern-Classic through `Inherits`; without the directory at all, the `default` theme
//! home-manager writes inherits it too. The pointer is never invisible.
//!
//! # Not here
//!
//! - Deciding when to write which part, and telling Hyprland: `kubaos-cli`'s `cursor.rs`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

use crate::Colors;
use crate::FALLBACK_THEME;
use crate::Shape;
use crate::Style;
use crate::hyprcursor;
use crate::raster::Drawing;
use crate::raster::RasterError;
use crate::recolour::recolour;
use crate::source::CANVAS;
use crate::tokens;
use crate::xcursor;

/// Bumped whenever the files written for the same inputs change, so every machine rewrites its
/// theme once after an update.
const FORMAT: u32 = 1;
const STAMP: &str = ".stamp";
const CURSORS_DIR: &str = "cursors";

/// Why a theme couldn't be written.
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("writing {}", .0.display())]
    Write(PathBuf, #[source] io::Error),
    #[error("swapping {} into place", .0.display())]
    Swap(PathBuf, #[source] io::Error),
    #[error("rendering the cursor {0}")]
    Render(String, #[source] RasterError),
}

/// What [`Theme::write_xcursor`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XcursorOutcome {
    /// The XCursor theme is live.
    Written,
    /// The theme changed while it rendered, so its work was dropped; the run for the new theme
    /// writes it.
    Superseded,
}

/// What a theme was made from: the style and its colours, and the sources. A theme whose stamp
/// matches the wanted one is left as it is.
pub fn stamp(style: Style, colors: Option<&Colors>, source: &Path) -> String {
    let head = format!("kubaos-cursor {FORMAT}\nstyle {style}\n");
    match colors {
        Some(Colors {
            fill,
            outline,
            watch,
            spinner: [a, b, c, d],
        }) => format!(
            "{head}fill {fill}\noutline {outline}\nwatch {watch}\nspinner {a} {b} {c} {d}\nsource {}\n",
            source.display()
        ),
        None => head,
    }
}

/// The stamp of the XCursor part: the theme's, and the sizes it's rendered at.
pub fn xcursor_stamp(theme_stamp: &str, size: u32) -> String {
    let sizes: Vec<String> = xcursor_sizes(size).iter().map(u32::to_string).collect();
    format!("{theme_stamp}sizes {}\n", sizes.join(" "))
}

/// The XCursor sizes for the setting `size`: itself at every display scale the desktop supports, so
/// an XWayland or GTK 3 app on any output gets a pointer drawn at its size rather than the nearest
/// one scaled. A size on the grid is whole device pixels at every one of them.
pub fn xcursor_sizes(size: u32) -> Vec<u32> {
    let mut sizes: Vec<u32> = tokens::SCALES
        .iter()
        .map(|&scale| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss,
                reason = "A cursor of at most 96 px at a scale of at most 2 is exact in f32 and fits u32."
            )]
            let px = (size as f32 * scale).round() as u32;
            px
        })
        .collect();
    sizes.dedup();
    sizes
}

/// The generated theme in an icons directory.
#[derive(Clone, Debug)]
pub struct Theme {
    icons: PathBuf,
    name: String,
}

impl Theme {
    #[must_use]
    pub fn new(icons: impl Into<PathBuf>, name: impl Into<String>) -> Self {
        Self {
            icons: icons.into(),
            name: name.into(),
        }
    }

    pub fn dir(&self) -> PathBuf {
        self.icons.join(&self.name)
    }

    /// The stamp of the live theme, if it has one.
    pub fn stamp(&self) -> Option<String> {
        fs::read_to_string(self.dir().join(STAMP)).ok()
    }

    /// The stamp of the live XCursor part, if it has one.
    pub fn xcursor_stamp(&self) -> Option<String> {
        fs::read_to_string(self.dir().join(CURSORS_DIR).join(STAMP)).ok()
    }

    /// Makes the theme the hyprcursor theme of `shapes` in `colors`, keeping the live XCursor
    /// part until its own render replaces it.
    ///
    /// # Errors
    ///
    /// [`ThemeError`] naming the file that couldn't be written or swapped in. The live theme is
    /// then unchanged.
    pub fn write_hyprcursor(&self, shapes: &[Shape], colors: &Colors, stamp: &str) -> Result<(), ThemeError> {
        let new = self.scratch("new");
        let result = (|| {
            self.write_base(&new, stamp)?;
            write(&new.join("manifest.hl"), hyprcursor::manifest(&self.name).as_bytes())?;
            let cursors = new.join(hyprcursor::CURSORS_DIR);
            create_dir(&cursors)?;
            for shape in shapes {
                write(
                    &cursors.join(format!("{}.hlc", shape.name)),
                    &hyprcursor::archive(shape, colors),
                )?;
            }
            self.keep_xcursor(&new);
            swap(&new, &self.dir())
        })();
        let _ = fs::remove_dir_all(&new);
        result
    }

    /// Makes the theme a stub that inherits Bibata-Modern-Classic: the `classic` style, and the
    /// fallback after a failed render.
    ///
    /// # Errors
    ///
    /// As in [`Theme::write_hyprcursor`].
    pub fn write_classic(&self, stamp: &str) -> Result<(), ThemeError> {
        let new = self.scratch("new");
        let result = self.write_base(&new, stamp).and_then(|()| swap(&new, &self.dir()));
        let _ = fs::remove_dir_all(&new);
        result
    }

    /// Renders the XCursor theme of `shapes` in `colors` at the sizes for `size`, and swaps it in
    /// while the theme's stamp is still `theme_stamp`. `threads` cursors render at once.
    ///
    /// The caller holds the theme's render lock (`kubaos-cli`'s `cursor.rs`), so this is the only
    /// writer of the scratch directory it renders into.
    ///
    /// # Errors
    ///
    /// [`ThemeError`] for a cursor that couldn't be rendered or a file that couldn't be written.
    pub fn write_xcursor(
        &self,
        shapes: &[Shape],
        colors: &Colors,
        size: u32,
        theme_stamp: &str,
        threads: usize,
    ) -> Result<XcursorOutcome, ThemeError> {
        let new = self.scratch_fixed("cursors");
        match fs::remove_dir_all(&new) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(ThemeError::Write(new, e)),
        }
        let result = (|| {
            create_dir(&new)?;
            let sizes = xcursor_sizes(size);
            let still_wanted = || self.stamp().as_deref() == Some(theme_stamp);
            if !render_all(shapes, colors, &sizes, &new, threads, &still_wanted)? || !still_wanted() {
                return Ok(XcursorOutcome::Superseded);
            }
            link_aliases(shapes, &new)?;
            write(&new.join(STAMP), xcursor_stamp(theme_stamp, size).as_bytes())?;
            swap(&new, &self.dir().join(CURSORS_DIR))?;
            Ok(XcursorOutcome::Written)
        })();
        let _ = fs::remove_dir_all(&new);
        result
    }

    /// `index.theme` and the stamp in a new theme directory.
    fn write_base(&self, dir: &Path, stamp: &str) -> Result<(), ThemeError> {
        create_dir(dir)?;
        let index = format!(
            "[Icon Theme]\nName={}\nComment=Bibata Modern in the colours of the wallpaper\nInherits={FALLBACK_THEME}\n",
            self.name
        );
        write(&dir.join("index.theme"), index.as_bytes())?;
        write(&dir.join(STAMP), stamp.as_bytes())
    }

    /// Moves the live XCursor part into the new theme, so XCursor clients keep the old colours,
    /// not the fallback's, until the new ones are rendered.
    fn keep_xcursor(&self, new: &Path) {
        let _ = fs::rename(self.dir().join(CURSORS_DIR), new.join(CURSORS_DIR));
    }

    /// A scratch directory beside the theme, unique to this process: dot names, so no theme
    /// lookup takes it for a theme by its name.
    fn scratch(&self, what: &str) -> PathBuf {
        self.icons.join(format!(".{}.{what}-{}", self.name, process::id()))
    }

    fn scratch_fixed(&self, what: &str) -> PathBuf {
        self.icons.join(format!(".{}.{what}", self.name))
    }
}

/// Renders every shape into `dir`, `threads` at a time, and returns `false` as soon as the theme is
/// no longer wanted.
fn render_all(
    shapes: &[Shape],
    colors: &Colors,
    sizes: &[u32],
    dir: &Path,
    threads: usize,
    still_wanted: &(dyn Fn() -> bool + Sync),
) -> Result<bool, ThemeError> {
    let next = AtomicUsize::new(0);
    let failure: Mutex<Option<ThemeError>> = Mutex::new(None);
    let superseded = AtomicBool::new(false);
    thread::scope(|scope| {
        for _ in 0..threads.clamp(1, shapes.len().max(1)) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(shape) = shapes.get(i) else { break };
                    if failure.lock().expect("render failure mutex poisoned").is_some() {
                        break;
                    }
                    if !still_wanted() {
                        superseded.store(true, Ordering::Relaxed);
                        break;
                    }
                    if let Err(e) = render_shape(shape, colors, sizes, dir) {
                        failure.lock().expect("render failure mutex poisoned").get_or_insert(e);
                        break;
                    }
                }
            });
        }
    });
    if let Some(e) = failure.into_inner().expect("render failure mutex poisoned") {
        return Err(e);
    }
    Ok(!superseded.load(Ordering::Relaxed))
}

/// One cursor file: every size, each with all its frames in order.
fn render_shape(shape: &Shape, colors: &Colors, sizes: &[u32], dir: &Path) -> Result<(), ThemeError> {
    let drawings = shape
        .frames
        .iter()
        .map(|svg| Drawing::parse(&recolour(svg, colors)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ThemeError::Render(shape.name.clone(), e))?;
    let delay_ms = if drawings.len() > 1 { shape.delay_ms } else { 0 };
    let mut images = Vec::with_capacity(sizes.len() * drawings.len());
    for &px in sizes {
        let hot = |at: u32| ((f64::from(at) * f64::from(px) / f64::from(CANVAS)).round() as u32).min(px - 1);
        for drawing in &drawings {
            images.push(xcursor::Image {
                nominal: px,
                width: px,
                height: px,
                xhot: hot(shape.hotspot.0),
                yhot: hot(shape.hotspot.1),
                delay_ms,
                pixels: drawing.render(px),
            });
        }
    }
    write(&dir.join(&shape.name), &xcursor::encode(&images))
}

/// The other names of each cursor, as links to its file. A name that's a cursor of its own, or an
/// alias of an earlier cursor, keeps that one.
fn link_aliases(shapes: &[Shape], dir: &Path) -> Result<(), ThemeError> {
    for shape in shapes {
        for alias in &shape.aliases {
            let link = dir.join(alias);
            if link.symlink_metadata().is_ok() {
                continue;
            }
            std::os::unix::fs::symlink(&shape.name, &link).map_err(|e| ThemeError::Write(link, e))?;
        }
    }
    Ok(())
}

fn create_dir(dir: &Path) -> Result<(), ThemeError> {
    fs::create_dir_all(dir).map_err(|e| ThemeError::Write(dir.to_owned(), e))
}

fn write(path: &Path, data: &[u8]) -> Result<(), ThemeError> {
    fs::write(path, data).map_err(|e| ThemeError::Write(path.to_owned(), e))
}

/// Puts the directory `new` at `dest` in one step, and removes what was there.
fn swap(new: &Path, dest: &Path) -> Result<(), ThemeError> {
    let swap_error = |e| ThemeError::Swap(dest.to_owned(), e);
    if dest.symlink_metadata().is_err() {
        return fs::rename(new, dest).map_err(swap_error);
    }
    exchange(new, dest).map_err(swap_error)?;
    let _ = fs::remove_dir_all(new);
    Ok(())
}

/// Trades `a` and `b` atomically, with `renameat2(RENAME_EXCHANGE)`: there is no moment without a
/// theme at `b`.
#[cfg(target_os = "linux")]
fn exchange(a: &Path, b: &Path) -> io::Result<()> {
    use rustix::fs::CWD;
    use rustix::fs::RenameFlags;

    match rustix::fs::renameat_with(CWD, a, CWD, b, RenameFlags::EXCHANGE) {
        Ok(()) => Ok(()),
        // A filesystem without RENAME_EXCHANGE: two renames, with a moment of no theme between
        // them, which the `default` theme covers.
        Err(rustix::io::Errno::INVAL | rustix::io::Errno::NOSYS) => exchange_by_renames(a, b),
        Err(e) => Err(e.into()),
    }
}

#[cfg(not(target_os = "linux"))]
fn exchange(a: &Path, b: &Path) -> io::Result<()> {
    exchange_by_renames(a, b)
}

fn exchange_by_renames(a: &Path, b: &Path) -> io::Result<()> {
    let mut parked = b.as_os_str().to_owned();
    parked.push(format!(".old-{}", process::id()));
    let parked = PathBuf::from(parked);
    fs::rename(b, &parked)?;
    fs::rename(a, b)?;
    fs::rename(&parked, a)
}

#[cfg(test)]
mod tests;
