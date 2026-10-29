import QtQuick
import QtTest
import KubaOS.Components as K

// A scrolling list's rows are drawn through the well's rounded shape, and still take the pointer,
// 12 in from the well's sides.
TestCase {
    id: suite

    name: "ScrollingList"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 300

    property var pressed: []

    K.ScrollingList {
        id: well

        anchors.fill: parent

        K.ListRows {
            id: list

            anchors.fill: parent
            topMargin: K.Tokens.space.s12
            bottomMargin: K.Tokens.space.s12
            model: 3

            delegate: K.ListRow {
                required property int index

                width: ListView.view.width
                title: `Row ${index}`
                onActivated: suite.pressed = suite.pressed.concat([index])
            }
        }
    }

    function test_a_click_reaches_the_row_under_it() {
        const row = list.itemAtIndex(1);

        mouseClick(row);

        compare(suite.pressed, [1]);
    }

    function test_the_rows_are_the_wells_padding_in_from_its_sides() {
        const row = list.itemAtIndex(0);
        const at = row.mapToItem(well, 0, 0);

        compare(at.x, K.Tokens.space.s12);
        compare(at.y, K.Tokens.space.s12);
        compare(row.width, well.width - K.Tokens.space.s12 * 2);
    }
}
