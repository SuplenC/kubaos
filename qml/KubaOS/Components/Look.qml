pragma Singleton

import QtQuick

// What every component draws with besides the token values: colours with an alpha or blended,
// corners nested inside one another, and lengths on whole device pixels. The QML twin of
// `kubaos-ui`'s `look`, `scale` and `color` modules; `Tokens` holds the values.
QtObject {
    id: look

    // The grid every symbol is drawn on, in units: each drawing's viewBox.
    readonly property int symbolGrid: 24

    // `c` with its alpha multiplied by `a`.
    function alpha(c: color, a: real): color {
        return Qt.alpha(c, c.a * a);
    }

    // The linear blend from `a` to `b`, `t` of the way, alpha included.
    function mix(a: color, b: color, t: real): color {
        const blend = Qt.color(a);
        blend.r = a.r + (b.r - a.r) * t;
        blend.g = a.g + (b.g - a.g) * t;
        blend.b = a.b + (b.b - a.b) * t;
        blend.a = a.a + (b.a - a.a) * t;
        return blend;
    }

    // WCAG 2.1's relative luminance of an opaque colour: `kubaos-ui`'s `color::luminance`.
    function luminance(c: color): real {
        const channel = v => v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
        return 0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b);
    }

    // WCAG 2.1's contrast ratio of two opaque colours, 1 to 21: `color::contrast_ratio`.
    function contrast(a: color, b: color): real {
        const la = look.luminance(a);
        const lb = look.luminance(b);
        return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
    }

    // The corner of a shape filling one with corner `outer` less `inset` on every side, so both
    // share a centre. A pill in a pill is a pill. Throws when the inset reaches the outer radius:
    // nothing is left to round, and the corners can't be concentric.
    function nested(outer: int, inset: int): int {
        if (outer >= Tokens.radius.full)
            return outer;
        if (inset >= outer)
            throw new Error(`an inset of ${inset} reaches the radius ${outer}: the corners can't be concentric`);
        return outer - inset;
    }

    // The radius of a shape `by` logical pixels outside one with corner `inner` on every side,
    // concentric with it: a glow's ring around its control. A pill around a pill is a pill.
    function outset(inner: int, by: real): real {
        return inner >= Tokens.radius.full ? inner : inner + by;
    }

    // Whether `item`'s whole box shows inside every ancestor that clips it. What floats over the
    // window for a control (a hint, a tooltip) asks this, so a control scrolled out of its view,
    // even part way, takes what floats for it along.
    function unclipped(item: Item): bool {
        for (let clipper = item.parent; clipper; clipper = clipper.parent) {
            if (!clipper.clip)
                continue;
            const at = item.mapToItem(clipper, 0, 0);
            // A hair over is float error, not a box past the edge.
            const slack = 1e-3;
            if (at.x < -slack || at.y < -slack || at.x + item.width > clipper.width + slack || at.y + item.height > clipper.height + slack)
                return false;
        }
        return true;
    }

    // `v` logical pixels moved to the nearest device pixel at `dpr`.
    function snap(v: real, dpr: real): real {
        return Math.round(v * dpr) / dpr;
    }

    // Half of `spare` logical pixels on a whole device pixel at `dpr`: how far in a box is
    // centred. An odd number of spare device pixels leaves the extra one after the box.
    // HAZARD: half of an odd count is a tie, which float error breaks either way; ties round down,
    // a hair under, as `kubaos-ui`'s `Scale::centre` does, so both toolkits pick the same pixel.
    function centre(spare: real, dpr: real): real {
        return Math.round(spare * dpr / 2 - 1e-3) / dpr;
    }

    // `v` logical pixels moved up to the next whole device pixel at `dpr`. A hair under a whole
    // pixel is that pixel: float error must not add a row.
    function snapUp(v: real, dpr: real): real {
        return Math.ceil(v * dpr - 1e-3) / dpr;
    }

    // One device pixel at `dpr`, in logical pixels.
    function hairline(dpr: real): real {
        return Tokens.stroke.hairline_device_px / dpr;
    }

    // A stroke of `logical` pixels as whole device pixels, rounded half up and never under one.
    function stroke(logical: real, dpr: real): real {
        return Math.max(1, Math.round(logical * dpr)) / dpr;
    }

    // The thin stroke of the spinner, a field's caret and the partial checkbox's dash.
    function thin(dpr: real): real {
        return stroke(Tokens.stroke.thin, dpr);
    }

    // A symbol's stroke on its 24 unit grid when drawn `size` logical pixels across at `dpr`: its
    // two units scaled to the size, rounded half up to whole device pixels, back in grid units.
    function symbolStroke(size: int, dpr: real): real {
        return stroke(Tokens.stroke.symbol * size / symbolGrid, dpr) * symbolGrid / size;
    }
}
