import QtQuick
import KubaOS.Components as K

// An analog face in the theme's colours: twelve ticks, hour and minute hands
// in the primary roles, the second hand in tertiary. Rectangles turned about
// the centre, so a tick redraws three items and runs no script. The quarter
// ticks are a thin stroke, the others a hairline, both on whole device pixels.
Item {
    id: root

    required property date time
    property int size: K.Tokens.icon.s48 * 2

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property real hours: time.getHours() % 12 + time.getMinutes() / 60
    readonly property real minutes: time.getMinutes() + time.getSeconds() / 60

    implicitWidth: size
    implicitHeight: size

    Rectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.full
        color: K.Tokens.color.surface_container_high
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Tokens.color.outline_variant
    }

    Repeater {
        model: 12

        Item {
            required property int index
            readonly property bool quarter: index % 3 === 0

            anchors.fill: parent
            rotation: index * 30

            Rectangle {
                x: (root.size - width) / 2
                y: K.Tokens.space.s4
                width: parent.quarter ? K.Look.thin(root.dpr) : K.Look.hairline(root.dpr)
                height: parent.quarter ? K.Tokens.space.s8 : K.Tokens.space.s4
                radius: K.Tokens.radius.full
                color: parent.quarter ? K.Tokens.color.on_surface_variant : K.Tokens.color.outline
            }
        }
    }

    Hand {
        length: root.size * 0.27
        thickness: K.Tokens.space.s4
        color: K.Tokens.color.primary
        angle: root.hours * 30
    }

    Hand {
        length: root.size * 0.38
        thickness: K.Look.thin(root.dpr)
        color: K.Tokens.color.on_surface
        angle: root.minutes * 6
    }

    Hand {
        length: root.size * 0.42
        thickness: K.Look.hairline(root.dpr)
        color: K.Tokens.color.tertiary
        angle: root.time.getSeconds() * 6
    }

    Rectangle {
        anchors.centerIn: parent
        width: K.Tokens.space.s8
        height: K.Tokens.space.s8
        radius: K.Tokens.radius.full
        color: K.Tokens.color.tertiary
    }

    // A hand from the centre out to `length`, reaching a little past the centre behind it.
    component Hand: Item {
        id: hand

        property real length
        property real thickness
        property color color
        property real angle

        anchors.fill: parent
        rotation: angle

        Rectangle {
            x: (root.size - width) / 2
            y: root.size / 2 - hand.length
            width: hand.thickness
            height: hand.length + K.Tokens.space.s4
            radius: K.Tokens.radius.full
            color: hand.color
            antialiasing: true
        }
    }
}
