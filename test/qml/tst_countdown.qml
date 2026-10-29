import QtQuick
import QtTest
import "../../config/quickshell/modules/notifications"

// A popup's time left: it runs down while nothing holds it, stops while the pointer is on the popup
// (or its menu holds it), and goes on from where it stopped.
TestCase {
    id: suite

    name: "Countdown"

    Countdown {
        id: countdown

        // design-allow(duration): a test's fixture, not a design value.
        duration: 300
    }

    SignalSpy {
        id: finished

        target: countdown
        signalName: "finished"
    }

    function init() {
        countdown.running = false;
        countdown.held = false;
        finished.clear();
        countdown.running = true;
    }

    function test_it_runs_down_to_the_end() {
        finished.wait(1000);

        compare(countdown.remaining, 0);
    }

    function test_hovering_stops_it_where_it_is() {
        wait(100);
        countdown.held = true;
        const left = countdown.remaining;

        wait(400);

        compare(countdown.remaining, left);
        compare(finished.count, 0);
        verify(left > 0 && left < 1);
    }

    function test_it_goes_on_from_where_it_stopped() {
        wait(100);
        countdown.held = true;
        wait(400);

        countdown.held = false;

        verify(countdown.remaining > 0);
        finished.wait(1000);
    }

    function test_held_from_the_start_it_never_runs() {
        countdown.running = false;
        countdown.held = true;
        countdown.running = true;

        wait(400);

        compare(countdown.remaining, 1);
        compare(finished.count, 0);
    }
}
