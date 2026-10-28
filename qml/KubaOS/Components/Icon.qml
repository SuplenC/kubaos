import QtQuick
import KubaOS.Components as K

// A symbol of KubaOS Symbols in a square box of its size: `kubaos-ui`'s `icon::icon`. Drawn from
// its SVG with the stroke set for its size and the scale it shows at, whole device pixels, so no
// line blurs between two.
Item {
    id: root

    // The symbol, by the Material Symbols name it replaces (`arrow_forward`).
    property string name
    // The icon beside body text by default; beside another style, that style's `icon`.
    property int size: Tokens.type.body.icon
    property color color: Tokens.color.on_surface

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "icon",
            name: root.name
        })

    implicitWidth: root.size
    implicitHeight: root.size

    Image {
        anchors.fill: parent
        sourceSize.width: Math.round(root.size * root.dpr)
        sourceSize.height: Math.round(root.size * root.dpr)
        // The colour's alpha goes to the opacity, so a disabled or fading icon dims like its label.
        opacity: root.color.a
        source: {
            const drawing = K.Symbols.svg[root.name] ?? "";
            const stroke = K.Look.symbolStroke(root.size, root.dpr);
            const svg = drawing.replace("stroke-width=\"2\"", `stroke-width="${stroke}"`).replace("currentColor", Qt.alpha(root.color, 1).toString());
            return drawing === "" ? "" : `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
        }
    }
}
