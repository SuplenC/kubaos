//! Where text draws its ink: the top and bottom of what its glyphs fill, measured from the glyphs
//! the font draws.
//!
//! # Owns
//!
//! - [`middle`]: the middle of a text's ink, top to bottom, above its baseline, measured once per
//!   style and words and kept.
//!
//! # Not here
//!
//! - Placing text by it: an icon tile's initial ([`crate::component::icon_tile`]).

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use iced::Font;
use iced::advanced::graphics::text::cosmic_text;
use iced::advanced::graphics::text::font_system;
use iced::advanced::graphics::text::to_attributes;
use iced::advanced::graphics::text::to_shaping;
use iced::advanced::text::Shaping;

/// The size the ink is drawn at to be measured, in pixels: a row of it is under a pixel of the
/// largest style it places, as the mockup's `inkMiddle` (`design/mockups/src/lib/text.ts`) draws
/// it.
const WEIGHED_AT: f32 = 64.0;

/// The alpha from which a pixel counts as ink: half covered, as the mockup reads its canvas.
const INKED: u8 = 128;

/// Every middle measured, by font and words.
static MIDDLES: LazyLock<Middles> = LazyLock::new(Middles::default);

#[derive(Default)]
struct Middles(Mutex<HashMap<(Font, String), f32>>);

impl Middles {
    fn of(&self, font: Font, words: &str) -> f32 {
        let key = (font, words.to_owned());
        if let Some(known) = self.0.lock().expect("ink middles mutex poisoned").get(&key) {
            return *known;
        }
        let measured = measure(font, words);
        self.0.lock().expect("ink middles mutex poisoned").insert(key, measured);
        measured
    }
}

/// The middle of `words`' ink in `font`, top to bottom, in ems above the baseline, below it
/// counted: where a lone glyph on a square looks centred. A capital's is half its cap height,
/// whatever its strokes weigh, so B, L and K sit alike; an ideograph's takes in what it draws below
/// the baseline. Words that draw nothing give 0.
pub(crate) fn middle(font: Font, words: &str) -> f32 {
    MIDDLES.of(font, words)
}

/// The rows `words` ink at [`WEIGHED_AT`], read from the glyphs' rasters, as ems above the
/// baseline.
fn measure(font: Font, words: &str) -> f32 {
    let mut system = font_system().write().expect("font system lock poisoned");
    let raw = system.raw();
    let mut buffer = cosmic_text::Buffer::new(raw, cosmic_text::Metrics::new(WEIGHED_AT, WEIGHED_AT * 2.0));
    buffer.set_size(raw, None, None);
    buffer.set_text(raw, words, &to_attributes(font), to_shaping(Shaping::Auto, words), None);
    buffer.shape_until_scroll(raw, false);
    let mut swash = cosmic_text::SwashCache::new();
    let mut inked: Option<(i32, i32)> = None;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let physical = glyph.physical((0.0, 0.0), 1.0);
            let Some(image) = swash.get_image_uncached(raw, physical.cache_key) else {
                continue;
            };
            let channels = match image.content {
                cosmic_text::SwashContent::Mask => 1,
                cosmic_text::SwashContent::Color | cosmic_text::SwashContent::SubpixelMask => 4,
            };
            let width = image.placement.width as usize * channels;
            if width == 0 {
                continue;
            }
            let rows = image.data.chunks_exact(width);
            for (index, row) in rows.enumerate() {
                if !row
                    .iter()
                    .skip(channels - 1)
                    .step_by(channels)
                    .any(|&alpha| alpha >= INKED)
                {
                    continue;
                }
                let y = physical.y - image.placement.top + i32::try_from(index).unwrap_or(i32::MAX);
                inked = Some(inked.map_or((y, y), |(top, bottom)| (top.min(y), bottom.max(y))));
            }
        }
    }
    let Some((top, bottom)) = inked else {
        return 0.0;
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "a glyph's rows at 64 px are far under f32's 2^24"
    )]
    let centre = (top + bottom + 1) as f32 / 2.0;
    -centre / WEIGHED_AT
}
