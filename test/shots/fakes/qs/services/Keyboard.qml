pragma Singleton
import QtQuick

QtObject {
    id: root

    property var layouts: ["us", "de"]
    property int active: 0
    readonly property bool several: layouts.length > 1
    readonly property string code: (layouts[active] ?? "").toUpperCase()

    signal reveal

    function show(): void {
        root.reveal();
    }

    function next(): void {
        root.active = (root.active + 1) % root.layouts.length;
    }

    function parse(text: string): void {
    }
}
