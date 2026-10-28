//! Display scales, and lengths in device pixels at them.
//!
//! Lengths on the grid are multiples of 4 logical pixels, so they land on whole device pixels at
//! every supported scale without help. Hairlines and thin strokes aren't on the grid; this is
//! where they become whole device pixels.
//!
//! # Owns
//!
//! - [`Scale`], and turning logical lengths into device pixels and back.
//!
//! # Not here
//!
//! - The lengths themselves: [`crate::tokens`].

use crate::tokens;

/// Device pixels per logical pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale(f32);

impl Scale {
    pub const X1: Scale = Scale(1.0);
    pub const X1_25: Scale = Scale(1.25);
    pub const X1_5: Scale = Scale(1.5);
    pub const X2: Scale = Scale(2.0);

    /// The scales every length is checked at.
    pub const SUPPORTED: [Scale; 4] = [Scale::X1, Scale::X1_25, Scale::X1_5, Scale::X2];

    /// A scale of `factor` device pixels per logical pixel; anything under 1/8 counts as 1/8, since
    /// no display goes below it.
    #[must_use]
    pub fn new(factor: f32) -> Scale {
        Scale(factor.max(0.125))
    }

    pub fn factor(self) -> f32 {
        self.0
    }

    /// `logical` pixels in device pixels.
    pub fn to_device(self, logical: f32) -> f32 {
        logical * self.0
    }

    /// `logical` moved to the nearest device pixel, in logical pixels.
    pub fn snap(self, logical: f32) -> f32 {
        (logical * self.0).round() / self.0
    }

    /// Half of `spare` logical pixels on a whole device pixel: how far in a box is centred. An odd
    /// number of spare device pixels leaves the extra one after the box, never before it.
    ///
    /// HAZARD: rounding half of an odd count to nearest is a tie, which float error breaks either
    /// way, and differently in f32 and in QML's doubles; this rounds ties down, a hair under, so
    /// both toolkits centre to the same pixel.
    pub fn centre(self, spare: f32) -> f32 {
        (spare * self.0 / 2.0 - 1e-3).round() / self.0
    }

    /// `logical` moved up to the next whole device pixel, in logical pixels.
    pub fn snap_up(self, logical: f32) -> f32 {
        // A hair under a whole pixel is that pixel: float error must not add a row.
        (logical * self.0 - 1e-3).ceil() / self.0
    }

    /// A hairline's width in logical pixels: one device pixel at this scale.
    pub fn hairline(self) -> f32 {
        f32::from(tokens::HAIRLINE_DEVICE_PX) / self.0
    }

    /// A stroke of `logical` pixels drawn as whole device pixels, rounded half up and never under
    /// one: 2 is 2, 3, 3 and 4 device pixels at 1, 1.25, 1.5 and 2.
    pub fn stroke(self, logical: f32) -> f32 {
        (logical * self.0).round().max(1.0) / self.0
    }

    /// The thin stroke of the focus ring, the spinner and the partial checkbox's dash, in logical
    /// pixels.
    pub fn thin(self) -> f32 {
        self.stroke(f32::from(tokens::STROKE_THIN))
    }

    /// Whether `logical` pixels are a whole number of device pixels.
    pub fn is_whole(self, logical: f32) -> bool {
        let device = logical * self.0;
        (device - device.round()).abs() < 1e-3
    }
}

/// The scale a window opens at before it knows its display's.
pub const DEFAULT_SCALE: Scale = Scale::X1;

impl Default for Scale {
    fn default() -> Self {
        DEFAULT_SCALE
    }
}

#[cfg(test)]
mod tests;
