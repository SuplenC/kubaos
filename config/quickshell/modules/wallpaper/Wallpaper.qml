import QtQuick
import Quickshell
import Quickshell.Wayland
import KubaOS.Components as K
import qs.components
import qs.services

// Desktop background for one screen. Crossfades to a new wallpaper once its
// image (or a video's poster) has loaded: the picker's preview or a newly
// applied one. Stills and posters are decoded at the screen's size in device
// pixels. A video plays over its poster while Motion allows it here (mode,
// AC, not locked, screens on, no fullscreen window) and falls back to the
// poster otherwise.
PanelWindow {
    id: root

    readonly property string source: Wallpapers.shown
    readonly property real dpr: screen?.devicePixelRatio ?? 1
    readonly property bool motion: Motion.desktopOn(screen)
    property WallpaperView front: first

    anchors {
        top: true
        bottom: true
        left: true
        right: true
    }
    exclusionMode: ExclusionMode.Ignore
    color: K.Tokens.color.surface
    WlrLayershell.layer: WlrLayer.Background
    WlrLayershell.namespace: "qs-wallpaper"

    // Load into the hidden layer; it becomes `front` when ready (see onReadyChanged).
    // If the hidden layer already holds it (going back from a preview), swap now.
    onSourceChanged: {
        const back = front === first ? second : first;
        if (back.path === source && back.ready)
            front = back;
        else
            back.path = source;
    }
    Component.onCompleted: first.path = source

    // What plays here, for `qs ipc call wallpaper status`.
    readonly property string reported: front.moving ? front.path : ""
    onReportedChanged: Motion.report(screen?.name ?? "", reported)
    Component.onDestruction: Motion.report(screen?.name ?? "", "")

    component Layer: WallpaperView {
        anchors.fill: parent
        sourceSize: Qt.size(Math.ceil(root.width * root.dpr), Math.ceil(root.height * root.dpr))
        playing: root.front === this && root.motion
        opacity: root.front === this ? 1 : 0
        z: root.front === this ? 1 : 0
        onReadyChanged: if (ready && path === root.source)
            root.front = this

        Behavior on opacity {
            NumberAnimation {
                duration: K.Tokens.duration.slow
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
    }

    Layer {
        id: first
    }

    Layer {
        id: second
    }
}
