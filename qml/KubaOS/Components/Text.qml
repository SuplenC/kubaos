import QtQuick
import KubaOS.Components as K

// Text in a type style: `kubaos-ui`'s `text::text`. Its box is as wide as the words' advances,
// rounded up to a device pixel, and as tall as its lines on the style's line height. Given less
// `room`, it breaks between words, at the same places iced does.
//
// Widths are measured as iced lays text out (`Advances`), so a box is as wide here as there.
Item {
    id: root

    property string text
    property QtObject style: Tokens.type.body
    property color color: Tokens.color.on_surface
    // The widest the text may be laid out; past it, words wrap onto the next line.
    property real room: Infinity
    // How the lines sit across the box: `Text.AlignLeft`, or `Text.AlignHCenter`.
    property int horizontalAlignment: Text.AlignLeft
    // The most lines it takes, 0 for as many as the words need. Past them, the last line ends in
    // an ellipsis where it reaches `room`: a window's title, a notification's body.
    property int maximumLines: 0
    // The same words as Qt's `StyledText` with runs of them in other colours, as Kotodama lights
    // the characters a query matched; empty for none. `text` is laid out and measured, and the
    // markup drawn in its place while the words keep to one line.
    property string markup: ""

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // Whether the text is shaped: iced shapes anything but plain ASCII. A style with tabular
    // figures is shaped too, since Qt applies font features only to shaped text.
    readonly property bool shaped: root.style.tabular || /[^\x00-\x7f]/.test(root.text)
    // The OpenType features the style turns on: `tnum` for tabular figures.
    readonly property var features: root.style.tabular ? {
        "tnum": 1
    } : {}
    // Each line's words and its width in logical pixels.
    // The words in lines as they wrap in `room`, before `maximumLines` cuts them.
    readonly property var laid: root.layOut(root.text, root.room, root.style, root.shaped)
    readonly property var lines: root.clamped(root.laid)
    // Whether `maximumLines` cuts the words: more lines than it keeps, or one wider than the room or
    // than a box its owner made narrower.
    // HAZARD: wrap and elide only then. Qt wraps a line laid out exactly as wide as its own
    // advance onto a second line now and then, and a kept line count of one drops what moved.
    readonly property bool cut: root.maximumLines > 0 && (root.laid.length > root.maximumLines || root.laid.some(line => line.width > Math.min(root.room, root.width + 1e-3)))
    readonly property bool lit: root.markup !== "" && root.lines.length === 1
    readonly property real naturalWidth: root.lines.reduce((widest, line) => Math.max(widest, line.width), 0)
    readonly property real naturalHeight: root.lines.length * root.style.line
    readonly property var probe: ({
            role: "text",
            content: root.text,
            natural: {
                width: root.naturalWidth,
                height: root.naturalHeight
            }
        })

    implicitWidth: K.Look.snapUp(root.naturalWidth, root.dpr)
    implicitHeight: root.naturalHeight

    function advance(words: string): real {
        return K.Advances.of(root.style, root.shaped, words);
    }

    // `lines` cut to `maximumLines`: the last one kept takes every word after it, and each is
    // ellipsised at `room` when drawn.
    function clamped(lines: var): var {
        const most = root.maximumLines;
        if (most <= 0)
            return lines;
        const kept = lines.slice(0, most);
        if (lines.length > most)
            kept[most - 1] = {
                text: lines.slice(most - 1).map(line => line.text).join(" "),
                width: NaN
            };
        // A last line wider than the room, one long word included, is cut at it.
        return kept.map(line => ({
                    text: line.text,
                    width: Math.min(isNaN(line.width) ? root.advance(line.text) : line.width, root.room)
                }));
    }

    // `text` in lines no wider than `room`, broken at spaces: each line takes words while they
    // fit, and a word wider than the room has a line of its own. A paragraph that fits is one
    // line, measured once: no part of a line is wider than the whole.
    function layOut(text: string, room: real, style: QtObject, shaped: bool): var {
        const lines = [];
        for (const paragraph of text.split("\n")) {
            const whole = K.Advances.of(style, shaped, paragraph);
            if (whole <= room) {
                lines.push({
                    text: paragraph,
                    width: whole
                });
                continue;
            }
            let line = "";
            for (const word of paragraph.split(" ")) {
                const longer = line === "" ? word : `${line} ${word}`;
                if (line !== "" && K.Advances.of(style, shaped, longer) > room) {
                    lines.push({
                        text: line,
                        width: K.Advances.of(style, shaped, line)
                    });
                    line = word;
                } else {
                    line = longer;
                }
            }
            lines.push({
                text: line,
                width: K.Advances.of(style, shaped, line)
            });
        }
        return lines;
    }

    Text {
        width: root.width
        height: root.height
        text: root.lit ? root.markup : root.lines.map(line => line.text).join("\n")
        textFormat: root.lit ? Text.StyledText : Text.AutoText
        color: root.color
        horizontalAlignment: root.horizontalAlignment
        // The lines laid out above already fit the room but the last kept, which Qt wraps and cuts
        // at the ellipsis.
        wrapMode: root.cut ? Text.Wrap : Text.NoWrap
        maximumLineCount: root.cut ? root.maximumLines : Number.MAX_SAFE_INTEGER
        elide: root.cut ? Text.ElideRight : Text.ElideNone
        font.family: root.style.family
        font.weight: root.style.weight
        font.pixelSize: root.style.size
        font.kerning: root.shaped
        font.preferShaping: root.shaped
        font.features: root.features
        font.hintingPreference: Font.PreferNoHinting
        // Drawn by the font engine, not as distance fields, which a text that changes every
        // second (the bar's readings) pays for on each change: over 30 s of the bar's sysmon and
        // recording ticks, 0.47 CPU s this way against 0.57 with Qt's own rendering (0.44 for the
        // shell's text before it moved here).
        renderType: Text.NativeRendering
        lineHeightMode: Text.FixedHeight
        lineHeight: root.style.line
        verticalAlignment: Text.AlignVCenter
    }
}
