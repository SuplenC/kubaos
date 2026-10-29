import QtQuick
import QtTest
import KubaOS.Components as K
import "../../config/quickshell/modules/notifications"

// A stack of popups under the pointer: pointing at one keeps every card on screen with its time
// stopped, crossing the gap between two keeps them, a new one waits, and leaving lets each go on
// with the time it had left. The stack is wired as `ToastList` wires it: the hover on the whole
// list, each card's countdown held while the stack holds.
TestCase {
    id: suite

    name: "PopupHold"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    readonly property var first: ({
            summary: "Proxy on"
        })
    readonly property var second: ({
            summary: "Backup finished"
        })
    readonly property var third: ({
            summary: "Update ready"
        })
    readonly property var fourth: ({
            summary: "Battery low"
        })

    PopupHold {
        id: hold

        pointed: stackHover.hovered
    }

    Item {
        id: stack

        // design-allow(size): a test's fixture, not a design value.
        width: 320
        height: column.height

        HoverHandler {
            id: stackHover
        }

        Column {
            id: column

            width: stack.width
            spacing: K.Tokens.space.s8

            Repeater {
                id: cards

                model: hold.shown

                delegate: CardFace {
                    id: card

                    required property var modelData
                    readonly property alias countdown: countdown

                    width: column.width
                    summary: modelData.summary
                    popup: true
                    remaining: countdown.remaining

                    Countdown {
                        id: countdown

                        running: true
                        held: card.pointed || hold.holding
                        // design-allow(duration): a test's fixture, not a design value.
                        duration: 400
                        // The repeater hands its delegates a copy of each entry, so it goes by its summary.
                        onFinished: hold.notifications = hold.notifications.filter(n => n.summary !== card.modelData.summary)
                    }
                }
            }
        }
    }

    function init() {
        mouseMove(suite, suite.width - 1, suite.height - 1);
        hold.notifications = [suite.first, suite.second, suite.third];
        // The column places its cards on the next polish.
        tryVerify(() => cards.count === 3 && card(2).y > card(1).y && card(1).y > 0, 1000, "the cards are laid out");
    }

    function cleanup() {
        mouseMove(suite, suite.width - 1, suite.height - 1);
        hold.notifications = [];
    }

    function card(index) {
        return cards.itemAt(index);
    }

    function times() {
        const out = [];
        for (let i = 0; i < cards.count; i++)
            out.push(card(i).countdown.remaining);
        return out;
    }

    function pointAt(index) {
        const c = card(index);
        mouseMove(c, c.width / 2, c.height / 2);
    }

    function test_pointing_at_one_popup_keeps_every_one_with_its_time_stopped() {
        pointAt(0);
        verify(hold.holding);
        const stopped = suite.times();

        wait(600);

        compare(cards.count, 3, "none left while the pointer was on the stack");
        compare(suite.times(), stopped);
        for (let i = 0; i < cards.count; i++)
            compare(card(i).opacity, 1, `card ${i} stays shown`);
    }

    function test_crossing_the_gap_between_two_popups_keeps_them() {
        pointAt(0);
        const stopped = suite.times();

        mouseMove(stack, stack.width / 2, card(0).height + K.Tokens.space.s8 / 2);
        verify(hold.holding, "the gap is the stack's");
        wait(100);
        pointAt(1);

        compare(suite.times(), stopped);
    }

    function test_a_new_popup_waits_while_the_pointer_is_on_the_stack() {
        pointAt(1);

        hold.notifications = [suite.fourth, ...hold.notifications];

        compare(cards.count, 3);
        compare(card(0).summary, suite.first.summary);

        mouseMove(suite, suite.width - 1, suite.height - 1);

        compare(card(0).summary, suite.fourth.summary);
    }

    function test_leaving_lets_each_go_on_with_the_time_it_had_left() {
        wait(100);
        pointAt(2);
        const stopped = suite.times();
        verify(stopped.every(t => t > 0 && t < 1), String(stopped));

        mouseMove(suite, suite.width - 1, suite.height - 1);

        verify(!hold.holding);
        tryVerify(() => suite.times().every((t, i) => t < stopped[i]), 1000, "every clock goes on");
        // A plain array as the model makes the repeater build its cards again when one goes,
        // their clocks with them, so only the first to run out is the stack's own.
        tryVerify(() => cards.count < 3, 1000, "a card runs out and goes");
    }

    function test_a_popup_that_goes_while_held_still_goes() {
        pointAt(0);

        hold.notifications = hold.notifications.filter(n => n !== suite.second);

        compare(cards.count, 2);
        verify(hold.holding);
    }
}
