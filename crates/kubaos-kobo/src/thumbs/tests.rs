use std::fs;

use image::Rgba;

use super::*;

/// A solid JPEG of `w` x `h` in `rgb` at `path`.
fn jpeg(path: &Path, w: u32, h: u32, rgb: [u8; 3]) {
    fs::create_dir_all(path.parent().expect("a parent dir")).expect("make the dir");
    image::RgbImage::from_pixel(w, h, image::Rgb(rgb))
        .save(path)
        .expect("write the JPEG");
}

fn set(names: &[&str]) -> WallpaperSet {
    WallpaperSet {
        id: "shinjuku".into(),
        images: u32::try_from(names.len()).unwrap(),
        preview: None,
        files: names
            .iter()
            .map(|n| WallpaperFile {
                name: (*n).into(),
                thumb: Some(format!("wallpapers/shinjuku/{n}.jpg")),
            })
            .collect(),
    }
}

fn reddish(px: Rgba<u8>) -> bool {
    px[0] > 200 && px[1] < 60
}

#[test]
fn the_payloads_thumbnail_comes_first() {
    let payload = tempfile::tempdir().unwrap();
    let originals = tempfile::tempdir().unwrap();
    jpeg(&payload.path().join("wallpapers/shinjuku/gate.jpg.jpg"), 480, 270, [230, 20, 20]);
    jpeg(&originals.path().join("shinjuku/gate.jpg"), 960, 540, [20, 20, 230]);

    let got = decode(payload.path(), originals.path(), &set(&["gate.jpg"]));

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].0, "shinjuku/gate.jpg");
    assert_eq!(got[0].1.dimensions(), (480, 270));
    assert!(reddish(*got[0].1.get_pixel(10, 10)), "the thumbnail, not the original");
}

#[test]
fn without_a_thumbnail_the_original_is_shrunk_to_the_thumbnail_width() {
    let payload = tempfile::tempdir().unwrap();
    let originals = tempfile::tempdir().unwrap();
    jpeg(&originals.path().join("shinjuku/gate.jpg"), 1920, 1080, [230, 20, 20]);

    let got = decode(payload.path(), originals.path(), &set(&["gate.jpg"]));

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].1.dimensions(), (WIDTH, 270));
}

#[test]
fn a_video_without_a_thumbnail_falls_back_to_its_poster() {
    let payload = tempfile::tempdir().unwrap();
    let originals = tempfile::tempdir().unwrap();
    jpeg(&originals.path().join("shinjuku/rain.poster.jpg"), 640, 360, [230, 20, 20]);
    fs::write(originals.path().join("shinjuku/rain.mp4"), b"not a picture").unwrap();

    let got = decode(payload.path(), originals.path(), &set(&["rain.mp4"]));

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].0, "shinjuku/rain.mp4");
    assert_eq!(got[0].1.dimensions(), (WIDTH, 270));
}

#[test]
fn a_wallpaper_with_no_picture_anywhere_is_left_out() {
    let payload = tempfile::tempdir().unwrap();
    let originals = tempfile::tempdir().unwrap();
    jpeg(&payload.path().join("wallpapers/shinjuku/b.jpg.jpg"), 480, 270, [230, 20, 20]);

    let got = decode(payload.path(), originals.path(), &set(&["a.jpg", "b.jpg"]));

    let keys: Vec<&str> = got.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, ["shinjuku/b.jpg"]);
}

#[test]
fn a_thumbnail_is_cut_about_its_middle_to_the_cards_shape() {
    let wide = RgbaImage::new(480, 200);
    let tall = RgbaImage::new(320, 480);

    assert_eq!(super::cropped(&wide).dimensions(), (320, 200));
    assert_eq!(super::cropped(&tall).dimensions(), (320, 200));
}
