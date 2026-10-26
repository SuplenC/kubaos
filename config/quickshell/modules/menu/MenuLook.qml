import QtQuick
import Quickshell
import qs.theme
import qs.settings

// The Command menu's sizes, type and motion for the screen it opens on. One
// of these lives in Menu.qml and every piece of the menu reads it, so the
// numbers are set here once.
//
// Sizes follow the screen's logical size. A screen about 1920 logical pixels
// wide (1080p at scale 1, or a denser panel at the scale that gets it there) is
// the reference, so those show the same menu; smaller screens scale it down a
// little, larger ones up. Every length lands on whole device pixels at the
// screen's scale.
QtObject {
    id: look

    required property ShellScreen screen

    readonly property real dpr: screen?.devicePixelRatio ?? 1
    readonly property int screenWidth: screen?.width ?? 1920
    readonly property int screenHeight: screen?.height ?? 1080
    readonly property real unit: Math.max(0.9, Math.min(1.15, screenWidth / 1920, screenHeight / 1080))

    // `v` logical pixels at the reference density, scaled and snapped.
    function px(v: real): real {
        return snap(v * unit);
    }

    // `v` on the device pixel grid, so edges and images stay sharp at 1.5.
    function snap(v: real): real {
        return Math.round(v * dpr) / dpr;
    }

    // Type, scaled like the rest. Text snaps to pixels on its own.
    readonly property QtObject font: QtObject {
        readonly property int query: Math.round(Theme.font.xl * look.unit)
        readonly property int title: Math.round(Theme.font.lg * look.unit)
        readonly property int body: Math.round(Theme.font.md * look.unit)
        readonly property int small: Math.round(Theme.font.sm * look.unit)
        readonly property int heading: Math.round(Theme.font.xl * look.unit)
    }

    readonly property real gap: px(Theme.spacing.sm)
    readonly property real inset: px(Theme.spacing.md)
    readonly property real pad: px(Theme.spacing.xl)

    readonly property real headerHeight: px(56)
    readonly property real footerHeight: px(36)
    readonly property real rowHeight: px(48)
    readonly property real iconSize: px(32)
    // A row's icon and badge from its ends. The selection's corners curve in
    // `rowRadius`; an icon at that distance sits on the curve and reads
    // off-centre, so it stands a third further in.
    readonly property real rowPad: px(Theme.spacing.lg)
    readonly property real smallRowHeight: px(36)
    readonly property real jobHeight: px(28)
    readonly property real fieldHeight: px(40)

    // Smaller steps of Theme's spacing, and two off its scale where the
    // menu's denser rows need them: a keycap from its label, one hint or
    // field from the next.
    readonly property real nudge: px(Theme.spacing.xs)
    readonly property real tight: px(6)
    readonly property real apart: px(14)
    readonly property real wide: px(Theme.spacing.lg)

    // Lines of the detail pane and the Action panel: the "Actions" label, an
    // Extension's heading over its Contributed actions, one action.
    readonly property real labelHeight: px(24)
    readonly property real headingHeight: px(28)
    readonly property real actionHeight: px(30)
    readonly property real actionPanelWidth: px(380)

    // Chips: the app the search is in, a kind or an Alias, a keycap.
    readonly property real chipHeight: px(30)
    readonly property real badgeHeight: px(22)
    readonly property real badgePad: px(10)
    readonly property real keyHeight: px(20)

    // Icons beside the rows' 32: the header's, a job's, the pane's and its
    // box, a progress View's, an intro example's arrow, a button's.
    readonly property real leadIcon: px(22)
    readonly property real jobIcon: px(16)
    readonly property real paneIconBox: px(56)
    readonly property real paneIcon: px(40)
    readonly property real paneGlyph: px(28)
    readonly property real progressIcon: px(36)
    readonly property real exampleIcon: px(14)
    readonly property real controlSize: px(28)

    // A job's bar under the header, and a progress View's, thicker.
    readonly property real barHeight: px(Theme.spacing.xs)
    readonly property real progressBarHeight: px(6)

    // A form's or a progress View's column, and a detail View's labels.
    readonly property real columnWidth: px(560)
    readonly property real labelWidth: px(120)

    // A checkbox, drawn as Kōbō's switch: its track, and its knob off and
    // on.
    readonly property real switchWidth: px(40)
    readonly property real switchHeight: px(24)
    readonly property real knobOff: px(Theme.spacing.md)
    readonly property real knobOn: px(Theme.spacing.lg)

    // Panel corners like the frame's panels; rows step down one size per
    // level of padding, so the corners stay concentric (24 -> 12).
    readonly property real radius: Theme.frame.panelRounding
    readonly property real rowRadius: Theme.rounding.md
    // The cards inside the panel (the Action panel, the intro card), the
    // Action panel's rows, and keycaps.
    readonly property real cardRadius: px(Theme.rounding.lg)
    readonly property real itemRadius: px(10)
    readonly property real keyRadius: px(6)

    readonly property real width: snap(Math.max(px(560), Math.min(screenWidth * 0.5, px(1040))))
    // The detail pane shows when the list keeps room for a title and a
    // subtitle beside it.
    readonly property real paneMinWidth: px(300)
    readonly property real listMinWidth: px(480)
    readonly property bool paneFits: width >= listMinWidth + paneMinWidth
    readonly property real paneWidth: paneFits ? snap(Math.max(paneMinWidth, width * 0.38)) : 0

    // Rows fill about two thirds of the screen's height: a taller screen
    // shows more of them.
    readonly property int rows: Math.max(6, Math.min(14, Math.floor((screenHeight * 0.64 - headerHeight - footerHeight - 2 * inset) / rowHeight)))
    readonly property real bodyHeight: rows * rowHeight + 2 * inset
    // Header, divider, body, divider, footer.
    readonly property real height: headerHeight + 1 + bodyHeight + 1 + footerHeight
    readonly property real top: snap((screenHeight - height) * 0.4)

    // The Material Symbol standing for an Extension's Items when they have
    // no icon of their own.
    function glyph(extension: string): string {
        return glyphs[extension] ?? "bolt";
    }

    readonly property var glyphs: ({
            apps: "apps",
            calculator: "calculate",
            clipboard: "content_paste",
            emoji: "mood",
            folders: "folder",
            hyprland: "keyboard_command_key",
            system: "settings_power",
            "web-search": "travel_explore"
        })

    // An Extension's name as a label: `web-search` reads "Web search".
    function kindLabel(extension: string): string {
        const words = extension.replace(/[-_.]/g, " ");
        return words.charAt(0).toUpperCase() + words.slice(1);
    }

    // An Action's shortcut as Kotodama's key hints write keys: `ctrl+l`
    // reads `⌃L`, as `⌃K Actions` does, and `ctrl+shift+backspace` `⌃⇧⌫`.
    function shortcutLabel(shortcut: string): string {
        if (shortcut === "")
            return "";
        return shortcut.split("+").map(part => {
            const name = part.toLowerCase();
            const known = modifiers[name] ?? keys[name];
            if (known !== undefined)
                return known;
            return part.length === 1 ? part.toUpperCase() : part.charAt(0).toUpperCase() + part.slice(1);
        }).join("");
    }

    readonly property var keys: ({
            backspace: "⌫",
            delete: "⌦",
            return: "↵",
            enter: "↵",
            escape: "Esc",
            esc: "Esc",
            tab: "Tab",
            space: "Space",
            up: "↑",
            down: "↓",
            left: "←",
            right: "→"
        })

    readonly property var modifiers: ({
            ctrl: "⌃",
            control: "⌃",
            shift: "⇧",
            alt: "⌥",
            super: "Super+",
            meta: "Super+"
        })

    readonly property QtObject motion: QtObject {
        // Whether things may move, not only fade.
        readonly property bool moves: Settings.menuMotion !== "fades"

        // Kōbō's glide (crates/kubaos-kobo/src/reveal.rs): 220 ms on the
        // emphasized-decelerate curve. The selection highlight, rows moving,
        // a View pushed or popped.
        readonly property int glide: 220
        // The window arrives on the glide and leaves quicker, so a run never
        // waits long on the window going.
        readonly property int open: 220
        readonly property int close: 120
        // Rows easing in and out as Results change by themselves, and the
        // detail pane's crossfade. What a keystroke changes shows at once.
        readonly property int fade: 120
        // One pass of the segment across a job's bar when the Extension
        // can't tell how far it has got.
        readonly property int sweep: 1200

        // How far the window drops in as it opens, and a View slides as it
        // is pushed or popped.
        readonly property real openTravel: moves ? look.px(10) : 0
        readonly property real viewTravel: moves ? look.px(24) : 0

        readonly property list<real> arrive: Theme.anim.emphasizedDecel
        readonly property list<real> leave: Theme.anim.emphasizedAccel
        readonly property list<real> effects: Theme.anim.effects
    }
}
