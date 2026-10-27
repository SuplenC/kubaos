import QtQuick
import qs.theme
import qs.components
import qs.services

// The Proxy, while it is on: a locked globe, with the tunnel's endpoint in
// the tooltip. Click opens the control center, where its tile switches it off.
Pill {
    id: root

    visible: Proxy.on
    interactive: true
    padding: Theme.spacing.sm
    tooltip: {
        const parts = ["Proxy on"];
        if (Proxy.endpoint !== "")
            parts.push(`${Proxy.kind.toLowerCase()} to ${Proxy.endpoint}`);
        return `${parts.join(": ")}. Click for the control center`;
    }
    onClicked: Panels.open("controlcenter")

    Icon {
        anchors.verticalCenter: parent.verticalCenter
        name: "vpn_lock"
        size: Theme.bar.icon
        color: Theme.m3tertiary
    }
}
