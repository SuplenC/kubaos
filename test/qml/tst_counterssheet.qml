import QtQuick
import QtTest
import KubaOS.Components as K
import qs.services
import "../../config/quickshell/modules/counters"

// The counters' sheet over the stand-in's four counters: a two-line row each, one height for all,
// then New counter; its keys count and its history opens in the list's place.
TestCase {
    id: suite

    name: "CountersSheet"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 800
    // design-allow(size): a test's fixture, not a design value.
    height: 900
    visible: true

    QtObject {
        id: frame

        readonly property real l: 0
        readonly property real t: 0
        readonly property real r: suite.width
        readonly property real b: suite.height
    }

    CountersSheet {
        id: sheet

        frame: frame
    }

    function find(item, test) {
        if (!item)
            return null;
        if (test(item))
            return item;
        for (const kid of item.children) {
            const hit = find(kid, test);
            if (hit)
                return hit;
        }
        return null;
    }

    function list() {
        return find(sheet, it => typeof it.itemAtIndex === "function" && it.model !== undefined);
    }

    function init() {
        sheet.shown = true;
        sheet.showList();
        tryCompare(sheet, "progress", 1);
    }

    function cleanup() {
        sheet.shown = false;
        tryCompare(sheet, "progress", 0);
    }

    function test_every_counter_has_a_row_of_one_height() {
        const rows = list();

        tryCompare(rows, "count", Counters.list.length);
        tryCompare(rows, "height", Counters.list.length * K.Tokens.size.row_two_line + (Counters.list.length - 1) * K.Tokens.space.s4);
        for (let i = 0; i < rows.count; i++)
            compare(rows.itemAtIndex(i).height, K.Tokens.size.row_two_line);
    }

    // A MouseArea always sets a cursor of its own: one over a row but a pressable's would take the
    // hand from the − and + under it.
    function test_every_mouse_area_in_a_row_shows_the_hand() {
        const rows = list();
        tryCompare(rows, "count", Counters.list.length);
        const areas = [];
        const collect = item => {
            if (item.cursorShape !== undefined && item.acceptedButtons !== undefined)
                areas.push(item);
            for (const kid of item.children)
                collect(kid);
        };

        collect(rows.itemAtIndex(0));

        verify(areas.length > 0);
        for (const area of areas)
            compare(area.cursorShape, Qt.PointingHandCursor);
    }

    function test_the_rows_take_the_sheet_s_width_less_its_padding() {
        tryCompare(list(), "width", K.Tokens.size.panel - K.Tokens.space.s8 * 2);
    }

    function test_new_counter_sits_under_the_rows() {
        const rows = list();
        const add = find(sheet, it => it.title === "New counter");

        tryVerify(() => rows.height > 0);
        verify(add.mapToItem(sheet, 0, 0).y >= rows.mapToItem(sheet, 0, rows.height).y);
    }

    function test_the_history_takes_the_list_s_place_and_esc_goes_back() {
        sheet.showHistory(Counters.list[0]);

        compare(sheet.page, "history");
        keyClick(Qt.Key_Escape);
        compare(sheet.page, "list");
    }
}
