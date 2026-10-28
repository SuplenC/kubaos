import QtQuick

// Stand-in for QsMenuOpener: the entries of `menu`, a stand-in menu with `entries`, as
// `children.values`.
QtObject {
    id: opener

    property var menu: null
    readonly property QtObject children: QtObject {
        readonly property var values: opener.menu?.entries ?? []
    }
}
