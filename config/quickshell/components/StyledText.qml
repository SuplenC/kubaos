import QtQuick
import qs.theme

// Interface text: Geist, on-surface color, hinted native rendering so small
// sizes stay crisp at scale 1 and 2.
Text {
    color: Theme.m3onSurface
    font.family: Theme.font.sans
    font.pixelSize: Theme.font.md
    textFormat: Text.PlainText
    renderType: Text.NativeRendering
    elide: Text.ElideRight
    verticalAlignment: Text.AlignVCenter
}
