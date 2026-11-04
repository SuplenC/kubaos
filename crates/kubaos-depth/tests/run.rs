//! A whole run with the real models, on a synthetic skyline. The models are too large for the
//! repository, so the run is skipped unless `KUBAOS_DEPTH_MODELS` names their folder (and
//! `ORT_DYLIB_PATH` ONNX Runtime, outside the devshell): `docs/TESTING.md`.

use std::path::PathBuf;

use image::Rgb;
use image::RgbImage;
use kubaos_depth::sidecar::CUTOUT_FILE;
use kubaos_depth::sidecar::Kept;
use kubaos_depth::sidecar::Sidecar;

fn models() -> Option<PathBuf> {
    std::env::var_os("KUBAOS_DEPTH_MODELS")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
}

/// A 3:2 dusk: a sky graded from violet to pink over a city of dark blocks with lit windows.
fn skyline() -> RgbImage {
    let (width, height) = (1536, 1024);
    RgbImage::from_fn(width, height, |x, y| {
        let roof = 560 + ((x / 96) * 37 % 160);
        if y < roof {
            let t = y as f32 / roof as f32;
            Rgb([
                (120.0 + 110.0 * t) as u8,
                (80.0 + 40.0 * t) as u8,
                (200.0 - 40.0 * t) as u8,
            ])
        } else if (x % 24) < 8 && (y % 30) < 10 {
            Rgb([250, 210, 140])
        } else {
            Rgb([28, 24, 40])
        }
    })
}

#[test]
fn a_run_writes_a_sidecar_and_a_cut_out_only_when_it_keeps_one() {
    let Some(models) = models() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let picture = dir.path().join("dusk.png");
    skyline().save(&picture).expect("write the picture");
    let out = dir.path().join("depth");

    let sidecar = kubaos_depth::run(&picture, &out, &models, 4).expect("the run");

    assert_eq!(Sidecar::read(&out).expect("read the sidecar"), sidecar);
    let cutout = out.join(CUTOUT_FILE);
    assert_eq!(cutout.exists(), sidecar.method != Kept::None);
    if sidecar.method != Kept::None {
        let cut = image::open(&cutout).expect("open the cut-out").to_rgba8();
        assert_eq!(cut.dimensions(), (1536, 1024));
        assert!((0.0..=1.0).contains(&sidecar.clock.x) && (0.0..=1.0).contains(&sidecar.clock.y));
    }
}

#[test]
fn a_run_without_its_models_says_which_is_missing() {
    let dir = tempfile::tempdir().unwrap();

    let error = kubaos_depth::run(&dir.path().join("none.png"), dir.path(), dir.path(), 1).unwrap_err();

    assert!(matches!(error, kubaos_depth::run::Error::Models(_)), "{error:?}");
}
