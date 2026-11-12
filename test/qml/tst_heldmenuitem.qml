import QtQuick
import QtTest
import KubaOS.Components as K

// A held menu item is taken only once held for `duration.hold`, by the pointer or by Enter in its
// menu; let go early, its sweep runs back and nothing is taken.
TestCase {
    id: suite

    name: "HeldMenuItem"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    // design-allow(size): a test's fixture, not a design value.
    height: 300
    visible: true

    K.MenuItem {
        id: item

        width: K.Tokens.size.menu
        text: "Hold to remove"
        held: true
    }

    K.Menu {
        id: menu

        y: K.Tokens.size.control_sm * 2
        width: K.Tokens.size.menu
        currentIndex: 1
        entries: [
            {
                text: "Rename"
            },
            {
                text: "Hold to remove",
                held: true
            }
        ]
    }

    SignalSpy {
        id: taken

        target: item
        signalName: "activated"
    }

    SignalSpy {
        id: picked

        target: menu
        signalName: "activated"
    }

    function init() {
        item.keyHolding = false;
        item.hold = 0;
        taken.clear();
        picked.clear();
    }

    function test_a_click_never_takes_it() {
        mouseClick(item);

        wait(K.Tokens.duration.short);
        compare(taken.count, 0);
    }

    function test_held_long_enough_it_is_taken() {
        item.keyHolding = true;

        taken.wait(K.Tokens.duration.hold * 2);
        compare(taken.count, 1);
        compare(item.hold, 1);
    }

    function test_let_go_early_its_sweep_runs_back_and_nothing_is_taken() {
        item.keyHolding = true;
        wait(K.Tokens.duration.hold / 2);
        verify(item.hold > 0);

        item.keyHolding = false;

        tryCompare(item, "hold", 0, K.Tokens.duration.short * 4);
        compare(taken.count, 0);
    }

    function test_enter_in_its_menu_holds_it_while_it_is_down() {
        menu.forceActiveFocus();

        keyPress(Qt.Key_Return);
        verify(menu.keyHeld);
        keyRelease(Qt.Key_Return);

        verify(!menu.keyHeld);
        compare(picked.count, 0);
    }

    function test_enter_kept_down_in_its_menu_takes_it() {
        menu.forceActiveFocus();

        keyPress(Qt.Key_Return);

        picked.wait(K.Tokens.duration.hold * 2);
        compare(picked.signalArguments[0][0], 1);
        keyRelease(Qt.Key_Return);
    }

    function test_the_keyboard_moving_off_it_lets_go() {
        menu.forceActiveFocus();
        keyPress(Qt.Key_Return);

        menu.currentIndex = 0;

        verify(!menu.keyHeld);
        keyRelease(Qt.Key_Return);
    }
}
