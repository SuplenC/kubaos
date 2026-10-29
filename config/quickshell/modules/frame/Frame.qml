import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland
import KubaOS.Components as K
import qs.services
import qs.modules.bar
import qs.modules.clock
import qs.modules.controlcenter
import qs.modules.notifications
import qs.modules.osd
import qs.modules.powermenu
import qs.modules.sysmon
import qs.modules.wallpaper

// The screen frame for one monitor: the bar along the top, a thin border on
// the other edges with rounded inner corners, and every panel, each growing
// out of an edge. It is one transparent full-screen layer; the input mask
// lets clicks through to windows except on the frame and open panels.
//
// Frame and panel backgrounds are paths in one Shape inside one layer, so
// they share a single shadow and opacity and read as one surface. Panel
// contents sit above that layer at full opacity.
//
// Exclusive zones come from four empty windows (Exclusions.qml), so this
// window never moves when they change.
Scope {
    id: scope

    required property ShellScreen screen

    Exclusions {
        screen: scope.screen
    }

    PanelWindow {
        id: root

        readonly property HyprlandMonitor monitor: Hyprland.monitorFor(screen)
        readonly property bool fullscreen: monitor?.activeWorkspace?.hasFullscreen ?? false
        readonly property bool focused: Panels.screen === screen
        // The open panel on this screen ("" if none). All of them are modal.
        readonly property string panel: focused ? Panels.current : ""
        readonly property bool modal: panel !== ""
        // 1 = frame shown, 0 = retracted (a fullscreen window, no panel open).
        property real reveal: fullscreen && !modal ? 0 : 1
        readonly property var panels: [controlCenter, notificationPanel, clockPopup, sysmonPanel, powerMenu, wallpaperPicker]
        // The panel on screen (opening, open or closing) that reaches into
        // the popups' corner column, or null. Popups never draw over it.
        readonly property K.Sheet toastCover: panels.find(d => (d.shown || d.active) && d.area.x + d.area.width + K.Tokens.radius.surface > geo.r - popups.contentWidth) ?? null

        screen: scope.screen
        color: "transparent"
        exclusionMode: ExclusionMode.Ignore
        anchors {
            top: true
            bottom: true
            left: true
            right: true
        }
        WlrLayershell.namespace: "qs-frame"
        WlrLayershell.layer: fullscreen && modal ? WlrLayer.Overlay : WlrLayer.Top
        // The panel holds the keyboard only while the pointer is on its
        // screen: an exclusive layer keeps Hyprland from focusing anything
        // else, so holding it everywhere left the other screens dead.
        WlrLayershell.keyboardFocus: modal && Hypr.focusedScreen === screen ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

        // Clicks reach the frame, the bar and the notification popups; with a
        // panel open, the whole screen (a click outside closes it).
        mask: Region {
            x: 0
            y: 0
            width: root.width
            height: root.height

            Region {
                intersection: Intersection.Subtract
                x: geo.l
                y: geo.t
                width: root.modal ? 0 : geo.r - geo.l
                height: root.modal ? 0 : geo.b - geo.t
            }

            Region {
                intersection: Intersection.Combine
                item: popups
            }
        }

        // The frame comes back as a panel arrives and retracts as one leaves. HAZARD: pick the
        // timing from where it is going, not from `fullscreen`: the animation starts before
        // bindings on it update.
        Behavior on reveal {
            id: revealing

            NumberAnimation {
                duration: revealing.targetValue > 0 ? K.Tokens.duration.long : K.Tokens.duration.medium
                easing.type: Easing.BezierSpline
                easing.bezierCurve: revealing.targetValue > 0 ? K.Tokens.easing.emphasized_decel : K.Tokens.easing.emphasized_accel
            }
        }

        // Inner edges of the frame in window coordinates; panels attach to these.
        QtObject {
            id: geo

            readonly property real l: K.Tokens.space.s8 * root.reveal
            readonly property real t: K.Tokens.size.bar * root.reveal
            readonly property real r: root.width - K.Tokens.space.s8 * root.reveal
            readonly property real b: root.height - K.Tokens.space.s8 * root.reveal
            readonly property ShellScreen screen: root.screen
        }

        // Dims the desktop behind modal panels. Panels float over the tiled
        // windows (no exclusive zone, so nothing reflows when one opens); the
        // scrim keeps a window's text from reading as part of the panel.
        // The wallpaper picker stays undimmed: the desktop is its preview. The power menu asks
        // what to do with the session, so it dims as a dialog does.
        Rectangle {
            anchors.fill: parent
            color: K.Tokens.color.scrim
            opacity: ({
                    powermenu: K.Tokens.opacity.scrim,
                    controlcenter: K.Tokens.opacity.panel_scrim,
                    notifications: K.Tokens.opacity.panel_scrim,
                    clock: K.Tokens.opacity.panel_scrim,
                    sysmon: K.Tokens.opacity.panel_scrim
                })[root.panel] ?? 0
            visible: opacity > 0

            Behavior on opacity {
                NumberAnimation {
                    duration: K.Tokens.duration.medium
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard
                }
            }
        }

        Item {
            id: background

            anchors.fill: parent
            opacity: K.Tokens.opacity.frame
            layer.enabled: true
            layer.effect: MultiEffect {
                shadowEnabled: true
                shadowColor: K.Look.alpha(K.Tokens.color.shadow, K.Tokens.shadow.panel.opacity)
                shadowBlur: 1
                blurMax: K.Tokens.shadow.panel.blur
                shadowVerticalOffset: K.Tokens.shadow.panel.y
            }

            Shape {
                anchors.fill: parent
                preferredRendererType: Shape.CurveRenderer

                ShapePath {
                    fillColor: K.Tokens.color.surface
                    // design-allow(size): -1 is ShapePath's "no stroke", not a width.
                    strokeWidth: -1
                    fillRule: ShapePath.OddEvenFill

                    PathSvg {
                        path: K.SheetPath.frame(root.width, root.height, geo, K.Tokens.radius.surface * root.reveal)
                    }
                }

                PanelPath {
                    drawer: controlCenter
                }

                PanelPath {
                    drawer: notificationPanel
                }

                PanelPath {
                    drawer: powerMenu
                }

                PanelPath {
                    drawer: clockPopup
                }

                PanelPath {
                    drawer: sysmonPanel
                }

                PanelPath {
                    drawer: wallpaperPicker
                }

                PanelPath {
                    drawer: popups
                }

                PanelPath {
                    drawer: popupsBelow
                }

                PanelPath {
                    drawer: osd
                }
            }
        }

        MouseArea {
            anchors.fill: parent
            enabled: root.modal
            onClicked: Panels.close()
        }

        // The peek shows while the pointer rests on the bell or on the peek, and a moment after it
        // leaves either, so it can cross the bar's edge from one to the other.
        readonly property bool peekWanted: bar.bellPeeking || popups.peekHovered

        onPeekWantedChanged: {
            if (!root.peekWanted)
                peekLeave.restart();
        }

        Timer {
            id: peekLeave

            interval: K.Tokens.duration.short
        }

        Bar {
            id: bar

            y: geo.t - K.Tokens.size.bar
            width: root.width
            height: K.Tokens.size.bar
            screen: root.screen
            window: root
            opacity: root.reveal
        }

        FocusScope {
            anchors.fill: parent
            focus: true
            Keys.onEscapePressed: Panels.close()

            Popups {
                id: popups

                frame: geo
                here: Hypr.focusedScreen === root.screen
                shown: (popups.hasPopups || popups.hasPeek) && !root.toastCover
                covered: root.toastCover !== null
                peeking: root.peekWanted || peekLeave.running
            }

            PopupsBelow {
                id: popupsBelow

                frame: geo
                cover: root.toastCover
            }

            Osd {
                id: osd

                frame: geo
                screen: root.screen
            }

            // The three panels out of the right edge are twins: switching between them keeps the
            // surface and swaps the content, each coming from the side of its button in the bar,
            // where the system monitor's pill, the bell and the status pill sit in that order.
            SysmonPanel {
                id: sysmonPanel

                frame: geo
                shown: root.panel === "sysmon"
                twins: [notificationPanel, controlCenter]
                side: 0
            }

            NotificationPanel {
                id: notificationPanel

                frame: geo
                shown: root.panel === "notifications"
                twins: [sysmonPanel, controlCenter]
                side: 1
            }

            ControlCenter {
                id: controlCenter

                frame: geo
                shown: root.panel === "controlcenter"
                twins: [sysmonPanel, notificationPanel]
                side: 2
            }

            ClockPopup {
                id: clockPopup

                frame: geo
                shown: root.panel === "clock"
            }

            PowerMenu {
                id: powerMenu

                frame: geo
                shown: root.panel === "powermenu"
            }

            WallpaperPicker {
                id: wallpaperPicker

                frame: geo
                shown: root.panel === "wallpaper"
            }

            // `c_notifications`'s probe (test/lib/e2e.sh), with KUBAOS_TOAST_PROBE=1 only.
            LazyLoader {
                active: Quickshell.env("KUBAOS_TOAST_PROBE") === "1"

                ToastProbe {
                    panels: root.panels
                    stacks: [popups, popupsBelow]
                }
            }
        }
    }

    component PanelPath: ShapePath {
        required property K.Sheet drawer

        fillColor: K.Tokens.color.surface
        // design-allow(size): -1 is ShapePath's "no stroke", not a width.
        strokeWidth: -1

        PathSvg {
            path: drawer.path
        }
    }
}
