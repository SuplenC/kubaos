pragma Singleton
import QtQuick

// Stand-in for Panels: the drivers open a panel by setting `current` and `screen`.
QtObject {
    id: root

    property string current: ""
    property var screen: null

    signal closed

    function open(name: string): void {
        root.current = name;
    }

    function close(): void {
        console.log("panels: close");
    }

    function toggle(name: string): void {
        root.current = root.current === name ? "" : name;
    }
}
