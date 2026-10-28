import QtQuick

// A plain object, as Quickshell's Scope is: a window declared in an Item would wait for that
// Item's own window before it shows.
QtObject {
    default property list<QtObject> data
}
