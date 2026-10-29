import QtQuick
import QtTest
import "../../config/quickshell/components"

// A live list while a scan fills it: it never scrolls by itself, so where the person left it
// stays put as rows arrive, leave and change places, and the Known rows at its top stay in view.
TestCase {
    id: suite

    name: "LiveScan"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 240

    function line(key, kind) {
        return {
            key: key,
            kind: kind,
            folded: false
        };
    }

    // Known, then Nearby, as the radio tabs list them.
    function radio(known, nearby) {
        const out = [];
        if (known.length > 0)
            out.push(line("label:known", "label"), ...known.map(key => line(key, "row")));
        out.push(line("label:nearby", "label"));
        if (nearby.length > 0)
            out.push(...nearby.map(key => line(key, "row")));
        else
            out.push(line("note:nearby", "note"));
        return out;
    }

    function names(prefix, count) {
        return Array.from({
            length: count
        }, (_, n) => `${prefix}${String(n).padStart(2, "0")}`);
    }

    function delegateFor(key) {
        for (const item of list.contentItem.children) {
            if (item.key === key && !item.leaving)
                return item;
        }
        return null;
    }

    // Where the row `key` shows in the list, or null while it has no delegate.
    function shownAt(key) {
        const item = delegateFor(key);
        return item ? item.mapToItem(list, 0, 0).y : null;
    }

    function inView(key) {
        const y = shownAt(key);
        return y !== null && y >= 0 && y + delegateFor(key).height <= list.height;
    }

    function settle() {
        // Every arrival and leave runs its course in under this.
        wait(700);
    }

    LiveList {
        id: list

        anchors.fill: parent

        delegate: LiveRow {
            required property string key
            required property string kind
            required property bool folded

            folds: folded
            // design-allow(size): a test's fixture, not a design value.
            rowHeight: 40
        }
    }

    function init() {
        list.rows = [];
        list.unfolded = true;
        list.currentIndex = -1;
        list.positionViewAtBeginning();
        settle();
    }

    function test_a_scan_finding_many_devices_leaves_the_known_ones_in_view() {
        list.rows = radio(["known-a", "known-b"], ["near-m"]);
        settle();
        // The selection on the first Nearby row, where the list puts it when its row goes from
        // under it.
        list.jump(3);
        const top = list.contentY;

        for (let n = 0; n < 30; n++) {
            list.rows = radio(["known-a", "known-b"], [...names("near-", n + 1), "near-m"]);
            wait(16);
        }
        settle();

        compare(list.contentY, top, "the list didn't move");
        verify(inView("known-a") && inView("known-b"), "the Known rows are in view");
    }

    function test_a_selected_device_moving_down_doesnt_take_the_list_with_it() {
        list.rows = radio(["known-a"], ["near-a", "near-b"]);
        settle();
        list.currentIndex = 3;
        const top = list.contentY;

        list.rows = radio(["known-a"], [...names("near-c", 12), "near-a", "near-b"]);
        settle();

        compare(list.contentY, top, "the list didn't follow the selected row");
        verify(inView("known-a"), "the Known row is in view");
    }

    function test_rows_arriving_above_where_the_person_scrolled_dont_move_what_they_see() {
        list.rows = radio(["known-a"], names("near-m", 20));
        settle();
        list.contentY = 600;
        settle();
        const watched = "near-m12";
        const before = shownAt(watched);
        verify(before !== null && inView(watched), "the watched row shows");

        list.rows = radio(["known-a"], [...names("near-a", 6), ...names("near-m", 20)]);
        settle();

        compare(shownAt(watched), before, "the row the person looked at is where it was");
    }

    function test_rows_leaving_above_where_the_person_scrolled_dont_move_what_they_see() {
        list.rows = radio(["known-a"], [...names("near-a", 6), ...names("near-m", 20)]);
        settle();
        list.contentY = 860;
        settle();
        const watched = "near-m12";
        const before = shownAt(watched);
        verify(before !== null && inView(watched), "the watched row shows");

        list.rows = radio(["known-a"], names("near-m", 20));
        settle();

        compare(shownAt(watched), before, "the row the person looked at is where it was");
    }

    // Nearby with the line before its unnamed block, as the Bluetooth tab lists it.
    function withBlock(count) {
        return [...radio(["known-a"], ["near-a", "near-b"]), line("action:unnamed", "action"), ...names("unnamed-", count).map(key => Object.assign(line(key, "row"), {
                    folded: true
                }))];
    }

    // The block's rows' heights through a fold, sampled each frame, and where the line stood.
    function fold(open) {
        const block = names("unnamed-", 4);
        const samples = [];
        const count = list.count;
        list.unfolded = open;
        for (let n = 0; n < 40; n++) {
            samples.push({
                heights: block.map(key => delegateFor(key)?.height ?? -1),
                line: shownAt("action:unnamed"),
                count: list.count
            });
            wait(16);
        }
        return {
            samples: samples,
            count: count
        };
    }

    function test_hide_folds_the_unnamed_block_away_in_one_motion_under_a_line_that_stays_put() {
        list.rows = withBlock(4);
        settle();
        const line = shownAt("action:unnamed");

        const run = fold(false);

        verify(run.samples.some(sample => sample.heights[0] > 0 && sample.heights[0] < delegateFor("near-a").height), "it folds over frames");
        for (const sample of run.samples) {
            compare(sample.count, run.count, "no row left the model");
            verify(sample.heights.every(height => height === sample.heights[0]), `the block folds as one: ${sample.heights}`);
            compare(sample.line, line, "the line didn't move");
        }
        compare(run.samples[run.samples.length - 1].heights[0], 0, "folded away");
    }

    function test_show_opens_the_unnamed_block_in_one_motion_under_a_line_that_stays_put() {
        list.unfolded = false;
        list.rows = withBlock(4);
        settle();
        const line = shownAt("action:unnamed");

        const run = fold(true);

        verify(run.samples.some(sample => sample.heights[0] > 0 && sample.heights[0] < delegateFor("near-a").height), "it opens over frames");
        for (const sample of run.samples) {
            verify(sample.heights.every(height => height === sample.heights[0]), `the block opens as one: ${sample.heights}`);
            compare(sample.line, line, "the line didn't move");
        }
        compare(run.samples[run.samples.length - 1].heights[0], delegateFor("near-a").height, "open");
        list.unfolded = true;
    }

    function test_the_keyboard_skips_the_folded_block_and_leaves_it_as_it_folds() {
        list.rows = withBlock(2);
        settle();
        list.jump(list.count - 1);

        list.unfolded = false;

        compare(list.model.get(list.currentIndex).key, "action:unnamed", "onto the line before the block");
        list.step(1);
        compare(list.model.get(list.currentIndex).key, "known-a", "round past the block");
        list.unfolded = true;
    }

    function test_the_keyboard_still_brings_its_row_into_view() {
        list.rows = radio(["known-a"], names("near-", 20));
        settle();
        list.currentIndex = 1;
        list.forceActiveFocus();

        for (let n = 0; n < 10; n++)
            keyClick(Qt.Key_Down);
        settle();

        verify(inView(list.model.get(list.currentIndex).key), "the selected row shows");
    }
}
