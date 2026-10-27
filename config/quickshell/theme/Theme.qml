pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Design tokens. Every component reads sizes, colors, type and motion from
// here; nothing else hardcodes them.
//
// Colors are Material 3 roles read from ~/.local/state/theme/colors.json
// (written by `dot-theme` through matugen) and reloaded when that file
// changes. Until it exists, the built-in palette below is used (the same as
// config/matugen/default-colors.json). Roles are prefixed with m3 (m3primary,
// m3onSurface, ...) because QML reads any property named on<Something> as a
// signal handler.
Singleton {
    id: root

    readonly property string stateDir: (Quickshell.env("XDG_STATE_HOME") || `${Quickshell.env("HOME")}/.local/state`) + "/theme"

    // Merged palette: generated colors on top of the defaults, so a partial file still works.
    property var palette: defaults

    readonly property color m3primary: palette.primary
    readonly property color m3onPrimary: palette.on_primary
    readonly property color m3primaryContainer: palette.primary_container
    readonly property color m3onPrimaryContainer: palette.on_primary_container
    readonly property color m3secondary: palette.secondary
    readonly property color m3onSecondary: palette.on_secondary
    readonly property color m3secondaryContainer: palette.secondary_container
    readonly property color m3onSecondaryContainer: palette.on_secondary_container
    readonly property color m3tertiary: palette.tertiary
    readonly property color m3onTertiary: palette.on_tertiary
    readonly property color m3tertiaryContainer: palette.tertiary_container
    readonly property color m3onTertiaryContainer: palette.on_tertiary_container
    readonly property color m3error: palette.error
    readonly property color m3onError: palette.on_error
    readonly property color m3errorContainer: palette.error_container
    readonly property color m3onErrorContainer: palette.on_error_container
    readonly property color m3background: palette.background
    readonly property color m3onBackground: palette.on_background
    readonly property color m3surface: palette.surface
    readonly property color m3onSurface: palette.on_surface
    readonly property color m3surfaceVariant: palette.surface_variant
    readonly property color m3onSurfaceVariant: palette.on_surface_variant
    readonly property color m3surfaceDim: palette.surface_dim
    readonly property color m3surfaceBright: palette.surface_bright
    readonly property color m3surfaceContainerLowest: palette.surface_container_lowest
    readonly property color m3surfaceContainerLow: palette.surface_container_low
    readonly property color m3surfaceContainer: palette.surface_container
    readonly property color m3surfaceContainerHigh: palette.surface_container_high
    readonly property color m3surfaceContainerHighest: palette.surface_container_highest
    readonly property color m3outline: palette.outline
    readonly property color m3outlineVariant: palette.outline_variant
    readonly property color m3shadow: palette.shadow
    readonly property color m3scrim: palette.scrim
    readonly property color m3inverseSurface: palette.inverse_surface
    readonly property color m3inverseOnSurface: palette.inverse_on_surface
    readonly property color m3inversePrimary: palette.inverse_primary
    // Amber, between fine and the error colour: a reading close to its limit.
    // Material 3 has no such role, so matugen never writes it and the default
    // below holds.
    readonly property color warning: palette.warning

    // Same color with a different alpha.
    function alpha(c: color, a: real): color {
        return Qt.rgba(c.r, c.g, c.b, a);
    }

    // 4 px grid.
    readonly property QtObject spacing: QtObject {
        readonly property int xxs: 2
        readonly property int xs: 4
        readonly property int sm: 8
        readonly property int md: 12
        readonly property int lg: 16
        readonly property int xl: 20
        readonly property int xxl: 28
        readonly property int xxxl: 40
    }

    // Corner radii. The design language's are roles (docs/DESIGN.md, "Space
    // and shape"): small 4 (xs), control 12 (md), surface 24 (the frame,
    // panels) and full; any other radius comes from nesting. sm, lg, xl and
    // xxl predate the roles and are moved onto them with the shell's
    // alignment.
    readonly property QtObject rounding: QtObject {
        readonly property int xs: 4
        readonly property int sm: 8
        readonly property int md: 12
        readonly property int lg: 16
        readonly property int xl: 20
        readonly property int xxl: 28
        readonly property int full: 1000
    }

    // The screen frame: bar on top, a thin border on the other three edges,
    // rounded inner corners. Panels grow out of it (modules/frame/).
    readonly property QtObject frame: QtObject {
        readonly property int thickness: 8
        readonly property int rounding: 24      // inner corners of the frame: a surface
        readonly property int panelRounding: 24 // outer corners of panels
        readonly property int flare: 20         // concave fillet where a panel meets the frame
        readonly property real opacity: 0.94    // frame + panels; Hyprland blurs what shows through
    }

    // Sized for Full HD density, about 1920 logical pixels across: every
    // screen whose scale brings it there shows the same sizes.
    readonly property QtObject bar: QtObject {
        readonly property int height: 36
        readonly property int pillHeight: 26
        readonly property int icon: 16
        readonly property int padding: 8        // left/right inset of the bar content
        readonly property int cellFont: 12      // the workspace numbers and the group names beside them
        readonly property int groupNameMax: 96  // widest a group's name draws before it elides
    }

    // Typography. Geist for the interface (tabular figures for anything that
    // ticks), M PLUS 1 for the few Japanese accents, Material Symbols for icons.
    readonly property QtObject font: QtObject {
        readonly property string sans: "Geist"
        readonly property string mono: "JetBrainsMono Nerd Font"
        readonly property string jp: "M PLUS 1"
        readonly property string icons: "Material Symbols Rounded"
        readonly property int xs: 10
        readonly property int sm: 11
        readonly property int md: 13
        readonly property int lg: 14
        readonly property int xl: 18
        readonly property int xxl: 24
        readonly property int display: 116
        readonly property var tabular: ({ tnum: 1 })
    }

    // Motion. Durations in ms; curves are Material 3 cubic beziers for
    // Easing.BezierSpline. "Spatial" curves overshoot a little and are used
    // for things that move or grow; "effects" curves for opacity and color.
    readonly property QtObject anim: QtObject {
        readonly property int fast: 150     // hover, press, small color changes
        readonly property int small: 200    // focus moves, toggles
        readonly property int normal: 300   // panel exit, list moves
        readonly property int large: 450    // panel enter (spatial)
        readonly property int slow: 600     // wallpaper crossfade, lock screen
        readonly property int stagger: 22   // per-item delay in staggered lists

        readonly property list<real> standard: [0.2, 0, 0, 1, 1, 1]
        readonly property list<real> standardAccel: [0.3, 0, 1, 1, 1, 1]
        readonly property list<real> standardDecel: [0, 0, 0, 1, 1, 1]
        readonly property list<real> emphasized: [0.05, 0, 2 / 15, 0.06, 1 / 6, 0.4, 5 / 24, 0.82, 0.25, 1, 1, 1]
        readonly property list<real> emphasizedAccel: [0.3, 0, 0.8, 0.15, 1, 1]
        readonly property list<real> emphasizedDecel: [0.05, 0.7, 0.1, 1, 1, 1]
        readonly property list<real> spatial: [0.38, 1.21, 0.22, 1, 1, 1]
        readonly property list<real> spatialFast: [0.42, 1.67, 0.21, 0.9, 1, 1]
        readonly property list<real> effects: [0.34, 0.8, 0.34, 1, 1, 1]
    }

    // Elevation for floating surfaces (RectangularShadow in components/Elevation).
    readonly property QtObject elevation: QtObject {
        readonly property real opacity: 0.65
        readonly property int frameBlur: 28     // shadow under the frame and its panels
    }

    // Opacity of the hover/press overlay drawn on interactive surfaces (M3 state layers).
    readonly property QtObject state: QtObject {
        readonly property real hover: 0.08
        readonly property real focus: 0.12
        readonly property real press: 0.14
    }

    readonly property var defaults: ({
            primary: "#ffb779",
            on_primary: "#4c2700",
            primary_container: "#d37b1b",
            on_primary_container: "#000000",
            secondary: "#f2bc8e",
            on_secondary: "#492806",
            secondary_container: "#643e1b",
            on_secondary_container: "#ffd9bb",
            tertiary: "#85cfff",
            on_tertiary: "#00344c",
            tertiary_container: "#0f9ad6",
            on_tertiary_container: "#000000",
            error: "#ffb4ab",
            on_error: "#690005",
            error_container: "#93000a",
            on_error_container: "#ffdad6",
            background: "#19120b",
            on_background: "#f0dfd4",
            surface: "#19120b",
            on_surface: "#f0dfd4",
            surface_variant: "#544436",
            on_surface_variant: "#dac2b1",
            surface_dim: "#19120b",
            surface_bright: "#413730",
            surface_container_lowest: "#140d07",
            surface_container_low: "#221a13",
            surface_container: "#261e17",
            surface_container_high: "#312821",
            surface_container_highest: "#3d332b",
            outline: "#a28d7d",
            outline_variant: "#544436",
            shadow: "#000000",
            scrim: "#000000",
            inverse_surface: "#f0dfd4",
            inverse_on_surface: "#382f27",
            inverse_primary: "#8e4e00",
            warning: "#f2c14e"
        })

    function load(text: string): void {
        try {
            root.palette = Object.assign({}, root.defaults, JSON.parse(text));
        } catch (e) {
            console.warn(`Theme: ignoring unreadable colors.json: ${e}`);
        }
    }

    FileView {
        path: `${root.stateDir}/colors.json`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.load(text())
        onLoadFailed: root.palette = root.defaults
    }
}
