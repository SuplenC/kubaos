pragma Singleton
import QtQuick

QtObject {
    id: root

    property string device: "intel_backlight"
    readonly property bool available: true
    property real value: 0.8
    readonly property string icon: value < 0.34 ? "brightness_low" : value < 0.67 ? "brightness_medium" : "brightness_high"

    function set(v: real): void {
        root.value = v;
    }
}
