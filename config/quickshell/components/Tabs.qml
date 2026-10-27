import QtQuick
import KubaOS.Components as K

// A row of tabs: small buttons, the picked one tonal and the rest text, each with an optional
// symbol before its label. The owner keeps the state: bind `currentIndex`, handle `picked()`.
// The row is one stop for Tab; Left and Right, or h and l, pick the neighbouring tab, and the
// focus ring goes round the picked one.
K.Row {
    id: root

    property list<string> labels
    // A symbol per label, or none.
    property list<string> icons
    property int currentIndex: 0

    signal picked(int index)

    function step(delta: int): void {
        root.picked((root.currentIndex + delta + root.labels.length) % root.labels.length);
    }

    gap: K.Tokens.space.s4
    align: "centre"
    activeFocusOnTab: true
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
            focused: root.activeFocus && index === root.currentIndex
            onActivated: root.picked(index)
        }
    }
}
