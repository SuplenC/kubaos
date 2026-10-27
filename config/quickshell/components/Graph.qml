import QtQuick
import KubaOS.Components as K

// A line graph of the last minute of a series, in the theme's colours, filled
// under the line: the system monitor's popup graphs and the bar pill's
// sparklines. Points are [t, value] pairs, oldest first; the newest sits at
// the right edge. It paints when a sample comes, its colour or size changes,
// or it shows again; never per frame.
Canvas {
    id: root

    property var points: []
    // A second series over the first, unfilled: upload over download.
    property var second: []
    // The arrays change in place, so this is what a repaint follows.
    property int revision: 0
    // The top of the scale; 0 fits the highest point.
    property real max: 100
    // The time across, in ms.
    property real span: 60000
    property color color: K.Tokens.color.primary
    property color secondColor: K.Tokens.color.tertiary
    property bool filled: true
    // Whether it takes the room a `K.Row` or `K.Stack` gives it.
    property bool fillWidth: false
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // The line: the thin stroke, whole device pixels at every scale.
    property real lineWidth: K.Look.thin(root.dpr)
    // How strong the fill under the line is: an accent's tint under itself.
    property real fillOpacity: K.Tokens.opacity.tint

    onRevisionChanged: requestPaint()
    onColorChanged: requestPaint()
    onSecondColorChanged: requestPaint()
    onLineWidthChanged: requestPaint()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onVisibleChanged: if (visible)
        requestPaint()

    function newest(series: var): real {
        return series.length > 0 ? series[series.length - 1][0] : 0;
    }

    function draw(ctx: var, series: var, stroke: color, fill: bool, end: real, top: real): void {
        if (series.length < 2)
            return;
        const x = t => width - (end - t) / span * width;
        const y = v => height - lineWidth / 2 - Math.min(1, Math.max(0, v / top)) * (height - lineWidth);
        ctx.beginPath();
        for (let i = 0; i < series.length; i++) {
            if (i === 0)
                ctx.moveTo(x(series[i][0]), y(series[i][1]));
            else
                ctx.lineTo(x(series[i][0]), y(series[i][1]));
        }
        ctx.globalAlpha = 1;
        ctx.strokeStyle = String(stroke);
        ctx.lineWidth = lineWidth;
        ctx.lineJoin = "round";
        ctx.stroke();
        if (!fill)
            return;
        ctx.lineTo(x(series[series.length - 1][0]), height);
        ctx.lineTo(x(series[0][0]), height);
        ctx.closePath();
        ctx.globalAlpha = fillOpacity;
        ctx.fillStyle = String(stroke);
        ctx.fill();
    }

    onPaint: {
        const ctx = getContext("2d");
        ctx.reset();
        if (!visible || points.length + second.length < 2)
            return;
        const end = Math.max(newest(points), newest(second));
        let top = max;
        if (top <= 0) {
            top = 1;
            for (const p of points)
                top = Math.max(top, p[1]);
            for (const p of second)
                top = Math.max(top, p[1]);
            top *= 1.15;
        }
        draw(ctx, points, color, filled, end, top);
        draw(ctx, second, secondColor, false, end, top);
    }
}
