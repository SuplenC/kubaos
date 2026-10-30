import QtQuick
import QtTest
import KubaOS.Components as K

// A flow lays its children out as iced's wrapping row: side by side `gap` apart, a new line `gap`
// under the last where the next child doesn't fit, as wide as its widest line.
TestCase {
    id: suite

    name: "Flow"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 200

    K.Stack {
        id: page

        width: suite.width
        fill: true

        K.Grid {
            id: cell

            columns: 1

            K.Flow {
                id: flow

                gap: K.Tokens.space.s8

                Repeater {
                    model: ["English (US)", "Italian", "German", "Japanese", "Polish", "French", "Spanish", "Portuguese (Brazil)"]

                    K.Chip {
                        required property string modelData

                        text: modelData
                    }
                }
            }
        }
    }

    function chips(): list<Item> {
        return Array.from(flow.children).filter(kid => kid.text !== undefined);
    }

    function test_children_wrap_onto_a_new_line_where_the_next_does_not_fit() {
        page.relayout();
        const chips = suite.chips();
        let x = 0;
        let y = 0;

        for (const chip of chips) {
            if (x > 0 && x + chip.width > cell.width) {
                x = 0;
                y += chip.height + K.Tokens.space.s8;
            }
            compare(chip.x, x, chip.text);
            compare(chip.y, y, chip.text);
            x += chip.width + K.Tokens.space.s8;
        }
        verify(y > 0, "the chips take more than one line");
    }

    function test_it_is_as_wide_as_its_widest_line_and_as_tall_as_its_lines() {
        page.relayout();
        const chips = suite.chips();
        const right = Math.max(...chips.map(chip => chip.x + chip.width));
        const bottom = Math.max(...chips.map(chip => chip.y + chip.height));

        compare(flow.width, right);
        compare(flow.height, bottom);
        verify(flow.width <= cell.width);
    }
}
