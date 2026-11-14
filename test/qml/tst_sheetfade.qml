import QtQuick
import QtTest
import KubaOS.Components as K

// A sheet with reduce motion: nothing moves. It stands open where it goes and fades in, surface
// and content together, and fades out before it closes.
TestCase {
    id: suite

    name: "SheetFade"
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
        id: sheet

        frame: frame
        edge: "top"
        contentWidth: K.Tokens.size.panel
        contentHeight: K.Tokens.size.panel
    }

    function init() {
        K.Motion.reduced = true;
        sheet.shown = false;
        tryCompare(sheet, "active", false, K.Tokens.duration.long * 4);
    }

    function cleanup() {
        K.Motion.reduced = false;
        sheet.shown = false;
        tryCompare(sheet, "active", false, K.Tokens.duration.long * 4);
    }

    function test_with_reduce_motion_it_opens_where_it_goes_and_fades_in() {
        sheet.shown = true;

        compare(sheet.progress, 1, "the surface whole at once");
        verify(sheet.path !== "");
        wait(K.Tokens.duration.short / 2);
        verify(sheet.fade > 0 && sheet.fade < 1, `part-way in: ${sheet.fade}`);
        compare(sheet.progress, 1, "it never grows");
        tryCompare(sheet, "fade", 1, K.Tokens.duration.short * 4);
    }

    function test_with_reduce_motion_it_fades_out_then_closes() {
        sheet.shown = true;
        tryCompare(sheet, "fade", 1, K.Tokens.duration.short * 4);

        sheet.shown = false;
        wait(K.Tokens.duration.short / 2);

        verify(sheet.fade > 0 && sheet.fade < 1, `part-way out: ${sheet.fade}`);
        compare(sheet.progress, 1, "it never shrinks");
        tryCompare(sheet, "active", false, K.Tokens.duration.short * 4);
        compare(sheet.fade, 1, "ready to open again");
    }

    function test_asked_for_again_while_fading_out_it_fades_back_from_where_it_is() {
        sheet.shown = true;
        tryCompare(sheet, "fade", 1, K.Tokens.duration.short * 4);
        sheet.shown = false;
        wait(K.Tokens.duration.short / 2);
        const part = sheet.fade;

        sheet.shown = true;

        verify(sheet.fade >= part, `no blink: ${sheet.fade} after ${part}`);
        tryCompare(sheet, "fade", 1, K.Tokens.duration.short * 4);
        verify(sheet.active);
    }

    function test_without_reduce_motion_it_grows_at_full_fade() {
        K.Motion.reduced = false;

        sheet.shown = true;
        wait(K.Tokens.duration.micro);

        verify(sheet.progress > 0 && sheet.progress < 1, `growing: ${sheet.progress}`);
        compare(sheet.fade, 1);
    }
}
