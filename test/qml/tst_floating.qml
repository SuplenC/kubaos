import QtQuick
import QtTest
import KubaOS.Components as K

// What floats over the window for a control: a hint shows only while its control shows whole in
// the scroll it is in, and a select's menu goes when the select is hidden with what holds it.
TestCase {
    id: suite

    name: "Floating"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 400

    Flickable {
        id: scroll

        // design-allow(size): a test's fixture, not a design value.
        width: 200
        // design-allow(size): a test's fixture, not a design value.
        height: 120
        contentHeight: content.height
        clip: true

        Item {
            id: content

            width: scroll.width
            // design-allow(size): a test's fixture, not a design value.
            height: 400

            K.IconButton {
                id: button

                // design-allow(size): a test's fixture, not a design value.
                y: 40
                icon: "add"
                label: "Add"
                kind: "Text"
                size: "Sm"
            }
        }
    }

    Item {
        id: holder

        // design-allow(size): a test's fixture, not a design value.
        x: 220
        // design-allow(size): a test's fixture, not a design value.
        width: 160
        // design-allow(size): a test's fixture, not a design value.
        height: 200

        K.Select {
            id: select

            width: holder.width
            options: ["None", "Poland", "Japan"]
            currentIndex: 0
        }
    }

    function hintOf(control) {
        return Array.from(control.children).find(kid => kid.inView !== undefined && kid.text === control.label);
    }

    // The bubble, which floats on the window's content item.
    function bubbleOf(hint) {
        return Array.from(hint.host.children).find(kid => kid.overlay === true && kid.shift !== undefined && kid.width === hint.width && kid.height === hint.height);
    }

    function init() {
        scroll.contentY = 0;
        button.focused = false;
        holder.visible = true;
        select.dismiss();
        wait(K.Tokens.duration.short * 2);
    }

    function test_a_hint_shows_while_its_control_is_in_view() {
        const hint = hintOf(button);

        button.focused = true;

        tryVerify(() => bubbleOf(hint)?.visible === true, 2000, "the bubble shows, and comes back after it was hidden");
        verify(hint.inView);
    }

    function test_a_hint_goes_with_its_control_scrolled_out_of_view() {
        const hint = hintOf(button);
        button.focused = true;
        tryVerify(() => bubbleOf(hint)?.visible === true, 2000);

        scroll.contentY = button.y + button.height;

        tryCompare(hint, "inView", false, 2000);
        tryVerify(() => bubbleOf(hint).visible === false, 2000, "no bubble floats for a control out of view");
    }

    function test_a_hint_goes_with_its_control_cut_part_way() {
        const hint = hintOf(button);
        button.focused = true;

        scroll.contentY = button.y + button.height / 2;

        tryCompare(hint, "inView", false, 2000);
    }

    function test_a_box_inside_every_clip_is_unclipped() {
        compare(K.Look.unclipped(button), true);

        scroll.contentY = button.y + 1;

        compare(K.Look.unclipped(button), false);
    }

    function test_a_select_hidden_with_what_holds_it_closes_its_menu() {
        select.show();
        verify(select.open);

        holder.visible = false;

        compare(select.open, false);
    }

    function test_dismissing_a_select_leaves_the_keyboard_where_it_is() {
        scroll.forceActiveFocus();
        select.show();

        select.dismiss();

        compare(select.open, false);
        verify(!select.activeFocus, "the select doesn't take the keyboard back");
    }
}
