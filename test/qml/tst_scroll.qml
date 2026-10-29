import QtQuick
import QtTest
import KubaOS.Components as K

// A page's or a panel's scroll: it has no edge of its own, so it clips only past the room it
// reaches into around its box, with its content lined up with the box.
TestCase {
    id: suite

    name: "Scroll"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 400

    K.Scroll {
        id: scroll

        // design-allow(size): a test's fixture, not a design value.
        x: 40
        // design-allow(size): a test's fixture, not a design value.
        y: 40
        // design-allow(size): a test's fixture, not a design value.
        width: 200
        // design-allow(size): a test's fixture, not a design value.
        height: 120

        Rectangle {
            id: content

            width: parent.width
            // design-allow(size): a test's fixture, not a design value.
            height: 300
        }

        // What a panel shows only at times, under the rest: a problem's note.
        Rectangle {
            id: hidden

            y: content.height
            width: parent.width
            // design-allow(size): a test's fixture, not a design value.
            height: 100
            visible: false
        }
    }

    // The box the scroll clips at, in the scroll's own coordinates.
    function clipped(): rect {
        const clip = Array.from(scroll.children).find(child => child.clip === true);
        verify(clip, "the scroll clips somewhere");
        const at = clip.mapToItem(scroll, 0, 0);
        return Qt.rect(at.x, at.y, clip.width, clip.height);
    }

    function init() {
        scroll.roomAbove = K.Tokens.focus.far;
        scroll.roomBelow = K.Tokens.focus.far;
        scroll.contentY = 0;
        tryCompare(scroll, "contentY", 0, 1000);
    }

    function test_its_content_lines_up_with_its_box() {
        const at = content.mapToItem(scroll, 0, 0);

        compare(at.x, 0);
        compare(at.y, 0);
        compare(content.width, scroll.width);
    }

    function test_it_reaches_past_its_box_as_far_as_a_glow_does() {
        const far = K.Tokens.focus.far;
        const clip = clipped();

        compare(clip.x, -far, "left");
        compare(clip.y, -far, "top");
        compare(clip.x + clip.width, scroll.width + far, "right");
        compare(clip.y + clip.height, scroll.height + far, "bottom");
    }

    function test_it_reaches_no_farther_than_the_room_left_to_its_neighbours() {
        scroll.roomAbove = K.Tokens.space.s12;
        scroll.roomBelow = K.Tokens.space.s16;
        const clip = clipped();

        compare(clip.y, -K.Tokens.space.s12, "top");
        compare(clip.y + clip.height, scroll.height + K.Tokens.space.s16, "bottom");
        compare(content.mapToItem(scroll, 0, 0).y, 0, "the content still lines up");
    }

    function test_the_room_it_reaches_into_adds_nothing_to_how_far_it_scrolls() {
        scroll.roomAbove = K.Tokens.space.s32;

        compare(scroll.travel, content.height - scroll.height);
    }

    function test_a_hidden_child_gives_it_nothing_to_scroll_and_a_shown_one_does() {
        compare(scroll.travel, content.height - scroll.height, "hidden");

        hidden.visible = true;
        compare(scroll.travel, content.height + hidden.height - scroll.height, "shown");
        hidden.visible = false;
    }

    function test_content_scrolled_past_its_box_shows_in_that_room_and_stops_at_its_end() {
        scroll.contentY = scroll.travel;
        tryCompare(scroll, "contentY", scroll.travel, 1000);

        const at = content.mapToItem(scroll, 0, 0);
        compare(at.y + content.height, scroll.height, "the end of the content at the box's foot");
        verify(at.y < clipped().y, "the content above the room is past the clip");
    }

    function test_content_that_shrinks_under_it_brings_it_back_to_its_end() {
        hidden.visible = true;
        scroll.contentY = scroll.travel;
        tryCompare(scroll, "contentY", scroll.travel, 1000);

        hidden.visible = false;

        tryCompare(scroll, "contentY", scroll.travel, 1000);
    }

    function test_a_glide_to_where_its_end_was_ends_at_its_end_once_the_content_shrinks() {
        hidden.visible = true;
        const before = scroll.travel;

        scroll.contentY = before;
        hidden.visible = false;

        verify(scroll.travel < before);
        tryCompare(scroll, "contentY", scroll.travel, 1000);
        wait(K.Tokens.duration.medium * 2);
        compare(scroll.contentY, scroll.travel, "it rests at its end, not where its end was");
    }

    function test_it_tells_the_geometry_check_how_far_it_reaches_above_below_and_on_its_left() {
        scroll.roomAbove = K.Tokens.space.s12;

        compare(scroll.probe.role, "scroll");
        compare(scroll.probe.reach.above, K.Tokens.space.s12);
        compare(scroll.probe.reach.below, K.Tokens.focus.far);
        compare(scroll.probe.reach.left, K.Tokens.focus.far);
    }
}
