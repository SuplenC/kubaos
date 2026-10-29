import QtQuick
import QtTest
import "../../config/quickshell/components"

// A live list's rows as a scan changes them: a leaving row stays, unpressable, until it has
// faded and given its room back; an arriving one opens its room.
TestCase {
    id: suite

    name: "LiveList"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 400

    function rows(keys) {
        return keys.map(key => ({
                    key: key,
                    kind: "row"
                }));
    }

    function delegateFor(key) {
        for (const item of list.contentItem.children) {
            if (item.key === key)
                return item;
        }
        return null;
    }

    LiveList {
        id: list

        anchors.fill: parent
        rows: suite.rows(["a", "b", "c"])

        delegate: LiveRow {
            required property string key
            required property string kind
            // design-allow(size): a test's fixture, not a design value.
            rowHeight: 40
        }
    }

    function init() {
        list.rows = suite.rows(["a", "b", "c"]);
        tryVerify(() => list.contentItem.children.filter(item => item.key !== undefined && item.visible).length === 3, 1000);
        wait(600);
    }

    function test_a_leaving_row_stays_unpressable_until_it_has_gone() {
        list.rows = suite.rows(["a", "c"]);

        tryVerify(() => delegateFor("b")?.leaving ?? false, 200, "the row is still in the view, leaving");
        const leaving = delegateFor("b");
        verify(!leaving.enabled, "a press can't land on it");
        tryVerify(() => delegateFor("b") === null || !delegateFor("b").visible, 1000);
    }

    function test_an_arriving_row_opens_its_room() {
        list.rows = suite.rows(["a", "d", "b", "c"]);

        tryVerify(() => (delegateFor("d")?.grow ?? 1) < 1, 200, "the row arrives with no room yet");
        const arriving = delegateFor("d");
        tryCompare(arriving, "grow", 1, 1000);
        tryCompare(arriving, "opacity", 1, 1500);
    }

    function test_the_keyboard_skips_what_isnt_a_row() {
        list.rows = [
            {
                key: "label",
                kind: "label"
            },
            ...suite.rows(["a", "b"])
        ];
        list.currentIndex = 1;

        list.step(1);
        list.step(1);

        compare(list.currentIndex, 1, "from the last row past the label, round to the first row");
    }

    function test_the_keyboard_reaches_an_action_line() {
        list.rows = [
            ...suite.rows(["a"]),
            {
                key: "note",
                kind: "note"
            },
            {
                key: "show",
                kind: "action"
            }
        ];
        list.currentIndex = 0;

        list.step(1);

        compare(list.currentIndex, 2, "past the note, onto the action line");
    }
}
