import QtQuick
import QtTest
import KubaOS.Components as K

// A row still being found keeps the place of the tiled two-line row that will take it, and takes
// no press while it waits.
TestCase {
    id: suite

    name: "ListRow"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 200

    property int presses: 0

    K.ListRow {
        id: row

        width: suite.width
        title: "Home-5G"
        onActivated: suite.presses++
    }

    K.ListRow {
        id: found

        y: K.Tokens.size.row_two_line
        width: suite.width
        title: "Home-5G"
        subtitle: "Secured"
        icon: "wifi"
        tile: true
    }

    // The first item under `item`, itself included, that `test` holds for.
    function find(item, test) {
        if (test(item))
            return item;
        for (const child of item.children) {
            const hit = suite.find(child, test);
            if (hit)
                return hit;
        }
        return null;
    }

    function init() {
        row.loading = false;
        suite.presses = 0;
    }

    function test_a_loading_row_is_as_tall_as_a_two_line_row() {
        compare(row.implicitHeight, K.Tokens.size.row);

        row.loading = true;

        compare(row.implicitHeight, K.Tokens.size.row_two_line);
    }

    function test_a_loading_row_takes_no_press() {
        row.loading = true;

        mouseClick(row);

        compare(suite.presses, 0);
    }

    function test_a_loading_rows_skeletons_sit_where_the_found_rows_tile_and_title_do() {
        row.loading = true;
        tryVerify(() => suite.find(row, item => item.bone === "block") !== null);

        const bone = suite.find(row, item => item.bone === "block");
        const line = suite.find(row, item => item.bone === "line");
        const tile = suite.find(found, item => item.probe?.name === "IconTile");
        const title = suite.find(found, item => item.text === "Home-5G");
        const boneAt = bone.mapToItem(row, 0, 0);
        const tileAt = tile.mapToItem(found, 0, 0);
        compare(boneAt.x, tileAt.x);
        compare(boneAt.y, tileAt.y);
        compare(bone.width, tile.width);
        compare(line.mapToItem(row, 0, 0).x, title.mapToItem(found, 0, 0).x);
    }

    function test_a_loaded_row_takes_a_press() {
        mouseClick(row);

        compare(suite.presses, 1);
    }
}
