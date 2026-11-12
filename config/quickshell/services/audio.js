.pragma library

// The logic behind `Audio.qml` that needs no PipeWire: which nodes are apps, what each app is
// called and plays to, a device's level and balance from its channel volumes, and reading
// `dot-audio headset status --json`. Nodes here are plain objects with a PwNode's fields.

// The master level goes past full to boost a quiet source; the slider holds at 1 on its way there.
const boostMax = 1.5;

function clamp(v, max) {
    return Math.max(0, Math.min(max, v));
}

// An app's stream, playing (`output`) or recording. Peak meters (Quickshell's own, pavucontrol's)
// are capture streams too, marked `stream.monitor`; they are no app.
function isApp(node, output) {
    return node.isStream && node.audio !== null && node.isSink === output && node.properties?.["stream.monitor"] !== "true";
}

function apps(nodes, output) {
    return nodes.filter(n => isApp(n, output));
}

// The name a person knows the app by. Not every client sets `application.name`; the binary's
// name is the next best.
function appName(props) {
    return props?.["application.name"] || props?.["application.process.binary"] || props?.["node.name"] || props?.["media.name"] || "";
}

// The names to look an app's desktop entry up by, best first: a Flatpak's or portal's app id,
// then the binary.
function entryKeys(props) {
    return [props?.["pipewire.access.portal.app_id"], props?.["application.id"], props?.["application.process.binary"], props?.["application.name"]].filter(Boolean);
}

// The device a stream plays to or records from: the far end of its link that is no stream. A
// playing stream is the source of its links, a recording one the target.
function deviceOf(stream, linkGroups) {
    for (const group of linkGroups) {
        if (stream.isSink && group.source === stream && !group.target?.isStream)
            return group.target;
        if (!stream.isSink && group.target === stream && !group.source?.isStream)
            return group.source;
    }
    return null;
}

// What `target.object` names a device by: its serial, which stays unique when two devices share
// a name, else its name.
function targetName(device) {
    return device.properties?.["object.serial"] || device.name;
}

// A device's level is its loudest channel, so moving the balance leaves the level where it was.
function level(volumes) {
    return volumes.length === 0 ? 0 : Math.max(...volumes);
}

// The channel volumes for level `v`, scaled together so the balance holds. Silent channels have
// no balance left to keep, so they all take `v`.
function leveled(volumes, v, max) {
    const target = clamp(v, max);
    const current = level(volumes);
    return current === 0 ? volumes.map(() => target) : volumes.map(x => x * target / current);
}

// A channel's side, from its name as `PwAudioChannel.toString` gives it ("Front Left", "Rear
// Right Center", "Low Frequency Effects"): -1 left, 1 right, 0 neither.
function sideOf(channelName) {
    return /\bLeft\b/.test(channelName) ? -1 : /\bRight\b/.test(channelName) ? 1 : 0;
}

function sideLevel(sides, volumes, side) {
    const ofSide = volumes.filter((_, i) => sides[i] === side);
    return ofSide.length === 0 ? 0 : ofSide.reduce((a, b) => a + b, 0) / ofSide.length;
}

// Balance from -1 (left only) to 1 (right only), from each channel's side: -1 left, 1 right,
// 0 neither (centre, LFE, mono). The quieter side's share of the louder one, as pavucontrol has it.
function balance(sides, volumes) {
    const left = sideLevel(sides, volumes, -1);
    const right = sideLevel(sides, volumes, 1);
    const loud = Math.max(left, right);
    if (loud === 0 || !sides.includes(-1) || !sides.includes(1))
        return 0;
    return right >= left ? 1 - left / loud : right / loud - 1;
}

// The channel volumes at balance `b`, the level kept: the louder side and the middle at the
// level, the other side lowered.
function balanced(sides, volumes, b) {
    const m = level(volumes);
    const towards = clamp(Math.abs(b), 1);
    return sides.map(side => side === 0 || Math.sign(b) === side || b === 0 ? m : m * (1 - towards));
}

// `dot-audio headset status --json`: `{"device": "...", "mode": "calls" | "music" | null}`. A
// device with no mode, or output that isn't the JSON, is no headset.
function headsetStatus(text) {
    try {
        const status = JSON.parse(text);
        const mode = status?.mode === "calls" || status?.mode === "music" ? status.mode : "";
        return {
            device: mode === "" ? "" : String(status.device ?? ""),
            mode: mode
        };
    } catch (e) {
        return {
            device: "",
            mode: ""
        };
    }
}
