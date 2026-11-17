use super::IniText;

/// A `zoomus.conf` as Zoom writes it: Qt's `[General]` first, `key=value` with no spaces, a
/// percent-escaped key, a quoted value with a comma, and a section of its own after a blank line.
const ZOOM: &str = "[General]\nGeoLocale=system\nQFmw49%2By6zXS%23=1616522911\nautoScale=true\nxwayland=true\n\n\
                    [zoom_new_im]\nmain_frame_pixel_pos_wide=\"1860,990\"\n";

#[test]
fn a_set_changes_only_the_keys_value_and_keeps_every_other_byte() {
    let mut ini = IniText::parse(ZOOM).unwrap();

    ini.set("General", "xwayland", Some("false"), true);

    assert_eq!(ini.text(), ZOOM.replace("xwayland=true", "xwayland=false"));
    assert_eq!(ini.get("General", "xwayland").as_deref(), Some("false"));
    assert_eq!(ini.get("zoom_new_im", "xwayland"), None);
}

#[test]
fn a_new_key_goes_after_its_sections_last_line_and_its_removal_leaves_the_text_as_it_was() {
    let mut ini = IniText::parse(ZOOM).unwrap();

    ini.set("General", "disableCef", Some("true"), true);
    assert_eq!(
        ini.text(),
        ZOOM.replace("xwayland=true\n", "xwayland=true\ndisableCef=true\n")
    );
    ini.set("General", "disableCef", None, true);

    assert_eq!(ini.text(), ZOOM);
}

#[test]
fn a_missing_section_is_appended_and_taken_out_with_its_blank_line_once_empty() {
    let mine = "[AS]\nshowframewindow=true";
    let mut ini = IniText::parse(mine).unwrap();

    ini.set("General", "xwayland", Some("false"), true);
    assert_eq!(ini.text(), "[AS]\nshowframewindow=true\n\n[General]\nxwayland=false\n");
    assert!(ini.has_section("General"));
    ini.set("General", "xwayland", None, false);

    assert_eq!(ini.text(), "[AS]\nshowframewindow=true\n");
    assert!(!ini.has_section("General"));
}

#[test]
fn only_the_header_a_step_appends_is_taken_out() {
    let mut ini = IniText::parse("[General]\n\n[AS]\nxwayland=false\n").unwrap();

    ini.set("General", "xwayland", None, false);
    ini.set("AS", "xwayland", None, false);

    assert_eq!(ini.text(), "[General]\n");
}

#[test]
fn an_empty_file_gets_the_section_and_the_key_and_is_empty_again_after() {
    let mut ini = IniText::empty();

    ini.set("General", "xwayland", Some("false"), false);
    assert_eq!(ini.text(), "[General]\nxwayland=false\n");
    ini.set("General", "xwayland", None, false);

    assert!(ini.is_empty());
}

#[test]
fn a_section_kept_stays_when_its_last_key_goes() {
    let mut ini = IniText::parse("[General]\nxwayland=false\n").unwrap();

    ini.set("General", "xwayland", None, true);

    assert_eq!(ini.text(), "[General]\n");
}

#[test]
fn a_key_goes_under_a_section_that_ends_the_file_without_a_line_break() {
    let mut ini = IniText::parse("[General]\nautoScale=true").unwrap();

    ini.set("General", "xwayland", Some("false"), true);

    assert_eq!(ini.text(), "[General]\nautoScale=true\nxwayland=false\n");
}

#[test]
fn a_key_there_twice_has_its_last_value_which_a_set_changes_and_the_others_go() {
    let mut ini = IniText::parse("[General]\na=1\nb=2\na=3\n[Other]\na=9\n").unwrap();

    assert_eq!(ini.get("General", "a").as_deref(), Some("3"));
    ini.set("General", "a", Some("4"), true);
    assert_eq!(ini.text(), "[General]\nb=2\na=4\n[Other]\na=9\n");
    ini.set("General", "a", None, true);

    assert_eq!(ini.text(), "[General]\nb=2\n[Other]\na=9\n");
}

#[test]
fn the_keys_before_any_header_and_a_general_in_any_case_are_general() {
    let mut top = IniText::parse("xwayland=true\n[AS]\nxwayland=true\n").unwrap();
    let mut lower = IniText::parse("[general]\nautoScale=true\n").unwrap();

    assert_eq!(top.get("General", "xwayland").as_deref(), Some("true"));
    top.set("General", "xwayland", Some("false"), true);
    lower.set("General", "xwayland", Some("false"), true);

    assert_eq!(top.text(), "xwayland=false\n[AS]\nxwayland=true\n");
    assert_eq!(lower.text(), "[general]\nautoScale=true\nxwayland=false\n");
}

#[test]
fn a_comment_before_the_first_header_isnt_generals_so_its_key_goes_in_a_section() {
    let mut ini = IniText::parse("; mine\n[AS]\nshowframewindow=true\n").unwrap();

    ini.set("General", "xwayland", Some("false"), true);

    assert_eq!(
        ini.text(),
        "; mine\n[AS]\nshowframewindow=true\n\n[General]\nxwayland=false\n"
    );
}

#[test]
fn a_semicolon_ends_a_value_outside_quotes_and_its_comment_is_kept() {
    let mut ini = IniText::parse("[General]\nxwayland=true ;mine\npos=\"1;2\"\n#key=1\n").unwrap();

    assert_eq!(ini.get("General", "xwayland").as_deref(), Some("true"));
    assert_eq!(ini.get("General", "pos").as_deref(), Some("\"1;2\""));
    assert_eq!(ini.get("General", "#key").as_deref(), Some("1"));
    ini.set("General", "xwayland", Some("false"), true);

    assert_eq!(ini.text(), "[General]\nxwayland=false ;mine\npos=\"1;2\"\n#key=1\n");
}

#[test]
fn a_byte_order_mark_is_kept_and_left_out_of_the_first_header() {
    let mut ini = IniText::parse("\u{feff}[General]\nautoScale=true\n").unwrap();

    ini.set("General", "xwayland", Some("false"), true);

    assert_eq!(ini.text(), "\u{feff}[General]\nautoScale=true\nxwayland=false\n");
}

#[test]
fn the_files_own_spacing_and_line_endings_are_used_for_a_new_line() {
    let mut ini = IniText::parse("; mine\r\n[General]\r\nname = den\r\n").unwrap();

    ini.set("General", "xwayland", Some("false"), true);

    assert_eq!(ini.text(), "; mine\r\n[General]\r\nname = den\r\nxwayland = false\r\n");
}

#[test]
fn an_empty_value_is_found_empty_and_set_after_its_equals_sign() {
    let mut ini = IniText::parse("[General]\nxwayland=\nname = \n").unwrap();

    assert_eq!(ini.get("General", "xwayland").as_deref(), Some(""));
    ini.set("General", "xwayland", Some("false"), true);
    ini.set("General", "name", Some("den"), true);

    assert_eq!(ini.text(), "[General]\nxwayland=false\nname = den\n");
}

#[test]
fn a_line_qt_cant_read_as_one_setting_is_refused() {
    for text in [
        "[General]\nxwayland\n",
        "= false\n",
        "# a comment Qt doesn't know\n",
        "[General]\nlist=a,\\\nb\n",
        "[General]\nname=\"two\nlines\"\n",
        "key;x=1\n",
    ] {
        assert!(IniText::parse(text).is_err(), "{text:?}");
    }
    assert!(IniText::parse("; comment\n\n[General]\nkey=\nescaped=a\\;b\n").is_ok());
}

#[test]
fn a_carriage_return_alone_is_refused_as_qt_ends_a_line_there() {
    for text in [
        "[General]\rxwayland=true\n",
        "[General]\nxwayland=true\r",
        "[General]\na=1\r\r\n",
    ] {
        assert!(IniText::parse(text).is_err(), "{text:?}");
    }
    assert!(IniText::parse("[General]\r\nxwayland=true\r\n").is_ok());
}

#[test]
fn only_asciis_space_tab_vertical_tab_and_form_feed_are_trimmed() {
    let ini = IniText::parse("[\u{b}General\u{c}]\n\txwayland\u{b}=\u{c}true\t\nname=\u{a0}den\u{a0}\n").unwrap();

    assert_eq!(ini.get("General", "xwayland").as_deref(), Some("true"));
    assert_eq!(ini.get("General", "name").as_deref(), Some("\u{a0}den\u{a0}"));
    assert_eq!(ini.get("General", "\u{a0}name"), None);
}

#[test]
fn a_header_is_read_up_to_its_first_bracket_after_its_comment_is_cut() {
    let mut ini = IniText::parse("[General] ;c\nautoScale=true\n[AS];x]\nshow=true\n").unwrap();

    ini.set("General", "xwayland", Some("false"), true);

    assert_eq!(
        ini.text(),
        "[General] ;c\nautoScale=true\nxwayland=false\n[AS];x]\nshow=true\n"
    );
    assert_eq!(ini.get("AS", "show").as_deref(), Some("true"));
    assert!(IniText::parse("[General ;c]\n").is_err());
}

#[test]
fn an_equals_sign_inside_quotes_doesnt_end_the_key() {
    let ini = IniText::parse("[General]\n\"a=b\"=1\n").unwrap();

    assert_eq!(ini.get("General", "\"a=b\"").as_deref(), Some("1"));
    assert_eq!(ini.get("General", "\"a"), None);
}
