import QtQuick
import KubaOS.Components as K
import qs.services

// The Proxy, while it is on or has failed: a locked shield, with the tunnel's endpoint in the
// hint, and a dot while something is wrong with it. Click opens the control center, where its
// tile switches it.
K.Pill {
    id: root

    visible: Proxy.on || Proxy.problem !== ""
    interactive: true
    padding: K.Tokens.space.s8
    tooltip: {
        if (Proxy.problem !== "")
            return `Proxy: ${Proxy.problem}. Click for the control center`;
        const parts = ["Proxy on"];
        if (Proxy.endpoint !== "")
            parts.push(`${Proxy.kind.toLowerCase()} to ${Proxy.endpoint}`);
        return `${parts.join(": ")}. Click for the control center`;
    }
    onClicked: Panels.open("controlcenter")

    K.BadgedIcon {
        name: Proxy.on ? "vpn_lock" : "key"
        color: K.Tokens.color.tertiary
        badged: Proxy.problem !== ""
    }
}
