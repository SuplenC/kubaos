import QtQuick
import QtTest
import KubaOS.Components as K

// A list's rows moved through with the keys (docs/DESIGN.md, "The keyboard"), and a list in a
// Loader, as the Keyboard tab's add view is, closed while its rows are still being made. Qt's
// message for a row whose making outlives the close is an info line, which no TestCase can catch:
// the check that runs these (`qml-logic`, crates/checks.nix) fails on it.
TestCase {
    id: suite

    name: "ListRows"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 400

    property int rows: 0
    // The rows of `keys` pressed, in order: by the list's `activated`, or by the focused row's own
    // press.
    property var activated: []

    K.ListRows {
        id: keys

        anchors.fill: parent
        model: 20
        // A section label's place, which the keys pass over.
        selectable: index => index !== 3
        onActivated: index => suite.activated = suite.activated.concat([index])

        delegate: K.ListRow {
            required property int index

            width: ListView.view.width
            title: `Row ${index}`
            onActivated: suite.activated = suite.activated.concat([index])
        }
    }

    function init() {
        loader.active = false;
        keys.visible = true;
        keys.vimKeys = true;
        keys.fieldKeys = false;
        keys.orientation = ListView.Vertical;
        keys.currentIndex = 0;
        suite.activated = [];
        keys.forceActiveFocus();
    }

    function test_the_arrows_ctrl_and_vim_keys_step_along_the_list() {
        keyClick(Qt.Key_Down);
        compare(keys.currentIndex, 1);
        keyClick(Qt.Key_J);
        compare(keys.currentIndex, 2);
        keyClick(Qt.Key_N, Qt.ControlModifier);
        compare(keys.currentIndex, 4, "past the row that can't be selected");
        keyClick(Qt.Key_K, Qt.ControlModifier);
        compare(keys.currentIndex, 2);
        keyClick(Qt.Key_P, Qt.ControlModifier);
        compare(keys.currentIndex, 1);
        keyClick(Qt.Key_Up);
        compare(keys.currentIndex, 0);
    }

    function test_a_step_past_either_end_wraps_round() {
        keyClick(Qt.Key_Up);
        compare(keys.currentIndex, 19);
        keyClick(Qt.Key_Down);
        compare(keys.currentIndex, 0);
    }

    function test_home_end_and_a_page_stop_at_the_ends() {
        keyClick(Qt.Key_End);
        compare(keys.currentIndex, 19);
        keyClick(Qt.Key_PageDown);
        compare(keys.currentIndex, 19);
        keyClick(Qt.Key_Home);
        compare(keys.currentIndex, 0);
        keyClick(Qt.Key_PageUp);
        compare(keys.currentIndex, 0);
    }

    function test_a_page_is_as_many_rows_as_show_whole() {
        const whole = Math.floor((keys.height + keys.spacing) / (K.Tokens.size.row + keys.spacing));

        keyClick(Qt.Key_PageDown);

        compare(keys.currentIndex, whole);
    }

    function test_a_jump_onto_a_row_that_cant_be_selected_lands_on_the_next() {
        keys.jump(3);

        compare(keys.currentIndex, 4);
    }

    function test_enter_presses_the_current_row_once() {
        keys.currentIndex = 5;

        keyClick(Qt.Key_Return);

        compare(suite.activated, [5]);
    }

    function test_plain_letters_are_left_alone_without_vim_keys() {
        keys.vimKeys = false;

        keyClick(Qt.Key_J);
        keyClick(Qt.Key_J, Qt.ControlModifier);

        compare(keys.currentIndex, 1, "Ctrl+J still steps");
    }

    function test_a_field_forwarding_its_keys_keeps_home_and_end() {
        keys.fieldKeys = true;
        keys.currentIndex = 5;

        keyClick(Qt.Key_Home);
        keyClick(Qt.Key_End);

        compare(keys.currentIndex, 5);
    }

    function test_a_list_laid_across_steps_with_left_right_h_and_l() {
        keys.orientation = ListView.Horizontal;

        keyClick(Qt.Key_Right);
        keyClick(Qt.Key_L);
        keyClick(Qt.Key_Down);

        compare(keys.currentIndex, 2);
    }

    Loader {
        id: loader

        anchors.fill: parent
        active: false

        sourceComponent: K.ListRows {
            model: suite.rows
            // Rows far past the edge, which the view makes a slice at a time between frames, so
            // a close lands while one is half made.
            cacheBuffer: 20000

            delegate: K.ListRow {
                required property int index

                width: ListView.view.width
                icon: "keyboard"
                tile: true
                title: `Layout ${index}`
                subtitle: `l${index}`
                checkable: true
            }
        }
    }

    function test_a_list_closed_while_its_rows_are_made_cancels_them() {
        keys.visible = false;
        for (let i = 0; i < 20; i++) {
            suite.rows = 0;
            loader.active = true;
            wait(20);
            suite.rows = 200;
            wait(10 + i * 10);

            loader.active = false;
            wait(20);
        }
    }
}
