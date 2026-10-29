import QtQuick
import QtTest
import Kotodama
import KubaOS.Components as K
import "../../config/quickshell/modules/menu"

// The Command menu's Action panel and form Views keep to the room they are given: more than fits
// scrolls, and the keys keep what they are on in view.
TestCase {
    id: suite

    name: "MenuHeights"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 600
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    // design-allow(size): a test's fixture, not a design value.
    readonly property int room: 240

    readonly property var manyActions: Array.from({
            length: 20
        }, (_, n) => ({
                title: `Action ${n + 1}`,
                group: n < 12 ? "" : "kubaos",
                shortcut: ""
            }))
    readonly property var fewActions: manyActions.slice(0, 2)
    readonly property var manyFields: Array.from({
            length: 12
        }, (_, n) => ({
                label: `Field ${n + 1}`,
                kind: "text",
                placeholder: "Type here",
                value: ""
            }))

    KotodamaSearch {
        id: search
    }

    ActionPanel {
        id: actions

        // design-allow(size): a test's fixture, not a design value.
        width: 280
        maxHeight: suite.room
    }

    FormView {
        id: form

        // design-allow(size): a test's fixture, not a design value.
        x: 300
        // design-allow(size): a test's fixture, not a design value.
        width: 280
        height: suite.room
        search: search
        fields: suite.manyFields
    }

    function init() {
        actions.actions = [];
        actions.current = 0;
        form.contentY = 0;
    }

    // Gives the panel `list` and waits for its entries to be laid out, as they are by the time a
    // key reaches it.
    function show(list) {
        actions.actions = list;
        waitForRendering(actions);
    }

    function test_an_action_panel_with_room_to_spare_is_as_tall_as_its_actions() {
        show(suite.fewActions);

        verify(actions.fullHeight < suite.room);
        compare(actions.height, actions.fullHeight);
    }

    function test_an_action_panel_is_never_taller_than_its_room() {
        show(suite.manyActions);

        verify(actions.fullHeight > suite.room);
        compare(actions.height, suite.room);
    }

    function test_the_keys_keep_the_last_action_in_view() {
        show(suite.manyActions);

        actions.current = suite.manyActions.length - 1;

        compare(actions.contentY, actions.fullHeight - suite.room);
    }

    function test_the_pointer_never_scrolls_the_action_panel() {
        show(suite.manyActions);

        actions.point(suite.manyActions.length - 1);

        compare(actions.contentY, 0);
    }

    function test_a_form_taller_than_its_room_scrolls_and_keeps_its_height() {
        tryVerify(() => form.travel > 0, 2000, "the fields don't fit, so the form scrolls");

        compare(form.height, suite.room);
    }

    function test_the_keys_on_the_last_field_scroll_it_into_view() {
        tryVerify(() => form.travel > 0, 2000);

        form.move(-1);

        tryVerify(() => form.contentY >= form.travel - K.Tokens.space.s24 - K.Tokens.size.control_md - 1, 2000, "scrolled down to the last field, the submit button under it");
    }
}
