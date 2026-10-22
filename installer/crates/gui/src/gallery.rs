//! A gallery of states the key tour never reaches (no system image, a
//! failed probe, long names, many disks, a blocked Continue), rendered
//! headlessly with iced_test at the sizes that matter. Not a regular test:
//!
//!   GALLERY=/tmp/gallery cargo test -p dotinstall-gui gallery -- --ignored
//!
//! then look at the PNGs.

use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::{Size, Theme};
use iced_test::simulator::Simulator;
use kubaos_core::probe::{DiskInfo, PartitionInfo, Region, WifiNetwork};
use kubaos_core::{Event, Probe, StepId};

use crate::app::{App, F, Message, Screen};
use crate::backend::{Demo, demo_probe};
use crate::progress::Install;

const GB: u64 = 1_000_000_000;

fn shot(dir: &std::path::Path, name: &str, app: &App, size: Size) {
    let settings = iced::Settings {
        fonts: crate::theme::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::theme::SANS,
        ..iced::Settings::default()
    };
    let mut ui = Simulator::with_size(settings, size, crate::view::view(app));
    let theme = Theme::custom("dotfiles", crate::theme::iced_palette());
    let path = dir.join(format!("{name}-{}x{}.png", size.width, size.height));
    let _ = std::fs::remove_file(&path);
    ui.snapshot(&theme)
        .expect("snapshot")
        .matches_image(&path)
        .expect("write");
    eprintln!("gallery: {}", path.display());
}

fn app_with(p: Result<Probe, String>, size: Size) -> App {
    let mut app = App::headless(Arc::new(Demo::new(40.0, None)));
    app.set_probe(p);
    app.load_images_now(size);
    app.transition.0 = Instant::now() - Duration::from_secs(5);
    app.now = Instant::now();
    app
}

fn go(app: &mut App, s: Screen) {
    let _ = app.go(s);
    app.transition.0 = Instant::now() - Duration::from_secs(5);
    app.now = Instant::now();
}

/// The demo machine, but harder: every glibc locale, a long SSID, six
/// disks with long names, one too small, one full.
fn hard_probe() -> Probe {
    let mut p = demo_probe();
    p.locales = include_str!("../assets/locales.tsv")
        .lines()
        .filter_map(|l| l.split('\t').next())
        .map(str::to_string)
        .collect();
    p.keymaps = include_str!("../assets/keymaps.tsv")
        .lines()
        .filter_map(|l| l.split('\t').next())
        .filter(|k| !k.contains('('))
        .map(str::to_string)
        .collect();
    p.wifi.insert(
        1,
        WifiNetwork {
            ssid: "FRITZ!Box 7590 XL Gästenetz OG".into(),
            signal: 80,
            secured: true,
        },
    );
    for i in 0..6 {
        p.wifi.push(WifiNetwork {
            ssid: format!("Neighbour-{i}"),
            signal: 10 + i * 5,
            secured: i % 2 == 0,
        });
    }
    p.disks.push(DiskInfo {
        path: "/dev/nvme1n1".into(),
        model: "WD_BLACK SN850X 4000GB with Heatsink, Gaming Edition, Firmware 620361WD".into(),
        size_bytes: 4000 * GB,
        removable: false,
        is_boot_medium: false,
        partitions: vec![PartitionInfo {
            path: "/dev/nvme1n1p1".into(),
            size_bytes: 4000 * GB - 2_000_000,
            fstype: Some("btrfs".into()),
            label: Some("a-very-long-filesystem-label-that-goes-on".into()),
            os_hint: Some("Linux (some other distro, unfortunately)".into()),
        }],
        free_regions: vec![Region {
            start: 0,
            end: 1_000_000,
        }],
    });
    p.disks.push(DiskInfo {
        path: "/dev/mmcblk0".into(),
        model: "".into(),
        size_bytes: 8 * GB,
        removable: true,
        is_boot_medium: false,
        partitions: vec![],
        free_regions: vec![],
    });
    p
}

#[test]
#[ignore]
fn gallery() {
    let Some(dir) = std::env::var_os("GALLERY") else {
        eprintln!("set GALLERY=DIR");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    for size in [Size::new(1024.0, 768.0), Size::new(1280.0, 800.0)] {
        // Welcome: refused (no image), probe failed.
        let mut p = demo_probe();
        p.payload = None;
        let app = app_with(Ok(p), size);
        shot(&dir, "welcome-no-image", &app, size);
        let app = app_with(
            Err("dotinstall probe failed (exit status: 1): lsblk: command not found".into()),
            size,
        );
        shot(&dir, "welcome-probe-failed", &app, size);

        let mut app = app_with(Ok(hard_probe()), size);
        go(&mut app, Screen::Language);
        shot(&dir, "language-all", &app, size);
        go(&mut app, Screen::Wifi);
        let _ = app.update_inner(Message::Pick(
            crate::app::List::Wifi,
            "FRITZ!Box 7590 XL Gästenetz OG".into(),
        ));
        shot(&dir, "wifi-long", &app, size);

        // Empty name, focused: placeholder with the caret in front.
        app.form.full_name.clear();
        go(&mut app, Screen::Account);
        app.focus = Some(F::FullName);
        app.caret = crate::edit::Caret::at(0);
        let _ = app.update_inner(Message::Key(crate::app::Key::Enter));
        shot(&dir, "account-empty-nudged", &app, size);
        app.form.full_name = "Kuba Example".into();
        app.form.password = "hunter2".into();
        app.form.password_confirm = "hunter3".into();
        app.focus = Some(F::PasswordConfirm);
        shot(&dir, "account-mismatch", &app, size);
        app.form.password_confirm = "hunter2".into();

        go(&mut app, Screen::Disk);
        let _ = app.update_inner(Message::Pick(crate::app::List::Disk, "/dev/nvme1n1".into()));
        shot(&dir, "disk-many", &app, size);
        let _ = app.update_inner(Message::Pick(crate::app::List::Disk, "/dev/mmcblk0".into()));
        app.form.encrypt = false;
        shot(&dir, "disk-too-small", &app, size);
        app.form.encrypt = true;
        let _ = app.update_inner(Message::Pick(crate::app::List::Disk, "/dev/nvme0n1".into()));

        app.form.git_email = "not-an-email".into();
        go(&mut app, Screen::Git);
        app.focus = Some(F::GitEmail);
        shot(&dir, "git-bad-email", &app, size);
        app.form.git_email = "kuba@example.com".into();

        go(&mut app, Screen::Wallpapers);
        app.form.toggle_wallpaper_set("japanese");
        app.focus = Some(F::WallpaperSet(1));
        shot(&dir, "wallpapers", &app, size);
        app.form.toggle_wallpaper_set("japanese");

        go(&mut app, Screen::Summary);
        shot(&dir, "summary-alongside", &app, size);

        // Failed with a long, real-looking error.
        let now = Instant::now();
        let mut i = Install::new(now - Duration::from_secs(40));
        i.apply(
            &Event::Plan {
                steps: StepId::ALL.iter().map(|s| s.info()).collect(),
            },
            now,
        );
        i.apply(
            &Event::StepStarted {
                id: StepId::Bootloader,
            },
            now,
        );
        i.apply(
            &Event::Failed {
                id: StepId::Bootloader,
                error: "grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=ubuntu failed with exit status 1: Installing for x86_64-efi platform. grub-install: error: efibootmgr failed to register the boot entry: Operation not permitted.".into(),
                hint: Some("The firmware refused a new boot entry. Turn off Secure Boot's \"lock boot order\" in the setup menu and try again.".into()),
            },
            now,
        );
        app.install = Some(i);
        app.log_open = (true, now - Duration::from_secs(5));
        go(&mut app, Screen::Failed);
        shot(&dir, "failed-long", &app, size);
    }
}

/// Every icon, drawn with the bundled font while a full Material Symbols
/// Rounded (the installed system's, from Nix) is loaded too: they must all
/// be ours. `FULL_SYMBOLS=path/to/MaterialSymbolsRounded.ttf` adds it.
#[test]
#[ignore]
fn icon_sheet() {
    use crate::theme::{DARK, Icon};
    use crate::widgets::icon;
    use iced::widget::{column, container, row, text};
    let Some(dir) = std::env::var_os("GALLERY") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut fonts: Vec<std::borrow::Cow<'static, [u8]>> =
        crate::theme::FONTS.iter().map(|f| (*f).into()).collect();
    if let Some(full) = std::env::var_os("FULL_SYMBOLS") {
        fonts.insert(0, std::fs::read(full).expect("FULL_SYMBOLS").into());
    }
    let settings = iced::Settings {
        fonts,
        default_font: crate::theme::SANS,
        ..iced::Settings::default()
    };
    let mut grid = column![].spacing(10);
    for chunk in Icon::ALL.chunks(8) {
        let mut r = row![].spacing(10);
        for i in chunk {
            r = r.push(
                column![
                    icon(*i, 32.0, DARK.on_surface),
                    text(format!("{i:?}")).size(10).color(DARK.outline)
                ]
                .spacing(4)
                .width(110),
            );
        }
        grid = grid.push(r);
    }
    let el: iced::Element<'_, Message> = container(grid)
        .padding(20)
        .style(|_t| container::Style::default().background(DARK.surface))
        .into();
    let mut ui = Simulator::with_size(settings, Size::new(960.0, 420.0), el);
    let theme = Theme::custom("dotfiles", crate::theme::iced_palette());
    let path = dir.join("icon-sheet.png");
    let _ = std::fs::remove_file(&path);
    ui.snapshot(&theme).unwrap().matches_image(&path).unwrap();
    eprintln!("gallery: {}", path.display());
}
