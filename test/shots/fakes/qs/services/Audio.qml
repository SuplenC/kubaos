pragma Singleton
import QtQuick

// Stand-in for Audio: two outputs, the laptop's speakers at 62%, and two inputs, the laptop's
// microphone the default at 80%. Firefox plays to the speakers and Spotify to the headphones;
// Discord records from the microphone. The devices are objects, as PipeWire's nodes are, so a
// row's device is the default one by identity. The headset is off until a scene turns it on.
QtObject {
    id: root

    component Device: QtObject {
        property string description
        property bool isSink
    }

    component Level: QtObject {
        property real volume
        property bool muted
    }

    component Stream: QtObject {
        property string appName
        property string appIcon
        property bool isSink
        property var device
        property Level audio
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
    readonly property Device headsetMic: Device {
        description: "WH-1000XM4"
        isSink: false
    }

    readonly property Stream firefox: Stream {
        appName: "Firefox"
        appIcon: "firefox"
        isSink: true
        device: root.speakers
        audio: Level {
            volume: 1
        }
    }
    readonly property Stream spotify: Stream {
        appName: "Spotify"
        appIcon: "spotify"
        isSink: true
        device: root.headphones
        audio: Level {
            volume: 0.45
        }
    }
    readonly property Stream discord: Stream {
        appName: "Discord"
        appIcon: "discord"
        isSink: false
        device: root.microphone
        audio: Level {
            volume: 1
        }
    }

    readonly property var sinks: [root.speakers, root.headphones]
    readonly property var sink: root.speakers
    readonly property var sources: [root.microphone, root.headsetMic]
    readonly property var source: root.microphone
    readonly property bool ready: true
    property real volume: 0.62
    property bool muted: false
    readonly property real boostMax: 1.5
    property real balance: 0
    property real inputVolume: 0.8
    property bool inputMuted: false
    readonly property string inputIcon: inputMuted || inputVolume <= 0 ? "mic_off" : "mic"
    readonly property string icon: muted || volume <= 0 ? "volume_off" : volume < 0.34 ? "volume_mute" : volume < 0.67 ? "volume_down" : "volume_up"

    property var outputApps: [root.firefox, root.spotify]
    property var inputApps: [root.discord]
    readonly property bool micInUse: root.inputApps.length > 0

    property bool headset: false
    readonly property string headsetDevice: root.headset ? "WH-1000XM4" : ""
    property string headsetMode: root.headset ? "calls" : ""

    function setVolume(v: real): void {
        root.muted = false;
        root.volume = Math.max(0, Math.min(root.boostMax, v));
    }

    function toggleMute(): void {
        root.muted = !root.muted;
    }

    function setBalance(b: real): void {
        root.balance = Math.max(-1, Math.min(1, b));
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

    function appName(stream: var): string {
        return stream.appName;
    }

    function appIcon(stream: var): string {
        return stream.appIcon;
    }

    function deviceOf(stream: var): var {
        return stream.device;
    }

    function setAppVolume(stream: var, v: real): void {
        stream.audio.muted = false;
        stream.audio.volume = Math.max(0, Math.min(1, v));
    }

    function setAppMuted(stream: var, muted: bool): void {
        stream.audio.muted = muted;
    }

    function toggleAppMute(stream: var): void {
        stream.audio.muted = !stream.audio.muted;
    }

    function moveApp(stream: var, device: var): void {
        stream.device = device;
    }

    function setHeadsetMode(mode: string): void {
        if (root.headset && (mode === "calls" || mode === "music"))
            root.headsetMode = mode;
    }

    function refreshHeadset(): void {
    }
}
