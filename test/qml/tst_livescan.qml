import QtQuick
import QtTest
import KubaOS.Components as K
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
        K.Motion.reduced = false;
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

    // Nearby with the line over its drawer of unnamed devices, as the Bluetooth tab lists it.
    function withDrawer(count) {
        return [...radio(["known-a"], ["near-a", "near-b"]), line("disclosure:unnamed", "disclosure"), ...names("unnamed-", count).map(key => Object.assign(line(key, "row"), {
                    folded: true
                }))];
    }

    // The drawer's rows through an open or a close, sampled each frame: where each shows, how
    // much of its room it has and how much it shows, and where the line stood.
    function toggle(open) {
        const drawer = names("unnamed-", 4);
        const samples = [];
        const count = list.count;
        list.unfolded = open;
        for (let n = 0; n < 40; n++) {
            // Laid out as the next frame draws it.
            list.forceLayout();
            samples.push({
                rows: drawer.map(key => ({
                            at: shownAt(key),
                            room: delegateFor(key)?.height ?? -1,
                            opacity: delegateFor(key)?.opacity ?? -1
                        })),
                line: shownAt("disclosure:unnamed"),
                count: list.count
            });
            wait(16);
        }
        return {
            samples: samples,
            count: count
        };
    }

    // How far below its line the drawer's edge was.
    function edge(sample) {
        return sample.rows.reduce((total, row) => total + row.room, 0);
    }

    // What every frame of an open or a close keeps: no row leaves the model, the line stays put,
    // the rows the edge uncovers sit where the open drawer has them, each after the one before,
    // and every row fades as one, by the same amount.
    function verifyDrawer(run, line, room) {
        for (const sample of run.samples) {
            compare(sample.count, run.count, "no row left the model");
            compare(sample.line, line, "the line didn't move");
            sample.rows.forEach((row, n) => {
                if (row.room > 0)
                    compare(row.at, line + room * (n + 1), `row ${n} is where it is in the open drawer, not moved`);
                compare(row.opacity, sample.rows[0].opacity, `the rows fade as one: ${sample.rows.map(row => row.opacity)}`);
            });
        }
    }

    function test_hide_slides_the_drawer_up_to_a_line_that_stays_put_with_its_rows_fading_as_one() {
        list.rows = withDrawer(4);
        settle();
        const line = shownAt("disclosure:unnamed");
        const room = delegateFor("near-a").height;

        const run = toggle(false);

        verifyDrawer(run, line, room);
        const edges = run.samples.map(edge);
        verify(edges.some(at => at > 0 && at < room * 4 && at % room !== 0), "the edge slides over frames, through a row");
        verify(edges.every((at, n) => n === 0 || at <= edges[n - 1]), `the edge only rises: ${edges}`);
        verify(run.samples.some(sample => sample.rows[0].opacity > 0 && sample.rows[0].opacity < 1), "the rows fade over frames");
        compare(edges[edges.length - 1], 0, "closed at the line");
        compare(run.samples[run.samples.length - 1].rows[0].opacity, 0, "faded out");
    }

    function test_show_slides_the_drawer_down_from_a_line_that_stays_put_with_its_rows_fading_in_as_one() {
        list.unfolded = false;
        list.rows = withDrawer(4);
        settle();
        const line = shownAt("disclosure:unnamed");
        const room = delegateFor("near-a").height;

        const run = toggle(true);

        verifyDrawer(run, line, room);
        const edges = run.samples.map(edge);
        verify(edges.some(at => at > 0 && at < room * 4 && at % room !== 0), "the edge slides over frames, through a row");
        verify(edges.every((at, n) => n === 0 || at >= edges[n - 1]), `the edge only falls: ${edges}`);
        verify(run.samples.some(sample => edge(sample) > 0 && sample.rows[0].opacity === 0), "the rows start to fade in after the edge starts");
        verify(run.samples.some(sample => sample.rows[0].opacity > 0 && sample.rows[0].opacity < 1), "the rows fade over frames");
        compare(edges[edges.length - 1], room * 4, "open under its last row");
        compare(run.samples[run.samples.length - 1].rows[0].opacity, 1, "faded in");
    }

    function test_with_reduced_motion_the_drawer_opens_and_closes_at_once() {
        list.rows = withDrawer(4);
        settle();
        K.Motion.reduced = true;

        list.unfolded = false;
        list.forceLayout();
        const closed = delegateFor("unnamed-00").height;
        list.unfolded = true;
        list.forceLayout();
        const open = delegateFor("unnamed-00").height;
        K.Motion.reduced = false;

        compare(closed, 0, "closed at once");
        compare(open, delegateFor("near-a").height, "open at once");
    }

    function test_hide_with_the_list_scrolled_into_the_drawer_leaves_the_line_where_it_was() {
        list.rows = withDrawer(4);
        settle();
        list.positionViewAtEnd();
        settle();
        const line = shownAt("disclosure:unnamed");
        verify(line !== null && inView("disclosure:unnamed"), "the line shows");

        list.unfolded = false;
        for (let n = 0; n < 40; n++) {
            list.forceLayout();
            compare(shownAt("disclosure:unnamed"), line, "the line didn't move");
            wait(16);
        }
        list.unfolded = true;
    }

    function test_devices_coming_and_going_in_the_open_drawer_arrive_and_leave_as_rows_do() {
        list.rows = withDrawer(2);
        settle();

        list.rows = [...withDrawer(2), Object.assign(line("unnamed-new", "row"), {
                folded: true
            })];
        wait(48);

        const arriving = delegateFor("unnamed-new");
        verify(arriving.grow > 0 && arriving.grow < 1, "a new device opens its room");
        settle();
        compare(delegateFor("unnamed-new").height, delegateFor("near-a").height, "and takes it all");

        list.rows = withDrawer(2);
        wait(48);

        const leaving = list.contentItem.children.find(item => item.key === "unnamed-new" && item.leaving);
        verify(leaving && leaving.height > 0 && leaving.height < delegateFor("near-a").height, "a device that goes gives its room back");
    }

    function test_the_keyboard_skips_the_closed_drawer_and_leaves_it_as_it_closes() {
        list.rows = withDrawer(2);
        settle();
        list.jump(list.count - 1);

        list.unfolded = false;

        compare(list.model.get(list.currentIndex).key, "disclosure:unnamed", "onto the drawer's line");
        list.step(1);
        compare(list.model.get(list.currentIndex).key, "known-a", "round past the drawer");
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
