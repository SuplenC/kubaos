import QtQuick
import qs.theme
import qs.components

// A month as the engine lays it out: six weeks, each led by its ISO week
// number, the first weekday from the settings. Today is filled, the picked
// day ringed, a holiday dotted with its name on hover; days of the months
// around are dimmed. Clicking the title swaps in the year's twelve months.
// The wheel moves a month; the new month slides in on the glide.
Item {
    id: root

    // `{"month": ...}` from the engine: year, month, title, weekdays, weeks.
    property var month: null
    property string selected: ""
    property bool yearView: false
    // +1 when the month moved forward, -1 back: where the new one comes from.
    property int direction: 0
    required property int glide
    required property list<real> arrive

    readonly property int cell: 40
    readonly property int weekColumn: 32
    readonly property var monthNames: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]

    signal picked(string date)
    signal moved(int months)
    signal pickedMonth(int month)

    implicitWidth: weekColumn + cell * 7
    implicitHeight: header.height + Theme.spacing.sm + names.height + cell * 6

    // A month the user moved to slides in; one read again (a new setting,
    // midnight) just shows.
    onMonthChanged: {
        if (direction !== 0) {
            slideFrom.x = direction * cell * 1.5;
            slide.restart();
        }
        direction = 0;
    }

    Item {
        id: header

        width: parent.width
        height: 32

        IconButton {
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            icon: "chevron_left"
            size: 30
            tooltip: root.yearView ? "Last year" : "Last month"
            onClicked: root.moved(root.yearView ? -12 : -1)
        }

        Rectangle {
            id: titleChip

            anchors.centerIn: parent
            width: titleText.implicitWidth + Theme.spacing.lg * 2
            height: 30
            radius: Theme.rounding.full
            color: root.yearView ? Theme.m3secondaryContainer : "transparent"

            Behavior on color {
                CAnim {}
            }

            StyledText {
                id: titleText

                anchors.centerIn: parent
                text: root.yearView ? String(root.month?.year ?? "") : (root.month?.title ?? "")
                font.pixelSize: Theme.font.lg
                font.weight: Font.DemiBold
                color: root.yearView ? Theme.m3onSecondaryContainer : Theme.m3onSurface
            }

            StateLayer {
                onClicked: root.yearView = !root.yearView
            }
        }

        IconButton {
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            icon: "chevron_right"
            size: 30
            tooltip: root.yearView ? "Next year" : "Next month"
            onClicked: root.moved(root.yearView ? 12 : 1)
        }
    }

    Item {
        id: body

        y: header.height + Theme.spacing.sm
        width: parent.width
        height: names.height + root.cell * 6
        clip: true

        WheelHandler {
            acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
            onWheel: event => {
                const steps = event.angleDelta.y !== 0 ? event.angleDelta.y : -event.angleDelta.x;
                if (steps !== 0)
                    root.moved((steps > 0 ? -1 : 1) * (root.yearView ? 12 : 1));
            }
        }

        // The days: weekday names, then a row a week.
        Item {
            id: days

            width: parent.width
            height: parent.height
            opacity: root.yearView ? 0 : 1
            visible: opacity > 0
            scale: root.yearView ? 0.94 : 1

            Behavior on opacity {
                Anim {
                    duration: root.glide
                    easing.bezierCurve: root.arrive
                }
            }

            Behavior on scale {
                Anim {
                    duration: root.glide
                    easing.bezierCurve: root.arrive
                }
            }

            Row {
                id: names

                height: 24

                StyledText {
                    width: root.weekColumn
                    height: parent.height
                    horizontalAlignment: Text.AlignHCenter
                    text: "Wk"
                    font.pixelSize: Theme.font.xs
                    color: Theme.m3outline
                }

                Repeater {
                    model: root.month?.weekdays ?? []

                    StyledText {
                        required property string modelData
                        required property int index

                        width: root.cell
                        height: names.height
                        horizontalAlignment: Text.AlignHCenter
                        text: modelData.slice(0, 2)
                        font.pixelSize: Theme.font.sm
                        font.weight: Font.Medium
                        color: modelData.startsWith("Sat") || modelData.startsWith("Sun") ? Theme.m3tertiary : Theme.m3onSurfaceVariant
                    }
                }
            }

            Column {
                id: weeks

                y: names.height

                transform: Translate {
                    id: shift
                }

                Repeater {
                    model: root.month?.weeks ?? []

                    Row {
                        id: week

                        required property var modelData

                        height: root.cell

                        StyledText {
                            width: root.weekColumn
                            height: root.cell
                            horizontalAlignment: Text.AlignHCenter
                            text: week.modelData.number
                            font.pixelSize: Theme.font.xs
                            font.features: Theme.font.tabular
                            color: Theme.m3outline
                        }

                        Repeater {
                            model: week.modelData.days

                            DayCell {}
                        }
                    }
                }
            }
        }

        // The year: twelve months, the one shown filled.
        Grid {
            id: year

            anchors.centerIn: parent
            columns: 3
            spacing: Theme.spacing.sm
            opacity: root.yearView ? 1 : 0
            visible: opacity > 0
            scale: root.yearView ? 1 : 1.06

            Behavior on opacity {
                Anim {
                    duration: root.glide
                    easing.bezierCurve: root.arrive
                }
            }

            Behavior on scale {
                Anim {
                    duration: root.glide
                    easing.bezierCurve: root.arrive
                }
            }

            Repeater {
                model: 12

                Rectangle {
                    id: monthChip

                    required property int index
                    readonly property bool current: (root.month?.month ?? 0) === index + 1

                    width: (root.width - Theme.spacing.sm * 2) / 3 - Theme.spacing.sm
                    height: 44
                    radius: Theme.rounding.md
                    color: current ? Theme.m3primary : Theme.m3surfaceContainerHigh

                    StyledText {
                        anchors.centerIn: parent
                        text: root.monthNames[monthChip.index]
                        font.weight: Font.Medium
                        color: monthChip.current ? Theme.m3onPrimary : Theme.m3onSurface
                    }

                    StateLayer {
                        color: monthChip.current ? Theme.m3onPrimary : Theme.m3onSurface
                        onClicked: {
                            root.pickedMonth(monthChip.index + 1);
                            root.yearView = false;
                        }
                    }
                }
            }
        }
    }

    QtObject {
        id: slideFrom

        property real x: 0
    }

    // The new month comes in from the side it lies on, on the glide.
    ParallelAnimation {
        id: slide

        NumberAnimation {
            target: shift
            property: "x"
            from: slideFrom.x
            to: 0
            duration: root.glide
            easing.type: Easing.BezierSpline
            easing.bezierCurve: root.arrive
        }

        NumberAnimation {
            target: weeks
            property: "opacity"
            from: 0
            to: 1
            duration: root.glide
            easing.type: Easing.BezierSpline
            easing.bezierCurve: root.arrive
        }
    }

    component DayCell: Item {
        id: cell

        required property var modelData
        readonly property bool isSelected: root.selected === modelData.date

        width: root.cell
        height: root.cell

        Rectangle {
            anchors.centerIn: parent
            width: root.cell - 6
            height: root.cell - 6
            radius: Theme.rounding.full
            color: cell.modelData.today ? Theme.m3primary : "transparent"
            border.width: cell.isSelected && !cell.modelData.today ? 1.5 : 0
            border.color: Theme.m3primary

            StyledText {
                anchors.centerIn: parent
                text: cell.modelData.day
                font.features: Theme.font.tabular
                font.weight: cell.modelData.today ? Font.Bold : Font.Normal
                color: cell.modelData.today ? Theme.m3onPrimary : cell.modelData.holiday ? Theme.m3tertiary : cell.modelData.weekend ? Theme.m3onSurfaceVariant : Theme.m3onSurface
                opacity: cell.modelData.inMonth ? 1 : 0.38
            }

            Rectangle {
                visible: !!cell.modelData.holiday
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 3
                width: 4
                height: 4
                radius: 2
                color: cell.modelData.today ? Theme.m3onPrimary : Theme.m3tertiary
                opacity: cell.modelData.inMonth ? 1 : 0.5
            }

            StateLayer {
                id: hover

                color: cell.modelData.today ? Theme.m3onPrimary : Theme.m3onSurface
                onClicked: root.picked(cell.modelData.date)
            }
        }

        Tooltip {
            text: cell.modelData.holiday ?? ""
            shown: text !== "" && hover.containsMouse
        }
    }
}
