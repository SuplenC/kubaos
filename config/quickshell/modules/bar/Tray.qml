import QtQuick
import QtQuick.Effects
import Quickshell
import Quickshell.Widgets
import Quickshell.Services.SystemTray
import KubaOS.Components as K
import qs.components

// StatusNotifierItem tray. Left click activates, middle click secondary action, right click (or
// left on menu-only items) opens the app's menu. Each app's icon sits in a circle as tall as a
// chip, which takes the state layer under the pointer, 4 in from the pill's curve.
Pill {
    id: root

    required property QsWindow window

    // Whether an icon's RGBA `pixels` are grey and under an icon's 3:1 on the bar's surface: their
    // opaque pixels' average, with little colour in it. A coloured icon's average can be dull
    // while its pixels aren't, so colour rules it out.
    function faint(pixels: var): bool {
        let r = 0, g = 0, b = 0, count = 0, spread = 0;
        for (let i = 0; i < pixels.length; i += 4) {
            if (pixels[i + 3] < 128)
                continue;
            r += pixels[i];
            g += pixels[i + 1];
            b += pixels[i + 2];
            spread += Math.max(pixels[i], pixels[i + 1], pixels[i + 2]) - Math.min(pixels[i], pixels[i + 1], pixels[i + 2]);
            count += 1;
        }
        if (count === 0)
            return false;
        // design-allow(colour): the average measured from an app's own pixels, not a colour we draw.
        const average = Qt.rgba(r / count / 255, g / count / 255, b / count / 255, 1);
        const grey = spread / count / 255 < 0.2;
        return grey && K.Look.contrast(average, K.Tokens.color.surface) < 3;
    }

    visible: SystemTray.items.values.length > 0
    padding: K.Tokens.space.s4
    spacing: 0

    Repeater {
        model: SystemTray.items

        delegate: Item {
            id: entry

            required property SystemTrayItem modelData

            implicitWidth: K.Tokens.size.chip
            implicitHeight: K.Tokens.size.chip

            K.StateLayer {
                radius: K.Tokens.radius.full
                hovered: mouse.containsMouse
                pressed: mouse.pressed
            }

            IconImage {
                x: (K.Tokens.size.chip - K.Tokens.icon.s16) / 2
                y: (K.Tokens.size.chip - K.Tokens.icon.s16) / 2
                implicitSize: K.Tokens.icon.s16
                source: entry.modelData.icon
                asynchronous: true
                // A grey icon that would vanish into the bar is drawn in the content colour, as
                // the bar's own symbols are; the layer exists only while it is.
                layer.enabled: reader.faint
                layer.effect: MultiEffect {
                    // Colourising keeps the drawing's lightness, so a dark icon is first lifted to
                    // white (in light, pressed to black), then takes the content colour.
                    brightness: K.Tokens.mode === "light" ? -1 : 1
                    colorization: 1
                    colorizationColor: K.Tokens.color.on_surface
                }
            }

            // Reads the icon's pixels once each time it changes, never per frame: whether the app
            // drew it grey and close to the bar's own surface (a GTK symbolic icon, drawn dark for
            // a light theme), under the 3:1 an icon needs. A coloured icon keeps its colours.
            Canvas {
                id: reader

                property bool faint: false
                // The icon whose pixels are loaded, unloaded when the next one arrives, so an app
                // that changes its icon often doesn't fill the cache.
                property url loaded: ""

                // Loads `source`, keeping the last reading until its pixels are in: the layer
                // doesn't flicker off and on with every update of the icon.
                function read(source: url): void {
                    if (reader.loaded.toString() !== "")
                        reader.unloadImage(reader.loaded);
                    reader.loaded = source;
                    if (source.toString() !== "")
                        reader.loadImage(source);
                }

                width: K.Tokens.icon.s16
                height: K.Tokens.icon.s16
                opacity: 0
                onImageLoaded: requestPaint()
                onPaint: {
                    const ctx = getContext("2d");
                    ctx.clearRect(0, 0, width, height);
                    if (!isImageLoaded(reader.loaded))
                        return;
                    ctx.drawImage(reader.loaded, 0, 0, width, height);
                    reader.faint = root.faint(ctx.getImageData(0, 0, width, height).data);
                }

                Connections {
                    target: entry.modelData

                    function onIconChanged(): void {
                        reader.read(entry.modelData.icon);
                    }
                }

                Component.onCompleted: reader.read(entry.modelData.icon)
            }

            MouseArea {
                id: mouse

                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
                cursorShape: Qt.PointingHandCursor
                // A wheel's notch is 120 (Qt's QWheelEvent::angleDelta).
                onWheel: wheel => entry.modelData.scroll(wheel.angleDelta.y / 120, false)
                onClicked: mouse => {
                    const item = entry.modelData;
                    if (mouse.button === Qt.MiddleButton) {
                        item.secondaryActivate();
                    } else if (mouse.button === Qt.LeftButton && !item.onlyMenu) {
                        item.activate();
                    } else if (item.hasMenu) {
                        const pos = entry.mapToItem(null, 0, entry.height + K.Tokens.space.s12);
                        item.display(root.window, pos.x, pos.y);
                    }
                }
            }

            K.Hint {
                text: entry.modelData.tooltipTitle || entry.modelData.title
                shown: text !== "" && mouse.containsMouse
            }
        }
    }
}
