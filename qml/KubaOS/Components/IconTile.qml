import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// A symbol, an app's initial or a picture on a tinted rounded square of one size: `kubaos-ui`'s
// `icon_tile::icon_tile` and `initial_tile`. A list of apps reads as one set whatever each vendor
// drew, and a row with no picture keeps the room a picture takes: the symbol, or the initial of
// an app that names none, shows until the picture has loaded, and stays when there is none. An
// app's icon (`source`) sits inset at the icon size, keeping its own margins; a photo (`photo`, a
// notification's image) covers the tile.
Item {
    id: root

    // The symbol, and the placeholder while a picture loads or when it fails; empty for an app
    // that names none, whose initial takes its place.
    property string icon: "notifications"
    // An app's name, whose first character, upper-cased, stands in for a symbol it doesn't name.
    property string name: ""
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
    // The letter in the style whose icon is the tile's symbol box, so its line is that box.
    readonly property QtObject letterStyle: root.size === "Sm" ? Tokens.type.body_strong : root.size === "Lg" ? Tokens.type.headline : Tokens.type.subheading
    readonly property string letter: root.icon === "" ? root.initial(root.name) : ""
    readonly property bool pictured: app.status === Image.Ready || (photoPath.item?.ready ?? false)
    readonly property var probe: ({
            role: "component",
            name: "IconTile",
            variant: root.letter === "" ? root.size : `${root.size} initials`,
            fixed: {
                width: true,
                height: true
            }
        })

    implicitWidth: root.side
    implicitHeight: root.side

    // The first character of `name`, upper-cased, with the marks that combine with it: one letter,
    // in whatever script the name is in. Qt's JavaScript has no grapheme segmenter, and its
    // strings index UTF-16 units, so a character past the BMP is read by its code point.
    function initial(name: string): string {
        const trimmed = name.trim();
        if (trimmed === "")
            return "";
        let first = String.fromCodePoint(trimmed.codePointAt(0));
        for (const mark of trimmed.slice(first.length)) {
            if (!/[\u0300-\u036f\u1ab0-\u1aff\u1dc0-\u1dff\u20d0-\u20ff\ufe00-\ufe0f\ufe20-\ufe2f\u200d]/.test(mark))
                break;
            first += mark;
        }
        return first.toUpperCase();
    }

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        color: root.danger ? Tokens.color.error : Tokens.color.surface_container_highest
    }

    K.Icon {
        x: K.Look.snap((root.side - width) / 2, root.dpr)
        y: K.Look.snap((root.side - height) / 2, root.dpr)
        visible: !root.pictured && root.icon !== ""
        name: root.icon
        size: root.glyph
        color: root.ink
    }

    // The initial, centred across on its advance and down on its ink, on whole device pixels as
    // iced places it: its box is the style's line tall and centred on the tile, and its baseline
    // sits where the middle of its ink lands in the middle of that line. Worked in device pixels,
    // half of an odd count going after the box.
    Loader {
        active: root.letter !== ""
        visible: !root.pictured

        sourceComponent: Item {
            objectName: "initial"

            // The letter's style in the family that draws it: M PLUS 1 for a kana or a kanji,
            // which Geist doesn't draw, as iced falls back to it. Qt's own fallback skips a font
            // that covers part of a language, and the kanji kubaos-ui bundles are a subset.
            readonly property QtObject font: QtObject {
                readonly property string family: /[\u3040-\u30ff\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff]/.test(root.letter) ? Tokens.font.jp : root.letterStyle.family
                readonly property int weight: root.letterStyle.weight
                readonly property int size: root.letterStyle.size
                readonly property int line: root.letterStyle.line
                readonly property bool tabular: false
            }
            readonly property real middle: K.Ink.middle(font, root.letter)
            readonly property real advance: K.Advances.of(font, /[^\x00-\x7f]/.test(root.letter), root.letter)
            readonly property int sideDevice: Math.round(root.side * root.dpr)
            readonly property int lineDevice: Math.round(root.letterStyle.line * root.dpr)
            readonly property int widthDevice: Math.ceil(advance * root.dpr - 1e-3)
            readonly property int leftDevice: Math.floor((sideDevice - widthDevice) / 2 + 0.5)
            readonly property int topDevice: Math.floor((sideDevice - lineDevice) / 2 + 0.5)
            readonly property int baselineDevice: Math.floor(lineDevice / 2 + middle * root.letterStyle.size * root.dpr + 0.5)
            readonly property var probe: ({
                    role: "text",
                    content: root.letter,
                    natural: {
                        width: advance,
                        height: root.letterStyle.line
                    }
                })

            x: leftDevice / root.dpr
            y: topDevice / root.dpr
            width: widthDevice / root.dpr
            height: lineDevice / root.dpr

            Text {
                y: parent.baselineDevice / root.dpr - baselineOffset
                text: root.letter
                color: root.ink
                font.family: parent.font.family
                font.weight: root.letterStyle.weight
                font.pixelSize: root.letterStyle.size
                font.hintingPreference: Font.PreferNoHinting
                renderType: Text.NativeRendering
            }
        }
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

                objectName: "cover"
                anchors.fill: parent
                visible: false
                source: root.photo
                asynchronous: true
                fillMode: Image.PreserveAspectCrop
                sourceSize: Qt.size(Math.ceil(width * root.dpr), Math.ceil(height * root.dpr))
            }

            // HAZARD: the mask's texture exists from the moment there is a photo, not from the
            // moment it has loaded. Made in the same frame as the effect starts drawing, the
            // effect kept an empty mask, and a card that didn't move again showed a blank tile.
            Rectangle {
                id: corners

                objectName: "corners"
                anchors.fill: parent
                radius: Tokens.radius.control
                visible: false
                layer.enabled: true
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
