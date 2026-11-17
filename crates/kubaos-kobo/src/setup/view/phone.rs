//! The phone card's words: pairing, the phone it knows, and adding the laptop by address.

use kubaos_core::setup::State;
use kubaos_ui::Look;
use kubaos_ui::icon::Icon;

use super::notice;
use super::para;
use super::warning;
use crate::setup::Setup;
use crate::ui::El;

/// The phone card: what the link is, the steps on the phone, the pairing request, and what works
/// each way once paired. iPhone's limits are said plainly: it never sends its clipboard on its own.
pub(super) fn phone_explain<'a>(s: &Setup, link: &kubaos_core::phone::Status, st: State, look: &Look) -> Vec<El<'a>> {
    let mut v: Vec<El> = vec![];
    let laptop = link.name.clone().unwrap_or_else(|| "this laptop".into());
    if !link.on {
        v.push(para(
            look,
            "Share the clipboard and files between this laptop and your phone (iPhone, iPad or Android) with KDE \
             Connect. It stays off until you set it up: until then nothing on this laptop listens for phones or \
             announces itself on the network.",
        ));
        v.push(notice(
            look,
            Icon::Download,
            "First, on the phone",
            vec![
                "iPhone and iPad: KDE Connect from the App Store.".into(),
                "Android: KDE Connect from Google Play or F-Droid.".into(),
                format!("Same Wi-Fi as the laptop. It will show up there as {laptop}."),
            ],
        ));
        return v;
    }
    if !link.running {
        v.push(para(
            look,
            "The phone link is on, but KDE Connect isn't running. Start it again; if it stops once more, \
             journalctl --user -u kubaos-phone says why.",
        ));
        return v;
    }
    if let Some(asking) = link.request() {
        let mut lines = vec![];
        if let Some(key) = &asking.verification_key {
            lines.push(format!("The phone shows a key too. It must be the same: {key}."));
        }
        lines.push("Accept only a phone you're holding: other KDE Connect devices on this network can ask too.".into());
        v.push(warning(look, Icon::Key, "A phone wants to pair", lines));
    } else if st == State::Waiting {
        v.push(notice(
            look,
            Icon::Wifi,
            "On the phone",
            vec![
                format!("Open KDE Connect. This laptop shows up as {laptop}: tap it, then Request pairing."),
                "The request shows up here within a few seconds. No need to press anything.".into(),
            ],
        ));
        if s.phone_by_address() {
            let address = link.addresses.first().map_or("this laptop's address", String::as_str);
            v.push(notice(
                look,
                Icon::Public,
                "No phone yet?",
                vec![
                    "Some networks, at offices and hotels, keep devices from finding each other. Restarting the app \
                     on the phone helps sometimes."
                        .into(),
                    format!("Or add the laptop by its address in the phone's app (Add devices by IP): {address}."),
                ],
            ));
        }
    }
    let others: Vec<&str> = link.others().map(|d| d.name.as_str()).collect();
    if st != State::Done && !others.is_empty() {
        v.push(para(
            look,
            format!("Also on this network: {}. Not yours? Leave them be.", others.join(", ")),
        ));
    }
    if st == State::Done {
        v.push(notice(
            look,
            Icon::CheckCircle,
            "What works each way",
            vec![
                "Laptop to phone: copy here and it's on the phone's clipboard. Files go from the Command menu's Send \
                 a file to the phone, and Ring the phone finds it."
                    .into(),
                "Phone to laptop on iPhone: open KDE Connect and tap Send clipboard, or share to KDE Connect. iOS \
                 never lets an app read the clipboard in the background, so it is always by hand."
                    .into(),
                "Phone to laptop on Android: the same from the app, its quick tile or its notification.".into(),
                "That's all it can do here: no moving the pointer, typing, running commands or browsing the \
                 laptop's files. KubaOS keeps those KDE Connect plugins off."
                    .into(),
                "Try it: Send a test copy and paste on the phone. Then send something back and press Check what the \
                 phone sent."
                    .into(),
            ],
        ));
    }
    v
}
