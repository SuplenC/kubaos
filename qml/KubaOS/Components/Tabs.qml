import QtQuick
import KubaOS.Components as K

// One of a few views or values, picked from a row of small buttons: `kubaos-ui`'s `tabs::tabs`.
// The picked one is tonal and the rest text, 4 apart, each with an optional symbol before its
// label. The same row is a setting's segmented control (Always, On AC, Never). The owner keeps the
// state: bind `currentIndex`, handle `picked()`. The row is one stop for Tab; Left and Right, or
// h and l, pick the neighbouring choice, and the focus tube lights the picked one. Disabled
// (`enabled: false`), every choice dims, the picked one still tonal, and the row takes no press
// and is no stop for Tab.
Item {
    id: root

    property list<string> labels
    // A symbol per label, or none.
    property list<string> icons
    property int currentIndex: 0
    // The widest the row may grow, offered by its parent.
    property alias room: line.room
    // Whether the keyboard is on it; Qt's focus by default, or the app's where it owns focus.
    property bool focused: root.activeFocus

    signal picked(int index)

    readonly property var probe: ({
            role: "component",
            name: "Tabs",
            variant: "",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitWidth: line.implicitWidth
    implicitHeight: line.implicitHeight
    activeFocusOnTab: root.enabled

    function relayout(): void {
        line.relayout();
    }

    function step(delta: int): void {
        root.picked((root.currentIndex + delta + root.labels.length) % root.labels.length);
    }

    Keys.onPressed: event => {
        const k = event.key;
        if (k === Qt.Key_Left || k === Qt.Key_H)
            root.step(-1);
        else if (k === Qt.Key_Right || k === Qt.Key_L)
            root.step(1);
        else
            return;
        event.accepted = true;
    }

    K.Row {
        id: line

        gap: Tokens.space.s4
        align: "centre"

        Repeater {
            model: root.labels

            K.Button {
                required property string modelData
                required property int index

                text: modelData
                icon: root.icons[index] ?? ""
                kind: index === root.currentIndex ? "Tonal" : "Text"
                size: "Sm"
                activeFocusOnTab: false
                focused: root.focused && index === root.currentIndex
                onActivated: root.picked(index)
            }
        }
    }
}
