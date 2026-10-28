import QtQuick
import KubaOS.Components as K

// A month as six weeks, each led by its ISO week number: `kubaos-ui`'s `calendar::calendar`. The
// first weekday is the month's own (the engine's, from the settings). Today is filled, the picked
// day ringed, a holiday dotted with its name in a hint on hover; days of the months around are
// dimmed. The title between the arrows swaps in the year's twelve months. The wheel moves a
// month; the new month slides in.
//
// A day is a control, as tall as a regular button, its disc a small one; the week numbers take a
// small control's width.
Item {
    id: root

    // The month: `{ year, month, title, weekdays, weeks: [{ number, days: [{ date, day, inMonth,
    // today, weekend, holiday }] }] }`, as `kubaos-clock` replies with it.
    property var month: null
    // The picked day, by its `date`.
    property string selected: ""
    property bool yearView: false
    // +1 when the month moved forward, -1 back: where the new one comes from.
    property int direction: 0
    // The day under the pointer, or null.
    property Item hovered: null

    readonly property int cell: Tokens.size.control_md
    readonly property int weekColumn: Tokens.size.control_sm
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var monthNames: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
    readonly property var probe: ({
            role: "component",
            name: "Calendar",
            variant: root.yearView ? "year" : "",
            fixed: {
                width: true,
                height: true
            }
        })

    signal picked(string date)
    signal moved(int months)
    signal pickedMonth(int month)

    implicitWidth: root.weekColumn + root.cell * 7
    implicitHeight: column.implicitHeight

    function relayout(): void {
        column.relayout();
    }

    // A month the user moved to slides in; one read again (a new setting, midnight) just shows.
    onMonthChanged: {
        if (direction !== 0) {
            slideFrom.x = direction * Tokens.space.s64;
            slide.restart();
        }
        direction = 0;
    }

    // The months and the year view trade places: over `short`, settling in.
    component Glide: NumberAnimation {
        duration: Tokens.duration.short
        easing.type: Easing.BezierSpline
        easing.bezierCurve: Tokens.easing.emphasized_decel
    }

    // `text` centred in its box on whole device pixels, as `kubaos-ui` centres it: across as a
    // stack centres, an odd spare device pixel after it, and down to the nearest device pixel.
    component Centred: Item {
        id: centred

        property alias text: label.text
        property alias style: label.style
        property alias color: label.color

        K.Text {
            id: label

            x: K.Look.centre(centred.width - width, root.dpr)
            y: K.Look.snap((centred.height - height) / 2, root.dpr)
        }
    }

    K.Stack {
        id: column

        width: root.implicitWidth
        fill: true
        gap: Tokens.space.s8

        // The arrows at either end and the title between them, a small control tall.
        Item {
            id: header

            readonly property bool fillWidth: true

            implicitHeight: Tokens.size.control_sm

            function relayout(): void {
                arrows.relayout();
            }

            K.Row {
                id: arrows

                width: header.width
                fill: true
                align: "centre"

                K.IconButton {
                    icon: "chevron_left"
                    label: root.yearView ? qsTr("Last year") : qsTr("Last month")
                    kind: "Text"
                    size: "Sm"
                    onActivated: root.moved(root.yearView ? -12 : -1)
                }

                K.Fill {
                }

                K.IconButton {
                    icon: "chevron_right"
                    label: root.yearView ? qsTr("Next year") : qsTr("Next month")
                    kind: "Text"
                    size: "Sm"
                    onActivated: root.moved(root.yearView ? 12 : 1)
                }
            }

            // The title: a pill that opens the year, tonal while it is open.
            K.Pressable {
                id: title

                x: K.Look.centre(header.width - width, root.dpr)
                width: K.Look.snapUp(titleText.implicitWidth + Tokens.space.s12 * 2, root.dpr)
                height: Tokens.size.control_sm
                activeFocusOnTab: false
                onActivated: root.yearView = !root.yearView

                Rectangle {
                    anchors.fill: parent
                    radius: Tokens.radius.full
                    color: root.yearView ? Tokens.color.secondary_container : "transparent"

                    K.StateLayer {
                        radius: parent.radius
                        content: root.yearView ? Tokens.color.on_secondary_container : Tokens.color.on_surface
                        hovered: title.hovered
                        pressed: title.down
                    }
                }

                K.Text {
                    id: titleText

                    x: Tokens.space.s12
                    y: K.Look.snap((Tokens.size.control_sm - implicitHeight) / 2, root.dpr)
                    text: root.yearView ? String(root.month?.year ?? "") : (root.month?.title ?? "")
                    style: Tokens.type.subheading
                    color: root.yearView ? Tokens.color.on_secondary_container : Tokens.color.on_surface
                }
            }
        }

        Item {
            id: body

            readonly property bool fillWidth: true

            implicitHeight: Tokens.space.s24 + root.cell * 6
            clip: true

            function relayout(): void {
                if (days.visible)
                    days.relayout();
                if (year.visible)
                    year.relayout();
            }

            WheelHandler {
                acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
                onWheel: event => {
                    const steps = event.angleDelta.y !== 0 ? event.angleDelta.y : -event.angleDelta.x;
                    if (steps !== 0)
                        root.moved((steps > 0 ? -1 : 1) * (root.yearView ? 12 : 1));
                }
            }

            // The weekdays' names, then the weeks, a row each.
            K.Stack {
                id: days

                width: body.width
                opacity: root.yearView ? 0 : 1
                visible: opacity > 0
                scale: root.yearView ? 0.94 : 1

                Behavior on opacity {
                    Glide {}
                }

                Behavior on scale {
                    Glide {}
                }

                K.Row {
                    Centred {
                        implicitWidth: root.weekColumn
                        implicitHeight: Tokens.space.s24
                        text: "Wk"
                        style: Tokens.type.caption
                        color: Tokens.color.on_surface_variant
                    }

                    Repeater {
                        model: root.month?.weekdays ?? []

                        Centred {
                            required property string modelData

                            implicitWidth: root.cell
                            implicitHeight: Tokens.space.s24
                            text: modelData.slice(0, 2)
                            style: Tokens.type.label
                            color: modelData.startsWith("Sat") || modelData.startsWith("Sun") ? Tokens.color.tertiary : Tokens.color.on_surface_variant
                        }
                    }
                }

                K.Stack {
                    id: weeks

                    transform: Translate {
                        id: shift
                    }

                    Repeater {
                        model: root.month?.weeks ?? []

                        K.Row {
                            id: week

                            required property var modelData

                            Centred {
                                implicitWidth: root.weekColumn
                                implicitHeight: root.cell
                                text: String(week.modelData.number)
                                style: Tokens.type.caption
                                color: Tokens.color.on_surface_variant
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

                y: K.Look.snap((body.height - height) / 2, root.dpr)
                width: body.width
                fill: true
                columns: 3
                gap: Tokens.space.s8
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

                        implicitHeight: Tokens.size.control_md
                        activeFocusOnTab: false
                        onActivated: {
                            root.pickedMonth(monthChip.index + 1);
                            root.yearView = false;
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: Tokens.radius.control
                            color: monthChip.current ? Tokens.color.primary : Tokens.color.surface_container_high

                            K.StateLayer {
                                radius: parent.radius
                                content: monthChip.current ? Tokens.color.on_primary : Tokens.color.on_surface
                                hovered: monthChip.hovered
                                pressed: monthChip.down
                            }
                        }

                        Centred {
                            width: parent.width
                            height: parent.height
                            text: root.monthNames[monthChip.index]
                            style: Tokens.type.body_strong
                            color: monthChip.current ? Tokens.color.on_primary : Tokens.color.on_surface
                        }
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

    component DayCell: K.Pressable {
        id: day

        required property var modelData
        readonly property bool isSelected: root.selected === modelData.date
        readonly property real dim: modelData.inMonth ? 1 : Tokens.opacity.disabled_content

        implicitWidth: root.cell
        implicitHeight: root.cell
        activeFocusOnTab: false
        onActivated: root.picked(day.modelData.date)

        Rectangle {
            id: disc

            x: Tokens.space.s4
            y: Tokens.space.s4
            width: Tokens.size.control_sm
            height: Tokens.size.control_sm
            radius: Tokens.radius.full
            color: day.modelData.today ? Tokens.color.primary : "transparent"
            border.width: day.isSelected && !day.modelData.today ? K.Look.thin(root.dpr) : 0
            border.color: Tokens.color.primary

            K.StateLayer {
                radius: parent.radius
                content: day.modelData.today ? Tokens.color.on_primary : Tokens.color.on_surface
                hovered: day.hovered
                pressed: day.down
            }

            Centred {
                width: parent.width
                height: parent.height
                text: String(day.modelData.day)
                style: day.modelData.today ? Tokens.type.body_strong : Tokens.type.body
                color: K.Look.alpha(day.modelData.today ? Tokens.color.on_primary : day.modelData.holiday ? Tokens.color.tertiary : day.modelData.weekend ? Tokens.color.on_surface_variant : Tokens.color.on_surface, day.dim)
            }

            // A holiday's dot, under the number.
            Rectangle {
                visible: !!day.modelData.holiday
                x: K.Look.centre(disc.width - width, root.dpr)
                y: disc.height - Tokens.space.s4 * 2
                width: Tokens.space.s4
                height: Tokens.space.s4
                radius: Tokens.radius.full
                color: K.Look.alpha(day.modelData.today ? Tokens.color.on_primary : Tokens.color.tertiary, day.dim)
            }
        }

        onPointerOverChanged: {
            if (day.pointerOver)
                root.hovered = day;
            else if (root.hovered === day)
                root.hovered = null;
        }
    }
}
