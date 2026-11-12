import QtQuick
import QtTest
import "../../config/quickshell/services/audio.js" as Audio

// The audio service's logic: which streams are apps and where each plays, the output's level
// with its boost, its balance, and the headset's status.
TestCase {
    id: suite

    name: "Audio"

    readonly property var speakers: ({
            name: "alsa_output.speakers",
            isSink: true,
            isStream: false,
            audio: {},
            properties: {
                "object.serial": "41"
            }
        })
    readonly property var microphone: ({
            name: "alsa_input.mic",
            isSink: false,
            isStream: false,
            audio: {},
            properties: {}
        })

    function stream(isSink, props) {
        return {
            isSink: isSink,
            isStream: true,
            audio: {},
            properties: props
        };
    }

    function test_apps_are_streams_split_by_playing_or_recording() {
        const firefox = stream(true, {
            "application.name": "Firefox"
        });
        const discord = stream(false, {
            "application.name": "Discord"
        });

        const nodes = [suite.speakers, suite.microphone, firefox, discord];

        compare(Audio.apps(nodes, true), [firefox]);
        compare(Audio.apps(nodes, false), [discord]);
    }

    function test_peak_meters_and_video_streams_are_no_app() {
        const meter = stream(false, {
            "application.name": "Quickshell Peak Detect",
            "stream.monitor": "true"
        });
        const video = stream(false, {});
        video.audio = null;

        compare(Audio.apps([meter, video], false), []);
    }

    function test_an_app_is_named_by_its_application_name_then_its_binary() {
        compare(Audio.appName({
            "application.name": "Firefox",
            "application.process.binary": "firefox-bin"
        }), "Firefox");
        compare(Audio.appName({
            "application.process.binary": "mpv",
            "node.name": "mpv.1"
        }), "mpv");
        compare(Audio.appName(undefined), "");
    }

    function test_desktop_entries_are_looked_up_by_app_id_first() {
        compare(Audio.entryKeys({
            "application.name": "Firefox",
            "application.process.binary": "firefox",
            "pipewire.access.portal.app_id": "org.mozilla.firefox"
        }), ["org.mozilla.firefox", "firefox", "Firefox"]);
    }

    function test_a_playing_stream_plays_to_the_device_it_links_to() {
        const firefox = stream(true, {});
        const meter = stream(false, {});
        const groups = [
            {
                source: firefox,
                target: meter
            },
            {
                source: firefox,
                target: suite.speakers
            }
        ];

        compare(Audio.deviceOf(firefox, groups), suite.speakers);
    }

    function test_a_recording_stream_records_from_the_device_linked_to_it() {
        const discord = stream(false, {});
        const groups = [
            {
                source: suite.microphone,
                target: discord
            }
        ];

        compare(Audio.deviceOf(discord, groups), suite.microphone);
    }

    function test_an_unlinked_stream_has_no_device() {
        compare(Audio.deviceOf(stream(true, {}), []), null);
    }

    function test_a_device_is_targeted_by_its_serial_else_its_name() {
        compare(Audio.targetName(suite.speakers), "41");
        compare(Audio.targetName(suite.microphone), "alsa_input.mic");
    }

    function test_the_level_is_the_loudest_channel() {
        compare(Audio.level([0.3, 0.6]), 0.6);
        compare(Audio.level([]), 0);
    }

    function test_the_level_boosts_to_150_percent_and_no_further() {
        compare(Audio.leveled([0.5, 0.5], 1.4, Audio.boostMax), [1.4, 1.4]);
        compare(Audio.leveled([0.5, 0.5], 2, Audio.boostMax), [1.5, 1.5]);
        compare(Audio.leveled([0.5, 0.5], -1, Audio.boostMax), [0, 0]);
    }

    function test_a_new_level_keeps_the_balance() {
        const volumes = Audio.leveled([0.4, 0.8], 1, Audio.boostMax);

        fuzzyCompare(volumes[0], 0.5, 1e-6);
        fuzzyCompare(volumes[1], 1, 1e-6);
    }

    function test_silent_channels_all_take_the_new_level() {
        compare(Audio.leveled([0, 0], 0.7, Audio.boostMax), [0.7, 0.7]);
    }

    function test_channels_take_their_side_from_their_name() {
        compare(["Front Left", "Front Right", "Front Center", "Low Frequency Effects", "Rear Left Center", "Mono"].map(Audio.sideOf), [-1, 1, 0, 0, -1, 0]);
    }

    function test_equal_sides_are_centred() {
        compare(Audio.balance([-1, 1], [0.6, 0.6]), 0);
    }

    function test_a_quieter_left_leans_right_by_its_share_of_the_right() {
        fuzzyCompare(Audio.balance([-1, 1], [0.42, 0.6]), 0.3, 1e-6);
        fuzzyCompare(Audio.balance([-1, 1], [0.6, 0.51]), -0.15, 1e-6);
        compare(Audio.balance([-1, 1], [0, 0.6]), 1);
    }

    function test_a_mono_or_silent_output_is_centred() {
        compare(Audio.balance([0], [0.6]), 0);
        compare(Audio.balance([-1, 1], [0, 0]), 0);
    }

    function test_balancing_lowers_the_other_side_and_keeps_the_level() {
        const sides = [-1, 1, 0];

        const right = Audio.balanced(sides, [0.8, 0.8, 0.8], 0.25);
        const left = Audio.balanced(sides, [0.8, 0.8, 0.8], -1);

        fuzzyCompare(right[0], 0.6, 1e-6);
        compare(right.slice(1), [0.8, 0.8]);
        compare(left, [0.8, 0, 0.8]);
    }

    function test_balance_reads_back_what_was_set() {
        const sides = [-1, 1];

        for (const b of [-0.7, -0.2, 0, 0.35, 1])
            fuzzyCompare(Audio.balance(sides, Audio.balanced(sides, [0.5, 0.5], b)), b, 1e-6);
    }

    function test_balancing_a_centred_output_changes_nothing() {
        compare(Audio.balanced([-1, 1], [0.3, 0.9], 0), [0.9, 0.9]);
    }

    function test_the_headset_status_gives_its_device_and_mode() {
        compare(Audio.headsetStatus('{"device": "WH-1000XM4", "mode": "music"}'), {
            device: "WH-1000XM4",
            mode: "music"
        });
    }

    function test_no_mode_or_no_answer_is_no_headset() {
        const none = {
            device: "",
            mode: ""
        };

        compare(Audio.headsetStatus('{"device": "WH-1000XM4", "mode": null}'), none);
        compare(Audio.headsetStatus(""), none);
        compare(Audio.headsetStatus("dot-audio: not found"), none);
    }
}
