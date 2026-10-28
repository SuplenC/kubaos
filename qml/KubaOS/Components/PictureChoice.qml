import QtQuick
import KubaOS.Components as K

// A picture to pick: `kubaos-ui`'s `picture::picture_choice`. The picture covers the whole tile,
// edge to edge in its rounded shape (a `Media` box at the control radius, held before the picture
// loads), with no frame round it. Picked, the ring goes round it and `mark` shows over its top
// right corner; the keyboard on it shows the ring too, cold when it is picked, since the accent
// already means picked. The name sits under the picture, or over its foot on a dark scrim.
//
// The picture is `source`, or content put in the tile, laid over the placeholder and under the
// name and the chips (a video).
Item {
    id: root

    signal activated

    property url source: ""
    property list<int> aspect: Tokens.aspect.wallpaper
    property string name
    // `under` the picture, or `over` its foot.
    property string namePlace: "under"
    property bool picked: false
    // What says it is picked, over the top right corner while it is: a symbol and a word.
    property string markIcon: ""
    property string markText: ""
    // A fact about the picture whatever its state, over the top left corner.
    property string badgeIcon: ""
    property string badgeText: ""
    // Whether the keyboard is on it; Qt's focus by default, or the app's where it owns focus.
    property bool focused: root.activeFocus
    default property alias content: media.content

    readonly property bool over: root.namePlace === "over"
    // Whether the pointer is over the tile.
    readonly property bool hovered: tile.hovered

    readonly property bool fillWidth: true
    readonly property var probe: ({
            role: "component",
            name: "PictureChoice",
            variant: root.over ? "name-over" : "name-under",
            fixed: {
                width: false,
                height: false,
                holds: true
            }
        })

    implicitHeight: column.implicitHeight

    function relayout(): void {
        column.relayout();
    }

    K.Stack {
        id: column

        width: root.width
        room: root.width
        gap: Tokens.space.s8
        centred: true

        K.Pressable {
            id: tile

            readonly property bool fillWidth: true

            implicitHeight: media.implicitHeight
            onActivated: root.activated()

            K.Media {
                id: media

                width: tile.width
                source: root.source
                aspect: root.aspect
                radius: Tokens.radius.control

                // Over a photo the name is always light on a dark foot, whatever the mode: the
                // scrim's colour at the backdrop's opacity, from nothing 40% down the tile.
                Rectangle {
                    anchors.fill: parent
                    z: 1
                    visible: root.over
                    radius: Tokens.radius.control
                    gradient: Gradient {
                        GradientStop {
                            position: 0.4
                            color: K.Look.alpha(Tokens.color.scrim, 0)
                        }
                        GradientStop {
                            position: 1
                            color: K.Look.alpha(Tokens.color.scrim, Tokens.opacity.backdrop)
                        }
                    }
                }

                K.Text {
                    x: Tokens.space.s12
                    y: tile.height - Tokens.space.s12 - height
                    z: 1
                    visible: root.over
                    room: tile.width - Tokens.space.s12 * 2
                    maximumLines: 1
                    text: root.name
                    style: Tokens.type.small
                    color: Tokens.palette.dark.on_surface
                }

                K.Chip {
                    x: Tokens.space.s8
                    y: Tokens.space.s8
                    z: 1
                    visible: root.badgeText !== ""
                    icon: root.badgeIcon
                    text: root.badgeText
                }

                K.Chip {
                    x: tile.width - Tokens.space.s8 - width
                    y: Tokens.space.s8
                    z: 1
                    visible: root.picked && root.markText !== ""
                    kind: "Selected"
                    icon: root.markIcon
                    text: root.markText
                }
            }

            K.StateLayer {
                radius: Tokens.radius.control
                hovered: tile.hovered
                pressed: tile.down
            }

            K.FocusRing {
                shown: root.picked || root.focused
                cold: root.picked && root.focused
                controlRadius: Tokens.radius.control
            }
        }

        K.Text {
            visible: !root.over
            room: root.width
            maximumLines: 1
            text: root.name
            style: Tokens.type.small
            color: root.picked ? Tokens.color.on_surface : Tokens.color.on_surface_variant
        }
    }
}
