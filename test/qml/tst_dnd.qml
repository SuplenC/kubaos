import QtQuick
import QtTest
import "../../config/quickshell/services/dnd.js" as Dnd

// Do not disturb: by hand it stays until turned off; by itself it holds while the screen is shared
// or a window is fullscreen, and turning it off during one keeps it off for that one only.
TestCase {
    id: suite

    name: "Dnd"

    function sensed(state, sharing, fullscreen) {
        return Dnd.sense(state, {
            sharing: sharing,
            fullscreen: fullscreen
        });
    }

    function test_it_starts_off() {
        const state = Dnd.initial();

        verify(!Dnd.on(state));
        compare(Dnd.reason(state), "");
    }

    function test_turned_on_by_hand_it_stays_on_until_turned_off() {
        let state = Dnd.toggle(Dnd.initial());
        verify(Dnd.on(state));
        compare(Dnd.reason(state), "manual");

        state = sensed(sensed(state, true, false), false, false);
        verify(Dnd.on(state), "a share coming and going leaves it on");

        state = Dnd.toggle(state);
        verify(!Dnd.on(state));
    }

    function test_it_is_on_while_the_screen_is_shared() {
        let state = sensed(Dnd.initial(), true, false);
        verify(Dnd.on(state));
        compare(Dnd.reason(state), "sharing");

        state = sensed(state, false, false);
        verify(!Dnd.on(state));
    }

    function test_it_is_on_while_a_window_is_fullscreen() {
        let state = sensed(Dnd.initial(), false, true);
        verify(Dnd.on(state));
        compare(Dnd.reason(state), "fullscreen");

        state = sensed(state, false, false);
        verify(!Dnd.on(state));
    }

    function test_sharing_is_the_reason_given_over_fullscreen() {
        compare(Dnd.reason(sensed(Dnd.initial(), true, true)), "sharing");
    }

    function test_turned_off_during_a_share_it_stays_off_for_that_share_only() {
        let state = Dnd.toggle(sensed(Dnd.initial(), true, false));
        verify(!Dnd.on(state));
        compare(Dnd.reason(state), "overridden");

        state = sensed(state, false, false);
        verify(!Dnd.on(state));
        compare(Dnd.reason(state), "");

        state = sensed(state, true, false);
        verify(Dnd.on(state), "the next share turns it on again");
    }

    function test_nothing_turns_it_on_twice_in_one_occurrence() {
        let state = Dnd.toggle(sensed(Dnd.initial(), true, false));

        state = sensed(state, true, true);
        verify(!Dnd.on(state), "a window going fullscreen during the share keeps the override");
        state = sensed(state, true, true);
        verify(!Dnd.on(state), "the same event again changes nothing");
        state = sensed(state, false, true);
        verify(!Dnd.on(state), "the share ending while the window stays fullscreen keeps it");
    }

    function test_turned_back_on_during_a_share_it_ends_with_the_share() {
        let state = Dnd.toggle(Dnd.toggle(sensed(Dnd.initial(), true, false)));
        verify(Dnd.on(state));
        compare(Dnd.reason(state), "sharing");

        state = sensed(state, false, false);
        verify(!Dnd.on(state));
    }

    function test_turning_it_off_clears_a_hand_on_during_a_share() {
        let state = sensed(Dnd.toggle(Dnd.initial()), true, false);

        state = Dnd.toggle(state);
        verify(!Dnd.on(state));

        state = sensed(state, false, false);
        verify(!Dnd.on(state), "it doesn't come back on by hand after the share");
    }

    function test_setting_it_to_what_it_already_is_changes_nothing() {
        const on = Dnd.set(Dnd.initial(), true);
        compare(Dnd.set(on, true), on);

        const shared = sensed(Dnd.initial(), true, false);
        compare(Dnd.set(shared, true), shared, "on already, so the share keeps it, not the hand");

        const off = Dnd.initial();
        compare(Dnd.set(off, false), off);
    }

    function test_set_reaches_the_state_asked_for() {
        verify(Dnd.on(Dnd.set(Dnd.initial(), true)));
        verify(!Dnd.on(Dnd.set(sensed(Dnd.initial(), false, true), false)));
    }

    function test_the_hand_kept_state_round_trips_through_its_file() {
        const state = Dnd.restore(Dnd.saved(Dnd.toggle(Dnd.initial())));

        verify(Dnd.on(state));
        verify(!Dnd.on(Dnd.restore("not json")));
        verify(!Dnd.on(Dnd.restore("")));
    }

    function test_the_file_says_whether_it_is_on_and_why() {
        const file = JSON.parse(Dnd.saved(sensed(Dnd.initial(), true, false)));

        compare(file.on, true);
        compare(file.manual, false);
        compare(file.reason, "sharing");
    }
}
