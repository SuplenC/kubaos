import QtQuick

// The type, for declarations that name it; `Hyprland.workspaces` holds plain objects.
QtObject {
    property string name
    property var monitor: null
    property bool active: false
    property bool focused: false
    property bool urgent: false
    property bool hasFullscreen: false
}
