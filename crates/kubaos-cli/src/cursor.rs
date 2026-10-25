//! `kubaos cursor`: the pointer in the wallpaper's colours.
//!
//!   kubaos cursor apply [--sync]                  the theme for the settings and the palette
//!   kubaos cursor style <dark | accent | classic> change the style, then apply
//!   kubaos cursor size <16-96>                    change the size, then apply
//!   kubaos cursor show                            print the style and the size
//!
//! `scripts/dot-theme` runs `apply` after matugen; the Command menu runs `style` and `size`.
//! `apply` writes the hyprcursor theme and points Hyprland at it, then leaves the XCursor render
//! to a background `kubaos cursor render-xcursor` at idle priority, so it returns in well under a
//! second. `--sync` renders in this process instead (the harness, a benchmark). Without a palette,
//! with the `classic` style, or when anything fails, the pointer is Bibata-Modern-Classic.
//! Exit codes: 0 ok, 1 the theme failed and the pointer fell back, 2 usage.
//!
//! # Owns
//!
//! - Where the files are, when each part is rewritten, the background render and its lock.
//! - Telling the session: `hyprctl setcursor`, GSettings and the environment of new apps.
//!
//! # Not here
//!
//! - The theme files and the settings file: the `kubaos-cursor` crate.

use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;
use std::thread;

use clap::Subcommand;
use kubaos_cursor::Colors;
use kubaos_cursor::FALLBACK_THEME;
use kubaos_cursor::Palette;
use kubaos_cursor::Settings;
use kubaos_cursor::Style;
use kubaos_cursor::THEME_NAME;
use kubaos_cursor::colors::PaletteError;
use kubaos_cursor::settings;
use kubaos_cursor::source;
use kubaos_cursor::source::SourceError;
use kubaos_cursor::theme;
use kubaos_cursor::theme::Theme;
use kubaos_cursor::theme::ThemeError;
use kubaos_cursor::theme::XcursorOutcome;
use kubaos_user_layer::App;

use crate::job::chain;
use crate::usage_error;

/// The background render's threads at most: enough to finish in a couple of seconds, few enough
/// to leave cores to the session while it runs.
const RENDER_THREADS: usize = 4;

#[derive(Subcommand)]
pub enum CursorCmd {
    /// Write the cursor theme for the settings and the wallpaper's palette, and use it.
    Apply {
        /// Render the XCursor theme in this process, not in the background.
        #[arg(long)]
        sync: bool,
    },
    /// Set the cursor style (dark, accent or classic) and apply it.
    Style { style: String },
    /// Set the cursor size in pixels at scale 1 (16 to 96) and apply it.
    Size { size: u32 },
    /// Print the cursor style and size.
    Show,
    /// The background half of `apply`: render the XCursor theme.
    #[command(hide = true)]
    RenderXcursor,
}

pub fn main(cmd: CursorCmd) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(msg) => return usage_error(msg),
    };
    match cmd {
        CursorCmd::Apply { sync } => apply(&paths, sync),
        CursorCmd::Style { style } => match style.parse::<Style>() {
            Ok(style) => change(&paths, settings::set_style(&paths.settings, style)),
            Err(e) => usage_error(e),
        },
        CursorCmd::Size { size } => change(&paths, settings::set_size(&paths.settings, size)),
        CursorCmd::Show => {
            let settings = load_settings(&paths);
            println!("style={} size={}", settings.style, settings.size);
            ExitCode::SUCCESS
        }
        CursorCmd::RenderXcursor => render_xcursor(&paths),
    }
}

/// Where the cursor's inputs and outputs are.
struct Paths {
    /// The user's settings, `~/.config/kubaos/desktop/cursor/cursor.toml`.
    settings: PathBuf,
    /// matugen's palette, next to the other theme outputs `scripts/dot-theme` keeps.
    palette: PathBuf,
    /// Bibata's sources, linked by `home/theme.nix`.
    source: PathBuf,
    icons: PathBuf,
    /// The background render's output.
    log: PathBuf,
}

impl Paths {
    fn from_env() -> Result<Self, String> {
        let home = env::var_os("HOME").map(PathBuf::from).ok_or("HOME isn't set")?;
        let xdg = |var: &str, default: &str| {
            env::var_os(var).filter(|v| !v.is_empty()).map_or_else(|| home.join(default), PathBuf::from)
        };
        let config = xdg("XDG_CONFIG_HOME", ".config");
        let data = xdg("XDG_DATA_HOME", ".local/share");
        let state = home.join(".local/state/theme");
        Ok(Self {
            settings: kubaos_user_layer::dir(&config, App::Cursor).join("cursor.toml"),
            palette: state.join("cursor.toml"),
            source: env::var_os("KUBAOS_CURSOR_SOURCE")
                .map_or_else(|| data.join("kubaos/cursor/bibata"), PathBuf::from),
            icons: data.join("icons"),
            log: state.join("cursor.log"),
        })
    }

    fn theme(&self) -> Theme {
        Theme::new(&self.icons, THEME_NAME)
    }
}

/// Why the recoloured theme couldn't be written.
#[derive(Debug, thiserror::Error)]
enum ApplyError {
    #[error("the palette {}", .0.display())]
    Palette(PathBuf, #[source] PaletteError),
    #[error("Bibata's sources")]
    Source(#[source] SourceError),
    #[error("the theme")]
    Theme(#[source] ThemeError),
}

/// The recoloured theme the settings and the palette ask for.
struct Wanted {
    colors: Colors,
    stamp: String,
    source: PathBuf,
}

/// What `style` asks for: `None` for `classic`.
fn wanted(paths: &Paths, style: Style) -> Result<Option<Wanted>, ApplyError> {
    if style == Style::Classic {
        return Ok(None);
    }
    let palette = Palette::load(&paths.palette).map_err(|e| ApplyError::Palette(paths.palette.clone(), e))?;
    let Some(colors) = palette.colors(style) else {
        return Ok(None);
    };
    // The store path behind the link, so a new Bibata release is a new stamp.
    let source = fs::canonicalize(&paths.source).unwrap_or_else(|_| paths.source.clone());
    let stamp = theme::stamp(style, Some(&colors), &source);
    Ok(Some(Wanted { colors, stamp, source }))
}

fn load_settings(paths: &Paths) -> Settings {
    Settings::load(&paths.settings).unwrap_or_else(|e| {
        eprintln!("kubaos: {}; the cursor is dark at 24", chain(&e));
        Settings::default()
    })
}

fn change(paths: &Paths, result: Result<(), settings::SettingsError>) -> ExitCode {
    match result {
        Ok(()) => apply(paths, false),
        Err(e @ settings::SettingsError::Size(_)) => usage_error(e),
        Err(e) => {
            eprintln!("kubaos: {}", chain(&e));
            ExitCode::FAILURE
        }
    }
}

fn apply(paths: &Paths, sync: bool) -> ExitCode {
    let settings = load_settings(paths);
    let theme = paths.theme();
    let written = wanted(paths, settings.style).and_then(|wanted| {
        let Some(wanted) = wanted else { return Ok(None) };
        if theme.stamp().as_deref() != Some(wanted.stamp.as_str()) {
            let shapes = source::load(&wanted.source).map_err(ApplyError::Source)?;
            theme.write_hyprcursor(&shapes, &wanted.colors, &wanted.stamp).map_err(ApplyError::Theme)?;
        }
        Ok(Some(wanted))
    });
    let failed = written.as_ref().is_err();
    if let Err(e) = &written {
        eprintln!("kubaos: writing the cursor theme: {}; the pointer is {FALLBACK_THEME}", chain(e));
    }
    if let Ok(Some(_)) = written {
        set_hyprland_cursor(THEME_NAME, settings.size);
    } else {
        write_classic(&theme);
        set_hyprland_cursor(FALLBACK_THEME, settings.size);
    }
    if sync {
        render_xcursor(paths);
    } else {
        spawn_render(paths);
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// The classic stub, unless the theme already is one: the pointer every XCursor client gets is
/// then Bibata-Modern-Classic, through `Inherits`.
fn write_classic(theme: &Theme) {
    let stamp = theme::stamp(Style::Classic, None, Path::new(""));
    if theme.stamp().as_deref() == Some(stamp.as_str()) {
        return;
    }
    if let Err(e) = theme.write_classic(&stamp) {
        eprintln!("kubaos: writing the classic cursor stub: {}", chain(&e));
    }
}

/// Starts `kubaos cursor render-xcursor` at idle CPU and I/O priority, detached from this
/// process, its output appended to the log.
fn spawn_render(paths: &Paths) {
    let Ok(exe) = env::current_exe() else {
        eprintln!("kubaos: can't find this kubaos to render the XCursor theme");
        return;
    };
    let mut cmd = if on_path("ionice") {
        let mut cmd = Command::new("ionice");
        cmd.args(["-c", "3", "nice", "-n", "19"]).arg(exe);
        cmd
    } else if on_path("nice") {
        let mut cmd = Command::new("nice");
        cmd.args(["-n", "19"]).arg(exe);
        cmd
    } else {
        Command::new(exe)
    };
    cmd.args(["cursor", "render-xcursor"]).stdin(Stdio::null()).stdout(Stdio::null());
    if let Some(dir) = paths.log.parent() {
        let _ = fs::create_dir_all(dir);
    }
    match OpenOptions::new().create(true).append(true).open(&paths.log) {
        Ok(log) => cmd.stderr(log),
        Err(_) => cmd.stderr(Stdio::null()),
    };
    // Its own process group: a terminal's Ctrl+C or hangup for the caller doesn't stop it.
    if let Err(e) = cmd.process_group(0).spawn() {
        eprintln!("kubaos: starting the XCursor render: {e}");
    }
}

fn on_path(program: &str) -> bool {
    env::var_os("PATH").is_some_and(|path| env::split_paths(&path).any(|dir| dir.join(program).is_file()))
}

/// The background half of `apply`: the XCursor theme for the theme that's live, then the session.
/// One render at a time, under the lock beside the theme; a render whose theme changed meanwhile
/// stops, and the newer one's render takes over.
fn render_xcursor(paths: &Paths) -> ExitCode {
    let lock = match lock(paths) {
        Ok(lock) => lock,
        Err(e) => {
            eprintln!("kubaos: taking the cursor render lock: {e}");
            return ExitCode::FAILURE;
        }
    };
    let settings = load_settings(paths);
    let theme = paths.theme();
    let mut rewritten = false;
    let mut code = ExitCode::SUCCESS;
    if let Ok(Some(wanted)) = wanted(paths, settings.style)
        && theme.stamp().as_deref() == Some(wanted.stamp.as_str())
        && theme.xcursor_stamp() != Some(theme::xcursor_stamp(&wanted.stamp, settings.size))
    {
        let threads = thread::available_parallelism().map_or(1, usize::from).min(RENDER_THREADS);
        let outcome = source::load(&wanted.source).map_err(ApplyError::Source).and_then(|shapes| {
            theme
                .write_xcursor(&shapes, &wanted.colors, settings.size, &wanted.stamp, threads)
                .map_err(ApplyError::Theme)
        });
        match outcome {
            Ok(XcursorOutcome::Written) => rewritten = true,
            Ok(XcursorOutcome::Superseded) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("kubaos: rendering the XCursor theme: {}; XCursor apps get {FALLBACK_THEME}", chain(&e));
                code = ExitCode::FAILURE;
            }
        }
    }
    drop(lock);
    tell_session(settings.size, rewritten);
    code
}

fn lock(paths: &Paths) -> std::io::Result<File> {
    fs::create_dir_all(&paths.icons)?;
    let file = File::create(paths.icons.join(format!(".{THEME_NAME}.lock")))?;
    file.lock()?;
    Ok(file)
}

/// Points Hyprland at `theme` at `size`. Hyprland re-reads a hyprcursor theme on every
/// `setcursor`, so the same name with new colours takes effect too.
fn set_hyprland_cursor(theme: &str, size: u32) {
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
        return;
    }
    quietly(Command::new("hyprctl").args(["setcursor", theme, &size.to_string()]));
}

/// The settings apps read the cursor from when they draw it themselves: GSettings for GTK, and
/// the environment of apps Hyprland and the user manager start from now on, for XWayland.
/// `bounce` switches the GSettings theme away and back, so running GTK 3 apps load the new
/// XCursor files under the same name.
fn tell_session(size: u32, bounce: bool) {
    let size = size.to_string();
    let has_bus = env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some()
        || env::var_os("XDG_RUNTIME_DIR").is_some_and(|dir| Path::new(&dir).join("bus").exists());
    if has_bus && on_path("gsettings") {
        let gsettings = |key: &str, value: &str| {
            quietly(Command::new("gsettings").args(["set", "org.gnome.desktop.interface", key, value]));
        };
        if bounce {
            gsettings("cursor-theme", FALLBACK_THEME);
        }
        gsettings("cursor-theme", THEME_NAME);
        gsettings("cursor-size", &size);
    }
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        let lua = format!(r#"hl.env("XCURSOR_SIZE", "{size}") hl.env("HYPRCURSOR_SIZE", "{size}")"#);
        quietly(Command::new("hyprctl").args(["eval", &lua]));
    }
    if has_bus && on_path("systemctl") {
        quietly(Command::new("systemctl").args([
            "--user",
            "set-environment",
            &format!("XCURSOR_THEME={THEME_NAME}"),
            &format!("XCURSOR_SIZE={size}"),
            &format!("HYPRCURSOR_SIZE={size}"),
        ]));
    }
}

/// Runs `cmd` for its effect alone: a session tool that's missing or fails changes nothing else.
fn quietly(cmd: &mut Command) {
    let _ = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}
