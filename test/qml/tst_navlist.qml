import QtQuick
import QtTest
import KubaOS.Components as K
import "../../config/quickshell/components"

// A list in a Loader, as the Keyboard tab's add view is, closed while its rows are still being
// made. Qt's message for a row whose making outlives the close is an info line, which no
// TestCase can catch: the check that runs these (`qml-logic`, crates/checks.nix) fails on it.
TestCase {
    id: suite

    name: "NavList"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 400

    property int rows: 0

    Loader {
        id: loader

        anchors.fill: parent
        active: false

        sourceComponent: NavList {
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
