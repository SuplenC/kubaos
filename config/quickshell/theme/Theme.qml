pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import KubaOS.Components as K

// The shell's theme. The values come from the design tokens, `KubaOS.Components`' `Tokens`
// (generated from design/tokens.toml); this file gives them the names the shell's older components
// read, and feeds the tokens the colours matugen derived from the wallpaper
// (~/.local/state/theme/colors.json, written by `dot-theme`, reloaded when it changes). Roles are
// prefixed with m3 (m3primary, m3onSurface, ...) because QML reads any property named
// on<Something> as a signal handler. New code reads `KubaOS.Components` directly.
//
// TODO(#149): drop the values below that have no token (the spacing, radii, type sizes and motion
// the shell predates the tokens with) once the Command menu reads `KubaOS.Components`; until then
// they keep it as it looks.
// TODO(#151): the same for the lock screen, the OSD, the power menu and the polkit prompt.
Singleton {
    id: root

    readonly property string stateDir: (Quickshell.env("XDG_STATE_HOME") || `${Quickshell.env("HOME")}/.local/state`) + "/theme"

    // matugen's colours as colors.json holds them, or nothing before it has been read.
    readonly property var palette: K.Tokens.matugen ?? ({})

    readonly property color m3primary: K.Tokens.color.primary
    readonly property color m3onPrimary: K.Tokens.color.on_primary
    readonly property color m3primaryContainer: K.Tokens.color.primary_container
    readonly property color m3onPrimaryContainer: K.Tokens.color.on_primary_container
    readonly property color m3secondary: K.Tokens.color.secondary
    readonly property color m3onSecondary: K.Tokens.color.on_secondary
    readonly property color m3secondaryContainer: K.Tokens.color.secondary_container
    readonly property color m3onSecondaryContainer: K.Tokens.color.on_secondary_container
    readonly property color m3tertiary: K.Tokens.color.tertiary
    readonly property color m3onTertiary: K.Tokens.color.on_tertiary
    readonly property color m3tertiaryContainer: K.Tokens.color.tertiary_container
    readonly property color m3onTertiaryContainer: K.Tokens.color.on_tertiary_container
    readonly property color m3error: K.Tokens.color.error
    readonly property color m3onError: K.Tokens.color.on_error
    readonly property color m3errorContainer: K.Tokens.color.error_container
    readonly property color m3onErrorContainer: K.Tokens.color.on_error_container
    readonly property color m3surface: K.Tokens.color.surface
    readonly property color m3onSurface: K.Tokens.color.on_surface
    readonly property color m3surfaceVariant: K.Tokens.color.surface_variant
    readonly property color m3onSurfaceVariant: K.Tokens.color.on_surface_variant
    readonly property color m3surfaceDim: K.Tokens.color.surface_dim
    readonly property color m3surfaceBright: K.Tokens.color.surface_bright
    readonly property color m3surfaceContainerLowest: K.Tokens.color.surface_container_lowest
    readonly property color m3surfaceContainerLow: K.Tokens.color.surface_container_low
    readonly property color m3surfaceContainer: K.Tokens.color.surface_container
    readonly property color m3surfaceContainerHigh: K.Tokens.color.surface_container_high
    readonly property color m3surfaceContainerHighest: K.Tokens.color.surface_container_highest
    readonly property color m3outline: K.Tokens.color.outline
    readonly property color m3outlineVariant: K.Tokens.color.outline_variant
    readonly property color m3shadow: K.Tokens.color.shadow
    readonly property color m3scrim: K.Tokens.color.scrim
    readonly property color m3inverseSurface: K.Tokens.color.inverse_surface
    readonly property color m3inverseOnSurface: K.Tokens.color.inverse_on_surface
    readonly property color m3inversePrimary: K.Tokens.color.inverse_primary
    // Amber, between fine and the error colour: a reading close to its limit.
    readonly property color warning: K.Tokens.color.warning

    // The same colour with alpha `a`.
    function alpha(c: color, a: real): color {
        return Qt.alpha(c, a);
    }

    readonly property QtObject spacing: QtObject {
        readonly property int xs: K.Tokens.space.s4
        readonly property int sm: K.Tokens.space.s8
        readonly property int md: K.Tokens.space.s12
        readonly property int lg: K.Tokens.space.s16
        readonly property int xl: K.Tokens.space.s20
        readonly property int xxxl: K.Tokens.space.s40
    }

    // Corner radii. The design language's are roles (docs/DESIGN.md, "Space and shape"): control
    // (md) and full; any other radius comes from nesting. sm, lg, xl and xxl predate the roles.
    readonly property QtObject rounding: QtObject {
        readonly property int sm: 8
        readonly property int md: K.Tokens.radius.control
        readonly property int lg: 16
        readonly property int xl: 20
        readonly property int xxl: 28
        readonly property int full: K.Tokens.radius.full
    }

    // The frame's surface, which the Command menu's window matches.
    readonly property QtObject frame: QtObject {
        readonly property int panelRounding: K.Tokens.radius.surface
        readonly property real opacity: K.Tokens.opacity.frame
    }

    // Typography. Geist for the interface (tabular figures for anything that ticks), M PLUS 1 for
    // the few Japanese accents, Material Symbols for icons.
    readonly property QtObject font: QtObject {
        readonly property string sans: K.Tokens.font.sans
        readonly property string mono: "JetBrainsMono Nerd Font"
        readonly property string jp: K.Tokens.font.jp
        readonly property string icons: "Material Symbols Rounded"
        readonly property int sm: 11
        readonly property int md: 13
        readonly property int lg: 14
        readonly property int xl: 18
        readonly property int display: 116
        readonly property var tabular: ({
                tnum: 1
            })
    }

    // Motion. Durations in ms; curves are cubic béziers for Easing.BezierSpline. "Spatial" curves
    // overshoot a little and are used for things that move or grow; "effects" curves for opacity
    // and colour.
    readonly property QtObject anim: QtObject {
        // Hover, press, small colour changes.
        readonly property int fast: 150
        // Focus moving, toggles.
        readonly property int small: K.Tokens.duration.short
        // A panel leaving, a list moving.
        readonly property int normal: K.Tokens.duration.medium
        // A panel arriving (spatial).
        readonly property int large: 450
        // The wallpaper's crossfade, the lock screen.
        readonly property int slow: 600

        readonly property list<real> standard: K.Tokens.easing.standard
        readonly property list<real> emphasizedAccel: K.Tokens.easing.emphasized_accel
        readonly property list<real> emphasizedDecel: K.Tokens.easing.emphasized_decel
        readonly property list<real> spatial: [0.38, 1.21, 0.22, 1, 1, 1]
        readonly property list<real> spatialFast: [0.42, 1.67, 0.21, 0.9, 1, 1]
        readonly property list<real> effects: [0.34, 0.8, 0.34, 1, 1, 1]
    }

    // Elevation for floating surfaces (RectangularShadow in components/Elevation).
    readonly property QtObject elevation: QtObject {
        readonly property real opacity: 0.65
    }

    // The opacity of the hover, focus and press layer drawn on interactive surfaces.
    readonly property QtObject state: QtObject {
        readonly property real hover: K.Tokens.opacity.hover
        readonly property real press: K.Tokens.opacity.press
    }

    function load(text: string): void {
        try {
            K.Tokens.matugen = JSON.parse(text);
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
        onLoadFailed: K.Tokens.matugen = null
    }
}
