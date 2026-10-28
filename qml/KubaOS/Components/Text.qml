import QtQuick
import KubaOS.Components as K

// Text in a type style: `kubaos-ui`'s `text::text`. Its box is as wide as the words' advances,
// rounded up to a device pixel, and as tall as its lines on the style's line height. Given less
// `room`, it breaks between words, at the same places iced does.
//
// Widths are measured as iced lays text out: plain ASCII from the font's own advances, unkerned
// and unshaped, and anything else shaped, kerning included. They are measured at `precision`
// times the size: Qt rounds each advance to 1/64 of a pixel, which over a line comes to a pixel
// off iced, and a pixel moves every box after it.
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

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // How much larger than the style the advances are measured, so Qt's rounding of each one to
    // 1/64 of a pixel shrinks to under a thousandth.
    readonly property int precision: 256
    // Whether the text is shaped: iced shapes anything but plain ASCII.
    readonly property bool shaped: /[^\x00-\x7f]/.test(root.text)
    // Each line's words and its width in logical pixels.
    // The words in lines as they wrap in `room`, before `maximumLines` cuts them.
    readonly property var laid: root.layOut(root.text, root.room, metrics.font)
    readonly property var lines: root.clamped(root.laid)
    // Whether `maximumLines` cuts the words: more lines than it keeps, or one wider than the room or
    // than a box its owner made narrower.
    // HAZARD: wrap and elide only then. Qt wraps a line laid out exactly as wide as its own
    // advance onto a second line now and then, and a kept line count of one drops what moved.
    readonly property bool cut: root.maximumLines > 0 && (root.laid.length > root.maximumLines || root.laid.some(line => line.width > Math.min(root.room, root.width + 1e-3)))
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
        return metrics.advanceWidth(words) / root.precision;
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
    // fit, and a word wider than the room has a line of its own.
    function layOut(text: string, room: real, font: font): var {
        // The font is read so a new one lays the text out again.
        font.family;
        font.weight;
        font.pixelSize;
        font.kerning;
        const lines = [];
        for (const paragraph of text.split("\n")) {
            let line = "";
            for (const word of paragraph.split(" ")) {
                const longer = line === "" ? word : `${line} ${word}`;
                if (line !== "" && root.advance(longer) > room) {
                    lines.push(line);
                    line = word;
                } else {
                    line = longer;
                }
            }
            lines.push(line);
        }
        return lines.map(line => ({
                    text: line,
                    width: root.advance(line)
                }));
    }

    FontMetrics {
        id: metrics

        font.family: root.style.family
        font.weight: root.style.weight
        font.pixelSize: root.style.size * root.precision
        font.kerning: root.shaped
        font.preferShaping: root.shaped
        font.hintingPreference: Font.PreferNoHinting
    }

    Text {
        width: root.width
        height: root.height
        text: root.lines.map(line => line.text).join("\n")
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
