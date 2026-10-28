import QtQuick
import KubaOS.Components as K
import qs.components

// The Wi-Fi and Bluetooth tabs' one well, which both share whole: a live list of Known and Nearby
// (`rows`, for a `LiveList`), or, while `formOpen`, the connection form in its place, the two
// cross-fading; or, while the radio is `off`, an empty state saying how to turn it on.
//
// A row is `{ key, kind, title, subtitle, subtitleIcon, icon, accent, checked, forgettable,
// failed }`. Its kind: "row" (a network or a device, on its icon tile, checked while in use;
// pressed, `picked(key)`; a forgettable one shows a quiet × under the pointer, and Delete on it
// does the same, `forgot(key)`), "label" (a section's small muted title), "skeleton" (a row still
// being found) or "note" (one line, in the error colour while `failed`).
Item {
    id: root

    property var rows: []
    property bool formOpen: false
    property bool off: false
    property string offIcon
    property string offTitle
    property string offBody
    // A button under the empty state's words that turns the radio on, or none.
    property Component offAction: null
    readonly property alias form: form
    readonly property alias list: list

    signal picked(string key)
    signal forgot(string key)

    function rowAt(index: int): var {
        return index >= 0 && index < list.count ? list.model.get(index) : null;
    }

    onFormOpenChanged: {
        if (root.formOpen) {
            form.begin();
        } else if (form.activeFocus) {
            list.forceActiveFocus();
        }
    }

    Well {
        id: well

        anchors.fill: parent
        focused: list.activeFocus

        LiveList {
            id: list

            anchors.fill: parent
            visible: !root.off && opacity > 0
            enabled: !root.formOpen
            opacity: root.formOpen ? 0 : 1
            highlightShown: activeFocus
            rows: root.rows
            onActivated: index => root.picked(root.rowAt(index).key)
            Keys.onDeletePressed: {
                const row = root.rowAt(list.currentIndex);
                if (row?.forgettable)
                    root.forgot(row.key);
            }

            Behavior on opacity {
                NumberAnimation {
                    duration: K.Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard
                }
            }

            delegate: LiveRow {
                id: live

                required property int index
                required property string key
                required property string kind
                required property string title
                required property string subtitle
                required property string subtitleIcon
                required property string icon
                required property bool accent
                required property bool checked
                required property bool forgettable
                required property bool failed

                // A label after rows stands 12 clear of them, with the 4 the row above carries.
                readonly property int labelRoom: live.index > 0 ? K.Tokens.space.s12 : 0

                rowHeight: live.kind === "label" ? live.labelRoom + K.Tokens.type.label.line : live.kind === "note" ? K.Tokens.size.row : K.Tokens.size.row_two_line

                K.ListRow {
                    id: row

                    visible: live.kind === "row"
                    width: live.width
                    height: live.rowHeight
                    title: live.title
                    subtitle: live.subtitle
                    subtitleIcon: live.subtitleIcon
                    icon: live.icon
                    tile: true
                    accent: live.accent
                    checkable: true
                    checked: live.checked
                    selected: live.current && list.highlightShown
                    onActivated: root.picked(live.key)
                }

                // Over the check's room, which a row that can be forgotten keeps empty.
                K.IconButton {
                    id: forget

                    x: row.width - K.Tokens.space.s12 - K.Tokens.type.body_strong.icon / 2 - width / 2
                    y: (row.height - height) / 2
                    visible: live.kind === "row" && live.forgettable && opacity > 0
                    opacity: row.hovered || forget.hovered || row.selected ? 1 : 0
                    icon: "close"
                    label: "Forget"
                    kind: "Text"
                    size: "Sm"
                    onActivated: root.forgot(live.key)

                    Behavior on opacity {
                        NumberAnimation {
                            duration: K.Tokens.duration.micro
                        }
                    }
                }

                K.Text {
                    visible: live.kind === "label"
                    x: K.Tokens.space.s12
                    y: live.labelRoom
                    text: live.title
                    style: K.Tokens.type.label
                    color: K.Tokens.color.on_surface_variant
                }

                SkeletonRow {
                    visible: live.kind === "skeleton"
                    width: live.width
                    height: live.rowHeight
                    share: live.index % 2 ? 0.45 : 0.7
                }

                K.Text {
                    visible: live.kind === "note"
                    x: K.Tokens.space.s12
                    y: (live.rowHeight - height) / 2
                    width: live.width - K.Tokens.space.s12 * 2
                    room: width
                    maximumLines: 1
                    text: live.title
                    style: K.Tokens.type.body
                    color: live.failed ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
                }
            }
        }

        ConnectForm {
            id: form

            anchors.fill: parent
            visible: opacity > 0
            enabled: root.formOpen
            opacity: root.formOpen ? 1 : 0

            Behavior on opacity {
                NumberAnimation {
                    duration: K.Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard
                }
            }
        }

        K.EmptyState {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width
            room: parent.width
            visible: root.off
            icon: root.offIcon
            title: root.offTitle
            body: root.offBody
            action: root.offAction
        }
    }
}
