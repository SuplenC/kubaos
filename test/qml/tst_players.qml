import QtQuick
import QtTest
import "../../config/quickshell/services/players.js" as Players

// The players service's logic: which player is active, and which window is a player's.
TestCase {
    id: suite

    name: "Players"

    function player(dbusName, isPlaying) {
        return {
            dbusName: dbusName,
            isPlaying: isPlaying
        };
    }

    function test_the_picked_player_is_active_while_it_is_there() {
        const spotify = player("spotify", true);
        const firefox = player("firefox", false);

        compare(Players.active([spotify, firefox], "firefox"), firefox);
    }

    function test_without_a_pick_a_playing_player_is_active() {
        const spotify = player("spotify", true);
        const firefox = player("firefox", false);

        compare(Players.active([firefox, spotify], ""), spotify);
    }

    function test_a_gone_pick_gives_way_to_a_playing_player_then_the_first() {
        const spotify = player("spotify", false);
        const firefox = player("firefox", false);

        compare(Players.active([spotify, firefox], "mpv"), spotify);
        compare(Players.active([], "mpv"), null);
    }

    function test_a_player_that_starts_playing_becomes_active() {
        compare(Players.played("firefox", player("spotify", true)), "spotify");
    }

    function test_a_player_that_stops_stays_active() {
        const key = Players.played("spotify", player("spotify", false));

        compare(key, "spotify");
        compare(Players.active([player("firefox", false), player("spotify", false)], key).dbusName, "spotify");
    }

    function test_another_player_stopping_leaves_the_active_one() {
        compare(Players.played("spotify", player("firefox", false)), "spotify");
    }

    function test_a_window_matches_a_player_by_class_ignoring_case() {
        const windows = [
            {
                address: "a1",
                appClass: "kitty"
            },
            {
                address: "b2",
                appClass: "Spotify"
            }
        ];

        compare(Players.windowOf(windows, ["spotify", undefined, "Spotify"]).address, "b2");
    }

    function test_a_reverse_dns_class_or_entry_matches_by_its_last_part() {
        const windows = [
            {
                address: "c3",
                appClass: "org.mozilla.firefox"
            }
        ];

        compare(Players.windowOf(windows, ["firefox"]).address, "c3");
        compare(Players.windowOf([
            {
                address: "d4",
                appClass: "firefox"
            }
        ], ["org.mozilla.firefox"]).address, "d4");
    }

    function test_a_player_named_only_by_identity_finds_the_entry_of_that_name_first() {
        const chrome = {
            name: "Google Chrome"
        };
        const exact = {
            name: "chrome"
        };

        compare(Players.entryNamed([chrome, exact], "Chrome"), exact);
        compare(Players.entryNamed([chrome], "Chrome"), chrome);
    }

    function test_an_entry_name_only_containing_the_identity_is_no_match() {
        compare(Players.entryNamed([
            {
                name: "Chrome Remote Desktop"
            }
        ], "Chrome"), null);
        compare(Players.entryNamed([
            {
                name: "Spotify"
            }
        ], ""), null);
    }

    function test_no_window_matches_a_player_with_no_names() {
        compare(Players.windowOf([
            {
                address: "e5",
                appClass: ""
            }
        ], ["", undefined]), null);
    }
}
