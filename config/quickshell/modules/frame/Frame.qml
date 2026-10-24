import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland
import qs.theme
import qs.components
import qs.services
import qs.modules.bar
import qs.modules.clock
import qs.modules.controlcenter
import qs.modules.notifications
import qs.modules.osd
import qs.modules.powermenu
import qs.modules.sysmon
import qs.modules.wallpaper
import "../../components/drawerpath.js" as DrawerPath

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
        WlrLayershell.keyboardFocus: modal ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

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

        Behavior on reveal {
            Anim {
                duration: Theme.anim.normal
                easing.bezierCurve: Theme.anim.emphasized
            }
        }

        // Inner edges of the frame in window coordinates; panels attach to these.
        QtObject {
            id: geo

            readonly property real l: Theme.frame.thickness * root.reveal
            readonly property real t: Theme.bar.height * root.reveal
            readonly property real r: root.width - Theme.frame.thickness * root.reveal
            readonly property real b: root.height - Theme.frame.thickness * root.reveal
            readonly property ShellScreen screen: root.screen
        }

        // Dims the desktop behind modal panels. Panels float over the tiled
        // windows (no exclusive zone, so nothing reflows when one opens); the
        // scrim keeps a window's text from reading as part of the panel.
        // The wallpaper picker stays undimmed: the desktop is its preview.
        Rectangle {
            anchors.fill: parent
            color: Theme.m3scrim
            opacity: ({
                    powermenu: 0.45,
                    controlcenter: 0.2,
                    clock: 0.2,
                    sysmon: 0.2
                })[root.panel] ?? 0
            visible: opacity > 0

            Behavior on opacity {
                Anim {
                    duration: Theme.anim.normal
                    easing.bezierCurve: Theme.anim.effects
                }
            }
        }

        Item {
            id: background

            anchors.fill: parent
            opacity: Theme.frame.opacity
            layer.enabled: true
            layer.effect: MultiEffect {
                shadowEnabled: true
                shadowColor: Theme.alpha(Theme.m3shadow, Theme.elevation.opacity)
                shadowBlur: 1
                blurMax: Theme.elevation.frameBlur
            }

            Shape {
                anchors.fill: parent
                preferredRendererType: Shape.CurveRenderer

                ShapePath {
                    fillColor: Theme.m3surface
                    strokeWidth: -1
                    fillRule: ShapePath.OddEvenFill

                    PathSvg {
                        path: DrawerPath.frame(root.width, root.height, geo, Theme.frame.rounding * root.reveal)
                    }
                }

                PanelPath {
                    drawer: controlCenter
                }

                PanelPath {
                    drawer: powerMenu
                }

                PanelPath {
                    drawer: clockPopup
                }

                PanelPath {
                    drawer: sysmonPopup
                }

                PanelPath {
                    drawer: wallpaperPicker
                }

                PanelPath {
                    drawer: popups
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

        Bar {
            y: geo.t - Theme.bar.height
            width: root.width
            height: Theme.bar.height
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
                shown: Notifs.popups.length > 0 && Hypr.focusedScreen === root.screen && root.panel !== "controlcenter"
            }

            Osd {
                id: osd

                frame: geo
                screen: root.screen
            }

            ControlCenter {
                id: controlCenter

                frame: geo
                shown: root.panel === "controlcenter"
            }

            ClockPopup {
                id: clockPopup

                frame: geo
                shown: root.panel === "clock"
            }

            SysmonPopup {
                id: sysmonPopup

                frame: geo
                shown: root.panel === "sysmon"
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
        }
    }

    component PanelPath: ShapePath {
        required property Drawer drawer

        fillColor: Theme.m3surface
        strokeWidth: -1

        PathSvg {
            path: drawer.path
        }
    }
}
