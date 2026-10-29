import QtQuick
import QtTest
import "../../config/quickshell/modules/controlcenter"

// When a scan runs: at once when its tab is shown, then every `every` for `window`, and never
// while the tab is hidden.
TestCase {
    id: suite

    name: "ScanPulse"

    ScanPulse {
        id: pulse

        every: 200
        window: 60
    }

    function init() {
        pulse.active = false;
    }

    function test_a_hidden_tab_never_scans() {
        wait(300);

        verify(!pulse.scanning);
    }

    function test_showing_the_tab_scans_at_once() {
        pulse.active = true;

        verify(pulse.scanning);
    }

    function test_a_scan_stops_after_its_window() {
        pulse.active = true;

        tryCompare(pulse, "scanning", false, 150);
    }

    function test_the_next_scan_comes_every_period() {
        pulse.active = true;
        tryCompare(pulse, "scanning", false, 150);

        tryCompare(pulse, "scanning", true, 250);
    }

    function test_hiding_the_tab_stops_a_running_scan_and_the_next_one() {
        pulse.active = true;

        pulse.active = false;

        verify(!pulse.scanning);
        wait(300);
        verify(!pulse.scanning);
    }

    function test_showing_the_tab_again_scans_at_once_again() {
        pulse.active = true;
        tryCompare(pulse, "scanning", false, 150);
        pulse.active = false;

        pulse.active = true;

        verify(pulse.scanning);
    }
}
