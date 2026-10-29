pragma Singleton

import QtQuick

// The width of words in a type style as iced lays them out, for `Text`: plain ASCII from the
// font's own advances, unkerned and unshaped, and anything else shaped, kerning included.
//
// Widths are measured at `precision` times the size: Qt rounds each advance to 1/64 of a pixel,
// which over a line comes to a pixel off iced, and a pixel moves every box after it.
//
// HAZARD: keep one metrics per font here, and the words' widths kept. Made headless, 42 texts
// (fourteen rows of three) took 2.2 ms with a `FontMetrics` in every `Text` and 1.5 ms with these,
// and building the font for each word measured cost 14 µs a word.
QtObject {
    id: advances

    // How much larger than the style the advances are measured, so Qt's rounding of each one to
    // 1/64 of a pixel shrinks to under a thousandth.
    readonly property int precision: 256
    // The most widths kept; past it they are all let go, as a list's words come round again
    // while it is typed in and others don't.
    readonly property int kept: 4096

    // One `FontMetrics` per font, by `key`.
    property var fonts: ({})
    // Widths measured, by the font's key and the words.
    property var widths: new Map()

    property Component metrics: Component {
        FontMetrics {}
    }

    // The width of `words` in `style`, shaped or not, in logical pixels.
    function of(style: QtObject, shaped: bool, words: string): real {
        const font = `${style.family}|${style.weight}|${style.size}|${shaped}|${style.tabular}`;
        const key = `${font}|${words}`;
        const known = advances.widths.get(key);
        if (known !== undefined)
            return known;
        const width = advances.metricsFor(font, style, shaped).advanceWidth(words) / advances.precision;
        if (advances.widths.size >= advances.kept)
            advances.widths.clear();
        advances.widths.set(key, width);
        return width;
    }

    function metricsFor(font: string, style: QtObject, shaped: bool): FontMetrics {
        const made = advances.fonts[font];
        if (made)
            return made;
        // A style with tabular figures is shaped too, since Qt applies font features only to
        // shaped text.
        const metrics = advances.metrics.createObject(advances, {
            font: Qt.font({
                family: style.family,
                weight: style.weight,
                pixelSize: style.size * advances.precision,
                kerning: shaped,
                preferShaping: shaped,
                features: style.tabular ? {
                    "tnum": 1
                } : {},
                hintingPreference: Font.PreferNoHinting
            })
        });
        advances.fonts[font] = metrics;
        return metrics;
    }
}
