pragma Singleton
import QtQuick

// Stand-in for Power: Balanced, of the five stops.
QtObject {
    id: root

    readonly property string stateFile: "/nonexistent/run/kubaos-power.json"
    readonly property var stops: [
        {
            id: "super-saver",
            title: "Super Saver",
            icon: "bedtime"
        },
        {
            id: "power-saver",
            title: "Power Saver",
            icon: "eco"
        },
        {
            id: "balanced",
            title: "Balanced",
            icon: "balance"
        },
        {
            id: "performance",
            title: "Performance",
            icon: "bolt"
        },
        {
            id: "max-performance",
            title: "Max Performance",
            icon: "rocket_launch"
        }
    ]
    readonly property var titles: stops.map(stop => stop.title)
    readonly property bool service: true
    property int index: 2
    readonly property string title: stops[index].title
    readonly property string icon: stops[index].icon
    readonly property bool quiet: false

    function set(to: int): void {
        root.index = to;
    }

    function setId(id: string): void {
    }
}
