import QtQuick
import QtTest
import Kotodama
import KubaOS.Components as K
import "../../config/quickshell/modules/menu"

// A layout whose child turns out to fill only as it is laid out fills too, from its first pass:
// a Command menu form's fields take the form's width, whichever of its layouts is laid out first.
TestCase {
    id: suite

    name: "FormWidth"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 600
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    // design-allow(size): a test's fixture, not a design value.
    readonly property int formWidth: 480
    readonly property var fields: Array.from({
            length: 6
        }, (_, n) => n % 3 === 2 ? {
                label: `Option ${n + 1}`,
                kind: "checkbox",
                value: false
            } : {
                label: `Field ${n + 1}`,
                kind: "text",
                placeholder: "Type here",
                value: ""
            })

    KotodamaSearch {
        id: engine
    }

    Component {
        id: formView

        FormView {
            width: suite.formWidth
            // design-allow(size): a test's fixture, not a design value.
            height: 400
            search: engine
            fields: suite.fields
            submitLabel: "Save"
        }
    }

    Component {
        id: labelledStack

        K.Stack {
            width: suite.formWidth
            fill: true

            K.Labelled {
                label: "Name"

                K.Field {
                    placeholder: "Type here"
                }
            }
        }
    }

    // The items under `item` for which `test` holds, depth first.
    function all(item, test) {
        const found = test(item) ? [item] : [];
        for (const kid of item.children)
            found.push(...all(kid, test));
        return found;
    }

    function isLayout(item) {
        return typeof item.relayout === "function" && item.laid !== undefined;
    }

    function isTextField(item) {
        return item.placeholder !== undefined && item.secret !== undefined && item.visible;
    }

    function test_a_labelled_field_laid_out_before_its_own_layouts_takes_the_stack_width() {
        const stack = createTemporaryObject(labelledStack, suite);
        const labelled = stack.children[0];

        stack.relayout();
        waitForRendering(stack);

        verify(labelled.fillWidth, "the field fills, so the labelled field around it does");
        compare(labelled.width, suite.formWidth);
        compare(all(labelled, isTextField)[0].width, suite.formWidth);
    }

    function test_a_form_laid_out_from_the_outside_in_gives_every_field_its_width() {
        const form = createTemporaryObject(formView, suite);
        const column = all(form, isLayout)[0];

        column.relayout();
        waitForRendering(form);

        const fields = all(column, isTextField);
        compare(fields.length, 4);
        for (const field of fields)
            compare(field.width, column.width, "a field is as wide as the form's column");
    }
}
