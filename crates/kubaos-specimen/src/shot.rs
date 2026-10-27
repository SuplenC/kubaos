//! Screenshots without a window: the specimen laid out and drawn by iced's tiny-skia renderer, on
//! the CPU, at any scale, then written as PNG.
//!
//! tiny-skia doesn't snap quads to pixels the way the wgpu renderer does with `crisp`, so a length
//! off the device grid shows here as the soft edge it is.
//!
//! # Owns
//!
//! - Rendering a [`Specimen`] to pixels ([`render`]), writing them as PNG ([`write_png`]), and the
//!   fonts both the window and the screenshots load ([`fonts`]).
//! - Laying a [`Specimen`] out as a geometry frame ([`frame`]), which `kubaos-specimen check` and
//!   the tests hold to the design language's rules.
//!
//! # Not here
//!
//! - What is drawn: [`crate::specimen`].

use std::borrow::Cow;
use std::fs;
use std::io;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use iced::Event;
use iced::Pixels;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::advanced::clipboard;
use iced::advanced::graphics::text::font_system;
use iced::advanced::renderer;
use iced::advanced::renderer::Headless;
use iced::advanced::widget::Id;
use iced::advanced::widget::Operation;
use iced::mouse;
use iced::window;
use iced_runtime::UserInterface;
use iced_runtime::user_interface;
use kubaos_geometry::Frame;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::Scale;
use kubaos_ui::probe::Collect;
use kubaos_ui::tokens::Type;

use crate::pages::icons;
use crate::specimen;
use crate::specimen::Message;
use crate::specimen::Specimen;

/// Every font the specimen draws with: kubaos-ui's, then the four icon sets of the comparison
/// sheet.
pub fn fonts() -> impl Iterator<Item = &'static [u8]> {
    kubaos_ui::text::FONTS.iter().copied().chain(icons::FONTS)
}

/// How tall the page may be laid out before it is measured; far taller than any page.
const MEASURE_HEIGHT: f32 = 100_000.0;

/// An image, its pixels in RGBA order, row by row, and the boxes in it that missed the device grid.
#[derive(Debug)]
pub struct Shot {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Every container whose edges fall between device pixels, in logical pixels.
    pub off_grid: Vec<Rectangle>,
}

/// Why a screenshot couldn't be made.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("starting the tiny-skia renderer")]
    Renderer,
    #[error("the page reported no boxes to check")]
    Empty,
    #[error("writing {}", path.display())]
    Write {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("encoding {}", path.display())]
    Encode {
        path: std::path::PathBuf,
        #[source]
        source: png::EncodingError,
    },
}

/// Loads [`fonts`] into the font system the renderers share. Once per process.
pub fn load_fonts() {
    let mut system = font_system().write().expect("font system lock poisoned");
    for font in fonts() {
        system.load_font(Cow::Borrowed(font));
    }
}

/// `specimen` drawn `width` logical pixels wide and as tall as its page, at its scale.
///
/// # Errors
///
/// [`Error::Renderer`] when the tiny-skia renderer can't start.
pub fn render(specimen: &Specimen, width: f32) -> Result<Shot, Error> {
    let mut renderer = headless()?;
    let look = Look::new(specimen.mode, specimen.scale);
    let factor = specimen.scale.factor();
    let (mut ui, physical) = lay_out(specimen, width, &mut renderer);
    let mut audit = Audit {
        scale: specimen.scale,
        off_grid: Vec::new(),
    };
    ui.operate(&renderer, &mut audit);

    let mut messages = Vec::new();
    let _ = ui.update(
        &[Event::Window(window::Event::RedrawRequested(Instant::now()))],
        mouse::Cursor::Unavailable,
        &mut renderer,
        &mut clipboard::Null,
        &mut messages,
    );
    ui.draw(
        &mut renderer,
        &Theme::Dark,
        &renderer::Style {
            text_color: look.colors.on_surface,
        },
        mouse::Cursor::Unavailable,
    );
    let rgba = renderer.screenshot(physical, factor, look.colors.surface);
    Ok(Shot {
        rgba,
        width: physical.width,
        height: physical.height,
        off_grid: audit.off_grid,
    })
}

/// `specimen` laid out `width` logical pixels wide, as the geometry check reads it: every box and
/// the role its probe gives it.
///
/// # Errors
///
/// [`Error::Renderer`] when the tiny-skia renderer can't start, [`Error::Empty`] when the page
/// reports no boxes.
pub fn frame(specimen: &Specimen, width: f32) -> Result<Frame, Error> {
    let mut renderer = headless()?;
    let (mut ui, _) = lay_out(specimen, width, &mut renderer);
    let mut collect = Collect::new();
    ui.operate(&renderer, &mut collect);
    let root = collect.into_tree().ok_or(Error::Empty)?;
    Ok(Frame {
        page: specimen.page.slug().to_owned(),
        mode: mode_name(specimen.mode).to_owned(),
        scale: specimen.scale.factor(),
        root,
    })
}

/// A mode's name in file and frame names.
pub fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Dark => "dark",
        Mode::Light => "light",
    }
}

/// The renderer screenshots draw with: tiny-skia, on the CPU, with no window.
fn headless() -> Result<Renderer, Error> {
    iced::futures::executor::block_on(<Renderer as Headless>::new(
        kubaos_ui::text::default_font(),
        Pixels(f32::from(Type::Body.size())),
        Some("tiny-skia"),
    ))
    .ok_or(Error::Renderer)
}

/// `specimen` laid out `width` wide and as tall as its page, both on whole device pixels, and that
/// size in device pixels.
fn lay_out<'a>(
    specimen: &'a Specimen,
    width: f32,
    renderer: &mut Renderer,
) -> (UserInterface<'a, Message, Theme, Renderer>, Size<u32>) {
    let factor = specimen.scale.factor();
    let mut ui = UserInterface::build(
        specimen::view(specimen),
        Size::new(width, MEASURE_HEIGHT),
        user_interface::Cache::default(),
        renderer,
    );
    let mut root = Find::new(specimen::ROOT);
    ui.operate(renderer, &mut root);
    let height = root.bounds.map_or(MEASURE_HEIGHT, |b| b.height);

    // Whole device pixels both ways; the page is laid out at what they come to in logical pixels.
    let physical = Size::new((width * factor).round() as u32, (height * factor).ceil() as u32);
    let logical = Size::new(physical.width as f32 / factor, physical.height as f32 / factor);
    (ui.relayout(logical, renderer), physical)
}

/// Writes `shot` to `path` as an RGBA PNG.
///
/// # Errors
///
/// When the file can't be created or written.
pub fn write_png(path: &Path, shot: &Shot) -> Result<(), Error> {
    let write = |source| Error::Write {
        path: path.to_path_buf(),
        source,
    };
    let file = fs::File::create(path).map_err(write)?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), shot.width, shot.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let encode = |source| Error::Encode {
        path: path.to_path_buf(),
        source,
    };
    let mut writer = encoder.write_header().map_err(encode)?;
    writer.write_image_data(&shot.rgba).map_err(encode)?;
    writer.finish().map_err(encode)
}

/// Finds the bounds of the container with an id.
struct Find {
    target: Id,
    bounds: Option<Rectangle>,
}

impl Find {
    fn new(id: &'static str) -> Self {
        Find {
            target: Id::new(id),
            bounds: None,
        }
    }
}

impl Operation for Find {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&self.target) {
            self.bounds = Some(bounds);
        }
    }
}

/// Collects every container whose edges aren't whole device pixels at `scale`.
struct Audit {
    scale: Scale,
    off_grid: Vec<Rectangle>,
}

impl Operation for Audit {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, _id: Option<&Id>, bounds: Rectangle) {
        let whole = |v: f32| self.scale.is_whole(v);
        if !(whole(bounds.x) && whole(bounds.y) && whole(bounds.width) && whole(bounds.height)) {
            self.off_grid.push(bounds);
        }
    }
}
