pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Services.Pipewire

// Default PipeWire output and input: each one's volume and mute, and the lists to pick the
// defaults from.
Singleton {
    id: root

    readonly property PwNode sink: Pipewire.defaultAudioSink
    readonly property bool ready: sink?.ready ?? false
    readonly property real volume: sink?.audio?.volume ?? 0
    readonly property bool muted: sink?.audio?.muted ?? false
    readonly property var sinks: Pipewire.nodes.values.filter(n => n.isSink && !n.isStream && n.audio)
    readonly property PwNode source: Pipewire.defaultAudioSource
    readonly property var sources: Pipewire.nodes.values.filter(n => !n.isSink && !n.isStream && n.audio)
    readonly property real inputVolume: source?.audio?.volume ?? 0
    readonly property bool inputMuted: source?.audio?.muted ?? false
    readonly property string inputIcon: inputMuted || inputVolume <= 0 ? "mic_off" : "mic"
    readonly property string icon: muted || volume <= 0 ? "volume_off" : volume < 0.34 ? "volume_mute" : volume < 0.67 ? "volume_down" : "volume_up"

    function setVolume(v: real): void {
        if (!sink?.audio)
            return;
        sink.audio.muted = false;
        sink.audio.volume = Math.max(0, Math.min(1, v));
    }

    function toggleMute(): void {
        if (sink?.audio)
            sink.audio.muted = !sink.audio.muted;
    }

    function setInputVolume(v: real): void {
        if (!source?.audio)
            return;
        source.audio.muted = false;
        source.audio.volume = Math.max(0, Math.min(1, v));
    }

    function toggleInputMute(): void {
        if (source?.audio)
            source.audio.muted = !source.audio.muted;
    }

    function setDefault(node: PwNode): void {
        Pipewire.preferredDefaultAudioSink = node;
    }

    function setDefaultSource(node: PwNode): void {
        Pipewire.preferredDefaultAudioSource = node;
    }

    function label(node: PwNode): string {
        return node?.description || node?.nickname || node?.name || "Unknown device";
    }

    // How the device is attached: "bluetooth", "usb", "hdmi", "builtin", or "" for a virtual one.
    // A Bluetooth headset's nodes are loopbacks WirePlumber makes, without the device's
    // `device.api`, so they are known by their `bluez_` names.
    function connection(node: PwNode): string {
        const props = node?.properties ?? {};
        const api = props["device.api"] ?? "";
        if (api === "bluez5" || (node?.name ?? "").startsWith("bluez_"))
            return "bluetooth";
        if (api === "")
            return "";
        if (props["device.bus"] === "usb")
            return "usb";
        if ((props["device.profile.name"] ?? "").includes("hdmi"))
            return "hdmi";
        return "builtin";
    }

    function connectionName(node: PwNode): string {
        switch (connection(node)) {
        case "bluetooth":
            return qsTr("Bluetooth");
        case "usb":
            return qsTr("USB");
        case "hdmi":
            return qsTr("HDMI");
        case "builtin":
            return qsTr("Built-in");
        }
        return "";
    }

    function connectionIcon(node: PwNode): string {
        switch (connection(node)) {
        case "bluetooth":
            return "bluetooth";
        case "usb":
            return "usb";
        case "hdmi":
            return "tv";
        }
        return node?.isSink ? "speaker" : "mic";
    }

    // Nodes only report volume and mute while bound.
    PwObjectTracker {
        objects: [...root.sinks, ...root.sources]
    }
}
