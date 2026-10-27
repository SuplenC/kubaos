import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// The Proxy, while it is on: a locked shield, with the tunnel's endpoint in the hint. Click opens
// the control center, where its tile switches it off.
Pill {
    id: root

    visible: Proxy.on
    interactive: true
    padding: K.Tokens.space.s8
    tooltip: {
        const parts = ["Proxy on"];
        if (Proxy.endpoint !== "")
            parts.push(`${Proxy.kind.toLowerCase()} to ${Proxy.endpoint}`);
        return `${parts.join(": ")}. Click for the control center`;
    }
    onClicked: Panels.open("controlcenter")

    K.Icon {
        name: "vpn_lock"
        size: K.Tokens.icon.s16
        color: K.Tokens.color.tertiary
    }
}
