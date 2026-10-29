import QtQuick
import QtTest
import KubaOS.Components as K

// Sheets in one place, as the system monitor, the notifications panel and the control center are,
// their buttons in that order along the bar: switching straight from one to another keeps the
// surface and swaps the content, each coming from its button's side; opening from nothing and
// closing slide the surface.
TestCase {
    id: suite

    name: "SheetTwins"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 800
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    QtObject {
        id: frame

        readonly property real l: K.Tokens.space.s8
        readonly property real t: K.Tokens.size.bar
        readonly property real r: suite.width - K.Tokens.space.s8
        readonly property real b: suite.height - K.Tokens.space.s8
    }

    K.Sheet {
        id: left

        frame: frame
        edge: "right"
        flushStart: true
        flushEnd: true
        contentWidth: K.Tokens.size.panel
        twins: [right, far]
        side: -1
    }

    K.Sheet {
        id: right

        frame: frame
        edge: "right"
        flushStart: true
        flushEnd: true
        contentWidth: K.Tokens.size.panel
        twins: [left, far]
        side: 1
    }

    // A third, its button right of both.
    K.Sheet {
        id: far

        frame: frame
        edge: "right"
        flushStart: true
        flushEnd: true
        contentWidth: K.Tokens.size.panel
        twins: [left, right]
        side: 2
    }

    // The longest a swap may take: `micro` / 2 + `short`, and a few frames.
    readonly property int swapTime: K.Tokens.duration.micro / 2 + K.Tokens.duration.short + K.Tokens.duration.micro

    function init() {
        K.Motion.reduced = false;
        left.shown = false;
        right.shown = false;
        far.shown = false;
        tryCompare(left, "active", false, K.Tokens.duration.long * 4);
        tryCompare(right, "active", false, K.Tokens.duration.long * 4);
        tryCompare(far, "active", false, K.Tokens.duration.long * 4);
    }

    function cleanup() {
        K.Motion.reduced = false;
    }

    function openFully(sheet) {
        sheet.shown = true;
        tryCompare(sheet, "progress", 1, K.Tokens.duration.long * 4);
    }

    // `from` closes and `to` opens, as the frame's bindings do when `Panels.current` changes, in
    // the order given: Qt updates the two bindings in either.
    function switchTo(from, to, toFirst) {
        if (toFirst) {
            to.shown = true;
            from.shown = false;
        } else {
            from.shown = false;
            to.shown = true;
        }
    }

    // Watches a swap from `from` to `to` until `to` holds the whole content, checking every few
    // milliseconds that the two are never drawn together and that each moves its own way.
    function watchSwap(from, to) {
        const ends = Date.now() + suite.swapTime * 4;
        // 1 where the new one's button is right of the old one's.
        const toward = Math.sign(to.side - from.side);
        let both = 0;
        let fromWrongWay = 0;
        let toWrongWay = 0;
        while (Date.now() < ends && !(to.shift === 0 && to.presence === 1 && !from.active)) {
            if (from.active && from.presence > 0 && to.presence > 0)
                both++;
            if (from.active && Math.sign(from.shift) === toward)
                fromWrongWay++;
            if (Math.sign(to.shift) === -toward)
                toWrongWay++;
            compare(to.progress, 1, "the surface stays open");
            compare(from.path, "", "only one surface is drawn");
            wait(4);
        }
        compare(both, 0, "the two contents are never drawn together");
        compare(fromWrongWay, 0, "the old content leaves away from the new one's button");
        compare(toWrongWay, 0, "the new content comes in from its button's side");
        compare(to.shift, 0, "the new content at rest");
        compare(to.presence, 1, "the new content shown");
        verify(!from.active, "the old sheet gone");
        verify(!from.leaving, "the old sheet settled");
    }

    function test_opening_from_nothing_slides_the_surface_out() {
        right.shown = true;
        wait(K.Tokens.duration.micro);

        verify(right.progress > 0 && right.progress < 1, `half-way out: ${right.progress}`);
        compare(right.shift, 0);
        compare(right.presence, 1);
        verify(right.path !== "");
        tryCompare(right, "progress", 1, K.Tokens.duration.long * 4);
    }

    function test_closing_slides_the_surface_back_in() {
        openFully(right);

        right.shown = false;
        wait(K.Tokens.duration.micro);

        verify(right.progress > 0 && right.progress < 1, `half-way in: ${right.progress}`);
        verify(!right.leaving);
        verify(right.path !== "");
        tryCompare(right, "active", false, K.Tokens.duration.medium * 4);
    }

    function test_switching_keeps_the_surface_and_starts_the_new_content_at_its_buttons_side() {
        openFully(left);

        switchTo(left, right, false);

        compare(right.progress, 1, "the surface where it was");
        verify(right.path !== "", "the new sheet draws the surface");
        compare(left.path, "", "the old one draws none");
        verify(left.leaving);
        compare(right.presence, 0, "the new content not shown yet");
        compare(right.shift, right.travel, "the new content to the right, by the status pill");
    }

    function test_switching_to_the_right_moves_the_old_content_left_and_the_new_in_from_the_right_data() {
        return [
            {
                tag: "the old one's binding updates first",
                toFirst: false
            },
            {
                tag: "the new one's binding updates first",
                toFirst: true
            }
        ];
    }

    function test_switching_to_the_right_moves_the_old_content_left_and_the_new_in_from_the_right(data) {
        openFully(left);

        switchTo(left, right, data.toFirst);

        watchSwap(left, right);
    }

    function test_switching_back_moves_the_old_content_right_and_the_new_in_from_the_left_data() {
        return test_switching_to_the_right_moves_the_old_content_left_and_the_new_in_from_the_right_data();
    }

    function test_switching_back_moves_the_old_content_right_and_the_new_in_from_the_left(data) {
        openFully(right);

        switchTo(right, left, data.toFirst);

        compare(left.shift, -left.travel, "the new content starts on the left, by the bell");
        watchSwap(right, left);
    }

    function test_a_swap_takes_about_a_quarter_second() {
        openFully(left);
        const started = Date.now();

        switchTo(left, right, false);
        tryCompare(right, "presence", 1, suite.swapTime * 4);
        tryCompare(right, "shift", 0, suite.swapTime * 4);
        const took = Date.now() - started;

        verify(took >= K.Tokens.duration.short, `not before short: ${took} ms`);
        verify(took <= suite.swapTime * 2, `about 250 ms: ${took} ms`);
    }

    function test_switching_back_mid_swap_ends_on_the_first_content() {
        openFully(left);
        switchTo(left, right, false);
        wait(K.Tokens.duration.micro);

        switchTo(right, left, false);

        watchSwap(right, left);
        compare(left.progress, 1);
    }

    function test_asked_for_again_before_the_new_content_shows_the_old_one_turns_back() {
        openFully(left);
        switchTo(left, right, false);
        wait(K.Tokens.duration.micro / 4);
        const part = left.presence;
        verify(part > 0 && part < 1, `part-way out: ${part}`);

        switchTo(right, left, false);

        verify(left.presence >= part, `no blink: ${left.presence} after ${part}`);
        verify(!right.active, "the other content never showed");
        compare(left.path !== "", true, "the surface stays drawn");
        let last = left.presence;
        const ends = Date.now() + suite.swapTime * 4;
        while (Date.now() < ends && !(left.presence === 1 && left.shift === 0)) {
            verify(left.presence >= last, "it only comes back");
            last = left.presence;
            wait(4);
        }
        compare(left.presence, 1);
        compare(left.shift, 0);
    }

    function test_with_reduce_motion_the_content_swaps_at_once() {
        openFully(left);
        K.Motion.reduced = true;

        switchTo(left, right, false);

        verify(!left.active, "the old content gone at once");
        verify(!left.leaving);
        compare(right.progress, 1);
        compare(right.shift, 0);
        compare(right.presence, 1);
    }

    function test_closing_mid_swap_closes_the_surface() {
        openFully(left);
        switchTo(left, right, false);
        wait(K.Tokens.duration.micro / 2);

        right.shown = false;

        tryCompare(right, "active", false, K.Tokens.duration.medium * 4);
        tryCompare(left, "active", false, suite.swapTime * 4);
    }

    function test_any_twin_takes_over_from_any_other_from_its_buttons_side_data() {
        return [
            {
                tag: "from the leftmost to the rightmost",
                from: "left",
                to: "far"
            },
            {
                tag: "from the rightmost to the leftmost",
                from: "far",
                to: "left"
            },
            {
                tag: "between the two on the right",
                from: "right",
                to: "far"
            },
            {
                tag: "and back",
                from: "far",
                to: "right"
            }
        ];
    }

    function test_any_twin_takes_over_from_any_other_from_its_buttons_side(data) {
        const sheets = {
            left: left,
            right: right,
            far: far
        };
        const from = sheets[data.from];
        const to = sheets[data.to];
        openFully(from);

        switchTo(from, to, false);

        compare(to.progress, 1, "the surface where it was");
        compare(to.shift, Math.sign(to.side - from.side) * to.travel, "the new content starts on its button's side");
        watchSwap(from, to);
    }

    // The middle one is still on its way in from the right when the third takes over, so its
    // content starts right of its place: what holds is that it only ever moves left from there.
    function test_a_third_asked_for_mid_swap_takes_over_and_the_first_goes_at_once() {
        openFully(left);
        switchTo(left, right, false);
        wait(K.Tokens.duration.micro / 2);

        switchTo(right, far, false);

        verify(!left.active, "the first gone");
        verify(!left.leaving, "the first settled");
        const ends = Date.now() + suite.swapTime * 4;
        let last = right.shift;
        let backwards = 0;
        let both = 0;
        while (Date.now() < ends && !(far.shift === 0 && far.presence === 1 && !right.active)) {
            if (right.active && right.shift > last)
                backwards++;
            if (right.active && right.presence > 0 && far.presence > 0)
                both++;
            last = right.shift;
            compare(far.progress, 1, "the surface stays open");
            compare(right.path, "", "only one surface is drawn");
            wait(4);
        }
        compare(backwards, 0, "the middle one's content only moves left, away from the third's button");
        compare(both, 0, "the two contents are never drawn together");
        compare(far.shift, 0);
        compare(far.presence, 1);
        verify(!right.active, "the middle one gone");
    }
}
