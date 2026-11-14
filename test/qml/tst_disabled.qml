import QtQuick
import QtTest
import KubaOS.Components as K

// A disabled part is no stop for Tab and takes no click: Tab goes from the button before them to
// the one after, and a click on each sends nothing.
TestCase {
    id: suite

    name: "Disabled"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 800
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 800

    K.Stack {
        id: page

        width: suite.width
        fill: true

        K.Button {
            id: before

            text: "Before"
        }

        K.Field {
            id: field

            status: "Disabled"
            text: "kuba"
        }

        K.Tabs {
            id: tabs

            enabled: false
            labels: ["Always", "On AC", "Never"]
        }

        K.Chip {
            id: chip

            enabled: false
            text: "Italian"
            pressable: true
            removable: true
        }

        K.Collapsible {
            id: collapsible

            enabled: false
            title: "firefox · pid 4127"
        }

        K.ChoiceCard {
            id: card

            enabled: false

            K.Text {
                text: "Install alongside"
            }
        }

        K.Grid {
            columns: 4

            K.PictureChoice {
                id: picture

                enabled: false
                name: "Golden Gai"
            }
        }

        K.Button {
            id: after

            text: "After"
        }
    }

    SignalSpy {
        id: tabsPicked

        target: tabs
        signalName: "picked"
    }

    SignalSpy {
        id: chipActivated

        target: chip
        signalName: "activated"
    }

    SignalSpy {
        id: chipRemoved

        target: chip
        signalName: "removed"
    }

    SignalSpy {
        id: toggled

        target: collapsible
        signalName: "toggled"
    }

    SignalSpy {
        id: cardPicked

        target: card
        signalName: "picked"
    }

    SignalSpy {
        id: picturePicked

        target: picture
        signalName: "activated"
    }

    function test_tab_passes_every_disabled_part() {
        page.relayout();
        before.forceActiveFocus();

        keyClick(Qt.Key_Tab);

        verify(after.activeFocus, "the focus is on the button after them");
    }

    function test_a_click_on_a_disabled_part_sends_nothing() {
        page.relayout();
        for (const target of [field, tabs, chip, collapsible, card, picture])
            mouseClick(target);
        mouseClick(chip, chip.width - K.Tokens.space.s8, chip.height / 2);

        compare(tabsPicked.count, 0);
        compare(chipActivated.count, 0);
        compare(chipRemoved.count, 0);
        compare(toggled.count, 0);
        compare(cardPicked.count, 0);
        compare(picturePicked.count, 0);
        verify(!field.focused, "the field took no focus");
    }
}
