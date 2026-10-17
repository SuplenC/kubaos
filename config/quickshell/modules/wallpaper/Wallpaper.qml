import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.theme
import qs.components
import qs.services

// Desktop background for one screen. Crossfades to a new wallpaper once its
// image (or a video's poster) has loaded: the picker's preview or a newly
// applied one. Stills and posters are decoded at the screen's size in device
// pixels. A video plays over its poster while Motion allows it here (mode,
// AC, not locked, screens on, no fullscreen window) and falls back to the
// poster otherwise.
//
// A ukiyo scene draws on its own background layer (namespace "ukiyo"). This
// one shows the scene's poster until ukiyo says it's on screen, then fades
// out and unmaps, so whichever of the two layers is on top, ukiyo shows, and
// there's never a black frame between them. A preview from the picker brings
// it back on top for as long as it lasts.
PanelWindow {
    id: root

    readonly property string source: Wallpapers.shown
    readonly property real dpr: screen?.devicePixelRatio ?? 1
    readonly property bool motion: Motion.desktopOn(screen)
    property WallpaperView front: first
    // ukiyo draws the current scene and nothing is being previewed.
    readonly property bool ceded: Ukiyo.shown && Wallpapers.preview === "" && Wallpapers.isScene(source)
    // 1 while shown, 0 once ceded to ukiyo; the window unmaps at 0.
    property real presence: ceded ? 0 : 1

    visible: presence > 0 || !ceded

    Behavior on presence {
        Anim {
            duration: Theme.anim.slow
            easing.bezierCurve: Theme.anim.effects
        }
    }

    anchors {
        top: true
        bottom: true
        left: true
        right: true
    }
    exclusionMode: ExclusionMode.Ignore
    color: root.ceded ? "transparent" : Theme.m3surface
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
            Anim {
                duration: Theme.anim.slow
                easing.bezierCurve: Theme.anim.effects
            }
        }
    }

    Item {
        anchors.fill: parent
        opacity: root.presence

        Layer {
            id: first
        }

        Layer {
            id: second
        }
    }
}
