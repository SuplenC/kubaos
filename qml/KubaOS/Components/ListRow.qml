import QtQuick
import KubaOS.Components as K

// One row of a list: `kubaos-ui`'s `list::list_row`. 44 tall with one line, 56 with a subtitle;
// 12 inside, a 20 icon (24 beside two lines) or the icon on an icon tile (`tile`), the title in
// `body_strong`, the subtitle in `small`, chips and a detail in `mono` at the end, and an optional
// check whose room is kept when unchecked, so the rows' ends line up. Each part is centred on the
// row's whole height, so a chip moves nothing else.
// Its corners are concentric inside the list's well. The selected row takes the secondary
// container, or the error colour on a row whose action can't be taken back (`danger`).
// A row still being found (`loading`) is a tiled two-line row's box with its tile and its two
// lines as skeletons, so the row that arrives takes the place it kept and nothing moves.
//
// HAZARD: no layout primitives in here; every part is placed with token arithmetic, as iced's row
// and stack would place it. The Command menu makes up to fourteen of these on every keystroke:
// fourteen take 6.1 ms to make this way against 10.4 ms on `K.Row` and `K.Stack` (offscreen,
// median of 40 rounds).
K.Pressable {
    id: root

    property string title
    // The title with the characters a query matched lit, as Kotodama writes it in Qt's
    // `StyledText`; empty for none.
    property string markup: ""
    property string subtitle
    // A symbol before the subtitle, the subtitle's 16, 4 from its words: a fact about it at a
    // glance (a lock on a secured network, a device's battery).
    property string subtitleIcon: ""
    // A short fact at the row's end, in mono: a device path, a size, a code.
    property string detail
    // Chips near the row's end, as `{ kind, text }`: facts that matter at a glance (Default).
    property var chips: []
    property string icon: ""
    // The icon on an icon tile, `Sm` beside one line and `Md` beside two, as the Command menu's
    // rows show theirs: a list of things of one kind, where the symbol tells one from the others.
    property bool tile: false
    // An app's own icon on the tile, with `icon` as its placeholder.
    property url source: ""
    // The tile's symbol in the accent: the one row of the list that comes first (the default).
    property bool accent: false
    // Always 56 tall, its icon as beside two lines, a title alone included, so the words of every
    // row start in one column: the Command menu's Results.
    property bool tall: false
    // The words 16 after the icon, not 12, so they stand clear of an app's own icon on its tile.
    property bool clear: false
    // The subtitle right under the title, not 4 below.
    property bool tight: false
    // The row the keyboard is on, or the one picked in a list of one.
    property bool selected: false
    // Whether the row fills itself while selected. Off where the list draws the selection behind
    // its rows and slides it between them, as the Command menu does; the row still takes the
    // selection's colours.
    property bool fills: true
    // Whether the pointer over the row lights it. Off where the pointer moves the list's selection
    // instead.
    property bool hoverLayer: true
    // Whether its action can be taken now. A row that can't is dimmed, its icon and title, and
    // still takes a press, so it can say why.
    property bool available: true
    // Its action can't be taken back (Shut down).
    property bool danger: false
    // Whether the row shows a check at its end, and whether it is ticked.
    property bool checkable: false
    property bool checked: false
    // Still being found: skeletons in a tiled two-line row's place, taking no press.
    property bool loading: false
    // How much of the row a loading row's lines take, so rows one under another don't read as one
    // block.
    property real share: 0.6

    readonly property bool fillWidth: true
    readonly property color onFill: root.danger ? Tokens.color.on_error : Tokens.color.on_secondary_container
    readonly property color main: root.selected ? root.onFill : Tokens.color.on_surface
    readonly property color muted: root.selected ? K.Look.alpha(root.onFill, Tokens.opacity.subdued_text) : Tokens.color.on_surface_variant
    readonly property real dim: root.available ? 1 : Tokens.opacity.disabled_content
    // Laid out as a title over a subtitle: 56 tall, the icon spanning both lines.
    readonly property bool twoLine: root.subtitle !== "" || root.tall
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "ListRow",
            variant: [root.subtitle === "" && !root.loading ? "one-line" : "two-line", root.icon === "" ? "" : root.tile ? "tile" : "icon"].filter(word => word !== "").join(" "),
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: root.twoLine || root.loading ? Tokens.size.row_two_line : Tokens.size.row
    enabled: !root.loading
    // The list's well shows where the keyboard is; a row draws no ring of its own.
    activeFocusOnTab: false

    // The room the parts after the icon share, less the gaps between them. Read from the row's own
    // width in each binding that needs it, never through another box's: a box's width updates
    // after the row's, and a text read in the room it had would measure its words one by one.
    function partsRoom(): real {
        return root.width - Tokens.space.s12 * 2 - line.leadWidth - line.leadGap - Tokens.space.s12 * (line.parts - (line.led ? 2 : 1));
    }

    // `v` centred down the row, on a whole device pixel.
    function middle(v: real): real {
        return K.Look.snap((root.height - v) / 2, root.dpr);
    }

    Rectangle {
        anchors.fill: parent
        visible: root.selected && root.fills
        radius: K.Look.nested(Tokens.radius.surface, Tokens.space.s12)
        color: root.danger ? Tokens.color.error : Tokens.color.secondary_container
    }

    // Made only where the pointer lights the row: the Command menu's rows have none.
    Loader {
        anchors.fill: parent
        active: root.hoverLayer

        sourceComponent: K.StateLayer {
            radius: K.Look.nested(Tokens.radius.surface, Tokens.space.s12)
            content: Tokens.color.on_surface
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    // The parts side by side, 12 apart and centred down the row, as iced's row lays them out: the
    // parts that don't fill are measured one after another, each in the room the ones before it
    // left, and the words take what remains.
    Item {
        id: line

        readonly property var probe: ({
                role: "layout",
                axis: "x",
                gap: Tokens.space.s12,
                align: "centre"
            })
        readonly property bool led: root.icon !== ""
        readonly property real leadWidth: !line.led ? 0 : root.tile ? lead.side : glyph.size
        readonly property real leadGap: !line.led ? 0 : root.clear ? Tokens.space.s16 : Tokens.space.s12
        readonly property int parts: (line.led ? 1 : 0) + 1 + chips.count + (root.detail !== "" ? 1 : 0) + (root.checkable ? 1 : 0)
        // The room left for the parts after the words, before each of them is measured.
        readonly property real room: root.partsRoom()
        readonly property real chipsWidth: {
            let sum = 0;
            for (let i = 0; i < chips.count; i++)
                sum += chips.itemAt(i)?.width ?? 0;
            return sum;
        }
        readonly property real endWidth: line.chipsWidth + end.width + check.width
        // Where the parts after the words start.
        readonly property real endStart: words.x + words.width + Tokens.space.s12

        visible: !root.loading
        x: Tokens.space.s12
        width: root.width - Tokens.space.s12 * 2
        height: root.height

        // Made only on a row that shows one: most rows have none, and a tile carries an image of
        // its own.
        Loader {
            active: line.led && root.tile
            visible: active
            y: root.middle(lead.side)

            sourceComponent: K.IconTile {
                icon: root.icon
                source: root.source
                size: lead.size
                accent: root.accent
                opacity: root.dim
            }
        }

        QtObject {
            id: lead

            readonly property string size: root.twoLine ? "Md" : "Sm"
            readonly property int side: root.twoLine ? Tokens.size.control_md : Tokens.size.control_sm
        }

        K.Icon {
            id: glyph

            visible: line.led && !root.tile
            y: root.middle(glyph.size)
            // Named only when shown: a named icon draws its symbol even while hidden.
            name: glyph.visible ? root.icon : ""
            // Beside a title over a subtitle, the icon spans both lines.
            size: root.twoLine ? Tokens.icon.two_line : Tokens.type.body_strong.icon
            color: K.Look.alpha(root.main, root.dim)
        }

        // The title over the subtitle, 4 apart (`tight`: none), as iced's stack lays them out.
        Item {
            id: words

            readonly property var probe: ({
                    role: "layout",
                    axis: "y",
                    gap: words.gap,
                    align: "start"
                })
            readonly property int gap: root.tight ? 0 : Tokens.space.s4
            readonly property bool subtitled: root.subtitle !== ""

            x: line.leadWidth + line.leadGap
            y: root.middle(words.height)
            width: K.Look.snapUp(Math.max(line.room - line.endWidth, 0), root.dpr)
            height: Tokens.type.body_strong.line + (words.subtitled ? words.gap + Tokens.type.small.line : 0)

            K.Text {
                text: root.title
                markup: root.markup
                // Cut by its box, not laid out in the room: the words are measured once, whatever
                // widths the row passes through as it is made.
                width: Math.min(implicitWidth, words.width)
                style: Tokens.type.body_strong
                maximumLines: 1
                color: K.Look.alpha(root.main, root.dim)
            }

            K.Text {
                visible: words.subtitled && root.subtitleIcon === ""
                y: Tokens.type.body_strong.line + words.gap
                text: root.subtitle
                width: Math.min(implicitWidth, words.width)
                style: Tokens.type.small
                maximumLines: 1
                color: root.muted
            }

            // The subtitle after its symbol, as iced's row of the two.
            Loader {
                active: words.subtitled && root.subtitleIcon !== ""
                visible: active
                y: Tokens.type.body_strong.line + words.gap

                sourceComponent: Item {
                    readonly property var probe: ({
                            role: "layout",
                            axis: "x",
                            gap: Tokens.space.s4,
                            align: "centre"
                        })

                    width: words.width
                    height: Tokens.type.small.line

                    K.Icon {
                        y: K.Look.snap((Tokens.type.small.line - Tokens.type.small.icon) / 2, root.dpr)
                        name: root.subtitleIcon
                        size: Tokens.type.small.icon
                        color: root.muted
                    }

                    K.Text {
                        x: Tokens.type.small.icon + Tokens.space.s4
                        text: root.subtitle
                        width: Math.min(implicitWidth, Math.max(words.width - Tokens.type.small.icon - Tokens.space.s4, 0))
                        style: Tokens.type.small
                        maximumLines: 1
                        color: root.muted
                    }
                }
            }
        }

        Repeater {
            id: chips

            model: root.chips

            K.Chip {
                id: chip

                required property var modelData
                required property int index
                readonly property Item before: chip.index > 0 ? chips.itemAt(chip.index - 1) : null

                x: chip.before ? chip.before.x + chip.before.width + Tokens.space.s12 : line.endStart
                y: root.middle(Tokens.size.chip)
                kind: modelData.kind
                text: modelData.text
            }
        }

        // The detail and the check are made only on a row that shows them: most rows have neither.
        Loader {
            id: end

            active: root.detail !== ""
            visible: active
            x: line.endStart + line.chipsWidth + Tokens.space.s12 * chips.count
            y: root.middle(end.height)

            sourceComponent: K.Text {
                text: root.detail
                // The room the chips before it left, as iced's row measures it; none until the row
                // has its width.
                room: root.width > 0 ? Math.max(root.partsRoom() - line.chipsWidth, 0) : Infinity
                style: Tokens.type.mono
                color: root.muted
            }
        }

        Loader {
            id: check

            active: root.checkable
            visible: active
            x: end.active ? end.x + end.width + Tokens.space.s12 : end.x
            y: root.middle(Tokens.type.body_strong.icon)

            sourceComponent: Item {
                readonly property var probe: ({
                        role: "box"
                    })

                implicitWidth: Tokens.type.body_strong.icon
                implicitHeight: Tokens.type.body_strong.icon

                K.Icon {
                    visible: root.checked
                    name: "check"
                    size: Tokens.type.body_strong.icon
                    color: root.main
                }
            }
        }
    }

    Loader {
        anchors.fill: parent
        active: root.loading
        sourceComponent: skeletons
    }

    Component {
        id: skeletons

        Item {
            id: bones

            readonly property real words: bones.width - Tokens.space.s12 * 3 - Tokens.size.control_md

            K.Skeleton {
                x: Tokens.space.s12
                y: root.middle(height)
                width: Tokens.size.control_md
                bone: "block"
                size: Tokens.size.control_md
                radius: Tokens.radius.control
            }

            Column {
                x: Tokens.space.s12 * 2 + Tokens.size.control_md
                y: root.middle(height)
                spacing: Tokens.space.s4

                K.Skeleton {
                    width: K.Look.snap(bones.words * root.share, root.dpr)
                    style: Tokens.type.body_strong
                }

                K.Skeleton {
                    width: K.Look.snap(bones.words * root.share * root.share, root.dpr)
                    style: Tokens.type.small
                }
            }
        }
    }
}
