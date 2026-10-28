import QtQuick

// A frozen evening, Tuesday 27 October 2026 at 21:47:32, so every shot shows the same time.
QtObject {
    enum Precision {
        Hours,
        Minutes,
        Seconds
    }

    property int precision
    property bool enabled: true
    readonly property date date: new Date(2026, 9, 27, 21, 47, 32)
    readonly property int hours: 21
    readonly property int minutes: 47
    readonly property int seconds: 32
}
