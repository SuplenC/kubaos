import QtQuick

// Stand-in for LazyLoader: what it holds is built only while `active`, as with the real one.
Loader {
    property bool loading: false
    default property Component component

    sourceComponent: active ? component : null
}
