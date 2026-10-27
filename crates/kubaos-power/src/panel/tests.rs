use super::*;
use crate::settings::PanelMode;

#[test]
fn sixty_hz_is_kept_where_the_switch_is_seamless() {
    let target = Target::of(Panel::of(PanelMode::Fixed60), true);

    assert_eq!(
        target,
        Target {
            refresh: Some(60),
            vrr: false,
            held_back: false
        }
    );
    assert_eq!(target.text(), "refresh=60\nvrr=0\n");
}

#[test]
fn sixty_hz_becomes_vrr_where_a_new_rate_would_blank_the_panel() {
    let target = Target::of(Panel::of(PanelMode::Fixed60), false);

    assert_eq!(
        target,
        Target {
            refresh: None,
            vrr: true,
            held_back: true
        }
    );
    assert_eq!(target.text(), "refresh=native\nvrr=1\n");
}

#[test]
fn vrr_and_native_never_change_the_rate() {
    for seamless in [true, false] {
        assert_eq!(
            Target::of(Panel::of(PanelMode::Vrr), seamless).text(),
            "refresh=native\nvrr=1\n"
        );
        assert_eq!(
            Target::of(Panel::of(PanelMode::Native), seamless).text(),
            "refresh=native\nvrr=0\n"
        );
    }
}

#[test]
fn the_words_say_the_rate_and_vrr() {
    assert_eq!(Target::of(Panel::of(PanelMode::Fixed60), true).words(), "60 Hz");
    assert_eq!(
        Target::of(Panel::of(PanelMode::Vrr), true).words(),
        "native rate with VRR"
    );
    assert_eq!(Target::of(Panel::of(PanelMode::Native), true).words(), "native rate");
}
