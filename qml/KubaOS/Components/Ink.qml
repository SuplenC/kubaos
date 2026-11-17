pragma Singleton

import QtQuick

// Where text draws its ink: the middle of what its glyphs fill, top to bottom, as `kubaos-ui`'s
// `text::ink` measures it, so a lone glyph on a square (an app's initial on its `IconTile`) sits
// where iced puts it.
//
// The glyphs are measured at `weighedAt` pixels with vertical hinting, whose outlines snap to
// whole rows as the hinted rasters iced measures do.
QtObject {
    id: ink

    // The size the ink is measured at, in pixels: a row of it is under a pixel of the largest
    // style it places.
    // design-allow(size): a size glyphs are measured at, never drawn: iced's and the mockup's.
    readonly property int weighedAt: 64
    // The most middles kept; past it they are all let go, as `Advances` lets its widths go.
    readonly property int kept: 4096

    // One `FontMetrics` per font, by its family and weight.
    property var fonts: ({})
    // Middles measured, by the font and the words.
    property var middles: new Map()

    property Component metrics: Component {
        FontMetrics {}
    }

    // The middle of `words`' ink in `style`, top to bottom, in ems above the baseline, below it
    // counted: a capital's is half its cap height, whatever its strokes weigh, so B, L and K sit
    // alike; an ideograph's takes in what it draws below the baseline. Words that draw nothing
    // give 0.
    function middle(style: QtObject, words: string): real {
        const font = `${style.family}|${style.weight}`;
        const key = `${font}|${words}`;
        const known = ink.middles.get(key);
        if (known !== undefined)
            return known;
        const box = ink.metricsFor(font, style).tightBoundingRect(words);
        const measured = box.height > 0 ? -(box.y + box.height / 2) / ink.weighedAt : 0;
        if (ink.middles.size >= ink.kept)
            ink.middles.clear();
        ink.middles.set(key, measured);
        return measured;
    }

    function metricsFor(font: string, style: QtObject): FontMetrics {
        const made = ink.fonts[font];
        if (made)
            return made;
        const metrics = ink.metrics.createObject(ink, {
            font: Qt.font({
                family: style.family,
                weight: style.weight,
                pixelSize: ink.weighedAt,
                hintingPreference: Font.PreferVerticalHinting
            })
        });
        ink.fonts[font] = metrics;
        return metrics;
    }
}
