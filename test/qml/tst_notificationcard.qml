import QtQuick
import QtTest
import KubaOS.Components as K
import "../../config/quickshell/modules/notifications"

// A notification card's face: cut-off text unfolds on a click, Enter or Space and folds again on
// another or Esc, the chevron shows only when there is more to see, and a click never runs an
// action: the main one is a filled button, the others text ones after it, then Mark read.
TestCase {
    id: suite

    name: "NotificationCard"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    readonly property string long: "The nightly backup of the home folder finished, but three folders were skipped because they changed while they were being read; run it again tonight or open the log to see which ones and why."

    CardFace {
        id: card

        // design-allow(size): a test's fixture, not a design value.
        width: 320
        appName: "Backups"
        summary: "Backup finished"
        onUnfoldRequested: open => card.unfolded = open
    }

    SignalSpy {
        id: unfolds

        target: card
        signalName: "unfoldRequested"
    }

    SignalSpy {
        id: expands

        target: card
        signalName: "expandRequested"
    }

    SignalSpy {
        id: mains

        target: card
        signalName: "mainPicked"
    }

    SignalSpy {
        id: others

        target: card
        signalName: "otherPicked"
    }

    SignalSpy {
        id: marks

        target: card
        signalName: "markReadPicked"
    }

    function init() {
        K.Motion.reduced = true;
        card.body = "";
        card.unfolded = false;
        card.count = 1;
        card.opens = false;
        card.unread = false;
        card.critical = false;
        card.mainAction = "";
        card.otherActions = [];
        for (const spy of [unfolds, expands, mains, others, marks])
            spy.clear();
    }

    function cleanup() {
        K.Motion.reduced = false;
    }

    // A point on the card's text, clear of its buttons and of the band's.
    function clickText() {
        mouseClick(card, K.Tokens.space.s12 * 2, K.Tokens.space.s8 + K.Tokens.size.control_sm + K.Tokens.space.s8 + K.Tokens.space.s4);
    }

    function button(name) {
        return findChild(card, name);
    }

    function test_text_that_fits_has_no_chevron_and_takes_no_click() {
        card.body = "Everything was copied.";

        clickText();

        verify(!card.foldable);
        verify(!findChild(card, "chevron").visible);
        compare(unfolds.count, 0);
    }

    function test_cut_off_text_has_a_chevron() {
        card.body = suite.long;

        verify(card.foldable);
        verify(findChild(card, "chevron").visible);
    }

    function test_a_click_unfolds_cut_off_text_to_all_of_it_and_another_folds_it() {
        card.body = suite.long;
        const folded = card.implicitHeight;

        clickText();

        compare(unfolds.count, 1);
        verify(card.unfolded);
        verify(card.implicitHeight > folded, `${card.implicitHeight} over ${folded}`);

        clickText();

        verify(!card.unfolded);
        compare(card.implicitHeight, folded);
    }

    function test_unfolded_the_whole_body_shows() {
        card.body = suite.long;
        const body = findChild(card, "body");
        verify(body.laid.length > card.foldedLines, "the fixture is longer than the fold");

        card.unfolded = true;

        compare(body.lines.length, body.laid.length);
        verify(!body.cut);
    }

    function test_enter_and_space_unfold_and_esc_folds() {
        card.body = suite.long;
        card.forceActiveFocus();

        keyClick(Qt.Key_Return);
        verify(card.unfolded);
        keyClick(Qt.Key_Escape);
        verify(!card.unfolded);
        keyClick(Qt.Key_Space);
        verify(card.unfolded);
        keyClick(Qt.Key_Space);
        verify(!card.unfolded);
    }

    function test_esc_on_a_folded_card_is_left_for_what_holds_it() {
        card.body = suite.long;

        verify(!card.foldBack());
        compare(unfolds.count, 0);
    }

    function test_with_motion_it_unfolds_over_time_and_reduced_at_once() {
        card.body = suite.long;
        const folded = card.implicitHeight;
        K.Motion.reduced = false;

        card.unfolded = true;

        compare(card.implicitHeight, folded, "the first frame is still folded");
        tryVerify(() => card.implicitHeight > folded);
        card.unfolded = false;
        tryCompare(card, "implicitHeight", folded);
    }

    function test_a_stack_that_opens_asks_to_open_rather_than_unfolding() {
        card.body = suite.long;
        card.count = 3;
        card.opens = true;

        clickText();

        compare(expands.count, 1);
        compare(unfolds.count, 0);
    }

    function test_the_main_action_is_filled_and_the_others_are_text_after_it_then_mark_read() {
        card.mainAction = "Open";
        card.otherActions = ["Retry", "Mute"];
        card.unread = true;

        const main = button("main");
        const retry = button("other0");
        const mute = button("other1");
        const mark = button("markRead");

        compare(main.kind, "Filled");
        compare(main.text, "Open");
        compare(retry.kind, "Text");
        compare(mute.text, "Mute");
        compare(mark.kind, "Text");
        tryVerify(() => main.x < retry.x && retry.x < mute.x && mute.x < mark.x, 1000, "in that order");
    }

    function test_a_critical_one_s_main_action_is_a_danger_button() {
        card.mainAction = "Plug in";
        card.critical = true;

        compare(button("main").kind, "Danger");
    }

    function test_without_actions_only_mark_read_shows() {
        card.unread = true;

        verify(!button("main").visible);
        verify(button("markRead").visible);
    }

    function test_a_sender_s_own_mark_read_stands_in_for_the_card_s() {
        card.mainAction = "Reply";
        card.otherActions = ["Mark as read"];
        card.unread = true;

        verify(!button("markRead").visible);
        verify(button("other0").visible);
    }

    function test_read_and_without_actions_it_has_no_buttons() {
        const withButtons = (() => {
                card.unread = true;
                return card.implicitHeight;
            })();

        card.unread = false;

        verify(!button("markRead").visible);
        compare(card.implicitHeight, withButtons - K.Tokens.space.s12 - K.Tokens.size.control_sm);
    }

    function test_a_click_on_the_card_never_runs_an_action() {
        card.body = suite.long;
        card.mainAction = "Open";
        card.otherActions = ["Retry"];

        clickText();
        keyClick(Qt.Key_Return);

        compare(mains.count, 0);
        compare(others.count, 0);
    }

    function test_the_buttons_run_their_actions() {
        card.mainAction = "Open";
        card.otherActions = ["Retry"];
        card.unread = true;
        tryVerify(() => button("markRead").x > button("other0").x && button("other0").x > button("main").x);

        mouseClick(button("main"));
        mouseClick(button("other0"));
        mouseClick(button("markRead"));

        compare(mains.count, 1);
        compare(others.count, 1);
        compare(others.signalArguments[0][0], 0);
        compare(marks.count, 1);
        compare(unfolds.count, 0);
    }
}
