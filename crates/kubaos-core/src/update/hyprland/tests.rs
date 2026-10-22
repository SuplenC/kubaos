use super::Pin;
use super::is_held_stack;
use super::parse;

#[test]
fn every_complete_record_of_the_pin_file_is_a_pin() {
    let text = "Package: hyprland\nPin: version 0.56.1-1~resolute1\nPin-Priority: 1001\n\n\
                Package: libaquamarine9\nPin: version 0.10.0-1\nPin-Priority: 1001\n\n\
                Package: *\nPin: release o=Ubuntu\nPin-Priority: 500\n\n\
                Package: hyprlang\nPin-Priority: 1001\n";

    let pins = parse(text);

    assert_eq!(
        pins,
        [
            Pin {
                package: "hyprland".into(),
                version: "0.56.1-1~resolute1".into()
            },
            Pin {
                package: "libaquamarine9".into(),
                version: "0.10.0-1".into()
            },
        ]
    );
    assert_eq!(pins[0].install_arg(), "hyprland=0.56.1-1~resolute1");
}

#[test]
fn the_held_stack_is_what_system_lib_sh_holds() {
    for held in [
        "hyprland",
        "hyprlock",
        "libhyprutils7",
        "libaquamarine9",
        "xdg-desktop-portal-hyprland",
    ] {
        assert!(is_held_stack(held), "{held}");
    }
    assert!(!is_held_stack("xdg-desktop-portal-gtk"));
}
