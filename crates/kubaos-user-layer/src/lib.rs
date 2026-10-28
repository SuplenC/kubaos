//! kubaos-user-layer: where each app's settings live in the user layer, `~/.config/kubaos`. The
//! layer is grouped by area, `desktop/clock/clock.toml`, `terminal/ghostty/config`, so a person
//! looking for a setting opens one of five folders, not one of twenty.
//!
//! # Owns
//!
//! - The apps with a folder in the user layer, each one's group and name ([`App`], [`Group`]), and
//!   the layer's root under the config home ([`root`], [`dir`], [`config_home`]).
//!
//! # Not here
//!
//! - The same table in the other languages: `home/user-layer.nix` (Nix, and through the layer file
//!   it writes, `scripts/dot-config`), `config/hypr/user_layer.lua`,
//!   `config/nvim/lua/config/user.lua` and `config/quickshell/settings/UserLayer.qml`.
//! - The seeds, `user-layer/` in the repo, which the tests hold this table to.
//! - Reading or writing an app's settings: the app's own crate.

#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

/// The user layer's folder under the config home.
pub const DIR: &str = "kubaos";

/// An area of the user layer, one folder of apps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Group {
    Desktop,
    Terminal,
    Editor,
    Tools,
    Menu,
}

impl Group {
    pub const ALL: [Group; 5] = [
        Group::Desktop,
        Group::Terminal,
        Group::Editor,
        Group::Tools,
        Group::Menu,
    ];

    /// The group's folder: `desktop`.
    pub const fn name(self) -> &'static str {
        match self {
            Group::Desktop => "desktop",
            Group::Terminal => "terminal",
            Group::Editor => "editor",
            Group::Tools => "tools",
            Group::Menu => "menu",
        }
    }
}

/// An app with a folder in the user layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum App {
    Hypr,
    Quickshell,
    Cursor,
    Clock,
    Keyboard,
    Sysmon,
    /// When the screen locks, turns off and suspends, and what the lid does.
    Idle,
    /// The power stops' settings: the laptop panel per stop, the USB skip list, the charge limit
    /// and the low-battery drop.
    Power,
    /// The file manager's switches: Tidy Downloads, the automount.
    Files,
    /// The wallpaper sets the picker shows.
    Wallpapers,
    /// The phone link's switch.
    Phone,
    /// The Proxy's profile: the work network's tunnel, DNS, HTTP proxy and company CA.
    Network,
    /// The workspace groups: which screens are pinned to which group.
    Workspaces,
    Ghostty,
    Foot,
    Tmux,
    Zsh,
    Fastfetch,
    Nvim,
    Git,
    Lazygit,
    Yazi,
    Bat,
    Bacon,
    Fuzzel,
    Kotodama,
}

impl App {
    pub const ALL: [App; 26] = [
        App::Hypr,
        App::Quickshell,
        App::Cursor,
        App::Clock,
        App::Keyboard,
        App::Sysmon,
        App::Idle,
        App::Power,
        App::Files,
        App::Wallpapers,
        App::Phone,
        App::Network,
        App::Workspaces,
        App::Ghostty,
        App::Foot,
        App::Tmux,
        App::Zsh,
        App::Fastfetch,
        App::Nvim,
        App::Git,
        App::Lazygit,
        App::Yazi,
        App::Bat,
        App::Bacon,
        App::Fuzzel,
        App::Kotodama,
    ];

    /// The app's folder in its group: `clock`.
    pub const fn name(self) -> &'static str {
        match self {
            App::Hypr => "hypr",
            App::Quickshell => "quickshell",
            App::Cursor => "cursor",
            App::Clock => "clock",
            App::Keyboard => "keyboard",
            App::Sysmon => "sysmon",
            App::Idle => "idle",
            App::Power => "power",
            App::Files => "files",
            App::Wallpapers => "wallpapers",
            App::Phone => "phone",
            App::Network => "network",
            App::Workspaces => "workspaces",
            App::Ghostty => "ghostty",
            App::Foot => "foot",
            App::Tmux => "tmux",
            App::Zsh => "zsh",
            App::Fastfetch => "fastfetch",
            App::Nvim => "nvim",
            App::Git => "git",
            App::Lazygit => "lazygit",
            App::Yazi => "yazi",
            App::Bat => "bat",
            App::Bacon => "bacon",
            App::Fuzzel => "fuzzel",
            App::Kotodama => "kotodama",
        }
    }

    pub const fn group(self) -> Group {
        match self {
            App::Hypr
            | App::Quickshell
            | App::Cursor
            | App::Clock
            | App::Keyboard
            | App::Sysmon
            | App::Idle
            | App::Power
            | App::Files
            | App::Wallpapers
            | App::Phone
            | App::Network
            | App::Workspaces => Group::Desktop,
            App::Ghostty | App::Foot | App::Tmux | App::Zsh | App::Fastfetch => Group::Terminal,
            App::Nvim => Group::Editor,
            App::Git | App::Lazygit | App::Yazi | App::Bat | App::Bacon | App::Fuzzel => Group::Tools,
            App::Kotodama => Group::Menu,
        }
    }

    /// The app's folder relative to the user layer: `desktop/clock`.
    pub fn relative(self) -> PathBuf {
        Path::new(self.group().name()).join(self.name())
    }

    /// The app whose folder is named `name`, if any.
    pub fn from_name(name: &str) -> Option<App> {
        App::ALL.into_iter().find(|app| app.name() == name)
    }
}

/// The user layer for the config home `config_home`, usually `~/.config`.
pub fn root(config_home: &Path) -> PathBuf {
    config_home.join(DIR)
}

/// `app`'s folder in the user layer for the config home `config_home`.
pub fn dir(config_home: &Path, app: App) -> PathBuf {
    root(config_home).join(app.relative())
}

/// The config home: `$XDG_CONFIG_HOME`, else `~/.config`; none without either.
pub fn config_home() -> Option<PathBuf> {
    config_home_from(env::var_os("XDG_CONFIG_HOME"), env::var_os("HOME"))
}

/// [`config_home`] from the two variables' values. An empty `XDG_CONFIG_HOME` counts as unset.
fn config_home_from(xdg_config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    xdg_config_home
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".config")))
}

#[cfg(test)]
mod tests;
