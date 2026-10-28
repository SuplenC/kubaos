//! kubaos-cursor: the pointer in the wallpaper's colours. Bibata's SVG sources, recoloured with
//! the palette matugen writes, become a hyprcursor theme (SVG, which Hyprland draws at any scale)
//! and an XCursor theme of the same name (bitmaps, for XWayland and GTK 3).
//!
//! # Owns
//!
//! - The cursor settings file and the palette file's format (`settings`, `colors`).
//! - Reading Bibata's sources: its build config and SVGs (`source`).
//! - Recolouring and rasterising them (`recolour`, `raster`).
//! - The hyprcursor and XCursor file formats (`hyprcursor`, `xcursor`, `zip`).
//! - Building a theme directory and swapping it in whole (`theme`).
//! - The design tokens the pointer is sized by (`tokens`, generated from `design/tokens.toml`).
//!
//! # Not here
//!
//! - The `kubaos cursor` command, `hyprctl` and the background render: `kubaos-cli`'s `cursor.rs`.
//! - Writing the palette: the matugen template `config/matugen/templates/cursor.toml`, run by
//!   `scripts/dot-theme`.
//! - Installing Bibata-Modern-Classic and pinning the sources: `home/theme.nix`.

pub mod colors;
#[cfg(test)]
mod fixture;
pub mod hyprcursor;
pub mod raster;
pub mod recolour;
pub mod settings;
pub mod source;
pub mod theme;
mod tokens;
pub mod xcursor;
mod zip;

pub use self::colors::Colors;
pub use self::colors::Palette;
pub use self::colors::Rgb;
pub use self::settings::Settings;
pub use self::settings::Style;
pub use self::source::Shape;

/// The generated theme's name, under `~/.local/share/icons/`.
pub const THEME_NAME: &str = "KubaOS-Bibata";

/// The theme the generated one inherits from, and the one the pointer falls back to: the
/// first-login pointer, the `classic` style, and the pointer after a failed render.
pub const FALLBACK_THEME: &str = "Bibata-Modern-Classic";
