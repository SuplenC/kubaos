import QtQuick
import QtTest
import KubaOS.Components as K

// A pressable chip that is no stop for Tab: Tab goes from the field before it to the one after,
// a click still presses it, and its tooltip shows while the pointer is on it.
TestCase {
    id: suite

    name: "Chip"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 200

    K.Row {
        K.Field {
            id: before
        }

        K.Chip {
            id: chip

            text: "Day first"
            pressable: true
            tabStop: false
            tooltip: "Your locale's"
        }

        K.Chip {
            id: stop

            text: "Month first"
            pressable: true
        }
    }

    SignalSpy {
        id: activated

        target: chip
        signalName: "activated"
    }

    function hint(): Item {
        return findChild(chip, "hint");
    }

    function test_tab_passes_a_chip_that_is_no_tab_stop() {
        before.forceActiveFocus();

        keyClick(Qt.Key_Tab);

        verify(!chip.focused);
        verify(stop.focused);
    }

    function test_a_click_presses_a_chip_that_is_no_tab_stop() {
        activated.clear();

        mouseClick(chip);

        compare(activated.count, 1);
    }

    function test_its_tooltip_shows_while_the_pointer_is_on_it() {
        mouseMove(chip);

        tryVerify(() => suite.hint()?.shown === true);
        mouseMove(suite, suite.width - 1, suite.height - 1);
        tryVerify(() => suite.hint()?.shown === false);
    }

    function test_a_chip_without_a_tooltip_makes_no_hint() {
        verify(!findChild(stop, "hint"));
    }
}
