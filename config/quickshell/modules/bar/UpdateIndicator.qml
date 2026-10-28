import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// A small marker while the daily check has a KubaOS release or apt security
// updates waiting. Click opens a terminal running kubaos update, or kubaos
// upgrade when only the next series is waiting.
Pill {
    id: root

    readonly property bool onlyUpgrade: Updates.update === "" && Updates.upgrade !== "" && Updates.aptSecurity === 0
    readonly property bool securityWaiting: Updates.update === "" && Updates.aptSecurity > 0

    visible: Updates.available
    interactive: true
    padding: K.Tokens.space.s8
    tooltip: onlyUpgrade ? `KubaOS ${Updates.upgradeCodename || Updates.upgrade} is out. Click and I'll run kubaos upgrade` : securityWaiting ? `${Updates.aptSecurity} security ${Updates.aptSecurity === 1 ? "update is" : "updates are"} waiting. Click and I'll run kubaos update. Or don't, I'm not your mum` : `KubaOS ${Updates.update} is ready. Click and I'll run kubaos update. Or don't, I'm not your mum`
    onClicked: Updates.run(onlyUpgrade ? "upgrade" : "update")

    // Security updates waiting are counted; a release waiting is a dot.
    BadgedIcon {
        name: "system_update_alt"
        color: K.Tokens.color.tertiary
        badged: true
        count: Updates.aptSecurity
    }
}
