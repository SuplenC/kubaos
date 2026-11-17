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

    // The window's device pixel ratio; a test sets another to place the tile at that scale.
    property real dpr: Window.window ? Window.window.devicePixelRatio : 1
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
            variant: root.icon === "" ? `${root.size} initials` : root.size,
            fixed: {
                width: true,
                height: true
            }
        })

    implicitWidth: root.side
    implicitHeight: root.side

    // The first character of `name`, upper-cased, as a reader sees one: a base and the marks
    // that combine with it (हि, ที่), a flag's two regional indicators, an emoji with its skin tone
    // or its joined sequence (👨‍👩‍👧). iced segments by Unicode's extended graphemes; Qt's
    // JavaScript has no segmenter and no `\p{M}`, so the rule is written out for the scripts an
    // app's name is in, and its strings index UTF-16 units, so it walks code points.
    function initial(name: string): string {
        const points = name.trim();
        const cps = [];
        for (let at = 0; at < points.length;) {
            const cp = points.codePointAt(at);
            cps.push(cp);
            at += cp > 0xffff ? 2 : 1;
        }
        if (cps.length === 0)
            return "";
        let end = 1;
        if (root.regional(cps[0]) && cps.length > 1 && root.regional(cps[1]))
            end = 2;
        while (end < cps.length) {
            if (cps[end] === 0x200d && end + 1 < cps.length)
                end += 2;
            else if (root.joins(cps[end]))
                end += 1;
            else
                break;
        }
        return String.fromCodePoint(...cps.slice(0, end)).toUpperCase();
    }

    // A flag's half.
    function regional(cp: int): bool {
        return cp >= 0x1f1e6 && cp <= 0x1f1ff;
    }

    // A code point that joins the one before it: a combining mark, a vowel sign or virama of the
    // Indic scripts, a Thai or Lao mark, a variation selector, a skin tone or an emoji tag.
    function joins(cp: int): bool {
        const ranges = [[0x0300, 0x036f], [0x0483, 0x0489], [0x0591, 0x05c7], [0x0610, 0x061a], [0x064b, 0x065f], [0x0670, 0x0670], [0x06d6, 0x06ed], [0x0e31, 0x0e31], [0x0e34, 0x0e3a], [0x0e47, 0x0e4e], [0x0eb1, 0x0eb1], [0x0eb4, 0x0ebc], [0x0ec8, 0x0ece], [0x1160, 0x11ff], [0x1ab0, 0x1aff], [0x1dc0, 0x1dff], [0x20d0, 0x20ff], [0x302a, 0x302f], [0x3099, 0x309a], [0xfe00, 0xfe0f], [0xfe20, 0xfe2f], [0x1f3fb, 0x1f3ff], [0xe0020, 0xe007f], [0xe0100, 0xe01ef]];
        if (ranges.some(([low, high]) => cp >= low && cp <= high))
            return true;
        // Devanagari to Malayalam share one layout: in each block of 128, the signs sit at these
        // offsets, the avagraha (3D) and OM (50) being letters.
        if (cp < 0x0900 || cp > 0x0d7f)
            return false;
        const offset = cp & 0x7f;
        return offset <= 0x03 || (offset >= 0x3a && offset <= 0x4f && offset !== 0x3d) || (offset >= 0x51 && offset <= 0x57) || offset === 0x62 || offset === 0x63;
    }

    // Whether `letter` is a kana or a kanji, which M PLUS 1 draws and Geist doesn't: CJK's
    // ideographs to Extension B and past, its compatibility ones, and full- and half-width kana.
    function japanese(letter: string): bool {
        const cp = letter.codePointAt(0) ?? 0;
        return (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff) || (cp >= 0xff66 && cp <= 0xff9f) || (cp >= 0x20000 && cp <= 0x3134f);
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
    // the odd one of an odd count going before the box. Made only while it shows: a Command menu
    // row with an app's icon, made on every keystroke, carries none.
    Loader {
        active: root.letter !== "" && !root.pictured

        sourceComponent: Item {
            objectName: "initial"

            // The letter's style in the family that draws it: M PLUS 1 for a kana or a kanji,
            // which Geist doesn't draw, as iced picks it. Qt's own fallback skips a font that
            // covers part of a language, as the kanji kubaos-ui bundles are, and on a machine with
            // another CJK font installed would draw it in that one.
            readonly property QtObject font: QtObject {
                readonly property string family: root.japanese(root.letter) ? Tokens.font.jp : root.letterStyle.family
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
