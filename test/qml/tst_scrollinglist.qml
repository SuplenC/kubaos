import QtQuick
import QtTest
import KubaOS.Components as K

// A scrolling list's rows are drawn through the well's rounded shape while they reach its corners,
// and still take the pointer, 12 in from the well's sides.
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

    Component {
        id: longWell

        K.ScrollingList {
            anchors.fill: parent

            K.ListRows {
                anchors.fill: parent
                topMargin: K.Tokens.space.s12
                bottomMargin: K.Tokens.space.s12
                model: 20

                delegate: K.ListRow {
                    required property int index

                    width: ListView.view.width
                    title: `Row ${index}`
                }
            }
        }
    }

    Component {
        id: changingWell

        K.ScrollingList {
            readonly property alias rows: rows

            anchors.fill: parent

            K.ListRows {
                anchors.fill: parent
                topMargin: K.Tokens.space.s12
                bottomMargin: K.Tokens.space.s12
                model: ListModel {
                    id: rows

                    ListElement {
                        name: "First"
                    }

                    ListElement {
                        name: "Second"
                    }
                }

                delegate: K.ListRow {
                    required property string name

                    width: ListView.view.width
                    title: name
                }
            }
        }
    }

    // The item that draws the well's rows through its shape.
    function layerOf(well: Item): Item {
        return Array.from(well.children).find(child => child.layer.effect !== null);
    }

    function test_rows_that_fit_are_drawn_as_they_are() {
        verify(!well.shaped);
        verify(!suite.layerOf(well).layer.enabled);
    }

    function test_rows_past_the_well_are_drawn_through_its_shape() {
        const long = createTemporaryObject(longWell, suite);

        tryVerify(() => long.shaped);
        verify(suite.layerOf(long).layer.enabled);
    }

    function test_rows_added_past_the_well_are_drawn_through_its_shape_until_they_go() {
        const changing = createTemporaryObject(changingWell, suite);
        waitForRendering(changing);
        verify(!changing.shaped);

        for (let i = 0; i < 10; i++)
            changing.rows.append({
                name: `Added ${i}`
            });
        tryVerify(() => changing.shaped);
        changing.rows.remove(2, 10);

        tryVerify(() => !changing.shaped);
    }

    function test_rows_its_owner_set_adrift_are_drawn_through_the_wells_shape() {
        well.adrift = true;

        verify(well.shaped);

        well.adrift = false;
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
