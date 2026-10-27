//! Kōbō's own pictures and words: the wallpaper it opens on, the logo, and the Japanese accents
//! above its titles.
//!
//! # Not here
//!
//! - Colours, lengths, type, motion and symbols: `kubaos-ui`, from `design/tokens.toml`.

pub const WALLPAPER: &[u8] = include_bytes!("../assets/wallpaper.jpg");
pub const WALLPAPER_BLUR: &[u8] = include_bytes!("../assets/wallpaper-blur.jpg");
/// クバ, neon, 120x120 (branding/svg/kubaos-mark.svg).
pub const LOGO_MARK: &[u8] = include_bytes!("../assets/logo/kubaos-mark.svg");
/// "KubaOS" (branding/svg/kubaos-wordmark.svg).
pub const LOGO_WORDMARK: &[u8] = include_bytes!("../assets/logo/kubaos-wordmark.svg");
/// The neon sign cut out of its wall, 1024x396, for the logo at large sizes
/// (branding/png/kubaos-sign-1024.png).
pub const LOGO_SIGN: &[u8] = include_bytes!("../assets/logo/kubaos-sign-1024.png");
/// [`LOGO_SIGN`]'s width over its height. Views size the sign with it, so the layout holds still
/// while the PNG decodes.
pub const LOGO_SIGN_ASPECT: f32 = 1024.0 / 396.0;

/// Small Japanese accents above screen titles, like the shell's clock. Every character here must
/// be in `kubaos-ui`'s M PLUS 1 subset (`crates/kubaos-ui/assets/fonts`).
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
