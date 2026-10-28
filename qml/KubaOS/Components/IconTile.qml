import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// A symbol or a picture on a tinted rounded square of one size: `kubaos-ui`'s
// `avatar::icon_tile`. A list of apps reads as one set whatever each vendor drew, and a row with
// no picture keeps the room a picture takes: the symbol shows until the picture has loaded, and
// stays when there is none. An app's icon (`source`) sits inset at the icon size, keeping its own
// margins; a photo (`photo`, a notification's image) covers the tile.
Item {
    id: root

    // The symbol, and the placeholder while a picture loads or when it fails.
    property string icon: "notifications"
    // An app's icon, inset.
    property url source: ""
    // A photo, covering the tile.
    property url photo: ""
    // `Sm` (32, beside one line), `Md` (40, beside a title over a subtitle) or `Lg` (48, over a
    // `headline` name: the one thing a view is about).
    property string size: "Md"
    // On the error colour: a critical notification's.
    property bool danger: false
    // Its symbol in the accent: the one of a list that comes first, as the default keyboard
    // layout's star.
    property bool accent: false

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property int side: root.size === "Sm" ? Tokens.size.control_sm : root.size === "Lg" ? Tokens.size.control_lg : Tokens.size.control_md
    readonly property int glyph: root.size === "Sm" ? Tokens.type.body.icon : root.size === "Lg" ? Tokens.type.headline.icon : Tokens.icon.two_line
    readonly property color ink: root.danger ? Tokens.color.on_error : root.accent ? Tokens.color.primary : Tokens.color.on_surface_variant
    readonly property bool pictured: app.status === Image.Ready || (photoPath.item?.ready ?? false)
    readonly property var probe: ({
            role: "component",
            name: "IconTile",
            variant: root.size,
            fixed: {
                width: true,
                height: true
            }
        })

    implicitWidth: root.side
    implicitHeight: root.side

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        color: root.danger ? Tokens.color.error : Tokens.color.surface_container_highest
    }

    K.Icon {
        x: K.Look.snap((root.side - width) / 2, root.dpr)
        y: K.Look.snap((root.side - height) / 2, root.dpr)
        visible: !root.pictured
        name: root.icon
        size: root.glyph
        color: root.ink
    }

    Image {
        id: app

        x: K.Look.snap((root.side - width) / 2, root.dpr)
        y: K.Look.snap((root.side - height) / 2, root.dpr)
        width: root.glyph
        height: root.glyph
        visible: status === Image.Ready && root.photo.toString() === ""
        source: root.photo.toString() === "" ? root.source : ""
        asynchronous: true
        fillMode: Image.PreserveAspectFit
        sourceSize: Qt.size(Math.ceil(width * root.dpr), Math.ceil(height * root.dpr))
        smooth: true
        mipmap: true
    }

    // The photo's path only exists while there is a photo: a row of the Command menu, made on
    // every keystroke, carries no effect it doesn't use.
    Loader {
        id: photoPath

        anchors.fill: parent
        active: root.photo.toString() !== ""

        sourceComponent: Item {
            readonly property bool ready: cover.status === Image.Ready

            Image {
                id: cover

                anchors.fill: parent
                visible: false
                source: root.photo
                asynchronous: true
                fillMode: Image.PreserveAspectCrop
                sourceSize: Qt.size(Math.ceil(width * root.dpr), Math.ceil(height * root.dpr))
            }

            Rectangle {
                id: corners

                anchors.fill: parent
                radius: Tokens.radius.control
                visible: false
                layer.enabled: cover.status === Image.Ready
            }

            MultiEffect {
                anchors.fill: parent
                visible: cover.status === Image.Ready
                source: cover
                maskEnabled: true
                maskSource: corners
                maskThresholdMin: 0.5
                maskSpreadAtMin: 1
            }
        }
    }
}
