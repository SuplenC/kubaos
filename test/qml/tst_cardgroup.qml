import QtQuick
import QtTest
import KubaOS.Components as K
import "../../config/quickshell/modules/notifications"

// A notification group: at rest a stack with one edge under it for a pair and two for more, a
// preview after the pointer rests on it a beat (nothing before, and nothing moving in the list),
// at most three cards with "+N more" past them, a click opening it under a heading whose Show less
// tucks it back, and a click in an opened group unfolding a card rather than closing it.
TestCase {
    id: suite

    name: "CardGroup"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 900

    readonly property string long: "The nightly backup of the home folder finished, but three folders were skipped because they changed while they were being read; run it again tonight or open the log to see which ones and why."
    property var unfolded: null
    // The notifications whose main action ran, in order.
    property var ran: []

    function notes(n) {
        const out = [];
        for (let i = 0; i < n; i++)
            out.push({
                summary: `Backup ${n - i}`,
                action: i === 2 ? "" : "Open",
                body: i === 1 ? suite.long : "Everything was copied."
            });
        return out;
    }

    Item {
        id: listBox

        // design-allow(size): a test's fixture, not a design value.
        width: 320
        height: suite.height

        Column {
            id: column

            width: listBox.width
            spacing: K.Tokens.space.s8

            CardGroup {
                id: group

                width: column.width
                onOpenRequested: open => group.open = open

                delegate: CardFace {
                    required property int index
                    readonly property var note: group.items[index] ?? ({})

                    mainAction: note.action ?? ""
                    onMainPicked: suite.ran.push(note)

                    appName: "Backups"
                    summary: note.summary ?? ""
                    body: note.body ?? ""
                    count: (group.standing[index] ?? []).length
                    unfolded: suite.unfolded === note
                    onUnfoldRequested: open => suite.unfolded = open ? note : null
                }
            }

            // The next row of the list, which the preview must never move.
            Rectangle {
                id: below

                width: column.width
                height: K.Tokens.size.control_md
            }
        }
    }

    SignalSpy {
        id: opens

        target: group
        signalName: "openRequested"
    }

    function init() {
        K.Motion.reduced = true;
        mouseMove(suite, suite.width - 1, suite.height - 1);
        group.open = false;
        suite.unfolded = null;
        suite.ran = [];
        group.items = suite.notes(3);
        opens.clear();
        tryVerify(() => !group.live && group.fan === 0 && group.opening === 0);
    }

    function cleanup() {
        K.Motion.reduced = false;
        mouseMove(suite, suite.width - 1, suite.height - 1);
    }

    function pointAtHead() {
        mouseMove(group.head, group.head.width / 2, K.Tokens.space.s12);
    }

    function cardsShown() {
        return group.cards.filter(c => c && c.visible && c.opacity === 1).length;
    }

    function test_a_pair_has_one_edge_and_three_or_more_two() {
        group.items = suite.notes(2);
        compare(group.edges, 1);
        compare(group.implicitHeight, group.headHeight + K.Tokens.space.s8);

        group.items = suite.notes(5);
        compare(group.edges, 2);
        compare(group.implicitHeight, group.headHeight + K.Tokens.space.s8 * 2);
        const [head, first, second] = group.boxes;
        verify(first.x > head.x && second.x > first.x, "each edge narrower than the one above");
        verify(first.w < head.w && second.w < first.w);
        compare(first.y + first.h, head.y + head.h + K.Tokens.space.s8);
        compare(second.y + second.h, first.y + first.h + K.Tokens.space.s8);
    }

    function test_one_notification_is_the_card_alone() {
        group.items = suite.notes(1);

        compare(group.edges, 0);
        compare(group.implicitHeight, group.headHeight);
        pointAtHead();
        wait(K.Tokens.duration.medium * 2);
        compare(group.fan, 0);
    }

    function test_at_rest_only_the_newest_card_is_made() {
        group.items = suite.notes(5);

        compare(group.made, 1);
        compare(group.head.count, 5, "it carries the group's count");
    }

    function test_the_preview_waits_for_the_pointer_to_rest() {
        pointAtHead();

        wait(K.Tokens.duration.medium / 2);
        compare(group.fan, 0, "nothing before the beat");
        tryCompare(group, "fan", 1);
    }

    function test_leaving_before_the_beat_shows_nothing() {
        pointAtHead();
        wait(K.Tokens.duration.medium / 2);

        mouseMove(suite, suite.width - 1, suite.height - 1);
        wait(K.Tokens.duration.medium);

        compare(group.fan, 0);
        verify(!group.armed);
    }

    function test_leaving_the_preview_tucks_it_back() {
        pointAtHead();
        tryCompare(group, "fan", 1);

        mouseMove(suite, suite.width - 1, suite.height - 1);

        tryCompare(group, "fan", 0);
    }

    function test_the_preview_never_moves_the_list() {
        group.items = suite.notes(5);
        waitForItemPolished(column);
        const height = group.implicitHeight;
        const at = below.y;

        pointAtHead();
        tryCompare(group, "fan", 1);

        compare(group.implicitHeight, height);
        compare(below.y, at);
        verify(group.extent > height, "the cards float past the group's own room");
    }

    function test_the_preview_spreads_its_cards_the_list_s_gap_apart() {
        pointAtHead();
        tryCompare(group, "fan", 1);

        const [head, first, second] = group.boxes;
        compare(first.y, head.y + head.h + K.Tokens.space.s8);
        compare(second.y, first.y + first.h + K.Tokens.space.s8);
        compare(first.x, 0);
        compare(first.w, group.width);
    }

    function test_the_preview_shows_at_most_three_and_how_many_more() {
        group.items = suite.notes(5);

        pointAtHead();
        tryCompare(group, "fan", 1);

        compare(suite.cardsShown(), 3);
        const more = findChild(group, "more");
        verify(more.visible);
        compare(more.text, "+2 more · click to open");
    }

    function test_a_preview_of_three_says_no_more() {
        pointAtHead();
        tryCompare(group, "fan", 1);

        compare(suite.cardsShown(), 3);
        verify(!findChild(group, "more").visible);
    }

    function test_with_motion_the_preview_spreads_over_time() {
        K.Motion.reduced = false;
        pointAtHead();

        tryVerify(() => group.fan > 0 && group.fan < 1, 2000, "it spreads, not jumps");
        tryCompare(group, "fan", 1);
    }

    function test_with_reduce_motion_the_preview_spreads_at_once() {
        pointAtHead();

        tryVerify(() => group.armed);

        compare(group.fan, 1);
        compare(suite.cardsShown(), 3);
    }

    function test_a_click_opens_it_under_a_heading() {
        group.items = suite.notes(5);

        mouseClick(group.head, group.head.width / 2, K.Tokens.space.s12);

        compare(opens.count, 1);
        verify(group.isOpen);
        compare(suite.cardsShown(), 5, "every card takes its place");
        const showLess = findChild(group, "showLess");
        verify(showLess.visible);
        for (let k = 1; k < 5; k++)
            compare(group.boxes[k].y, group.boxes[k - 1].y + group.boxes[k - 1].h + K.Tokens.space.s8);
        compare(group.boxes[0].y, K.Tokens.size.control_sm + K.Tokens.space.s8, "the heading is on top");
        waitForItemPolished(column);
        compare(below.y, group.implicitHeight + K.Tokens.space.s8, "what is below moves down");
        compare(group.head.count, 1, "each card is on its own");
    }

    function test_a_click_on_the_preview_opens_it() {
        group.items = suite.notes(5);
        pointAtHead();
        tryCompare(group, "fan", 1);
        const first = group.boxes[1];

        mouseClick(group, group.width / 2, first.y + first.h / 2);

        verify(group.isOpen);
    }

    function test_opening_moves_nothing_above_it() {
        const top = group.mapToItem(suite, 0, 0).y;

        group.open = true;

        compare(group.mapToItem(suite, 0, 0).y, top);
    }

    function test_show_less_tucks_it_back() {
        group.open = true;
        mouseClick(findChild(group, "showLess"));

        verify(!group.open);
        tryCompare(group, "opening", 0);
        compare(group.implicitHeight, group.headHeight + K.Tokens.space.s8 * 2);
    }

    function test_after_show_less_the_pointer_must_leave_before_it_previews_again() {
        group.open = true;
        mouseClick(findChild(group, "showLess"));

        wait(K.Tokens.duration.medium * 2);

        compare(group.fan, 0);
    }

    function test_a_second_click_on_show_less_never_dismisses_the_group() {
        const dismisses = [];
        group.open = true;
        waitForRendering(group);
        const showLess = findChild(group, "showLess");
        const at = showLess.mapToItem(group, showLess.width - K.Tokens.space.s4, showLess.height / 2);
        group.head.dismissRequested.connect(() => dismisses.push(true));

        mouseMove(group, at.x, at.y);
        mouseClick(group, at.x, at.y);
        tryCompare(group, "opening", 0);
        mouseClick(group, at.x, at.y);

        compare(dismisses.length, 0);
        verify(!group.open);
    }

    function test_a_click_in_an_opened_group_unfolds_a_card_rather_than_closing_it() {
        group.open = true;
        const card = group.cards[1];
        verify(card.foldable, "the fixture's second card is cut off");

        waitForRendering(group);
        mouseClick(card, K.Tokens.space.s12 * 2, K.Tokens.space.s8 + K.Tokens.size.control_sm + K.Tokens.space.s12);

        verify(group.open);
        compare(suite.unfolded, group.items[1]);
    }

    function test_shift_enter_on_a_stack_runs_its_newest_s_main_action() {
        verify(group.runMain());

        compare(suite.ran, [group.items[0]]);
        verify(!group.isOpen, "it runs the action, it doesn't open");
    }

    function test_shift_enter_in_an_opened_group_runs_the_current_card_s_main_action() {
        group.open = true;
        group.current = 1;

        verify(group.runMain());

        compare(suite.ran, [group.items[1]]);
        verify(group.isOpen);
    }

    function test_shift_enter_on_a_card_without_a_main_action_does_nothing() {
        group.open = true;
        group.current = 2;

        verify(!group.runMain());

        compare(suite.ran.length, 0);
    }

    function test_press_opens_a_stack_and_then_unfolds_the_current_card() {
        group.press();
        verify(group.isOpen);

        group.current = 1;
        group.press();

        verify(group.isOpen);
        compare(suite.unfolded, group.items[1]);
    }

    function test_the_keyboard_selects_the_newest_at_rest_and_the_current_one_open() {
        group.selected = true;
        verify(group.head.selected);

        group.open = true;
        group.current = 2;

        verify(!group.head.selected);
        verify(group.cards[2].selected);
        group.selected = false;
    }

    function test_with_motion_opening_moves_over_time_and_reduced_at_once() {
        K.Motion.reduced = false;
        const rest = group.implicitHeight;

        group.open = true;

        compare(group.implicitHeight, rest, "the first frame is still at rest");
        tryVerify(() => group.implicitHeight > rest);
        tryCompare(group, "opening", 1);
        K.Motion.reduced = true;
        group.open = false;
        compare(group.implicitHeight, rest);
    }

    function test_a_group_down_to_one_asks_to_close() {
        group.items = suite.notes(2);
        group.open = true;

        group.items = suite.notes(1);

        verify(!group.open);
    }
}
