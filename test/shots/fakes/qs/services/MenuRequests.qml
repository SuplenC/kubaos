pragma Singleton
import QtQuick

QtObject {
    id: root

    signal toggleRequested
    signal openRequested(string query)

    function toggle(): void {
        root.toggleRequested();
    }

    function open(query: string): void {
        root.openRequested(query);
    }
}
