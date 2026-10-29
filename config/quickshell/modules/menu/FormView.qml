import QtQuick
import Kotodama
import KubaOS.Components as K

// A form View, in a column: each field under its label, Kōbō's way, and the submit button at the
// end. Tab and Shift+Tab (or Down and Up) move between fields, Space ticks a checkbox, Left and
// Right pick an option, Enter submits from anywhere; the menu's own keys come first, through
// `keys`.
//
// A checkbox or a select keeps the keyboard on an item of its own, which draws its focus on the
// control: the library's controls press on Enter, and here Enter submits.
//
// It is never taller than the room the menu gives it: a form with more fields than that scrolls,
// reaching into the room around it (`roomAbove`, `roomBelow`), and the keys moving to a field
// scroll it into view. A click never scrolls, so the field clicked stays under the pointer.
FocusScope {
    id: form

    required property KotodamaSearch search
    // The menu's keys, `Menu.qml`'s `key(event)`: true if it took the event.
    property var keys: event => false
    // Each `{label, kind, placeholder, secret, options, value}`.
    property var fields: []
    property string submitLabel: ""
    // The field that has the keys.
    property int current: 0
    // The room between it and what the menu draws above and below it, which its scroll reaches
    // into.
    property alias roomAbove: scroll.roomAbove
    property alias roomBelow: scroll.roomBelow
    // How far it is scrolled, and how far it can be.
    property alias contentY: scroll.contentY
    readonly property alias travel: scroll.travel

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    signal submitted

    function move(by: int): void {
        if (repeater.count > 0)
            repeater.itemAt((current + by + repeater.count) % repeater.count).take();
        form.reveal();
    }

    function takeFirst(): bool {
        if (repeater.count === 0)
            return false;
        repeater.itemAt(0).take();
        form.reveal();
        return true;
    }

    // Scrolls the field with the keys into view, its label with it.
    function reveal(): void {
        const field = repeater.itemAt(form.current);
        if (!field)
            return;
        const top = field.mapToItem(column, 0, 0).y;
        const bottom = top + field.height;
        if (top < scroll.contentY)
            scroll.contentY = top;
        else if (bottom > scroll.contentY + scroll.height)
            scroll.contentY = Math.min(scroll.travel, bottom - scroll.height);
    }

    K.Scroll {
        id: scroll

        anchors.fill: parent

        K.Stack {
            id: column

            x: K.Look.snap((form.width - width) / 2, form.dpr)
            width: Math.min(form.width, K.Tokens.size.column)
            fill: true
            gap: K.Tokens.space.s24

            K.Stack {
                fill: true
                gap: K.Tokens.space.s16

                Repeater {
                    id: repeater

                    model: form.fields

                    K.Stack {
                        id: field

                        required property int index
                        required property var modelData
                        readonly property string kind: modelData.kind
                        readonly property bool current: typed.activeFocus || toggle.activeFocus || choice.activeFocus

                        function take(): void {
                            (field.kind === "checkbox" ? toggle : field.kind === "select" ? choice : typed).forceActiveFocus();
                        }

                        fill: true
                        onCurrentChanged: {
                            if (current)
                                form.current = index;
                        }

                        K.Labelled {
                            visible: field.kind !== "checkbox"
                            label: field.modelData.label

                            K.Field {
                                id: typed

                                visible: field.kind === "text"
                                placeholder: field.modelData.placeholder ?? ""
                                secret: field.modelData.secret === true
                                Component.onCompleted: {
                                    if (field.kind === "text")
                                        text = field.modelData.value;
                                }
                                onTextChanged: form.search.setText(field.index, text)
                                onKeyPressed: event => event.accepted = form.keys(event)
                            }

                            // A select: one radio per option, Left and Right step through them.
                            Item {
                                id: choice

                                readonly property var options: field.modelData.options ?? []
                                property int chosen: Math.max(0, options.findIndex(option => option.value === field.modelData.value))
                                readonly property bool fillWidth: true

                                function step(by: int): void {
                                    if (options.length > 0)
                                        choice.pick((chosen + by + options.length) % options.length);
                                }

                                function pick(n: int): void {
                                    chosen = n;
                                    form.search.setText(field.index, options[n].value);
                                }

                                function relayout(): void {
                                    radios.relayout();
                                }

                                visible: field.kind === "select"
                                implicitHeight: radios.implicitHeight
                                Keys.onPressed: event => {
                                    const by = event.key === Qt.Key_Right ? 1 : event.key === Qt.Key_Left ? -1 : 0;
                                    if (form.keys(event)) {
                                        event.accepted = true;
                                    } else if (by !== 0) {
                                        choice.step(by);
                                        event.accepted = true;
                                    }
                                }

                                K.Grid {
                                    id: radios

                                    width: choice.width
                                    columns: 3
                                    gap: K.Tokens.space.s12

                                    Repeater {
                                        model: choice.options

                                        K.Radio {
                                            required property int index
                                            required property var modelData

                                            text: modelData.label
                                            checked: index === choice.chosen
                                            focused: choice.activeFocus && checked
                                            activeFocusOnTab: false
                                            onPicked: {
                                                choice.forceActiveFocus();
                                                choice.pick(index);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // A checkbox: Space ticks it.
                        Item {
                            id: toggle

                            property bool checked: field.modelData.value === true

                            function flip(): void {
                                checked = !checked;
                                form.search.setChecked(field.index, checked);
                            }

                            visible: field.kind === "checkbox"
                            implicitWidth: box.implicitWidth
                            implicitHeight: box.implicitHeight
                            Keys.onPressed: event => {
                                if (form.keys(event)) {
                                    event.accepted = true;
                                } else if (event.key === Qt.Key_Space) {
                                    toggle.flip();
                                    event.accepted = true;
                                }
                            }

                            K.Checkbox {
                                id: box

                                text: field.modelData.label
                                tick: toggle.checked ? "Full" : "Empty"
                                focused: toggle.activeFocus
                                activeFocusOnTab: false
                                onToggled: {
                                    toggle.forceActiveFocus();
                                    toggle.flip();
                                }
                            }
                        }
                    }
                }
            }

            K.Row {
                fill: true

                K.Fill {}

                K.Button {
                    text: form.submitLabel
                    kind: "Filled"
                    activeFocusOnTab: false
                    onActivated: form.submitted()
                }
            }
        }
    }
}
