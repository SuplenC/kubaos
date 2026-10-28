.pragma library

// How the Wi-Fi and Bluetooth tabs group and order what is in range: Known (saved or paired), then
// Nearby, always by signal. Both order by a coarse signal (bars, a distance) and then by name, so a
// reading nudged by a point or two moves no row; a row moves when its bars or distance change.

// Signal bars from 0 to 4 for a `signalStrength` (0 to 1), at the thresholds `nmcli` draws its
// BARS column with, so the icons agree with `nmcli device wifi`.
// https://github.com/NetworkManager/NetworkManager/blob/main/src/libnm-core-impl/nm-utils.c
// (`nm_utils_wifi_strength_bars`)
function bars(strength) {
    // NetworkManager reports whole percents; rounding undoes the float division (0.55 * 100 is
    // 55.00000000000001).
    const percent = Math.round(strength * 100);
    if (percent > 80)
        return 4;
    if (percent > 55)
        return 3;
    if (percent > 30)
        return 2;
    return percent > 5 ? 1 : 0;
}

function byName(a, b) {
    return a.name.localeCompare(b.name);
}

// Saved networks in range: the one in use first, then by signal.
function wifiKnown(networks) {
    return networks.filter(n => n.known).sort((a, b) => (b.connected - a.connected) || (bars(b.signalStrength) - bars(a.signalStrength)) || byName(a, b));
}

// Networks in range that aren't saved, by signal.
function wifiNearby(networks) {
    return networks.filter(n => !n.known).sort((a, b) => (bars(b.signalStrength) - bars(a.signalStrength)) || byName(a, b));
}

// The Wi-Fi band of a frequency in MHz.
function band(mhz) {
    if (mhz >= 5925)
        return "6 GHz";
    if (mhz >= 4900)
        return "5 GHz";
    return mhz >= 2400 ? "2.4 GHz" : "";
}

// How near a device is from its RSSI in dBm: 3 very close, 2 nearby, 1 far, 0 not heard. A
// device across a room reads about -60; past a wall or two, under -75.
function nearness(rssi) {
    if (rssi === undefined || rssi === null)
        return 0;
    if (rssi >= -60)
        return 3;
    return rssi >= -75 ? 2 : 1;
}

function distance(rssi) {
    return ["", "Far", "Nearby", "Very close"][nearness(rssi)];
}

// Paired devices: the connected first, then by name.
function btKnown(devices) {
    return devices.filter(d => d.paired).sort((a, b) => (b.connected - a.connected) || byName(a, b));
}

// Devices in range to pair, nearest first, by their last RSSI (`rssi`, by object path). An
// unnamed discovery (its name is its address) is noise.
function btNearby(devices, rssi) {
    const near = d => nearness(rssi[d.dbusPath]);
    return devices.filter(d => !d.paired && d.name !== d.address).sort((a, b) => (near(b) - near(a)) || byName(a, b));
}

// KubaOS Symbols' drawing of a BlueZ device icon (`org.bluez.Device1.Icon`, freedesktop icon names).
function deviceSymbol(icon) {
    if (icon.startsWith("audio-head"))
        return "headphones";
    if (icon.startsWith("audio"))
        return "speaker";
    if (icon === "input-keyboard")
        return "keyboard";
    if (icon === "input-mouse" || icon === "input-tablet")
        return "mouse";
    if (icon === "phone")
        return "smartphone";
    if (icon === "computer")
        return "computer";
    return "bluetooth";
}

// The battery symbol for a charge from 0 to 1.
function batterySymbol(charge) {
    if (charge > 0.9)
        return "battery_full";
    return `battery_${Math.min(3, Math.floor(charge * 4))}_bar`;
}
