import QtQuick
import QtTest
import "../../config/quickshell/services/sysmon.js" as Levels
import "../../config/quickshell/modules/sysmon/panel.js" as Panel

// What the system monitor's panel says of a sample: the colour steps of its readings, the subtitle
// from the worst of them, and the process rows that fill its room while the list is counted.
TestCase {
    id: suite

    name: "Sysmon"

    readonly property var thresholds: ({
            cpuTemp: 90,
            ram: 90,
            disk: 95,
            batteryDraw: 30
        })

    // A sample with every reading fine; `change` changes what a test needs.
    function sample(change) {
        const s = {
            cpu: {
                temp: 55
            },
            memory: {
                percent: 40
            },
            disks: [
                {
                    percent: 50
                },
                {
                    percent: 60
                }
            ],
            battery: {
                status: "Discharging",
                watts: 10
            }
        };
        change?.(s);
        return s;
    }

    function test_a_fine_machine_is_cool_as_a_cucumber() {
        const level = Levels.worst(sample(), suite.thresholds);

        compare(level, 0);
        compare(Panel.mood(level), "Cool as a cucumber");
    }

    function test_the_worst_reading_sets_the_subtitle_data() {
        return [
            {
                tag: "the CPU within 10 °C of its limit",
                change: s => s.cpu.temp = 85,
                level: 1,
                mood: "Breaking a sweat"
            },
            {
                tag: "the CPU over its limit",
                change: s => s.cpu.temp = 91,
                level: 2,
                mood: "This is fine"
            },
            {
                tag: "memory within 10 points of its limit",
                change: s => s.memory.percent = 81,
                level: 1,
                mood: "Breaking a sweat"
            },
            {
                tag: "memory over its limit",
                change: s => s.memory.percent = 95,
                level: 2,
                mood: "This is fine"
            },
            {
                tag: "the second disk within 5 points of full",
                change: s => s.disks[1].percent = 92,
                level: 1,
                mood: "Breaking a sweat"
            },
            {
                tag: "a disk over its limit",
                change: s => s.disks[0].percent = 97,
                level: 2,
                mood: "This is fine"
            },
            {
                tag: "the battery draining fast",
                change: s => s.battery.watts = 40,
                level: 2,
                mood: "This is fine"
            },
            {
                tag: "one reading warm and another over its limit",
                change: s => {
                    s.cpu.temp = 85;
                    s.disks[1].percent = 99;
                },
                level: 2,
                mood: "This is fine"
            }
        ];
    }

    function test_the_worst_reading_sets_the_subtitle(data) {
        const level = Levels.worst(sample(data.change), suite.thresholds);

        compare(level, data.level);
        compare(Panel.mood(level), data.mood);
    }

    function test_the_battery_on_its_charger_counts_for_nothing_however_much_it_takes() {
        const charging = sample(s => {
            s.battery.status = "Charging";
            s.battery.watts = 60;
        });

        compare(Levels.drawLevel(charging, suite.thresholds), 0);
        compare(Levels.worst(charging, suite.thresholds), 0);
    }

    function test_a_machine_without_sensors_disks_or_battery_is_fine() {
        compare(Levels.worst({}, suite.thresholds), 0);
        compare(Levels.worst(null, suite.thresholds), 0);
    }

    function test_the_rows_fill_the_room_left() {
        compare(Panel.slots(280, 28, 5, 40), 10);
        compare(Panel.slots(290, 28, 5, 40), 10, "a part row's room is left over");
    }

    function test_a_tall_room_gets_no_more_rows_than_the_engine_lists() {
        compare(Panel.slots(28 * 60, 28, 5, 40), 40);
    }

    function test_a_short_room_still_gets_the_fewest_rows_and_scrolls() {
        compare(Panel.slots(60, 28, 5, 40), 5);
        compare(Panel.slots(-200, 28, 5, 40), 5);
    }

    function test_while_the_list_is_counted_every_row_is_a_placeholder() {
        for (let i = 0; i < 8; i++)
            compare(Panel.slot(i, false, 0), "placeholder", `row ${i}`);
    }

    function test_once_counted_the_rows_show_processes_and_any_left_over_stay_empty() {
        compare(Panel.slot(0, true, 3), "process");
        compare(Panel.slot(2, true, 3), "process");
        compare(Panel.slot(3, true, 3), "empty");
        compare(Panel.slot(7, true, 3), "empty");
    }

    function test_neighbouring_placeholders_differ_in_width_and_stay_in_their_column() {
        for (let i = 0; i < 10; i++) {
            verify(Panel.share(i, false) !== Panel.share(i + 1, false), `rows ${i} and ${i + 1}`);
            verify(Panel.share(i, true) !== Panel.share(i + 1, true), `numbers in rows ${i} and ${i + 1}`);
            for (const numeric of [false, true])
                verify(Panel.share(i, numeric) > 0 && Panel.share(i, numeric) <= 1, `row ${i}, numeric ${numeric}`);
        }
    }

    function test_the_snapshots_line_says_how_many_and_their_size_data() {
        return [
            {
                tag: "quotas off",
                snaps: {
                    count: 68,
                    bytes: null
                },
                line: "68 snapshots · size unknown"
            },
            {
                tag: "one, quotas on",
                snaps: {
                    count: 1,
                    bytes: 2048
                },
                line: "1 snapshot · 2 KB"
            },
            {
                tag: "uncounted",
                snaps: {
                    count: null,
                    bytes: null
                },
                line: "Snapshots · size unknown"
            }
        ];
    }

    function test_the_snapshots_line_says_how_many_and_their_size(data) {
        compare(Panel.snapshots(data.snaps, bytes => `${bytes / 1024} KB`), data.line);
    }
}
