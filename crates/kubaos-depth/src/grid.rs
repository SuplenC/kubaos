//! A grid of numbers over a picture: a depth map, a mask's chances, a cut-out's alpha. Values sit
//! at pixel centres, row by row from the top left.
//!
//! # Owns
//!
//! - Resampling a grid to another size ([`Grid::resized`]), the summed-area table that sums a box
//!   in constant time ([`Sums`]), and Otsu's split of a grid's values ([`otsu`]).
//!
//! # Not here
//!
//! - What a grid means: the candidates (`crate::choose`) and the models (`crate::models`).

/// Values over a `width` by `height` picture, row by row from the top left.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    width: usize,
    height: usize,
    values: Vec<f32>,
}

impl Grid {
    /// A grid of `values`, row by row.
    ///
    /// # Panics
    ///
    /// When `values` doesn't hold `width * height` numbers: the caller built them for another size.
    #[must_use]
    pub fn new(width: usize, height: usize, values: Vec<f32>) -> Self {
        assert_eq!(values.len(), width * height, "a {width}×{height} grid's values");
        Self { width, height, values }
    }

    /// A grid whose value at each column and row is `value(x, y)`.
    #[must_use]
    pub fn from_fn(width: usize, height: usize, mut value: impl FnMut(usize, usize) -> f32) -> Self {
        let mut values = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                values.push(value(x, y));
            }
        }
        Self { width, height, values }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.values[y * self.width + x]
    }

    /// Each value through `f`.
    #[must_use]
    pub fn map(&self, f: impl Fn(f32) -> f32) -> Self {
        Self {
            width: self.width,
            height: self.height,
            values: self.values.iter().map(|&v| f(v)).collect(),
        }
    }

    /// The share of values above one half: how much of the picture a mask holds.
    pub fn share_above_half(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().filter(|&&v| v > 0.5).count() as f32 / self.values.len() as f32
    }

    /// The grid resampled to `width` by `height`, bilinearly, the corners' centres on each other's.
    /// Bilinear keeps a mask's chances between its own values, so an edge never rings.
    #[must_use]
    pub fn resized(&self, width: usize, height: usize) -> Self {
        if width == self.width && height == self.height {
            return self.clone();
        }
        let sx = self.width as f32 / width as f32;
        let sy = self.height as f32 / height as f32;
        let columns: Vec<(usize, usize, f32)> = (0..width).map(|x| taps(x, sx, self.width)).collect();
        let mut values = Vec::with_capacity(width * height);
        for y in 0..height {
            let (y0, y1, fy) = taps(y, sy, self.height);
            let top = &self.values[y0 * self.width..(y0 + 1) * self.width];
            let bottom = &self.values[y1 * self.width..(y1 + 1) * self.width];
            for &(x0, x1, fx) in &columns {
                let upper = top[x0] + (top[x1] - top[x0]) * fx;
                let lower = bottom[x0] + (bottom[x1] - bottom[x0]) * fx;
                values.push(upper + (lower - upper) * fy);
            }
        }
        Self { width, height, values }
    }
}

/// The two source samples output sample `i` falls between, at a `scale` of source per output
/// sample, and how far it is from the first.
fn taps(i: usize, scale: f32, len: usize) -> (usize, usize, f32) {
    let at = ((i as f32 + 0.5) * scale - 0.5).clamp(0.0, (len - 1) as f32);
    let first = at.floor() as usize;
    (first, (first + 1).min(len - 1), at - first as f32)
}

/// A summed-area table: the sum of a grid's values over any box, in four lookups.
#[derive(Debug)]
pub struct Sums {
    width: usize,
    height: usize,
    /// `(width + 1) × (height + 1)`, the sum of everything above and left of each corner, in f64 so
    /// a 4K picture's sums keep their last pixel.
    table: Vec<f64>,
}

impl Sums {
    #[must_use]
    pub fn new(grid: &Grid) -> Self {
        let stride = grid.width + 1;
        let mut table = vec![0.0; stride * (grid.height + 1)];
        for y in 0..grid.height {
            let mut row = 0.0;
            for x in 0..grid.width {
                row += f64::from(grid.get(x, y));
                table[(y + 1) * stride + x + 1] = table[y * stride + x + 1] + row;
            }
        }
        Self {
            width: grid.width,
            height: grid.height,
            table,
        }
    }

    /// The mean value over the box from `(x0, y0)` to `(x1, y1)` in the grid's pixels, clipped to
    /// the grid, the part outside counting as `outside`. Fractional edges round to whole pixels.
    pub fn mean(&self, x0: f32, y0: f32, x1: f32, y1: f32, outside: f32) -> f32 {
        let clip = |v: f32, len: usize| (v.round().max(0.0) as usize).min(len);
        let (cx0, cx1) = (clip(x0, self.width), clip(x1, self.width));
        let (cy0, cy1) = (clip(y0, self.height), clip(y1, self.height));
        let whole = ((x1.round() - x0.round()) * (y1.round() - y0.round())).max(1.0);
        let inside = (cx1.saturating_sub(cx0) * cy1.saturating_sub(cy0)) as f32;
        let sum = if inside > 0.0 {
            let stride = self.width + 1;
            let at = |x: usize, y: usize| self.table[y * stride + x];
            (at(cx1, cy1) - at(cx0, cy1) - at(cx1, cy0) + at(cx0, cy0)) as f32
        } else {
            0.0
        };
        (sum + (whole - inside).max(0.0) * outside) / whole
    }
}

impl Sums {
    /// The mean value over the part of the box from `(x0, y0)` to `(x1, y1)` inside the grid.
    pub fn mean_inside(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
        let clip = |v: f32, len: usize| v.clamp(0.0, len as f32);
        let (x0, x1) = (clip(x0, self.width), clip(x1, self.width));
        let (y0, y1) = (clip(y0, self.height), clip(y1, self.height));
        self.mean(x0, y0, x1, y1, 0.0)
    }
}

/// Otsu's split of values in `[0, 1]`: the threshold that best separates them into two classes,
/// and how well it does, the between-class variance as a share of the whole (1 for two spikes, 0.75
/// for values spread evenly).
pub fn otsu(values: &[f32]) -> (f32, f32) {
    const BINS: usize = 256;
    let mut histogram = [0u64; BINS];
    for &v in values {
        histogram[((v.clamp(0.0, 1.0) * (BINS - 1) as f32).round()) as usize] += 1;
    }
    let total = values.len() as f64;
    if values.is_empty() {
        return (0.5, 0.0);
    }
    let centre = |bin: usize| bin as f64 / (BINS - 1) as f64;
    let mean = histogram
        .iter()
        .enumerate()
        .map(|(bin, &n)| centre(bin) * n as f64)
        .sum::<f64>()
        / total;
    let variance = histogram
        .iter()
        .enumerate()
        .map(|(bin, &n)| (centre(bin) - mean).powi(2) * n as f64)
        .sum::<f64>()
        / total;
    let (mut below, mut below_sum) = (0u64, 0.0);
    let (mut best, mut best_between) = (0, 0.0);
    for (bin, &n) in histogram.iter().enumerate().take(BINS - 1) {
        below += n;
        below_sum += centre(bin) * n as f64;
        if below == 0 || below == values.len() as u64 {
            continue;
        }
        let w0 = below as f64 / total;
        let m0 = below_sum / below as f64;
        let m1 = (mean * total - below_sum) / (total - below as f64);
        let between = w0 * (1.0 - w0) * (m0 - m1).powi(2);
        if between > best_between {
            best_between = between;
            best = bin;
        }
    }
    let threshold = f64::midpoint(centre(best), centre(best + 1));
    let separation = if variance > 0.0 { best_between / variance } else { 0.0 };
    (threshold as f32, separation as f32)
}

#[cfg(test)]
mod tests;

/// The mean of `values` (`width` by `height`, row by row) over a square `2 * radius + 1` across
/// round each pixel, clipped to the picture: running sums along each row, then a window of rows
/// slid down the picture, so the radius costs nothing and every pass reads memory in order.
pub fn box_mean(values: &[f32], width: usize, height: usize, radius: usize) -> Vec<f32> {
    let mut across = vec![0.0f32; values.len()];
    let mut prefix = vec![0.0f64; width + 1];
    for (row, out) in values.chunks_exact(width).zip(across.chunks_exact_mut(width)) {
        for (x, &v) in row.iter().enumerate() {
            prefix[x + 1] = prefix[x] + f64::from(v);
        }
        for (x, out) in out.iter_mut().enumerate() {
            let (start, end) = (x.saturating_sub(radius), (x + radius + 1).min(width));
            *out = ((prefix[end] - prefix[start]) / (end - start) as f64) as f32;
        }
    }
    let mut out = vec![0.0f32; values.len()];
    let mut window = vec![0.0f64; width];
    let row = |y: usize| &across[y * width..(y + 1) * width];
    for y in 0..radius.min(height) {
        for (sum, &v) in window.iter_mut().zip(row(y)) {
            *sum += f64::from(v);
        }
    }
    for y in 0..height {
        if y + radius < height {
            for (sum, &v) in window.iter_mut().zip(row(y + radius)) {
                *sum += f64::from(v);
            }
        }
        if y > radius {
            for (sum, &v) in window.iter_mut().zip(row(y - radius - 1)) {
                *sum -= f64::from(v);
            }
        }
        let count = ((y + radius + 1).min(height) - y.saturating_sub(radius)) as f64;
        for (out, &sum) in out[y * width..(y + 1) * width].iter_mut().zip(&window) {
            *out = (sum / count) as f32;
        }
    }
    out
}
