import QtQuick
import Kotodama
import qs.theme
import qs.components

// A form View: each field's label above its control, Kōbō's way (`field` in
// crates/kubaos-kobo/src/widgets.rs), and the submit button. Tab and
// Shift+Tab (or Down and Up) move between fields, Space ticks a checkbox,
// Left and Right choose an option, Enter submits; the menu's own keys come
// first, through `keys`.
FocusScope {
    id: form

    property MenuLook look
    required property KotodamaSearch search
    // The menu's keys, `Menu.qml`'s `key(event)`: true if it took the event.
    property var keys: event => false
    // Each `{label, kind, placeholder, secret, options, value}`.
    property var fields: []
    property string submitLabel: ""
    // The field that has the keys.
    property int current: 0

    signal submitted

    function move(by: int): void {
        if (repeater.count > 0)
            repeater.itemAt((current + by + repeater.count) % repeater.count).take();
    }

    function takeFirst(): bool {
        if (repeater.count === 0)
            return false;
        repeater.itemAt(0).take();
        return true;
    }

    Column {
        id: column

        width: Math.min(parent.width - 2 * form.look.pad, form.look.columnWidth)
        anchors.horizontalCenter: parent.horizontalCenter
        y: form.look.pad
        spacing: form.look.apart

        Repeater {
            id: repeater

            model: form.fields

            Column {
                id: field

                required property int index
                required property var modelData
                readonly property bool current: typed.activeFocus || toggle.activeFocus || choice.activeFocus

                function take(): void {
                    const kind = modelData.kind;
                    (kind === "checkbox" ? toggle : kind === "select" ? choice : typed).forceActiveFocus();
                }

                width: column.width
                spacing: form.look.tight
                onCurrentChanged: {
                    if (current)
                        form.current = index;
                }

                StyledText {
                    width: parent.width
                    text: field.modelData.label
                    color: field.current ? Theme.m3primary : Theme.m3onSurfaceVariant
                    font.pixelSize: form.look.font.small
                    font.weight: Font.Medium
                }

                Rectangle {
                    id: box

                    width: parent.width
                    height: form.look.fieldHeight
                    radius: form.look.rowRadius
                    color: Theme.m3surfaceContainerHigh
                    border.width: 1
                    border.color: Theme.alpha(Theme.m3outlineVariant, 0.8)

                    // Kōbō's focus ring: 2 px of primary, 3 px outside.
                    FocusRing {
                        shown: field.current
                    }

                    TextInput {
                        id: typed

                        anchors.fill: parent
                        anchors.leftMargin: form.look.inset
                        anchors.rightMargin: form.look.inset
                        visible: field.modelData.kind === "text"
                        verticalAlignment: TextInput.AlignVCenter
                        clip: true
                        color: Theme.m3onSurface
                        selectionColor: Theme.m3primary
                        selectedTextColor: Theme.m3onPrimary
                        renderType: Text.NativeRendering
                        font.family: Theme.font.sans
                        font.pixelSize: form.look.font.title
                        passwordCharacter: "•"
                        echoMode: field.modelData.secret ? TextInput.Password : TextInput.Normal
                        Component.onCompleted: {
                            if (field.modelData.kind === "text")
                                text = field.modelData.value;
                        }
                        onTextChanged: form.search.setText(field.index, text)
                        Keys.onPressed: event => event.accepted = form.keys(event)

                        StyledText {
                            anchors.fill: parent
                            visible: typed.text === "" && !typed.preeditText
                            text: field.modelData.placeholder
                            color: Theme.alpha(Theme.m3onSurfaceVariant, 0.7)
                            font.pixelSize: form.look.font.title
                        }
                    }

                    // A checkbox, drawn as Kōbō's switch.
                    Item {
                        id: toggle

                        property bool checked: field.modelData.value === true

                        anchors.fill: parent
                        anchors.leftMargin: form.look.inset
                        anchors.rightMargin: form.look.inset
                        visible: field.modelData.kind === "checkbox"
                        Keys.onPressed: event => {
                            if (form.keys(event)) {
                                event.accepted = true;
                            } else if (event.key === Qt.Key_Space) {
                                toggle.flip();
                                event.accepted = true;
                            }
                        }

                        function flip(): void {
                            checked = !checked;
                            form.search.setChecked(field.index, checked);
                        }

                        StyledText {
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            text: toggle.checked ? "On" : "Off"
                            font.pixelSize: form.look.font.title
                        }

                        Rectangle {
                            id: track

                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            width: form.look.switchWidth
                            height: form.look.switchHeight
                            radius: height / 2
                            color: toggle.checked ? Theme.m3primary : Theme.m3surfaceContainerHighest
                            border.width: toggle.checked ? 0 : 2
                            border.color: Theme.m3outline

                            Rectangle {
                                width: toggle.checked ? form.look.knobOn : form.look.knobOff
                                height: width
                                radius: width / 2
                                anchors.verticalCenter: parent.verticalCenter
                                x: toggle.checked ? track.width - width - form.look.nudge : form.look.tight
                                color: toggle.checked ? Theme.m3onPrimary : Theme.m3outline

                                Behavior on x {
                                    enabled: form.look.motion.moves
                                    NumberAnimation {
                                        duration: form.look.motion.glide
                                        easing.type: Easing.BezierSpline
                                        easing.bezierCurve: form.look.motion.arrive
                                    }
                                }
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                toggle.forceActiveFocus();
                                toggle.flip();
                            }
                        }
                    }

                    // A select: Left and Right step through the options.
                    Item {
                        id: choice

                        readonly property var options: field.modelData.options ?? []
                        property int chosen: Math.max(0, options.findIndex(option => option.value === field.modelData.value))

                        function step(by: int): void {
                            if (options.length === 0)
                                return;
                            chosen = (chosen + by + options.length) % options.length;
                            form.search.setText(field.index, options[chosen].value);
                        }

                        anchors.fill: parent
                        anchors.leftMargin: form.look.inset
                        anchors.rightMargin: form.look.tight
                        visible: field.modelData.kind === "select"
                        Keys.onPressed: event => {
                            const by = event.key === Qt.Key_Right ? 1 : event.key === Qt.Key_Left ? -1 : 0;
                            if (form.keys(event)) {
                                event.accepted = true;
                            } else if (by !== 0) {
                                choice.step(by);
                                event.accepted = true;
                            }
                        }

                        StyledText {
                            anchors.left: parent.left
                            anchors.right: arrows.left
                            anchors.verticalCenter: parent.verticalCenter
                            text: choice.options.length > 0 ? choice.options[choice.chosen].label : ""
                            font.pixelSize: form.look.font.title
                        }

                        Row {
                            id: arrows

                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter

                            IconButton {
                                size: form.look.controlSize
                                icon: "chevron_left"
                                onClicked: {
                                    choice.forceActiveFocus();
                                    choice.step(-1);
                                }
                            }

                            IconButton {
                                size: form.look.controlSize
                                icon: "chevron_right"
                                onClicked: {
                                    choice.forceActiveFocus();
                                    choice.step(1);
                                }
                            }
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        visible: !field.current && field.modelData.kind !== "checkbox"
                        onClicked: field.take()
                    }
                }
            }
        }

        Item {
            width: parent.width
            height: submit.height + form.look.tight

            Rectangle {
                id: submit

                anchors.right: parent.right
                anchors.bottom: parent.bottom
                width: form.look.snap(submitRow.implicitWidth + 2 * form.look.pad)
                height: form.look.fieldHeight
                radius: height / 2
                color: Theme.m3primary

                Row {
                    id: submitRow

                    anchors.centerIn: parent
                    spacing: form.look.gap

                    StyledText {
                        anchors.verticalCenter: parent.verticalCenter
                        text: form.submitLabel
                        color: Theme.m3onPrimary
                        font.pixelSize: form.look.font.title
                        font.weight: Font.Medium
                    }

                    Keycap {
                        anchors.verticalCenter: parent.verticalCenter
                        look: form.look
                        tint: Theme.m3onPrimary
                        key: "↵"
                    }
                }

                StateLayer {
                    color: Theme.m3onPrimary
                    onClicked: form.submitted()
                }
            }
        }
    }
}
