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
        compare(names(Radios.btShown(Radios.btNearby(devices, {}), false).shown), ["Speaker"]);
    }

    function test_a_device_bluez_names_by_its_address_is_unnamed() {
        verify(Radios.unnamed(device("4C-2F-9A-01-BE-77", false, false, "4C:2F:9A:01:BE:77")), "BlueZ's alias, dashes for colons");
        verify(Radios.unnamed(device("4c-2f-9a-01-be-77", false, false, "4C:2F:9A:01:BE:77")), "whatever its case");
        verify(Radios.unnamed(device("", false, false, "4C:2F:9A:01:BE:77")), "no name at all");
        verify(!Radios.unnamed(device("Speaker")), "a name of its own");
    }

    function test_unnamed_devices_are_hidden_and_counted() {
        const devices = [device("Speaker"), device("11-22", false, false, "11:22"), device("33-44", false, false, "33:44"), device("Watch")];

        const hidden = Radios.btShown(Radios.btNearby(devices, {}), false);

        compare(names(hidden.shown), ["Speaker", "Watch"]);
        compare(hidden.unnamed, 2);
        compare(Radios.unnamedLine(hidden.unnamed, false), "2 unnamed devices hidden");
    }

    function test_show_lists_the_unnamed_devices_after_the_named_ones_and_hide_leaves_them_out_again() {
        const devices = [device("11-22", false, false, "11:22"), device("Speaker"), device("Watch")];
        const rssi = {
            "/org/bluez/hci0/11-22": -40,
            "/org/bluez/hci0/Watch": -88
        };
        const nearby = Radios.btNearby(devices, rssi);

        const shown = Radios.btShown(nearby, true);
        compare(names(shown.shown), ["Watch", "Speaker", "11-22"], "nearest first, the unnamed after, however near");
        compare(shown.unnamed, 1, "counted while shown too");
        compare(Radios.unnamedLine(shown.unnamed, true), "1 unnamed device shown");

        compare(names(Radios.btShown(nearby, false).shown), ["Watch", "Speaker"]);
    }

    function test_there_is_no_line_without_unnamed_devices() {
        compare(Radios.unnamedLine(0, false), "");
        compare(Radios.unnamedLine(0, true), "");
    }

    function test_known_devices_show_named_or_not() {
        const devices = [device("11-22", true, false, "11:22"), device("Headphones", true), device("33-44", false, false, "33:44")];

        compare(names(Radios.btKnown(devices)), ["11-22", "Headphones"]);
        compare(Radios.btShown(Radios.btNearby(devices, {}), false).unnamed, 1, "only the unpaired one counts");
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
