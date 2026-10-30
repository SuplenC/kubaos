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
        id: unfolding

        // As a notification's card unfolds: as many lines as the words wrap to.
        K.Text {
            id: unfolded

            style: K.Tokens.type.body
            room: 120
            maximumLines: Math.max(unfolded.laid.length, 1)
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

    function test_texts_in_one_style_draw_in_the_one_font_made_from_it() {
        createTemporaryObject(text, suite, {
            text: "First"
        });
        const styles = K.Advances.styles.size;

        const second = createTemporaryObject(text, suite, {
            text: "Second"
        });

        compare(K.Advances.styles.size, styles);
        const font = second.children[0].font;
        compare(font.family, K.Tokens.type.body_strong.family);
        compare(font.weight, K.Tokens.type.body_strong.weight);
        compare(font.pixelSize, K.Tokens.type.body_strong.size);
        compare(font.kerning, false);
        compare(font.preferShaping, false);
        compare(font.hintingPreference, Font.PreferNoHinting);
    }

    function test_words_past_ascii_and_tabular_figures_draw_shaped() {
        const accented = createTemporaryObject(text, suite, {
            text: "Zażółć"
        });
        const figures = createTemporaryObject(text, suite, {
            text: "12:30",
            style: K.Tokens.type.label_number
        });

        compare(accented.children[0].font.kerning, true);
        compare(accented.children[0].font.preferShaping, true);
        compare(figures.children[0].font.preferShaping, true);
        compare(figures.children[0].font.features, {
            "tnum": 1
        });
    }

    function test_a_text_draws_on_the_surface_colour_unless_given_one() {
        const plain = createTemporaryObject(text, suite, {
            text: "Plain"
        });
        const coloured = createTemporaryObject(text, suite, {
            text: "Coloured",
            color: K.Tokens.color.primary
        });

        compare(plain.children[0].color, K.Tokens.color.on_surface);
        compare(coloured.children[0].color, K.Tokens.color.primary);
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

    function test_a_text_can_keep_as_many_lines_as_its_words_wrap_to() {
        const made = createTemporaryObject(unfolding, suite, {
            text: "Words enough to wrap onto a few lines in the room"
        });

        verify(made.laid.length > 1);
        compare(made.measured.lines, made.laid.length);
        verify(!made.cut);
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

    // The hints' delegates, in order, in the hints' one line; the Repeater is a child there too.
    function hintsOf(hints: Item): var {
        return Array.from(hints.children[0].children).filter(child => child.index !== undefined);
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
