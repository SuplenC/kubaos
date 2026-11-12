import QtQuick
import QtTest
import KubaOS.Components as K

// A prompt takes only a value it can take: Enter on one it can't turns its Field invalid with the
// reason in its Note and keeps it open; Esc cancels.
TestCase {
    id: suite

    name: "Prompt"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    // design-allow(size): a test's fixture, not a design value.
    height: 300
    visible: true

    K.Text {
        id: anchor

        x: K.Tokens.space.s48
        y: K.Tokens.space.s24
        text: "Coffee"
    }

    K.Prompt {
        id: prompt

        floating: true
        label: "Rename"
        check: typed => typed.trim() === "" ? "A name can't be empty." : ""
    }

    SignalSpy {
        id: saved

        target: prompt
        signalName: "saved"
    }

    SignalSpy {
        id: cancelled

        target: prompt
        signalName: "cancelled"
    }

    function init() {
        saved.clear();
        cancelled.clear();
        prompt.text = "Coffee";
        prompt.show(anchor, "start");
        waitForRendering(prompt);
    }

    function test_it_opens_under_its_text_lined_up_with_its_start() {
        const card = anchor.mapToItem(null, 0, anchor.height);

        verify(prompt.open);
        compare(prompt.popover.y, card.y + K.Tokens.space.s8);
        compare(prompt.popover.x, card.x - K.Tokens.space.s12);
    }

    function test_enter_takes_a_value_it_can_take_and_closes() {
        keyClick(Qt.Key_T);
        keyClick(Qt.Key_Return);

        compare(saved.count, 1);
        compare(saved.signalArguments[0][0], "t");
        verify(!prompt.open);
    }

    function test_enter_on_a_value_it_cannot_take_keeps_it_open_and_says_why() {
        keyClick(Qt.Key_Backspace);
        keyClick(Qt.Key_Return);

        compare(saved.count, 0);
        verify(prompt.open);
        verify(prompt.invalid);
    }

    function test_a_value_it_can_take_is_no_longer_invalid() {
        keyClick(Qt.Key_Backspace);
        keyClick(Qt.Key_Return);

        keyClick(Qt.Key_T);

        verify(!prompt.invalid);
    }

    function test_esc_cancels() {
        keyClick(Qt.Key_Escape);

        compare(cancelled.count, 1);
        compare(saved.count, 0);
        verify(!prompt.open);
    }
}
