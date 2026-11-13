import QtQuick
import QtTest
import KubaOS.Components as K

// A rolling number's slots are whole device pixels wide and its box as wide as its widest value;
// only the figures that change roll, the way the count went.
TestCase {
    id: suite

    name: "RollingNumber"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    // design-allow(size): a test's fixture, not a design value.
    height: 200
    visible: true

    K.RollingNumber {
        id: count

        value: 9
        widest: "000"
    }

    function init() {
        K.Motion.reduced = false;
        count.value = 9;
        wait(K.Tokens.duration.short * 2);
    }

    function cleanup() {
        K.Motion.reduced = false;
    }

    function test_each_slot_is_a_whole_device_pixel_wide() {
        const dpr = count.dpr;

        fuzzyCompare(count.slot * dpr, Math.round(count.slot * dpr), 1e-6);
        compare(count.width, count.slot * 3);
    }

    function test_its_box_holds_still_as_the_count_grows_a_figure() {
        const before = count.width;

        count.value = 10;

        compare(count.width, before);
        compare(count.shown, "10");
    }

    function test_going_down_rolls_the_other_way() {
        count.value = 10;
        count.value = 8;

        compare(count.way, -1);
    }

    function test_a_value_past_its_widest_widens_it() {
        count.value = 1000;

        compare(count.width, count.slot * 4);
    }
}
