//! Design tokens, mirroring config/quickshell/theme/Theme.qml: Material 3
//! roles from the default wallpaper (config/matugen/default-colors.json),
//! the shell's fonts, radii and motion curves.

use iced::font::{Family, Weight};
use iced::{Color, Font, color};

/// Material 3 roles. `Copy`, so views can carry a faded copy around.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    pub tertiary: Color,
    pub tertiary_container: Color,
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,
    pub surface: Color,
    pub on_surface: Color,
    pub on_surface_variant: Color,
    pub surface_container_lowest: Color,
    pub surface_container_low: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,
    pub surface_bright: Color,
    pub outline: Color,
    pub outline_variant: Color,
    pub scrim: Color,
}

/// config/matugen/default-colors.json: matugen `scheme-fidelity` on
/// cyberpunk/shinjuku-neon-canyon.jpg (SuplenC/kubaos-wallpapers).
pub const DARK: Palette = Palette {
    primary: color!(0xfdb882),
    on_primary: color!(0x4e2600),
    primary_container: color!(0xa36a3b),
    secondary: color!(0xe4bfa5),
    secondary_container: color!(0x5e4430),
    on_secondary_container: color!(0xffdec7),
    tertiary: color!(0x8dd1e2),
    tertiary_container: color!(0x398090),
    error: color!(0xffb4ab),
    on_error: color!(0x690005),
    error_container: color!(0x93000a),
    on_error_container: color!(0xffdad6),
    surface: color!(0x18120f),
    on_surface: color!(0xece0da),
    on_surface_variant: color!(0xd6c3b6),
    surface_container_lowest: color!(0x120d0a),
    surface_container_low: color!(0x201a17),
    surface_container: color!(0x241e1b),
    surface_container_high: color!(0x2f2925),
    surface_container_highest: color!(0x3a332f),
    surface_bright: color!(0x3f3833),
    outline: color!(0x9f8d82),
    outline_variant: color!(0x52443b),
    scrim: color!(0x000000),
};

impl Palette {
    /// Every role with its alpha multiplied by `a` (screen fade-in).
    pub fn faded(mut self, a: f32) -> Self {
        let a = a.clamp(0.0, 1.0);
        for c in [
            &mut self.primary,
            &mut self.on_primary,
            &mut self.primary_container,
            &mut self.secondary,
            &mut self.secondary_container,
            &mut self.on_secondary_container,
            &mut self.tertiary,
            &mut self.tertiary_container,
            &mut self.error,
            &mut self.on_error,
            &mut self.error_container,
            &mut self.on_error_container,
            &mut self.surface,
            &mut self.on_surface,
            &mut self.on_surface_variant,
            &mut self.surface_container_lowest,
            &mut self.surface_container_low,
            &mut self.surface_container,
            &mut self.surface_container_high,
            &mut self.surface_container_highest,
            &mut self.surface_bright,
            &mut self.outline,
            &mut self.outline_variant,
            &mut self.scrim,
        ] {
            c.a *= a;
        }
        self
    }
}

/// The palette iced's built-in widgets fall back to.
pub fn iced_palette() -> iced::theme::Palette {
    iced::theme::Palette {
        background: DARK.surface,
        text: DARK.on_surface,
        primary: DARK.primary,
        success: DARK.tertiary,
        warning: DARK.primary,
        danger: DARK.error,
    }
}

/// Same color, alpha multiplied.
pub fn alpha(c: Color, a: f32) -> Color {
    Color { a: c.a * a, ..c }
}

/// Linear blend from `a` to `b`.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

// Opacity of the hover/press overlay (M3 state layers), as in Theme.qml.
pub const STATE_HOVER: f32 = 0.08;
pub const STATE_PRESS: f32 = 0.14;

pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 20.0;
    pub const XXL: f32 = 28.0;
    pub const XXXL: f32 = 40.0;
}

pub mod radius {
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    /// Outer corners of panels (Theme.qml frame.panelRounding).
    pub const PANEL: f32 = 24.0;
    pub const FULL: f32 = 1000.0;
}

pub mod size {
    pub const XS: f32 = 11.0;
    pub const SM: f32 = 12.0;
    pub const MD: f32 = 14.0;
    pub const LG: f32 = 16.0;
    pub const TITLE: f32 = 30.0;
    pub const DISPLAY: f32 = 64.0;
}

const fn geist(weight: Weight) -> Font {
    Font {
        family: Family::Name("Geist"),
        weight,
        ..Font::DEFAULT
    }
}

pub const SANS: Font = geist(Weight::Normal);
pub const SANS_LIGHT: Font = geist(Weight::Light);
pub const SANS_MEDIUM: Font = geist(Weight::Medium);
pub const SANS_SEMIBOLD: Font = geist(Weight::Semibold);
pub const MONO: Font = Font::with_name("JetBrains Mono");
pub const JP: Font = Font {
    family: Family::Name("M PLUS 1"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};
/// The bundled Material Symbols Rounded subset, renamed (assets/
/// rename-icon-font.py): under its own name, Nix's full Material Symbols on
/// the installed system drew the wrong glyphs for some codepoints.
pub const ICONS: Font = Font::with_name("KubaOS Symbols");

/// The bundled fonts (assets/make-assets.sh), loaded at startup.
pub const FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../assets/fonts/Geist-Light.ttf"),
    include_bytes!("../assets/fonts/Geist-Medium.ttf"),
    include_bytes!("../assets/fonts/Geist-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Geist-Bold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/MPLUS1-subset.ttf"),
    include_bytes!("../assets/fonts/MaterialSymbolsRounded-subset.ttf"),
];

pub const WALLPAPER: &[u8] = include_bytes!("../assets/wallpaper.jpg");
pub const WALLPAPER_BLUR: &[u8] = include_bytes!("../assets/wallpaper-blur.jpg");
/// クバ, neon, 120x120 (branding/svg/kubaos-mark.svg).
pub const LOGO_MARK: &[u8] = include_bytes!("../assets/logo/kubaos-mark.svg");
/// "KubaOS" (branding/svg/kubaos-wordmark.svg).
pub const LOGO_WORDMARK: &[u8] = include_bytes!("../assets/logo/kubaos-wordmark.svg");

/// Material Symbols Rounded codepoints. The bundled font is a subset; keep
/// this list and the `icons` list in assets/make-assets.sh in sync. A few
/// are spares for screens still to come (first-boot setup).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    ArrowBack,
    ArrowForward,
    Badge,
    Bolt,
    Check,
    CheckCircle,
    ChevronRight,
    Circle,
    Close,
    Code,
    Computer,
    DeployedCode,
    Download,
    Error,
    ExpandLess,
    ExpandMore,
    HardDrive,
    Key,
    Keyboard,
    Language,
    Lock,
    LockOpen,
    Mail,
    Person,
    Power,
    Public,
    RadioChecked,
    RadioUnchecked,
    Restart,
    Rocket,
    Save,
    Schedule,
    Search,
    Storage,
    Terminal,
    Timer,
    Visibility,
    VisibilityOff,
    Wallpaper,
    Warning,
    Wifi,
    WifiLock,
    WifiOff,
    Wifi1,
    Wifi2,
    Wifi3,
    Wifi4,
}

impl Icon {
    /// Every icon, for the icon sheet (gallery.rs).
    #[cfg(test)]
    pub const ALL: [Icon; 47] = [
        Icon::ArrowBack,
        Icon::ArrowForward,
        Icon::Badge,
        Icon::Bolt,
        Icon::Check,
        Icon::CheckCircle,
        Icon::ChevronRight,
        Icon::Circle,
        Icon::Close,
        Icon::Code,
        Icon::Computer,
        Icon::DeployedCode,
        Icon::Download,
        Icon::Error,
        Icon::ExpandLess,
        Icon::ExpandMore,
        Icon::HardDrive,
        Icon::Key,
        Icon::Keyboard,
        Icon::Language,
        Icon::Lock,
        Icon::LockOpen,
        Icon::Mail,
        Icon::Person,
        Icon::Power,
        Icon::Public,
        Icon::RadioChecked,
        Icon::RadioUnchecked,
        Icon::Restart,
        Icon::Rocket,
        Icon::Save,
        Icon::Schedule,
        Icon::Search,
        Icon::Storage,
        Icon::Terminal,
        Icon::Timer,
        Icon::Visibility,
        Icon::VisibilityOff,
        Icon::Wallpaper,
        Icon::Warning,
        Icon::Wifi,
        Icon::WifiLock,
        Icon::WifiOff,
        Icon::Wifi1,
        Icon::Wifi2,
        Icon::Wifi3,
        Icon::Wifi4,
    ];

    pub fn glyph(self) -> char {
        let cp = match self {
            Icon::ArrowBack => 0xe5c4,
            Icon::ArrowForward => 0xe5c8,
            Icon::Badge => 0xea67,
            Icon::Bolt => 0xea0b,
            Icon::Check => 0xe668,
            Icon::CheckCircle => 0xf0be,
            Icon::ChevronRight => 0xe5cc,
            Icon::Circle => 0xef4a,
            Icon::Close => 0xe5cd,
            Icon::Code => 0xe86f,
            Icon::Computer => 0xe31e,
            Icon::DeployedCode => 0xf720,
            Icon::Download => 0xf090,
            Icon::Error => 0xf8b6,
            Icon::ExpandLess => 0xe5ce,
            Icon::ExpandMore => 0xe5cf,
            Icon::HardDrive => 0xf80e,
            Icon::Key => 0xe73c,
            Icon::Keyboard => 0xe312,
            Icon::Language => 0xea07,
            Icon::Lock => 0xe899,
            Icon::LockOpen => 0xe898,
            Icon::Mail => 0xe159,
            Icon::Person => 0xf0d3,
            Icon::Power => 0xf8c7,
            Icon::Public => 0xe80b,
            Icon::RadioChecked => 0xe837,
            Icon::RadioUnchecked => 0xe836,
            Icon::Restart => 0xf053,
            Icon::Rocket => 0xeb9b,
            Icon::Save => 0xe161,
            Icon::Schedule => 0xefd6,
            Icon::Search => 0xef7a,
            Icon::Storage => 0xe1db,
            Icon::Terminal => 0xeb8e,
            Icon::Timer => 0xe425,
            Icon::Visibility => 0xe8f4,
            Icon::VisibilityOff => 0xe8f5,
            Icon::Wallpaper => 0xe1bc,
            Icon::Warning => 0xf083,
            Icon::Wifi => 0xe63e,
            Icon::WifiLock => 0xe1e1,
            Icon::WifiOff => 0xe648,
            Icon::Wifi1 => 0xebe4,
            Icon::Wifi2 => 0xebd6,
            Icon::Wifi3 => 0xebe1,
            Icon::Wifi4 => 0xf065,
        };
        char::from_u32(cp).expect("valid codepoint")
    }
}

/// Small Japanese accents above screen titles, like the shell's clock.
/// Every character here must be in the M PLUS 1 subset (make-assets.sh `jp`).
pub mod jp {
    pub const WELCOME: &str = "ようこそ";
    pub const LANGUAGE: &str = "言語と入力";
    pub const TIMEZONE: &str = "時間帯";
    pub const NETWORK: &str = "接続";
    pub const ACCOUNT: &str = "アカウント";
    pub const DISK: &str = "ディスク";
    pub const GIT: &str = "署名";
    pub const WALLPAPERS: &str = "壁紙";
    pub const SUMMARY: &str = "確認";
    pub const INSTALLING: &str = "インストール中";
    pub const DONE: &str = "完了";
    pub const FAILED: &str = "失敗";
    pub const SETUP: &str = "初期設定";
}

/// Material 3 motion: cubic béziers from Theme.qml `anim`.
pub mod motion {
    use std::time::Duration;

    /// A switch's knob sliding across.
    pub const SWITCH: Duration = Duration::from_millis(180);
    pub const NORMAL: Duration = Duration::from_millis(300);
    pub const LARGE: Duration = Duration::from_millis(450);
    pub const SLOW: Duration = Duration::from_millis(600);

    /// `emphasizedDecel`: things arriving on screen.
    pub const EMPHASIZED_DECEL: (f32, f32, f32, f32) = (0.05, 0.7, 0.1, 1.0);
    /// `standard`: small state changes.
    pub const STANDARD: (f32, f32, f32, f32) = (0.2, 0.0, 0.0, 1.0);

    /// CSS-style cubic bézier easing at `x` in 0..=1.
    pub fn bezier((x1, y1, x2, y2): (f32, f32, f32, f32), x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        if x <= 0.0 || x >= 1.0 {
            return x;
        }
        let sample = |a: f32, b: f32, t: f32| {
            let u = 1.0 - t;
            3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
        };
        // Bisection on x(t); 24 rounds is well under a pixel.
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        let mut t = x;
        for _ in 0..24 {
            let xt = sample(x1, x2, t);
            if (xt - x).abs() < 1e-5 {
                break;
            }
            if xt < x {
                lo = t;
            } else {
                hi = t;
            }
            t = (lo + hi) / 2.0;
        }
        sample(y1, y2, t)
    }
}

#[cfg(test)]
mod tests {
    use super::motion::*;

    #[test]
    fn bezier_endpoints_and_monotonic() {
        assert_eq!(bezier(EMPHASIZED_DECEL, 0.0), 0.0);
        assert_eq!(bezier(EMPHASIZED_DECEL, 1.0), 1.0);
        let mut last = 0.0;
        for i in 0..=100 {
            let y = bezier(STANDARD, i as f32 / 100.0);
            assert!(y + 1e-4 >= last);
            last = y;
        }
    }
}
