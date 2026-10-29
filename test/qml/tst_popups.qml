import QtQuick
import QtTest
import "../../config/quickshell/services/popups.js" as Popups

// The popups against the notifications panel: opening it is seeing every popup, critical ones too,
// so they leave and don't come back when it closes; what arrives while it is open is listed there
// and never pops up; what arrives after it closes pops up.
TestCase {
    name: "Popups"

    function notification(summary, transient) {
        return {
            summary: summary,
            transient: transient === true
        };
    }

    function test_opening_the_panel_takes_every_popup_away_for_good() {
        const backup = notification("Backup finished");
        const battery = notification("Battery low");
        let state = Popups.arrived(Popups.initial(), backup, true);
        state = Popups.arrived(state, battery, true);
        compare(state.popups, [battery, backup]);

        state = Popups.listed(state, true);
        compare(state.popups, []);

        state = Popups.listed(state, false);
        compare(state.popups, [], "none comes back as it closes");
    }

    function test_one_that_arrives_while_the_panel_is_open_never_pops_up() {
        let state = Popups.listed(Popups.initial(), true);

        state = Popups.arrived(state, notification("Update ready"), true);
        compare(state.popups, []);

        state = Popups.listed(state, false);
        compare(state.popups, []);
    }

    function test_one_that_arrives_after_the_panel_closes_pops_up() {
        let state = Popups.listed(Popups.arrived(Popups.initial(), notification("Old"), true), true);
        state = Popups.listed(state, false);
        const fresh = notification("New");

        state = Popups.arrived(state, fresh, true);

        compare(state.popups, [fresh]);
    }

    function test_do_not_disturb_still_holds_one_back_with_the_panel_closed() {
        const state = Popups.arrived(Popups.initial(), notification("Quiet"), false);

        compare(state.popups, []);
    }

    function test_a_transient_one_seen_in_the_panel_waits_to_go_until_it_has_gone() {
        const volume = notification("Volume", true);
        const later = notification("Brightness", true);
        let state = Popups.arrived(Popups.initial(), volume, true);
        state = Popups.arrived(Popups.arrived(state, notification("Mail"), true), notification("Kept"), true);

        state = Popups.listed(state, true);
        state = Popups.arrived(state, later, true);
        state = Popups.listed(state, false);

        compare(state.seen, [volume, later], "kept while the panel slides away");
        compare(Popups.gone(state).seen, []);
    }

    function test_one_that_leaves_is_no_longer_held() {
        const volume = notification("Volume", true);
        let state = Popups.listed(Popups.arrived(Popups.initial(), volume, true), true);

        state = Popups.left(state, volume);

        compare(state.seen, []);
    }

    function test_it_never_changes_the_state_it_is_given() {
        const backup = notification("Backup finished");
        const state = Popups.arrived(Popups.initial(), backup, true);

        Popups.listed(state, true);

        compare(state.popups, [backup]);
        verify(!state.listing);
    }
}
