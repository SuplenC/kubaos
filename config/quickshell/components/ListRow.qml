import QtQuick
import QtQuick.Layouts
import Quickshell.Widgets
import qs.theme

// One row in a list: leading glyph or image, title, optional subtitle, trailing items.
// `current` marks the keyboard selection (NavList draws the highlight behind it),
// `active` marks e.g. the connected network. Cascades in when the list's
// enterSerial changes (NavList.playEnter()), and fades in when the model adds it.
Item {
    id: root

    property string icon
    property string image
    property string title
    property string subtitle
    property bool current: false
    property bool active: false
    property int enterSerial: ListView.view?.enterSerial ?? 0
    property int enterIndex: 0
    property real enter: 1
    readonly property real radius: Theme.rounding.md
    default property alias trailing: trailingRow.data

    signal clicked

    implicitHeight: subtitle ? 52 : 44
    opacity: enter
    transform: Translate {
        y: (1 - root.enter) * 10
    }

    onEnterSerialChanged: {
        arrival.stop();
        entrance.stop();
        if (enterIndex > 12)
            return;
        enter = 0;
        entrance.start();
    }

    ListView.onAdd: {
        entrance.stop();
        enter = 0;
        arrival.start();
    }

    Anim {
        id: arrival

        target: root
        property: "enter"
        to: 1
        duration: Theme.anim.small
        easing.bezierCurve: Theme.anim.effects
    }

    SequentialAnimation {
        id: entrance

        PauseAnimation {
            duration: Theme.anim.stagger * root.enterIndex + 60
        }

        Anim {
            target: root
            property: "enter"
            to: 1
            duration: Theme.anim.normal
            easing.bezierCurve: Theme.anim.emphasizedDecel
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.spacing.md
        anchors.rightMargin: Theme.spacing.md
        spacing: Theme.spacing.md

        IconImage {
            visible: root.image !== ""
            source: root.image
            implicitSize: 30
            asynchronous: true
        }

        Rectangle {
            visible: root.image === "" && root.icon !== ""
            implicitWidth: 32
            implicitHeight: 32
            radius: Theme.rounding.full
            color: root.active ? Theme.m3primary : "transparent"

            Behavior on color {
                CAnim {}
            }

            Icon {
                anchors.centerIn: parent
                name: root.icon
                size: 20
                filled: root.active
                color: root.active ? Theme.m3onPrimary : root.current ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 1

            StyledText {
                Layout.fillWidth: true
                text: root.title
                color: root.current ? Theme.m3onSecondaryContainer : Theme.m3onSurface
                font.weight: root.active || root.current ? Font.Medium : Font.Normal
            }

            StyledText {
                Layout.fillWidth: true
                visible: root.subtitle !== ""
                text: root.subtitle
                color: root.current ? Theme.alpha(Theme.m3onSecondaryContainer, 0.75) : Theme.m3onSurfaceVariant
                font.pixelSize: Theme.font.sm
            }
        }

        Row {
            id: trailingRow

            spacing: Theme.spacing.xs
        }
    }

    StateLayer {
        radius: root.radius
        onClicked: root.clicked()
    }
}
