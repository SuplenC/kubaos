pragma Singleton
import QtQuick

// Stand-in for Audio: two outputs, the laptop's speakers at 62%.
QtObject {
    id: root

    readonly property var sinks: [
        {
            description: "Built-in Audio Analog Stereo",
            name: "alsa_output.pci"
        },
        {
            description: "WH-1000XM4",
            name: "bluez_output"
        }
    ]
    readonly property var sink: root.sinks[0]
    readonly property bool ready: true
    property real volume: 0.62
    property bool muted: false
    readonly property string icon: muted || volume <= 0 ? "volume_off" : volume < 0.34 ? "volume_mute" : volume < 0.67 ? "volume_down" : "volume_up"

    function setVolume(v: real): void {
        root.volume = v;
    }

    function toggleMute(): void {
        root.muted = !root.muted;
    }

    function setDefault(node: var): void {
    }

    function label(node: var): string {
        return node.description;
    }
}
