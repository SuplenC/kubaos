use std::fs;
use std::path::Path;

use kubaos_design::geometry;
use kubaos_geometry::Rule;

/// A frame as the shell will write one: a row of two icon buttons at 1.25, the second smaller and
/// off the device grid.
const QML_FRAME: &str = r#"{
  "page": "osd", "mode": "dark", "scale": 1.25,
  "root": {
    "role": "layout", "axis": "x", "gap": 8, "align": "centre",
    "bounds": { "x": 0, "y": 0, "width": 96, "height": 40 },
    "children": [
      { "role": "component", "name": "IconButton", "variant": "Text Md",
        "fixed": { "width": true, "height": true },
        "bounds": { "x": 0, "y": 0, "width": 40, "height": 40 } },
      { "role": "component", "name": "IconButton", "variant": "Text Md",
        "fixed": { "width": true, "height": true },
        "bounds": { "x": 50, "y": 2, "width": 36, "height": 36 } }
    ]
  }
}"#;

#[test]
fn a_frame_written_by_another_toolkit_is_checked_by_the_same_rules() {
    let dir = tempfile::tempdir().unwrap();
    let one = dir.path().join("osd.json");
    let many = dir.path().join("all.json");
    fs::write(&one, QML_FRAME).unwrap();
    fs::write(&many, format!("[{QML_FRAME}]")).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rules = geometry::rules(&root).expect("the tokens' rules");

    let from_one = geometry::check_files(&[one.as_path()], &rules).expect("one frame");
    let from_many = geometry::check_files(&[many.as_path()], &rules).expect("an array of frames");

    let broken: Vec<Rule> = from_one.iter().map(|f| f.rule).collect();
    assert_eq!(broken, [Rule::SiblingGap, Rule::DeviceGrid, Rule::Identical]);
    assert_eq!(from_one, from_many);
}
