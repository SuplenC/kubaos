//! A small Bibata source tree for the tests: the layout and placeholder colours of the real one,
//! with three cursors (a still arrow, a hand with aliases, and a two-frame busy cursor).

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use crate::Colors;
use crate::Rgb;

pub const CONFIG: &str = r#"
[cursors]
[cursors.fallback_settings]
x11_sizes = [16, 24, 32]
x_hotspot = 128
y_hotspot = 128
x11_delay = 40

[cursors.left_ptr]
png = 'left_ptr.png'
x_hotspot = 55
y_hotspot = 17
x11_name = 'left_ptr'
x11_symlinks = ["arrow", "default"]

[cursors.hand2]
png = 'hand2.png'
x_hotspot = 114
y_hotspot = 18
x11_name = 'hand2'
x11_symlinks = ["pointer"]

[cursors.wait]
png = 'wait-*.png'
x11_name = 'wait'
x11_symlinks = ['watch']
"#;

/// A square of the fill with an outline, as Bibata's arrow uses its placeholders.
pub const ARROW: &str = r##"<svg width="256" height="256" viewBox="0 0 256 256" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M64 64H192V192H64Z" fill="#00FF00" stroke="#0000FF" stroke-width="16"/>
</svg>
"##;

/// A disc in the busy cursors' placeholder, under Bibata's four spinner segments.
pub const WATCH: &str = r##"<svg width="256" height="256" viewBox="0 0 256 256" fill="none" xmlns="http://www.w3.org/2000/svg">
<circle cx="128" cy="128" r="96" fill="#FF0000" stroke="#0000FF" stroke-width="16"/>
<path d="M128 56V128H200C200 88 168 56 128 56Z" fill="#F05024" fill-opacity="0.8"/>
<path d="M200 128H128V200C168 200 200 168 200 128Z" fill="#FCB813" fill-opacity="0.8"/>
<path d="M128 200V128H56C56 168 88 200 128 200Z" fill="#7EBA41" fill-opacity="0.8"/>
<path d="M56 128H128V56C88 56 56 88 56 128Z" fill="#32A0DA" fill-opacity="0.8"/>
</svg>
"##;

/// Bibata's spinner and corner colours, none of which a recoloured theme keeps.
pub const RAINBOW: [&str; 8] = ["#F05024", "#FCB813", "#7EBA41", "#32A0DA", "#F1613A", "#FDBE2A", "#96C865", "#4FADDF"];

/// The Shinjuku wallpaper's `dark` colours.
pub const DARK: Colors = Colors {
    fill: Rgb([0x1e, 0x0f, 0x0e]),
    outline: Rgb([0xfa, 0xdb, 0xd9]),
    watch: Rgb([0x1e, 0x0f, 0x0e]),
    spinner: [Rgb([0xff, 0xb3, 0xae]), Rgb([0xff, 0xda, 0xd6]), Rgb([0xab, 0x35, 0x37]), Rgb([0x89, 0x1d, 0x22])],
};

/// A Bibata source tree in a temporary directory.
pub fn bibata() -> TempDir {
    let dir = tempfile::tempdir().expect("make the fixture's directory");
    write(dir.path(), "configs/normal/x.build.toml", CONFIG);
    write(dir.path(), "svg/modern/left_ptr.svg", ARROW);
    write(dir.path(), "svg/modern/hand2.svg", ARROW);
    write(dir.path(), "svg/modern/wait/wait-02.svg", WATCH);
    write(dir.path(), "svg/modern/wait/wait-01.svg", WATCH);
    dir
}

fn write(root: &Path, file: &str, text: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().expect("a file in a folder")).expect("make the fixture's folders");
    fs::write(path, text).expect("write a fixture file");
}
