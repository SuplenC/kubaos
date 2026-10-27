import QtQuick
import KubaOS.Components as K

// A hairline across all the room it is given, in the soft outline: `kubaos-ui`'s
// `layout::divider`. One device pixel at every scale.
Rectangle {
    id: root

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Divider",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    implicitHeight: K.Look.hairline(root.dpr)
    color: K.Look.alpha(Tokens.color.outline_variant, Tokens.opacity.outline_soft)
}
