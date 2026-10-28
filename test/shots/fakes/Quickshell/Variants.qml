import QtQuick

// Stand-in for Variants: it never builds its delegate; the drivers build what they shoot.
QtObject {
    property var model
    default property Component delegate
}
