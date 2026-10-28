import QtQuick
import QtTest
import KubaOS.Components as K

// A meter's fill as a new reading arrives: the same bar moves from the old value to the new one,
// never rebuilt, so never through 0.
TestCase {
    id: suite

    name: "Meter"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 100

    function fills() {
        return meter.children.filter(item => item instanceof Rectangle);
    }

    function reading(value) {
        return [
            {
                value: value,
                color: K.Tokens.color.primary
            }
        ];
    }

    K.Meter {
        id: meter

        // design-allow(size): a test's fixture, not a design value.
        width: 200
    }

    function test_new_reading_keeps_the_fill_and_never_dips_to_zero() {
        meter.parts = reading(0.5);
        wait(400);
        const before = fills();
        compare(before.length, 1);
        const bar = before[0];
        const from = bar.width;
        verify(from > 90);

        let lowest = from;
        meter.parts = reading(0.8);
        for (let i = 0; i < 20; i++) {
            lowest = Math.min(lowest, bar.width);
            wait(15);
        }
        compare(fills().length, 1);
        verify(fills()[0] === bar, "the fill is the same object");
        verify(lowest >= from - 1, "the fill never fell below its old width: " + lowest);
        wait(400);
        verify(bar.width > from + 50);
    }
}
