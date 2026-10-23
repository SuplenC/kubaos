import QtQuick
import qs.theme
import qs.components
import qs.services

// A small marker while the daily check has a KubaOS release waiting. Click
// opens a terminal running kubaos update, or kubaos upgrade when only the
// next series is waiting.
Pill {
    id: root

    readonly property bool onlyUpgrade: Updates.update === "" && Updates.upgrade !== ""

    visible: Updates.available
    interactive: true
    padding: Theme.spacing.sm
    tooltip: onlyUpgrade ? `KubaOS ${Updates.upgradeCodename || Updates.upgrade} is out. Click and I'll run kubaos upgrade` : `KubaOS ${Updates.update} is ready. Click and I'll run kubaos update. Or don't, I'm not your mum`
    onClicked: Updates.run(onlyUpgrade ? "upgrade" : "update")

    Icon {
        anchors.verticalCenter: parent.verticalCenter
        name: "system_update_alt"
        size: Theme.bar.icon
        color: Theme.m3tertiary
    }
}
