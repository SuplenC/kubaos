pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Pipewire
import "audio.js" as Logic

// Default PipeWire output and input: each one's level and mute, the output's boost and balance,
// the lists to pick the defaults from, the apps playing and recording, and a Bluetooth headset's
// mode (`dot-audio headset`).
Singleton {
    id: root

    readonly property PwNode sink: Pipewire.defaultAudioSink
    readonly property bool ready: sink?.ready ?? false
    // The output's level, from 0 to `boostMax`: its loudest channel, so balance leaves it be.
    readonly property real volume: Logic.level(root.sinkVolumes)
    readonly property bool muted: sink?.audio?.muted ?? false
    readonly property real boostMax: Logic.boostMax
    // From -1 (left only) to 1 (right only); 0 for a mono output.
    readonly property real balance: Logic.balance(root.sinkSides, root.sinkVolumes)
    readonly property var sinks: Pipewire.nodes.values.filter(n => n.isSink && !n.isStream && n.audio)
    readonly property PwNode source: Pipewire.defaultAudioSource
    readonly property var sources: Pipewire.nodes.values.filter(n => !n.isSink && !n.isStream && n.audio)
    readonly property real inputVolume: source?.audio?.volume ?? 0
    readonly property bool inputMuted: source?.audio?.muted ?? false
    readonly property string inputIcon: inputMuted || inputVolume <= 0 ? "mic_off" : "mic"
    readonly property string icon: muted || volume <= 0 ? "volume_off" : volume < 0.34 ? "volume_mute" : volume < 0.67 ? "volume_down" : "volume_up"

    // Apps playing and apps recording, as their PipeWire streams.
    readonly property var outputApps: Logic.apps(root.streams, true)
    readonly property var inputApps: Logic.apps(root.streams, false)
    readonly property bool micInUse: root.inputApps.length > 0

    // The default output's Bluetooth headset, when it is one: its name and "calls" or "music".
    readonly property bool headset: root.headsetState.device !== ""
    readonly property string headsetDevice: root.headsetState.device
    readonly property string headsetMode: root.headsetState.mode

    readonly property var sinkVolumes: sink?.audio ? [...sink.audio.volumes] : []
    readonly property var sinkSides: sink?.audio ? [...sink.audio.channels].map(c => Logic.sideOf(PwAudioChannel.toString(c))) : []
    readonly property var streams: Pipewire.nodes.values.filter(n => n.isStream && n.audio)
    property var headsetState: Logic.headsetStatus("")

    function setVolume(v: real): void {
        if (!sink?.audio)
            return;
        sink.audio.muted = false;
        sink.audio.volumes = Logic.leveled(root.sinkVolumes, v, root.boostMax);
    }

    function toggleMute(): void {
        if (sink?.audio)
            sink.audio.muted = !sink.audio.muted;
    }

    function setBalance(b: real): void {
        if (sink?.audio)
            sink.audio.volumes = Logic.balanced(root.sinkSides, root.sinkVolumes, b);
    }

    function setInputVolume(v: real): void {
        if (!source?.audio)
            return;
        source.audio.muted = false;
        source.audio.volume = Logic.clamp(v, 1);
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

    function appName(stream: PwNode): string {
        return Logic.appName(stream?.properties) || "Unknown app";
    }

    // An icon name for the theme: the one the app gives its stream, else its desktop entry's.
    // Reading the entries makes a binding over this look again once they are scanned, which
    // finishes seconds after the shell starts.
    function appIcon(stream: PwNode): string {
        const props = stream?.properties;
        if (DesktopEntries.applications.values.length === 0)
            return props?.["application.icon-name"] ?? "";
        if (props?.["application.icon-name"])
            return props["application.icon-name"];
        for (const key of Logic.entryKeys(props)) {
            const entry = DesktopEntries.heuristicLookup(key);
            if (entry?.icon)
                return entry.icon;
        }
        return "";
    }

    // The device an app plays to or records from; null while it isn't linked.
    function deviceOf(stream: PwNode): PwNode {
        return Logic.deviceOf(stream, Pipewire.linkGroups.values);
    }

    function setAppVolume(stream: PwNode, v: real): void {
        if (!stream?.audio)
            return;
        stream.audio.muted = false;
        stream.audio.volume = Logic.clamp(v, 1);
    }

    function setAppMuted(stream: PwNode, muted: bool): void {
        if (stream?.audio)
            stream.audio.muted = muted;
    }

    function toggleAppMute(stream: PwNode): void {
        if (stream?.audio)
            stream.audio.muted = !stream.audio.muted;
    }

    // Sends an app to another device. Quickshell has no write to PipeWire's metadata, so it goes
    // through `pw-metadata`; WirePlumber moves the stream and keeps it there.
    function moveApp(stream: PwNode, device: PwNode): void {
        if (stream && device)
            Quickshell.execDetached(["pw-metadata", String(stream.id), "target.object", Logic.targetName(device)]);
    }

    function setHeadsetMode(mode: string): void {
        if (!root.headset || (mode !== "calls" && mode !== "music"))
            return;
        headsetSwitch.command = root.headsetCommand([mode]);
        headsetSwitch.running = true;
    }

    function refreshHeadset(): void {
        headsetStatus.running = true;
    }

    // `dot-audio headset` with its arguments: `calls`, `music`, or `status --json`.
    function headsetCommand(args: var): var {
        return ["dot-audio", "headset", ...args];
    }

    // A headset's mode changes the default devices (A2DP and HFP are nodes of their own), so a
    // new default is when to read it again.
    onSinkChanged: root.refreshHeadset()
    onSourceChanged: root.refreshHeadset()

    // Nodes only report volume, mute and properties while bound.
    PwObjectTracker {
        objects: [...root.sinks, ...root.sources, ...root.streams]
    }

    Process {
        id: headsetStatus

        command: root.headsetCommand(["status", "--json"])
        running: true
        stdout: StdioCollector {
            onStreamFinished: root.headsetState = Logic.headsetStatus(text)
        }
    }

    Process {
        id: headsetSwitch

        onExited: root.refreshHeadset()
    }
}
