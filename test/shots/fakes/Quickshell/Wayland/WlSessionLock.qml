import QtQuick

QtObject {
    property bool locked
    readonly property bool secure: locked
    default property Component surface
}
