import QtQuick
import qs.theme

// An analog face in the theme's colours: twelve ticks, hour and minute hands
// in the primary roles, the second hand in tertiary. Rectangles turned about
// the centre, so a tick redraws three items and runs no script.
Item {
    id: root

    required property date time
    property real size: 112

    readonly property real hours: time.getHours() % 12 + time.getMinutes() / 60
    readonly property real minutes: time.getMinutes() + time.getSeconds() / 60

    implicitWidth: size
    implicitHeight: size

    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: Theme.m3surfaceContainerHigh
        border.width: 1
        border.color: Theme.m3outlineVariant
    }

    Repeater {
        model: 12

        Item {
            required property int index

            anchors.fill: parent
            rotation: index * 30

            Rectangle {
                x: (root.size - width) / 2
                y: 6
                width: index % 3 === 0 ? 3 : 2
                height: index % 3 === 0 ? 9 : 5
                radius: width / 2
                color: index % 3 === 0 ? Theme.m3onSurfaceVariant : Theme.m3outline
            }
        }
    }

    Hand {
        length: root.size * 0.27
        thickness: 4
        color: Theme.m3primary
        angle: root.hours * 30
    }

    Hand {
        length: root.size * 0.38
        thickness: 3
        color: Theme.m3onSurface
        angle: root.minutes * 6
    }

    Hand {
        length: root.size * 0.42
        thickness: 1.5
        color: Theme.m3tertiary
        angle: root.time.getSeconds() * 6
    }

    Rectangle {
        anchors.centerIn: parent
        width: 8
        height: 8
        radius: 4
        color: Theme.m3tertiary
    }

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
            height: hand.length + 6
            radius: width / 2
            color: hand.color
            antialiasing: true
        }
    }
}
