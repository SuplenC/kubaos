//! Memory and swap, from `/proc/meminfo`.
//!
//! RAM splits three ways: used (what the kernel can't give back), cache (what it can drop when
//! asked) and the rest of what is available. Used is `MemTotal - MemAvailable`, the kernel's own
//! estimate, so the split adds up to what `free` shows.
//!
//! # Owns
//!
//! - Parsing `/proc/meminfo`.
//!
//! # Not here
//!
//! - Which processes hold the memory: [`crate::processes`].

use serde::Deserialize;
use serde::Serialize;

/// RAM and swap, in bytes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub total: u64,
    pub used: u64,
    /// The part of what is available that is cache.
    pub cache: u64,
    pub available: u64,
    /// Used, as a percentage of the total.
    pub percent: f32,
    pub swap_total: u64,
    pub swap_used: u64,
}

/// Parses `/proc/meminfo`; `None` when it has no `MemTotal`.
pub fn parse_meminfo(text: &str) -> Option<Memory> {
    let mut fields = Fields::default();
    for line in text.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let kib: u64 = rest
            .split_ascii_whitespace()
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let bytes = kib * 1024;
        match key {
            "MemTotal" => fields.total = Some(bytes),
            "MemFree" => fields.free = bytes,
            "MemAvailable" => fields.available = Some(bytes),
            "Buffers" => fields.buffers = bytes,
            "Cached" => fields.cached = bytes,
            "SReclaimable" => fields.reclaimable = bytes,
            "Shmem" => fields.shared = bytes,
            "SwapTotal" => fields.swap_total = bytes,
            "SwapFree" => fields.swap_free = bytes,
            _ => {}
        }
    }
    let total = fields.total?;
    // Kernels before 3.14 have no MemAvailable; free and cache is the old estimate.
    let cache = (fields.buffers + fields.cached + fields.reclaimable).saturating_sub(fields.shared);
    let available = fields.available.unwrap_or(fields.free + cache).min(total);
    let used = total - available;
    Some(Memory {
        total,
        used,
        cache: cache.min(available),
        available,
        percent: if total == 0 {
            0.0
        } else {
            used as f32 * 100.0 / total as f32
        },
        swap_total: fields.swap_total,
        swap_used: fields.swap_total.saturating_sub(fields.swap_free),
    })
}

#[derive(Default)]
struct Fields {
    total: Option<u64>,
    free: u64,
    available: Option<u64>,
    buffers: u64,
    cached: u64,
    reclaimable: u64,
    shared: u64,
    swap_total: u64,
    swap_free: u64,
}

#[cfg(test)]
mod tests;
