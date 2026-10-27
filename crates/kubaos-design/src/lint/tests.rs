use super::Rule;
use super::scan_text;
use super::source::Language;

/// The longest duration token, `scene`.
const SCENE_MS: u32 = 1200;

fn qml(text: &str) -> Vec<(usize, Rule, String)> {
    found("config/quickshell/modules/x/X.qml", text, Language::Qml)
}

fn rust(text: &str) -> Vec<(usize, Rule, String)> {
    found("crates/kubaos-kobo/src/view.rs", text, Language::Rust)
}

fn found(path: &str, text: &str, language: Language) -> Vec<(usize, Rule, String)> {
    let (violations, bad) = scan_text(path, text, language, SCENE_MS);
    assert!(bad.is_empty(), "{bad:?}");
    violations.into_iter().map(|v| (v.line, v.rule, v.what)).collect()
}

#[test]
fn qml_literal_sizes_spacing_radii_and_durations_are_found() {
    let text = "Rectangle {\n    implicitHeight: 40\n    spacing: Theme.spacing.md + 6\n    radius: 8\n    \
                NumberAnimation { duration: 200 }\n    anchors.leftMargin: 12\n}\n";

    let rules: Vec<Rule> = qml(text).into_iter().map(|(_, rule, _)| rule).collect();

    assert_eq!(rules, [Rule::Size, Rule::Space, Rule::Radius, Rule::Duration, Rule::Space]);
}

#[test]
fn qml_tokens_zero_and_halving_are_not_literals() {
    let text = "Item {\n    width: Theme.size.control\n    x: 0\n    radius: height / 2\n    \
                y: (parent.height - height) / 2\n    opacity: 0.5\n    z: 3\n    interval: 1000\n}\n";

    assert!(qml(text).is_empty(), "{:?}", qml(text));
}

#[test]
fn qml_colours_in_hex_channels_and_names_are_found_and_transparent_is_not() {
    let text = "Rectangle {\n    color: \"#1e1e2e\"\n    border.color: Qt.rgba(1, 1, 1, 0.1)\n    \
                property color glow: \"white\"\n    readonly property color none: \"transparent\"\n}\n";

    let found = qml(text);

    assert_eq!(found.iter().map(|(line, rule, _)| (*line, *rule)).collect::<Vec<_>>(), [
        (2, Rule::Colour),
        (3, Rule::Colour),
        (4, Rule::Colour)
    ]);
}

#[test]
fn qml_fonts_named_in_place_are_found() {
    let text = "Text {\n    font.family: \"JetBrains Mono\"\n    font.pixelSize: 13\n    font.weight: Font.Medium\n}\n";

    let rules: Vec<Rule> = qml(text).into_iter().map(|(_, rule, _)| rule).collect();

    assert_eq!(rules, [Rule::Font, Rule::Font, Rule::Font]);
}

#[test]
fn qml_custom_number_properties_named_for_a_size_are_found() {
    let text = "Item {\n    property int iconSize: 22\n    readonly property real cardRadius: 12\n    \
                property int count: 3\n}\n";

    let rules: Vec<Rule> = qml(text).into_iter().map(|(_, rule, _)| rule).collect();

    assert_eq!(rules, [Rule::Size, Rule::Radius]);
}

#[test]
fn qml_controls_are_stray_components_only_where_qt_quick_controls_is_imported() {
    let with_controls = "import QtQuick\nimport QtQuick.Controls\n\nItem {\n    Button {\n        text: \"Go\"\n    }\n}\n";
    let without = "import QtQuick\nimport qs.components\n\nItem {\n    Button {\n    }\n}\n";

    assert_eq!(qml(with_controls).len(), 1);
    assert_eq!(qml(with_controls)[0].1, Rule::Component);
    assert!(qml(without).is_empty());
}

#[test]
fn the_component_library_may_build_on_qt_controls() {
    let text = "import QtQuick.Controls\n\nButton {\n}\n";

    let (violations, _) = scan_text("config/quickshell/KubaOS/Components/Button.qml", text, Language::Qml, SCENE_MS);

    assert!(violations.is_empty());
}

#[test]
fn literals_in_comments_and_unrelated_strings_are_ignored() {
    let text = "Item {\n    // width: 40 was too wide\n    /* radius: 8 */\n    property string label: \"width: 40\"\n}\n";

    assert!(qml(text).is_empty(), "{:?}", qml(text));
}

#[test]
fn rust_literal_layout_values_are_found() {
    let text = "fn view() {\n    container(x).padding([10, 22]).width(Length::Fixed(18.0));\n    \
                let b = Border { width: 1.0, radius: 8.0.into(), ..Border::default() };\n    \
                text(\"Hi\").size(14);\n}\n";

    let found: Vec<(usize, Rule)> = rust(text).into_iter().map(|(line, rule, _)| (line, rule)).collect();

    assert_eq!(found, [
        (2, Rule::Space),
        (2, Rule::Space),
        (2, Rule::Size),
        (3, Rule::Size),
        (3, Rule::Radius),
        (4, Rule::Font)
    ]);
}

#[test]
fn rust_tokens_and_zero_are_not_literals() {
    let text = "fn view() {\n    row(look, Space::S8).padding(Inset::all(Space::S16)).width(Length::Fill);\n    \
                let x = self.scale.snap((room - size) / 2.0);\n    widget::container(x).padding(0);\n}\n";

    assert!(rust(text).is_empty(), "{:?}", rust(text));
}

#[test]
fn rust_colours_durations_and_fonts_are_found() {
    let text = "fn look() {\n    let a = Color::from_rgb8(0x1e, 0x1e, 0x2e);\n    \
                let t = Duration::from_millis(180);\n    let f = Font::with_name(\"JetBrains Mono\");\n    \
                let c = Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 };\n}\n";

    let rules: Vec<Rule> = rust(text).into_iter().map(|(_, rule, _)| rule).collect();

    assert_eq!(rules, [Rule::Colour, Rule::Duration, Rule::Font, Rule::Colour]);
}

#[test]
fn rust_icedss_own_controls_are_stray_components_and_their_style_types_are_not() {
    let text = "use iced::widget::button;\nuse iced::widget::{container, row, text_input};\n\
                fn v() {\n    widget::button(label).style(|_, s: widget::button::Status| x);\n    \
                let r = column![a, b];\n    widget::container(x);\n}\n";

    let found: Vec<(usize, String)> = rust(text)
        .into_iter()
        .map(|(line, rule, what)| {
            assert_eq!(rule, Rule::Component);
            (line, what)
        })
        .collect();

    assert_eq!(found, [
        (1, "iced::widget::button".to_owned()),
        (2, "iced::widget::row".to_owned()),
        (2, "iced::widget::text_input".to_owned()),
        (4, "iced::widget::button".to_owned()),
        (5, "column!".to_owned())
    ]);
}

#[test]
fn rust_inline_tests_are_not_scanned() {
    let text = "fn a() {}\n\n#[cfg(test)]\nmod tests {\n    fn b() { x.width(40.0); }\n}\n";

    assert!(rust(text).is_empty());
}

#[test]
fn a_marker_above_allows_its_rule_on_the_next_line_only() {
    let text = "Item {\n    // design-allow(size): the tray's icon size is the StatusNotifierItem spec's.\n    \
                implicitSize: 22\n    implicitWidth: 22\n}\n";

    let found = qml(text);

    assert_eq!(found.iter().map(|(line, _, _)| *line).collect::<Vec<_>>(), [4]);
}

#[test]
fn a_marker_without_a_reason_or_with_nothing_to_allow_is_bad() {
    let no_reason = "Item {\n    // design-allow(size)\n    implicitSize: 22\n}\n";
    let nothing = "Item {\n    // design-allow(colour): the old palette.\n    implicitSize: 22\n}\n";
    let unknown = "Item {\n    // design-allow(pixels): why.\n    implicitSize: 22\n}\n";

    for text in [no_reason, nothing, unknown] {
        let (_, bad) = scan_text("X.qml", text, Language::Qml, SCENE_MS);
        assert_eq!(bad.len(), 1, "{text}: {bad:?}");
    }
}

#[test]
fn padding_on_both_sides_of_a_token_is_not_a_literal_and_a_ratio_is() {
    let text = "Item {\n    implicitWidth: label.implicitWidth + Theme.spacing.md * 2\n    y: target.y + level * 0.6\n}\n";

    let lines: Vec<usize> = qml(text).into_iter().map(|(line, _, _)| line).collect();

    assert_eq!(lines, [3]);
}

#[test]
fn a_string_compared_in_a_colour_binding_is_not_a_colour() {
    let text = "Text {\n    color: day.startsWith(\"Sat\") ? Theme.m3tertiary : Theme.m3onSurface\n}\n";

    assert!(qml(text).is_empty(), "{:?}", qml(text));
}

#[test]
fn rust_timeouts_and_conversions_are_not_durations_or_colours() {
    let text = "fn f() {\n    const GRACE: Duration = Duration::from_secs(30);\n    \
                let d = Duration::from_micros((ms * 1000.0).round() as u64);\n    \
                let c = Color::from_rgb8(c.r, c.g, c.b);\n    const BLINK: Duration = Duration::from_millis(530);\n}\n";

    let found: Vec<(usize, Rule)> = rust(text).into_iter().map(|(line, rule, _)| (line, rule)).collect();

    assert_eq!(found, [(5, Rule::Duration)]);
}
