import QtQuick
import KubaOS.Components as K

// Text in a type style: `kubaos-ui`'s `text::text`. Its box is as wide as the words' advances,
// rounded up to a device pixel, and as tall as its lines on the style's line height. Given less
// `room`, it breaks between words, at the same places iced does.
//
// Widths are measured as iced lays text out (`Advances`), so a box is as wide here as there.
//
// HAZARD: keep a text's own properties few, its words measured in `laid` and `measured` alone.
// A list's rows make their texts many at a time, and each property costs about as much to make as
// a binding: made headless, 42 texts (fourteen rows of three) take 1.1 ms this way against 1.7 ms
// with a property for each measure and one for each of the font's.
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
    // The words in lines as they wrap in `room`, before `maximumLines` cuts them.
    // HAZARD: keep them apart from `maximumLines`: a notification's card unfolds to as many lines
    // as these.
    readonly property var laid: root.layOut(root.text, root.room, root.style)
    // The lines measured once for everything that reads them: the lines drawn, the font they are
    // drawn in and the box they take.
    readonly property var measured: root.measure(root.laid, root.text, root.room, root.style, root.maximumLines)
    // Whether `maximumLines` cuts the words: more lines than it keeps, or one wider than the room or
    // than a box its owner made narrower.
    // HAZARD: wrap and elide only then. Qt wraps a line laid out exactly as wide as its own
    // advance onto a second line now and then, and a kept line count of one drops what moved.
    readonly property bool cut: root.maximumLines > 0 && (root.laid.length > root.maximumLines || root.measured.widest > Math.min(root.room, root.width + 1e-3))
    readonly property bool lit: root.markup !== "" && root.measured.lines === 1
    readonly property var probe: ({
            role: "text",
            content: root.text,
            natural: {
                width: root.measured.width,
                height: root.measured.height
            }
        })

    implicitWidth: K.Look.snapUp(root.measured.width, root.dpr)
    implicitHeight: root.measured.height

    // Everything the text's box and its drawing take from its lines, in one pass.
    function measure(laid: var, text: string, room: real, style: QtObject, most: int): var {
        const fonts = K.Advances.fontsOf(style);
        const font = root.fontOf(text, fonts);
        const lines = root.clamped(laid, most, room, font);
        return {
            widest: root.widest(laid),
            lines: lines.length,
            drawn: lines.length === 1 ? lines[0].text : lines.map(line => line.text).join("\n"),
            width: root.widest(lines),
            height: lines.length * fonts.line,
            face: font.face
        };
    }

    // The width of the widest of `lines`.
    function widest(lines: var): real {
        let widest = 0;
        for (let i = 0; i < lines.length; i++)
            widest = Math.max(widest, lines[i].width);
        return widest;
    }

    // Which of a style's fonts `text` is measured and drawn in. Iced shapes anything but plain
    // ASCII, and a style with tabular figures is shaped too, since Qt applies font features only to
    // shaped text.
    function fontOf(text: string, fonts: var): var {
        return fonts.tabular || /[^\x00-\x7f]/.test(text) ? fonts.shaped : fonts.plain;
    }

    // `lines` cut to `maximumLines`: the last one kept takes every word after it, and each is
    // ellipsised at `room` when drawn.
    function clamped(lines: var, most: int, room: real, font: var): var {
        if (most <= 0 || lines.length === 1 && lines[0].width <= room)
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
                    width: Math.min(isNaN(line.width) ? K.Advances.width(font, line.text) : line.width, room)
                }));
    }

    // `text` in lines no wider than `room`, broken at spaces: each line takes words while they
    // fit, and a word wider than the room has a line of its own. A paragraph that fits is one
    // line, measured once: no part of a line is wider than the whole.
    function layOut(text: string, room: real, style: QtObject): var {
        const font = root.fontOf(text, K.Advances.fontsOf(style));
        // Most texts are a line that fits: measured once, with nothing to break.
        if (text.indexOf("\n") < 0) {
            const whole = K.Advances.width(font, text);
            if (whole <= room)
                return [{
                        text: text,
                        width: whole
                    }];
        }
        const lines = [];
        for (const paragraph of text.split("\n")) {
            const whole = K.Advances.width(font, paragraph);
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
                if (line !== "" && K.Advances.width(font, longer) > room) {
                    lines.push({
                        text: line,
                        width: K.Advances.width(font, line)
                    });
                    line = word;
                } else {
                    line = longer;
                }
            }
            lines.push({
                text: line,
                width: K.Advances.width(font, line)
            });
        }
        return lines;
    }

    Text {
        id: drawn

        width: root.width
        height: root.height
        text: root.lit ? root.markup : root.measured.drawn
        textFormat: root.lit ? Text.StyledText : Text.AutoText
        color: root.color
        horizontalAlignment: root.horizontalAlignment
        // The lines laid out above already fit the room but the last kept, which Qt wraps and cuts
        // at the ellipsis.
        wrapMode: root.cut ? Text.Wrap : Text.NoWrap
        maximumLineCount: root.cut ? root.maximumLines : Number.MAX_SAFE_INTEGER
        elide: root.cut ? Text.ElideRight : Text.ElideNone
        font: root.measured.face
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
