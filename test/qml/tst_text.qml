import QtQuick
import QtTest
import KubaOS.Components as K

// What keeps `K.Text` cheap enough for the Command menu's keystroke: a text is measured once, in
// the shared metrics, and key hints past their room are left out whole.
TestCase {
    id: suite

    name: "Text"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 600
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 200

    Component {
        id: text

        K.Text {
            style: K.Tokens.type.body_strong
            maximumLines: 1
        }
    }

    Component {
        id: hints

        K.KeyHints {
            text: "↵ Open · Tab Enter app · ⌃K Actions"
        }
    }

    function init() {
        K.Advances.widths.clear();
    }

    function test_a_text_that_fits_is_measured_once_as_it_is_made() {
        const before = K.Advances.widths.size;

        const made = createTemporaryObject(text, suite, {
            text: `Measured once ${Date.now()}`
        });

        verify(made.implicitWidth > 0);
        compare(K.Advances.widths.size, before + 1);
    }

    function test_the_same_words_again_are_not_measured_again() {
        const words = `Measured before ${Date.now()}`;
        createTemporaryObject(text, suite, {
            text: words
        });
        const before = K.Advances.widths.size;

        const again = createTemporaryObject(text, suite, {
            text: words
        });

        verify(again.implicitWidth > 0);
        compare(K.Advances.widths.size, before);
    }

    function test_a_text_measures_as_its_words_do_rounded_up_to_a_pixel() {
        const made = createTemporaryObject(text, suite, {
            text: "Firefox Web Browser"
        });

        const words = K.Advances.of(K.Tokens.type.body_strong, false, "Firefox Web Browser");
        compare(made.implicitWidth, K.Look.snapUp(words, 1));
    }

    function test_markup_is_drawn_where_the_words_keep_to_one_line() {
        const made = createTemporaryObject(text, suite, {
            text: "Firefox",
            markup: "<font color=\"red\">Fi</font>refox"
        });

        verify(made.lit);
        compare(made.implicitWidth, K.Look.snapUp(K.Advances.of(K.Tokens.type.body_strong, false, "Firefox"), 1));
    }

    function test_a_word_wider_than_the_room_is_cut_on_its_one_line() {
        const made = createTemporaryObject(text, suite, {
            text: "/home/someone/Projects/a/very/long/path/without/a/single/space/in/it",
            maximumLines: 3,
            room: 120
        });
        const drawn = made.children[0];

        compare(made.implicitHeight, K.Tokens.type.body_strong.line);
        compare(drawn.lineCount, 1);
        verify(drawn.truncated);
    }

    // The hints' delegates, in order; the Repeater is a child of the hints too.
    function hintsOf(line: Item): var {
        return Array.from(line.children).filter(child => child.index !== undefined);
    }

    function test_hints_past_the_room_are_left_out_whole() {
        const all = createTemporaryObject(hints, suite);

        const cut = createTemporaryObject(hints, suite, {
            room: all.implicitWidth - 1
        });

        compare(suite.hintsOf(cut).map(hint => hint.visible), [true, true, false]);
        const second = suite.hintsOf(cut)[1];
        compare(cut.implicitWidth, second.x + second.width);
    }
}
