//! The Wi-Fi screen: the networks in range, their signal, and the password for the picked one.

use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::list::list_row;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::header;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use super::account::secret_field;
use super::choices;
use crate::app::App;
use crate::app::F;
use crate::app::List;
use crate::app::Message;
use crate::app::ids;
use crate::theme::jp;
use crate::ui::El;

/// A network's signal as one of the Wi-Fi symbols.
fn signal(strength: u8) -> Icon {
    match strength {
        75.. => Icon::SignalWifi4Bar,
        50.. => Icon::NetworkWifi3Bar,
        25.. => Icon::NetworkWifi2Bar,
        _ => Icon::NetworkWifi1Bar,
    }
}

pub(super) fn wifi<'a>(app: &'a App, look: &Look) -> El<'a> {
    let Some(probe) = app.ready_probe() else {
        return widget::Space::new().into();
    };
    let none = app.form.wifi_ssid.is_none();
    let mut rows: Vec<El> = vec![
        list_row("Fuck Wi-Fi, I've got a cable")
            .icon(Icon::WifiOff)
            .selected(none)
            .checked(none)
            .on_press(Message::WifiNone)
            .view(look),
    ];
    for w in &probe.wifi {
        let sel = app.form.wifi_ssid.as_deref() == Some(&w.ssid);
        rows.push(
            list_row(w.ssid.clone())
                .icon(signal(w.signal))
                .detail(if w.secured { "Secured" } else { "Open" })
                .selected(sel)
                .checked(sel)
                .on_press(Message::Pick(List::Wifi, w.ssid.clone()))
                .view(look),
        );
    }
    let mut page = stack(look, Space::S24)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::NETWORK,
            "Wi-Fi",
            "Optional. I don't need the internet to install; this only saves a network for your first boot.",
        ))
        .push(choices(ids::WIFI_LIST, rows, app.is_focused(F::WifiList), look));
    if app.wifi_secured(app.form.wifi_ssid.as_deref()) {
        page = page.push(secret_field(app, "Wi-Fi password", F::WifiPsk, look));
    }
    page.into()
}
