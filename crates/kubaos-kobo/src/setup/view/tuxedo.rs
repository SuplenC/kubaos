//! The TUXEDO drivers card's Secure Boot notice: the MOK password and the blue screen after the
//! reboot.

use kubaos_core::setup::State;
use kubaos_ui::Look;
use kubaos_ui::icon::Icon;

use super::warning;
use crate::ui::El;

/// Secure Boot and the blue screen. Dead serious: this is the one that leaves you without drivers
/// if it goes wrong.
pub(super) fn mok_notice<'a>(st: State, look: &Look) -> El<'a> {
    let steps = "Enroll MOK → Continue → Yes → type the password → Reboot.";
    if st == State::Reboot {
        warning(
            look,
            Icon::RestartAlt,
            "Reboot, then enroll the key on the blue screen",
            vec![
                "Before KubaOS starts, a blue screen called MokManager appears. It waits about 10 seconds; press a \
                 key to stop it."
                    .into(),
                steps.into(),
                "Type the one-time password you picked here. If you miss the screen, the drivers won't load: open \
                 setup again and enroll the key again."
                    .into(),
            ],
        )
    } else {
        warning(
            look,
            Icon::Lock,
            "Secure Boot is on",
            vec![
                "The drivers are built on this machine, so they're signed with a key made here, and the firmware \
                 has to be told to trust it. Pick a one-time password below."
                    .into(),
                format!("After the next reboot a blue screen called MokManager appears before KubaOS starts. {steps}"),
                "You need this password exactly once, on that screen. It assumes a US keyboard, so letters and \
                 digits only."
                    .into(),
            ],
        )
    }
}
