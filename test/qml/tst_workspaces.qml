import QtQuick
import QtTest
import Quickshell
import Quickshell.Hyprland
import "../../config/quickshell/modules/bar"
import "../../config/quickshell/services/occupied.js" as Occupied

// The bar's workspaces over the stand-in's: 1, 2 (shown) and 3 in the screen's own group, 11 and
// 12 in the group named Desk. Entries that leave together all shrink out and go, a named group's
// name follows its first workspace, and the indicator shows only on a workspace the pill draws.
TestCase {
    id: suite

    name: "Workspaces"

    // The stand-in's workspaces as it starts, put back after each test.
    property var start: null

    ShellScreen {
        id: headless
    }

    Workspaces {
        id: pill

        screen: headless
    }

    // The stand-in with workspaces `ids`, the second of them shown.
    function keep(ids) {
        Hyprland.spaces = ids.map(n => ({
                    id: n,
                    name: `${n}`,
                    monitor: Hyprland.monitor,
                    active: n === 2,
                    focused: n === 2,
                    urgent: false,
                    hasFullscreen: false
                }));
    }

    function find(item, test) {
        if (test(item))
            return item;
        for (const kid of item.children) {
            const hit = find(kid, test);
            if (hit)
                return hit;
        }
        return null;
    }

    // The entries the pill draws, in order, leaving ones included.
    function slots() {
        const repeater = find(pill, it => typeof it.itemAt === "function" && it.model !== undefined);
        const items = [];
        for (let i = 0; i < repeater.count; i++)
            items.push(repeater.itemAt(i));
        return items;
    }

    // The sliding indicator of the shown workspace.
    function indicator() {
        return find(pill, it => it.lastId !== undefined && typeof it.place === "function");
    }

    function keys() {
        return slots().map(slot => slot.key);
    }

    function initTestCase() {
        suite.start = Hyprland.spaces;
    }

    function cleanup() {
        Hyprland.spaces = suite.start;
        tryVerify(() => slots().length === 7 && slots().every(slot => !slot.leaving));
        tryVerify(() => indicator().opacity === 1 && indicator().x === pill.cell);
    }

    function test_entries_leaving_together_all_go() {
        compare(keys(), ["1", "2", "3", "gap1", "label1", "11", "12"]);

        keep([1, 2]);

        tryVerify(() => keys().join(",") === "1,2", 5000, `still drawn: ${keys()}`);
    }

    function test_a_named_group_goes_to_its_first_workspace_after_the_old_one_is_gone() {
        keep([1, 2, 3, 12]);

        const label = slots().find(slot => slot.kind === "label");
        compare(label.workspace, 12);
    }

    function test_the_indicator_hides_while_the_shown_workspace_is_unknown() {
        compare(indicator().opacity, 1);

        keep([1]);

        tryVerify(() => indicator().opacity === 0, 5000, `opacity ${indicator().opacity}`);
    }

    function test_the_indicator_appears_on_its_cell_without_sliding_from_where_it_was() {
        const bar = indicator();
        keep([1]);
        tryVerify(() => bar.opacity === 0);

        keep([3, 1, 2]);
        wait(0);

        compare([bar.x, bar.width], [0, pill.cell]);
        wait(pill.glide);
        compare([bar.x, bar.width, bar.opacity], [0, pill.cell, 1]);
    }

    function test_a_pinned_window_holds_no_workspace() {
        const windows = [
            {
                workspace: {
                    id: 3
                },
                lastIpcObject: {
                    pinned: false
                }
            },
            {
                workspace: {
                    id: 5
                },
                lastIpcObject: {
                    pinned: true
                }
            },
            {
                workspace: {
                    id: 1
                },
                lastIpcObject: {}
            },
            {
                workspace: null,
                lastIpcObject: {}
            }
        ];

        compare(Occupied.ids(windows), ",1,3,");
    }
}
