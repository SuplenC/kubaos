pragma Singleton
import QtQuick

// Stand-in for Battery: 64%, discharging, 3 h 12 min left.
QtObject {
    readonly property bool available: true
    readonly property real percentage: 0.64
    readonly property bool charging: false
    readonly property bool low: false
    readonly property string icon: `battery_${Math.min(3, Math.floor(percentage * 4))}_bar`
    readonly property string timeLeft: "3h 12m left"
}
