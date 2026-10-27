pragma Singleton

import QtQuick

// How many times a layout has arranged its children since `count` was last reset. The QML
// specimen reads it to hold a pass over the page to a bound linear in its layouts.
QtObject {
    property int count: 0
}
