import QtQuick
import KubaOS.Components as K

// A count that rolls as it changes, as an odometer does: `kubaos-ui`'s `rolling::rolling_number`.
// Its figures are tabular, each in a slot one figure wide and the style's line tall that cuts what
// leaves it: the figure's advance rounded up to a whole device pixel, so each figure starts on one.
// The box is as many slots wide as the widest value it is given (`widest`, its figures all 0), its
// figures held to its end, so nothing beside it moves as the count grows a figure.
//
// Only the figures that change roll. Going up, the new one comes in from below as the old one
// leaves upward; going down, from above as the old one leaves downward, the pair moving together
// over `short` on `standard`. A figure the count grows comes in from nothing, and one it sheds
// leaves into nothing. With reduce motion the figures change in place.
Item {
    id: root

    property int value: 0
    // The widest value it shows, its figures all 0: "000" for a count up to 999. A value wider
    // than it widens the box.
    property string widest: "0"
    property QtObject style: Tokens.type.mono_large
    property color color: Tokens.color.on_surface

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // How much larger than the style a figure's advance is measured, as `Text` measures its words.
    readonly property int precision: 256
    // A figure's advance, rounded up to a whole device pixel.
    readonly property real slot: K.Look.snapUp(metrics.advanceWidth("0") / root.precision, root.dpr)
    // The figures shown, the locale's, and which way the last change went: 1 up, -1 down.
    // Set as the value changes, never bound, so the way is set before the figures change.
    property string shown: ""
    property int way: 1
    // The value `shown` holds, to tell which way the next change goes.
    property int shownValue: 0
    // Whether it has shown its first value: that one is in place, not rolled in.
    property bool ready: false
    readonly property int slots: Math.max(root.widest.length, root.shown.length)
    readonly property var probe: ({
            role: "component",
            name: "RollingNumber",
            variant: "",
            fixed: {
                width: false,
                height: true
            }
        })

    // `count` in the locale's figures, never grouped or padded.
    function format(count: int): string {
        const locale = Qt.locale();
        locale.numberOptions = Locale.OmitGroupSeparator;
        return count.toLocaleString(locale, "f", 0);
    }

    // The figure in slot `index`, counted from the box's start, or "" for a slot the count hasn't
    // reached.
    function figureAt(index: int): string {
        const at = index - (root.slots - root.shown.length);
        return at >= 0 ? root.shown[at] : "";
    }

    implicitWidth: root.slots * root.slot
    implicitHeight: root.style.line
    Accessible.role: Accessible.StaticText
    Accessible.name: root.shown

    Component.onCompleted: {
        root.shownValue = root.value;
        root.shown = root.format(root.value);
        root.ready = true;
    }
    // The way is known before the figures change, so each slot rolls the right way.
    onValueChanged: {
        const next = root.format(root.value);
        if (next === root.shown)
            return;
        root.way = root.value > root.shownValue ? 1 : -1;
        root.shownValue = root.value;
        root.shown = next;
    }

    FontMetrics {
        id: metrics

        font.family: root.style.family
        font.weight: root.style.weight
        font.pixelSize: root.style.size * root.precision
        font.features: {
            "tnum": 1
        }
        font.hintingPreference: Font.PreferNoHinting
    }

    Repeater {
        model: root.slots

        Item {
            id: slot

            required property int index
            readonly property string figure: root.figureAt(slot.index)
            // The figure it shows, and the one rolling out of it.
            property string arriving: slot.figure
            property string leaving: ""
            // How far the pair has rolled, 0 to 1.
            property real roll: 1

            x: slot.index * root.slot
            width: root.slot
            height: root.height
            clip: true

            onFigureChanged: {
                rolling.stop();
                if (K.Motion.reduced || !root.ready) {
                    slot.leaving = "";
                    slot.arriving = slot.figure;
                    slot.roll = 1;
                    return;
                }
                slot.leaving = slot.arriving;
                slot.arriving = slot.figure;
                slot.roll = 0;
                rolling.start();
            }

            NumberAnimation {
                id: rolling

                target: slot
                property: "roll"
                to: 1
                duration: Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
                onFinished: slot.leaving = ""
            }

            Figure {
                typeStyle: root.style
                width: slot.width
                height: slot.height
                color: root.color
                text: slot.leaving
                y: -slot.roll * root.way * slot.height
                visible: slot.leaving !== "" && slot.roll < 1
            }

            Figure {
                typeStyle: root.style
                width: slot.width
                height: slot.height
                color: root.color
                text: slot.arriving
                y: root.way * (slot.height - slot.roll * slot.height)
            }
        }
    }

    // One figure in `typeStyle`, from the slot's start, on the style's line.
    component Figure: Text {
        id: figure

        property QtObject typeStyle

        font.family: figure.typeStyle.family
        font.weight: figure.typeStyle.weight
        font.pixelSize: figure.typeStyle.size
        font.features: {
            "tnum": 1
        }
        font.hintingPreference: Font.PreferNoHinting
        renderType: Text.NativeRendering
        lineHeightMode: Text.FixedHeight
        lineHeight: figure.typeStyle.line
        verticalAlignment: Text.AlignVCenter
    }
}
