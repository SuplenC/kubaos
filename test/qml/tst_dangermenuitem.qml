import QtQuick
import QtTest
import KubaOS.Components as K

// A danger menu item lit by the keyboard lies on the error with its words `on_error`, and pressed
// it takes `error_pressed`; unlit it is the error on the popover, and a held item keeps its tint.
TestCase {
    id: suite

    name: "DangerMenuItem"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    // design-allow(size): a test's fixture, not a design value.
    height: 300
    visible: true

    K.MenuItem {
        id: item

        width: K.Tokens.size.menu
        text: "Kill"
        danger: true
    }

    function init() {
        item.highlighted = false;
        item.held = false;
        item.preview = "";
    }

    function test_an_unlit_danger_item_is_the_error_on_the_popover() {
        compare(item.ink, K.Tokens.color.error);
    }

    function test_a_lit_danger_item_lies_on_the_error_with_on_error_words() {
        item.highlighted = true;

        compare(item.ink, K.Tokens.color.on_error);
        compare(item.litFill, K.Tokens.color.error);
    }

    function test_a_lit_danger_item_pressed_takes_error_pressed() {
        item.highlighted = true;
        item.preview = "pressed";

        compare(item.litFill, K.Tokens.color.error_pressed_fixed);
        compare(item.ink, K.Tokens.color.on_error);
    }

    function test_a_held_item_lit_keeps_its_tint_and_its_error_words() {
        item.held = true;
        item.highlighted = true;

        compare(item.ink, K.Tokens.color.error);
        compare(item.litFill, K.Look.alpha(K.Tokens.color.error, K.Tokens.opacity.tint));
    }
}
