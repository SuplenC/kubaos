pragma Singleton
import QtQuick

// Stand-in for Audio: two outputs, the laptop's speakers at 62%, and two inputs, the laptop's
// microphone the default at 80%. The devices are objects, as PipeWire's nodes are, so a row's device
// is the default one by identity.
QtObject {
    id: root

    component Device: QtObject {
        property string description
        property bool isSink
    }

    readonly property Device speakers: Device {
        description: "Built-in Audio Analog Stereo"
        isSink: true
    }
    readonly property Device headphones: Device {
        description: "WH-1000XM4"
        isSink: true
    }
    readonly property Device microphone: Device {
        description: "Built-in Audio Digital Microphone"
        isSink: false
    }
    readonly property Device headset: Device {
        description: "WH-1000XM4"
        isSink: false
    }

    readonly property var sinks: [root.speakers, root.headphones]
    readonly property var sink: root.speakers
    readonly property var sources: [root.microphone, root.headset]
    readonly property var source: root.microphone
    readonly property bool ready: true
    property real volume: 0.62
    property bool muted: false
    property real inputVolume: 0.8
    property bool inputMuted: false
    readonly property string inputIcon: inputMuted || inputVolume <= 0 ? "mic_off" : "mic"
    readonly property string icon: muted || volume <= 0 ? "volume_off" : volume < 0.34 ? "volume_mute" : volume < 0.67 ? "volume_down" : "volume_up"

    function setVolume(v: real): void {
        root.volume = v;
    }

    function toggleMute(): void {
        root.muted = !root.muted;
    }

    function setInputVolume(v: real): void {
        root.inputVolume = v;
    }

    function toggleInputMute(): void {
        root.inputMuted = !root.inputMuted;
    }

    function setDefault(node: var): void {
    }

    function setDefaultSource(node: var): void {
    }

    function label(node: var): string {
        return node.description;
    }
}
