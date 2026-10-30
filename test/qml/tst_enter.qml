import QtQuick
import QtTest
import "../../config/quickshell/modules/menu/enter.js" as Enter

// Enter on a row runs the Item's own first Action while the row can't show the one a verb in the
// query chose, and the footer and the Action panel say so.
TestCase {
    id: suite

    name: "Enter"

    function entry(id, title, group, role) {
        return {
            id: id,
            title: title,
            group: group,
            shortcut: "",
            role: role,
            keywords: []
        };
    }

    // "remove chrome": the apps Extension's Chrome, with KubaOS's Remove chosen.
    readonly property var removeChrome: [entry("remove", "Remove", "kubaos", "destructive"), entry("open", "Open", "", "normal"), entry("new-window", "New Window", "", "normal"), entry("add-to-notes", "Add to notes", "notes", "normal")]
    // "wifi off": Wi-Fi, with its own Turn off chosen over Turn Wi-Fi on or off.
    readonly property var wifiOff: [entry("off", "Turn off", "", "normal"), entry("toggle", "Turn Wi-Fi on or off", "", "normal"), entry("on", "Turn on", "", "normal")]

    function ids(actions) {
        return actions.map(action => action.id);
    }

    function test_a_row_nothing_was_chosen_on_is_as_kotodama_gives_it() {
        compare(Enter.usual(wifiOff, false), 0);
        compare(ids(Enter.unchosen(wifiOff, false)), ["off", "toggle", "on"]);
        compare(Enter.hints("↵ Turn off · ⌃K Actions", wifiOff, false), "↵ Turn off · ⌃K Actions");
    }

    function test_enter_runs_the_items_own_first_action_over_a_chosen_contributed_one() {
        compare(Enter.usual(removeChrome, true), 1);
    }

    function test_enter_runs_the_items_own_first_action_over_a_chosen_own_one() {
        compare(Enter.usual(wifiOff, true), 1);
    }

    function test_a_chosen_destructive_action_goes_back_last_in_the_panel() {
        compare(ids(Enter.unchosen(removeChrome, true)), ["open", "new-window", "add-to-notes", "remove"]);
    }

    function test_a_chosen_action_follows_the_items_own_first_in_the_panel() {
        compare(ids(Enter.unchosen(wifiOff, true)), ["toggle", "off", "on"]);
    }

    function test_the_footer_names_what_enter_runs() {
        compare(Enter.hints("↵ Remove · Tab Enter app · ⌃K Actions", removeChrome, true), "↵ Open · Tab Enter app · ⌃K Actions");
        compare(Enter.hints("↵ Turn off · ⌃K Actions", wifiOff, true), "↵ Turn Wi-Fi on or off · ⌃K Actions");
    }

    function test_an_item_with_no_action_of_its_own_runs_nothing() {
        const contributedOnly = [entry("remove", "Remove", "kubaos", "destructive")];

        compare(Enter.usual(contributedOnly, true), -1);
        compare(Enter.usual([], false), -1);
        compare(ids(Enter.unchosen(contributedOnly, true)), ["remove"]);
    }
}
