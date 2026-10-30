pragma Singleton

import QtQuick

// The width of words in a type style as iced lays them out, for `Text`: plain ASCII from the
// font's own advances, unkerned and unshaped, and anything else shaped, kerning included.
//
// Widths are measured at `precision` times the size: Qt rounds each advance to 1/64 of a pixel,
// which over a line comes to a pixel off iced, and a pixel moves every box after it.
//
// HAZARD: keep one metrics per font here, and the words' widths kept. Made headless, 42 texts
// (fourteen rows of three) took 0.7 ms longer with a `FontMetrics` in every `Text`, and building
// the font for each word measured cost 14 µs a word.
QtObject {
    id: advances

    // How much larger than the style the advances are measured, so Qt's rounding of each one to
    // 1/64 of a pixel shrinks to under a thousandth.
    readonly property int precision: 256
    // The most widths kept; past it they are all let go, as a list's words come round again
    // while it is typed in and others don't.
    readonly property int kept: 4096

    // Each type style's two fonts, plain and shaped, by the style's name.
    // HAZARD: read a style's properties only here, once. A style's measures never change, and
    // reading them from a `Text` for every word cost more than measuring the words.
    property var styles: new Map()
    // Widths measured, by the font's key and the words.
    property var widths: new Map()

    property Component metrics: Component {
        FontMetrics {}
    }

    // The width of `words` in `style`, shaped or not, in logical pixels.
    function of(style: QtObject, shaped: bool, words: string): real {
        const fonts = advances.fontsOf(style);
        return advances.width(shaped ? fonts.shaped : fonts.plain, words);
    }

    // The width of `words` in one of a style's fonts from `fontsOf`, in logical pixels.
    function width(font: var, words: string): real {
        const key = `${font.key}|${words}`;
        const known = advances.widths.get(key);
        if (known !== undefined)
            return known;
        if (font.metrics === null)
            font.metrics = advances.metrics.createObject(advances, {
                font: advances.fontFor(font.spec, advances.precision)
            });
        const width = font.metrics.advanceWidth(words) / advances.precision;
        if (advances.widths.size >= advances.kept)
            advances.widths.clear();
        advances.widths.set(key, width);
        return width;
    }

    // A style's line height, whether its figures are tabular, and its two fonts: each with its
    // key among the widths, the face a `Text` draws in, and the metrics it is measured with, made
    // with its first width.
    function fontsOf(style: QtObject): var {
        const known = advances.styles.get(style.name);
        if (known !== undefined)
            return known;
        const font = shaped => {
            const spec = {
                family: style.family,
                weight: style.weight,
                size: style.size,
                tabular: style.tabular,
                shaped: shaped
            };
            return {
                key: `${style.name}|${shaped}`,
                spec: spec,
                face: advances.fontFor(spec, 1),
                metrics: null
            };
        };
        const made = {
            line: style.line,
            tabular: style.tabular,
            plain: font(false),
            shaped: font(true)
        };
        advances.styles.set(style.name, made);
        return made;
    }

    // A style's font at `scale` times its size.
    function fontFor(spec: var, scale: real): font {
        // A style with tabular figures is shaped too, since Qt applies font features only to
        // shaped text.
        return Qt.font({
            family: spec.family,
            weight: spec.weight,
            pixelSize: spec.size * scale,
            kerning: spec.shaped,
            preferShaping: spec.shaped,
            features: spec.tabular ? {
                "tnum": 1
            } : {},
            hintingPreference: Font.PreferNoHinting
        });
    }
}
