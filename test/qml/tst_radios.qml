import QtQuick
import QtTest
import "../../config/quickshell/services/radios.js" as Radios

// How the Wi-Fi and Bluetooth tabs group and order what is in range.
TestCase {
    id: suite

    name: "radios"

    function network(name, strength, known, connected) {
        return {
            name: name,
            signalStrength: strength,
            known: known ?? false,
            connected: connected ?? false
        };
    }

    function device(name, paired, connected, address) {
        return {
            name: name,
            address: address ?? `AA:${name}`,
            dbusPath: `/org/bluez/hci0/${name}`,
            paired: paired ?? false,
            connected: connected ?? false
        };
    }

    function names(list) {
        return list.map(item => item.name);
    }

    function test_bars_follow_nmcli() {
        compare([0.05, 0.06, 0.31, 0.56, 0.81].map(Radios.bars), [0, 1, 2, 3, 4]);
        compare(Radios.bars(0.55), 2, "0.55 * 100 rounds to 55, which is two bars");
    }

    function test_nearby_networks_are_sorted_by_signal() {
        const networks = [network("Weak", 0.2), network("Strong", 0.9), network("Middle", 0.6)];

        compare(names(Radios.wifiNearby(networks)), ["Strong", "Middle", "Weak"]);
    }

    function test_nearby_networks_with_the_same_bars_keep_their_order_by_name() {
        const networks = [network("b", 0.62), network("a", 0.7)];

        compare(names(Radios.wifiNearby(networks)), ["a", "b"]);
        compare(names(Radios.wifiNearby([network("b", 0.7), network("a", 0.62)])), ["a", "b"], "a strength nudged within its bars moves nothing");
    }

    function test_known_networks_are_not_nearby() {
        const networks = [network("Home", 0.9, true), network("Cafe", 0.5)];

        compare(names(Radios.wifiKnown(networks)), ["Home"]);
        compare(names(Radios.wifiNearby(networks)), ["Cafe"]);
    }

    function test_the_connected_network_leads_the_known_ones() {
        const networks = [network("Strong", 0.9, true), network("Used", 0.3, true, true)];

        compare(names(Radios.wifiKnown(networks)), ["Used", "Strong"]);
    }

    function test_distance_reads_the_rssi() {
        compare(Radios.distance(-45), "Very close");
        compare(Radios.distance(-70), "Nearby");
        compare(Radios.distance(-90), "Far");
        compare(Radios.distance(undefined), "");
    }

    function test_nearby_devices_are_sorted_by_distance_then_name() {
        const devices = [device("Far"), device("Close"), device("Unheard"), device("Also close")];
        const rssi = {
            "/org/bluez/hci0/Far": -88,
            "/org/bluez/hci0/Close": -40,
            "/org/bluez/hci0/Also close": -55
        };

        compare(names(Radios.btNearby(devices, rssi)), ["Also close", "Close", "Far", "Unheard"]);
    }

    function test_paired_devices_are_known_and_unnamed_ones_are_left_out() {
        const devices = [device("Headphones", true), device("AA:BB", false, false, "AA:BB"), device("Speaker")];

        compare(names(Radios.btKnown(devices)), ["Headphones"]);
        compare(names(Radios.btNearby(devices, {})), ["Speaker"]);
    }

    function test_connected_devices_lead_the_known_ones() {
        const devices = [device("A mouse", true), device("Z headphones", true, true)];

        compare(names(Radios.btKnown(devices)), ["Z headphones", "A mouse"]);
    }

    function test_band_names_the_frequency() {
        compare(Radios.band(2412), "2.4 GHz");
        compare(Radios.band(5180), "5 GHz");
        compare(Radios.band(5955), "6 GHz");
        compare(Radios.band(0), "");
    }

    function test_a_device_icon_names_a_symbol() {
        compare(Radios.deviceSymbol("audio-headset"), "headphones");
        compare(Radios.deviceSymbol("input-mouse"), "mouse");
        compare(Radios.deviceSymbol("phone"), "smartphone");
        compare(Radios.deviceSymbol("something-else"), "bluetooth");
    }

    function test_battery_symbol_follows_the_charge() {
        compare(Radios.batterySymbol(0.05), "battery_0_bar");
        compare(Radios.batterySymbol(0.5), "battery_2_bar");
        compare(Radios.batterySymbol(0.95), "battery_full");
    }
}
