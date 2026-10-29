import QtQuick
import QtTest
import KubaOS.Components as K
import qs.services
import "../../config/quickshell/modules/sysmon"

// The system monitor's body, its readings over its processes, holds still from the first frame to
// the readings: the light sample a panel opens on keeps a place for every line a detailed one
// fills, a line the machine lacks is left out from the start, and the snapshots line stays on one
// line with its info button.
TestCase {
    id: suite

    name: "SysmonReadings"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 500
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 1600

    property var shown: null

    K.Stack {
        id: body

        width: K.Tokens.size.panel
        fill: true
        gap: K.Tokens.space.s16

        Readings {
            id: readings

            sample: suite.shown
            within: body
        }

        Processes {
            id: processes

            sample: suite.shown
            tallest: 400
        }
    }

    // The stand-in's detailed sample, changed by `change`.
    function detailed(change) {
        const s = JSON.parse(JSON.stringify(Sysmon.sample));
        change?.(s);
        return s;
    }

    // `s` as the pill's pace leaves it: no cores, clock, traffic, address, snapshots, fans or
    // processes, only how many fans and whether snapshots.
    function light(s) {
        const l = JSON.parse(JSON.stringify(s));
        l.detail = false;
        l.cpu.cores = [];
        l.cpu.mhz = null;
        l.io = null;
        l.net.address = null;
        l.snapshots = null;
        l.fans = [];
        l.top = null;
        return l;
    }

    // A machine with no GPU, battery, fans or Btrfs.
    function bare(s) {
        s.gpus = [];
        s.battery = null;
        s.fans = [];
        s.fanCount = 0;
        s.snapshots = null;
        s.hasSnapshots = false;
        s.quotas = false;
    }

    function find(item, test) {
        if (test(item))
            return item;
        for (const child of item.children) {
            const found = suite.find(child, test);
            if (found)
                return found;
        }
        return null;
    }

    // Where each row of the body's stacks is and how tall, depth first: what must not move. Rows
    // only: what is in a row may change width, and a placeholder gives way to its text over it.
    function rows() {
        const out = [];
        const walk = it => {
            if (!it.visible)
                return;
            if (it.parent?.centred !== undefined && it.parent.columns === undefined) {
                const at = it.mapToItem(body, 0, 0);
                out.push(`${at.y}+${it.height}`);
            }
            for (const child of it.children)
                walk(child);
        };
        walk(body);
        return out;
    }

    function settle() {
        waitForRendering(body);
        wait(K.Tokens.duration.slow * 2);
    }

    function show(s) {
        suite.shown = s;
        suite.settle();
    }

    function test_the_body_holds_still_as_the_readings_arrive_data() {
        return [
            {
                tag: "a laptop on Btrfs with a GPU and a fan",
                change: null
            },
            {
                tag: "with snapshot sizes unknown",
                change: s => {
                    s.snapshots.bytes = null;
                    s.quotas = false;
                }
            },
            {
                tag: "a bare machine",
                change: s => suite.bare(s)
            }
        ];
    }

    function test_the_body_holds_still_as_the_readings_arrive(data) {
        const full = suite.detailed(data.change);

        suite.show(suite.light(full));
        const first = suite.rows();
        const firstHeight = body.implicitHeight;
        suite.show(full);

        const loaded = suite.rows();
        for (let i = 0; i < Math.max(loaded.length, first.length); i++)
            compare(loaded[i], first[i], `row ${i}`);
        compare(body.implicitHeight, firstHeight);
    }

    function test_a_reading_still_on_its_way_is_a_placeholder() {
        suite.show(suite.light(suite.detailed()));

        const waiting = suite.find(readings, it => it.waiting === true && it.place !== undefined);

        verify(waiting !== null);
        compare(waiting.height, K.Tokens.type.mono.line);
    }

    function test_the_snapshots_line_stays_on_one_line_with_its_info_button() {
        const full = suite.detailed(s => {
            s.snapshots = {
                count: 68,
                bytes: null
            };
            s.quotas = false;
        });

        suite.show(full);

        const line = suite.find(readings, it => it.place !== undefined && it.text === "68 snapshots · size unknown");
        verify(line !== null);
        compare(line.height, K.Tokens.type.mono.line);
        const label = suite.find(line, it => it.cut !== undefined);
        compare(label.lines.length, 1);
        verify(!label.cut);
        const button = suite.find(readings, it => it.label === "Why the size is unknown");
        verify(button.visible);
        compare(button.y + button.height / 2, line.y + line.height / 2);
    }
}
