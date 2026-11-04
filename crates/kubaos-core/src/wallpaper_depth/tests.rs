use super::*;

const SEED: &str = include_str!("../../../../user-layer/desktop/wallpapers/depth.toml");

fn file_with(text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("desktop/wallpapers/depth.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    (dir, path)
}

#[test]
fn without_a_file_depth_is_on_with_the_default_push_and_no_skips() {
    let dir = tempfile::tempdir().unwrap();

    let settings = Settings::load(&dir.path().join("depth.toml")).unwrap();

    assert_eq!(settings, Settings::default());
    assert!(settings.enabled);
    assert!((settings.push - DEFAULT_PUSH).abs() < f64::EPSILON);
    assert!(settings.skip.is_empty());
}

#[test]
fn the_seed_in_the_user_layer_is_the_defaults() {
    assert_eq!(Settings::parse(SEED).unwrap(), Settings::default());
}

#[test]
fn the_file_sets_every_key() {
    let settings = Settings::parse("enabled = false\npush = 0.04\nskip = [\"/p/a.jpg\", \"/p/b.png\"]\n").unwrap();

    assert!(!settings.enabled);
    assert!((settings.push - 0.04).abs() < f64::EPSILON);
    assert_eq!(settings.skip, [PathBuf::from("/p/a.jpg"), PathBuf::from("/p/b.png")]);
}

#[test]
fn an_unknown_key_a_wrong_type_or_a_push_out_of_bounds_is_refused() {
    for text in ["depth = true\n", "enabled = \"no\"\n", "skip = \"/p/a.jpg\"\n"] {
        assert!(matches!(Settings::parse(text), Err(SettingsError::Parse(_))), "{text}");
    }
    for text in ["push = -0.01\n", "push = 0.051\n", "push = nan\n"] {
        assert!(matches!(Settings::parse(text), Err(SettingsError::Push(_))), "{text}");
    }
}

#[test]
fn the_bounds_of_push_are_taken() {
    assert!(Settings::parse("push = 0\n").is_ok());
    assert!(Settings::parse("push = 0.05\n").is_ok());
}

#[test]
fn a_picture_of_the_users_own_is_cut_out_unless_depth_is_off_or_it_is_skipped() {
    let sets = Path::new("/usr/share/kubaos/wallpapers");
    let picture = Path::new("/home/k/Pictures/wallpapers/tower.jpg");
    let mut settings = Settings::default();

    assert_eq!(settings.refusal(picture, sets), None);

    settings.enabled = false;
    assert_eq!(settings.refusal(picture, sets), Some(Refusal::Off));

    settings.enabled = true;
    settings.skip.push(picture.to_path_buf());
    assert_eq!(settings.refusal(picture, sets), Some(Refusal::Skipped));
}

#[test]
fn a_video_or_a_shipped_wallpaper_is_never_cut_out() {
    let sets = Path::new("/usr/share/kubaos/wallpapers");
    let settings = Settings::default();

    assert_eq!(
        settings.refusal(Path::new("/home/k/Pictures/wallpapers/rain.MP4"), sets),
        Some(Refusal::Video)
    );
    assert_eq!(
        settings.refusal(Path::new("/usr/share/kubaos/wallpapers/shinjuku/neon.jpg"), sets),
        Some(Refusal::Shipped)
    );
    assert_eq!(
        settings.refusal(Path::new("/usr/share/kubaos/wallpapers-of-mine/neon.jpg"), sets),
        None
    );
}

#[test]
fn turning_depth_off_and_on_keeps_the_comments_and_writes_the_line_once() {
    let (_dir, path) = file_with(SEED);

    set_enabled(&path, false).unwrap();
    assert!(!Settings::load(&path).unwrap().enabled);
    set_enabled(&path, true).unwrap();

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.starts_with(SEED), "{text}");
    assert_eq!(&text[SEED.len()..], "enabled = true\n");
}

#[test]
fn a_set_line_is_rewritten_in_place() {
    let (_dir, path) = file_with("# mine\nenabled = false # for now\npush = 0.01\n");

    set_push(&path, 0.03).unwrap();
    set_enabled(&path, true).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# mine\nenabled = true # for now\npush = 0.03\n"
    );
}

#[test]
fn a_push_out_of_bounds_is_refused_and_leaves_the_file() {
    let (_dir, path) = file_with("push = 0.01\n");

    assert!(matches!(set_push(&path, 0.2), Err(SettingsError::Push(_))));
    assert!(matches!(set_push(&path, f64::NAN), Err(SettingsError::Push(_))));
    assert_eq!(fs::read_to_string(&path).unwrap(), "push = 0.01\n");
}

#[test]
fn a_missing_file_is_created_by_a_change() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("desktop/wallpapers/depth.toml");

    set_push(&path, 0.0).unwrap();

    assert!(Settings::load(&path).unwrap().push.abs() < f64::EPSILON);
}

#[test]
fn skipping_lists_the_picture_once_a_line_and_unskipping_takes_it_out() {
    let (_dir, path) = file_with(SEED);
    let tower = Path::new("/home/k/Pictures/wallpapers/tower.jpg");
    let harbour = Path::new("/home/k/Pictures/wallpapers/a \"quoted\" harbour.jpg");

    assert!(skip(&path, tower).unwrap());
    assert!(!skip(&path, tower).unwrap());
    assert!(skip(&path, harbour).unwrap());

    let settings = Settings::load(&path).unwrap();
    assert_eq!(settings.skip, [tower, harbour]);
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.ends_with(
            "skip = [\n  \"/home/k/Pictures/wallpapers/tower.jpg\",\n  \
             '/home/k/Pictures/wallpapers/a \"quoted\" harbour.jpg',\n]\n"
        ),
        "{text}"
    );

    assert!(unskip(&path, tower).unwrap());
    assert!(!unskip(&path, tower).unwrap());
    assert_eq!(Settings::load(&path).unwrap().skip, [harbour]);
    assert!(unskip(&path, harbour).unwrap());
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.starts_with(SEED), "{text}");
    assert_eq!(&text[SEED.len()..], "skip = []\n");
}

#[test]
fn a_hand_written_skip_list_over_several_lines_is_rewritten_whole() {
    let (_dir, path) = file_with("skip = [\n  \"/p/a.jpg\", # the sea\n  \"/p/b.jpg\",\n]\nenabled = true\n");

    unskip(&path, Path::new("/p/a.jpg")).unwrap();

    let settings = Settings::load(&path).unwrap();
    assert_eq!(settings.skip, [PathBuf::from("/p/b.jpg")]);
    assert!(fs::read_to_string(&path).unwrap().ends_with("]\nenabled = true\n"));
}

#[test]
fn a_file_that_isnt_toml_is_refused_and_left_as_it_is() {
    let (_dir, path) = file_with("enabled = \n");

    assert!(matches!(set_enabled(&path, false), Err(SettingsError::Edit(_))));
    assert_eq!(fs::read_to_string(&path).unwrap(), "enabled = \n");
}

#[test]
fn the_current_wallpaper_is_the_first_line_of_the_themes_record() {
    let dir = tempfile::tempdir().unwrap();
    let file = wallpaper_file(dir.path());

    assert_eq!(current_wallpaper(&file).unwrap(), None);

    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "/home/k/Pictures/wallpapers/tower.jpg\n").unwrap();
    assert_eq!(
        current_wallpaper(&file).unwrap(),
        Some(PathBuf::from("/home/k/Pictures/wallpapers/tower.jpg"))
    );

    fs::write(&file, "").unwrap();
    assert_eq!(current_wallpaper(&file).unwrap(), None);
}
