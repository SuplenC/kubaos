import QtQuick
import KubaOS.Components as K

// A month as the engine lays it out: six weeks, each led by its ISO week
// number, the first weekday from the settings. Today is filled, the picked
// day ringed, a holiday dotted with its name in a hint on hover; days of the
// months around are dimmed. Clicking the title swaps in the year's twelve
// months. The wheel moves a month; the new month slides in.
//
// A day is a control, as tall as a regular button, its disc a small one; the
// week numbers take a small control's width.
Item {
    id: root

    // `{"month": ...}` from the engine: year, month, title, weekdays, weeks.
    property var month: null
    property string selected: ""
    property bool yearView: false
    // +1 when the month moved forward, -1 back: where the new one comes from.
    property int direction: 0
    // The day under the pointer, or null.
    property Item hovered: null

    readonly property int cell: K.Tokens.size.control_md
    readonly property int weekColumn: K.Tokens.size.control_sm
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var monthNames: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]

    signal picked(string date)
    signal moved(int months)
    signal pickedMonth(int month)

    implicitWidth: weekColumn + cell * 7
    implicitHeight: header.height + K.Tokens.space.s8 + body.height

    // A month the user moved to slides in; one read again (a new setting,
    // midnight) just shows.
    onMonthChanged: {
        if (direction !== 0) {
            slideFrom.x = direction * K.Tokens.space.s64;
            slide.restart();
        }
        direction = 0;
    }

    // The months and the year view trade places as focus moves: over `short`, settling in.
    component Glide: NumberAnimation {
        duration: K.Tokens.duration.short
        easing.type: Easing.BezierSpline
        easing.bezierCurve: K.Tokens.easing.emphasized_decel
    }

    Item {
        id: header

        width: parent.width
        height: K.Tokens.size.control_sm

        K.IconButton {
            anchors.left: parent.left
            icon: "chevron_left"
            label: root.yearView ? qsTr("Last year") : qsTr("Last month")
            kind: "Text"
            size: "Sm"
            onActivated: root.moved(root.yearView ? -12 : -1)
        }

        // The title: a pill that opens the year, tonal while it is open.
        K.Pressable {
            id: title

            x: K.Look.snap((parent.width - width) / 2, root.dpr)
            width: K.Look.snapUp(titleText.implicitWidth + K.Tokens.space.s12 * 2, root.dpr)
            height: K.Tokens.size.control_sm
            activeFocusOnTab: false
            onActivated: root.yearView = !root.yearView

            Rectangle {
                anchors.fill: parent
                radius: K.Tokens.radius.full
                color: root.yearView ? K.Tokens.color.secondary_container : "transparent"

                K.StateLayer {
                    radius: parent.radius
                    content: root.yearView ? K.Tokens.color.on_secondary_container : K.Tokens.color.on_surface
                    hovered: title.hovered
                    pressed: title.down
                }
            }

            K.Text {
                id: titleText

                x: K.Tokens.space.s12
                y: K.Look.snap((K.Tokens.size.control_sm - implicitHeight) / 2, root.dpr)
                text: root.yearView ? String(root.month?.year ?? "") : (root.month?.title ?? "")
                style: K.Tokens.type.subheading
                color: root.yearView ? K.Tokens.color.on_secondary_container : K.Tokens.color.on_surface
            }
        }

        K.IconButton {
            anchors.right: parent.right
            icon: "chevron_right"
            label: root.yearView ? qsTr("Next year") : qsTr("Next month")
            kind: "Text"
            size: "Sm"
            onActivated: root.moved(root.yearView ? 12 : 1)
        }
    }

    Item {
        id: body

        y: header.height + K.Tokens.space.s8
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
                Glide {}
            }

            Behavior on scale {
                Glide {}
            }

            Row {
                id: names

                height: K.Tokens.space.s24

                Centred {
                    width: root.weekColumn
                    text: "Wk"
                    style: K.Tokens.type.caption
                    color: K.Tokens.color.outline
                }

                Repeater {
                    model: root.month?.weekdays ?? []

                    Centred {
                        required property string modelData

                        width: root.cell
                        text: modelData.slice(0, 2)
                        style: K.Tokens.type.label
                        color: modelData.startsWith("Sat") || modelData.startsWith("Sun") ? K.Tokens.color.tertiary : K.Tokens.color.on_surface_variant
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

                        Centred {
                            width: root.weekColumn
                            height: root.cell
                            text: String(week.modelData.number)
                            style: K.Tokens.type.caption
                            color: K.Tokens.color.outline
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
        K.Grid {
            id: year

            y: K.Look.snap((parent.height - height) / 2, root.dpr)
            width: parent.width
            height: implicitHeight
            columns: 3
            gap: K.Tokens.space.s8
            opacity: root.yearView ? 1 : 0
            visible: opacity > 0
            scale: root.yearView ? 1 : 1.06

            Behavior on opacity {
                Glide {}
            }

            Behavior on scale {
                Glide {}
            }

            Repeater {
                model: 12

                K.Pressable {
                    id: monthChip

                    required property int index
                    readonly property bool current: (root.month?.month ?? 0) === index + 1
                    readonly property bool fillWidth: true

                    implicitHeight: K.Tokens.size.control_md
                    activeFocusOnTab: false
                    onActivated: {
                        root.pickedMonth(monthChip.index + 1);
                        root.yearView = false;
                    }

                    Rectangle {
                        anchors.fill: parent
                        radius: K.Tokens.radius.control
                        color: monthChip.current ? K.Tokens.color.primary : K.Tokens.color.surface_container_high

                        K.StateLayer {
                            radius: parent.radius
                            content: monthChip.current ? K.Tokens.color.on_primary : K.Tokens.color.on_surface
                            hovered: monthChip.hovered
                            pressed: monthChip.down
                        }
                    }

                    Centred {
                        width: parent.width
                        height: parent.height
                        text: root.monthNames[monthChip.index]
                        style: K.Tokens.type.body_strong
                        color: monthChip.current ? K.Tokens.color.on_primary : K.Tokens.color.on_surface
                    }
                }
            }
        }
    }

    // The hovered day's holiday, in a hint over the grid: drawn inside its cell, the next week's
    // row would paint over it and the month's clip would cut it.
    Item {
        readonly property point at: root.hovered ? root.hovered.mapToItem(root, 0, 0) : Qt.point(0, 0)

        x: at.x
        y: at.y
        width: root.cell
        height: root.cell

        K.Hint {
            text: root.hovered?.modelData.holiday ?? ""
            shown: text !== ""
        }
    }

    QtObject {
        id: slideFrom

        property real x: 0
    }

    // The new month comes in from the side it lies on.
    ParallelAnimation {
        id: slide

        Glide {
            target: shift
            property: "x"
            from: slideFrom.x
            to: 0
        }

        Glide {
            target: weeks
            property: "opacity"
            from: 0
            to: 1
        }
    }

    // A text centred in the box it is given, on whole device pixels.
    component Centred: Item {
        id: centred

        property alias text: label.text
        property alias style: label.style
        property alias color: label.color

        height: names.height

        K.Text {
            id: label

            x: K.Look.snap((centred.width - implicitWidth) / 2, root.dpr)
            y: K.Look.snap((centred.height - implicitHeight) / 2, root.dpr)
        }
    }

    component DayCell: K.Pressable {
        id: cell

        required property var modelData
        readonly property bool isSelected: root.selected === modelData.date
        readonly property real dim: modelData.inMonth ? 1 : K.Tokens.opacity.disabled_content

        width: root.cell
        height: root.cell
        activeFocusOnTab: false
        onActivated: root.picked(cell.modelData.date)

        Rectangle {
            id: disc

            x: K.Tokens.space.s4
            y: K.Tokens.space.s4
            width: K.Tokens.size.control_sm
            height: K.Tokens.size.control_sm
            radius: K.Tokens.radius.full
            color: cell.modelData.today ? K.Tokens.color.primary : "transparent"
            border.width: cell.isSelected && !cell.modelData.today ? K.Look.thin(root.dpr) : 0
            border.color: K.Tokens.color.primary

            K.StateLayer {
                radius: parent.radius
                content: cell.modelData.today ? K.Tokens.color.on_primary : K.Tokens.color.on_surface
                hovered: cell.hovered
                pressed: cell.down
            }

            Centred {
                width: parent.width
                height: parent.height
                text: String(cell.modelData.day)
                style: cell.modelData.today ? K.Tokens.type.body_strong : K.Tokens.type.body
                color: K.Look.alpha(cell.modelData.today ? K.Tokens.color.on_primary : cell.modelData.holiday ? K.Tokens.color.tertiary : cell.modelData.weekend ? K.Tokens.color.on_surface_variant : K.Tokens.color.on_surface, cell.dim)
            }

            // A holiday's dot, under the number.
            Rectangle {
                visible: !!cell.modelData.holiday
                x: K.Look.snap((disc.width - width) / 2, root.dpr)
                y: disc.height - K.Tokens.space.s4 * 2
                width: K.Tokens.space.s4
                height: K.Tokens.space.s4
                radius: K.Tokens.radius.full
                color: K.Look.alpha(cell.modelData.today ? K.Tokens.color.on_primary : K.Tokens.color.tertiary, cell.dim)
            }
        }

        onPointerOverChanged: {
            if (cell.pointerOver)
                root.hovered = cell;
            else if (root.hovered === cell)
                root.hovered = null;
        }
    }
}
