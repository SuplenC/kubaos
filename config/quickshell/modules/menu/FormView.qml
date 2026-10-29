import QtQuick
import Kotodama
import KubaOS.Components as K

// A form View, in a column: each field under its label, Kōbō's way, and the submit button at the
// end. Tab and Shift+Tab (or Down and Up) move between fields, Space ticks a checkbox, Left and
// Right pick an option, Enter submits from anywhere; the menu's own keys come first, through
// `keys`. A file or folder field (`FileField`) takes the arrows, Tab and Enter for its list of
// paths while it shows one, and has Browse… at its label's right, as a password field has its
// Show, which asks the menu for the system's picker through `browse`.
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
    // Each `{label, kind, placeholder, secret, options, accept, value}`.
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

    // Whether the field with the keys is a file or folder field, and whether it shows its paths.
    readonly property bool pathCurrent: repeater.count > form.current && (repeater.itemAt(form.current)?.path ?? false)
    readonly property bool pathsOpen: repeater.count > form.current && (repeater.itemAt(form.current)?.pathsOpen ?? false)

    signal submitted
    // The system's picker, for the `field`th field: a folder, or a file of a kind in `accept`.
    signal browse(int field, bool folder, var accept)

    // Each field's item and what it holds, for the menu's probe: `{item, value}`.
    function shown(): var {
        const fields = [];
        for (let n = 0; n < repeater.count; n++) {
            const field = repeater.itemAt(n);
            if (field)
                fields.push({
                    item: field,
                    value: field.value()
                });
        }
        return fields;
    }

    // Gives the `field`th field the keys again once the picker is gone, `path` in it as the picker
    // chose it, or as it was when the pick was cancelled (`path` empty).
    function fill(field: int, path: string): void {
        const item = repeater.itemAt(field);
        if (!item?.path)
            return;
        if (path === "")
            item.take();
        else
            item.fillPath(path);
        form.reveal();
    }

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

    // Scrolls the field with the keys into view, its label with it; the last field's with the
    // submit button under it too.
    function reveal(): void {
        const field = repeater.itemAt(form.current);
        if (!field)
            return;
        const top = field.mapToItem(column, 0, 0).y;
        // The last field's paths can be taller than the room: the scroll stops at its label then.
        if (form.current === repeater.count - 1) {
            scroll.contentY = Math.min(scroll.travel, top);
            return;
        }
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
                        // In sentence case: the Daemon labels an Argument's field with its name, as `path`.
                        readonly property string label: (modelData.label ?? "").charAt(0).toUpperCase() + (modelData.label ?? "").slice(1)
                        readonly property bool path: kind === "file" || kind === "folder"
                        readonly property bool pathsOpen: field.path && pathField.listOpen
                        readonly property bool current: typed.activeFocus || toggle.activeFocus || choice.activeFocus || pathField.hasKeys

                        function take(): void {
                            if (field.path)
                                pathField.focusField();
                            else
                                (field.kind === "checkbox" ? toggle : field.kind === "select" ? choice : typed).forceActiveFocus();
                        }

                        function fillPath(path: string): void {
                            pathField.focusField();
                            pathField.take(pathField.homeRelative(path));
                        }

                        function value(): var {
                            if (field.path)
                                return pathField.text;
                            if (field.kind === "checkbox")
                                return toggle.checked;
                            if (field.kind === "select")
                                return choice.options[choice.chosen]?.value ?? "";
                            return typed.text;
                        }

                        function browse(): void {
                            form.browse(field.index, field.kind === "folder", field.modelData.accept ?? []);
                        }

                        fill: true
                        onCurrentChanged: {
                            if (current)
                                form.current = index;
                        }
                        // A file field's paths open under it: they scroll into view with it.
                        onHeightChanged: {
                            if (current)
                                Qt.callLater(form.reveal);
                        }

                        K.Labelled {
                            visible: field.kind !== "checkbox"
                            label: field.label
                            action: field.path ? browseButton : null

                            Component {
                                id: browseButton

                                K.Button {
                                    text: "Browse…"
                                    kind: "Text"
                                    size: "Sm"
                                    icon: "folder_open"
                                    activeFocusOnTab: false
                                    onActivated: field.browse()
                                }
                            }

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

                            FileField {
                                id: pathField

                                visible: field.path
                                active: field.path
                                search: form.search
                                place: field.index
                                folder: field.kind === "folder"
                                placeholder: field.modelData.placeholder ?? ""
                                keys: form.keys
                                Component.onCompleted: {
                                    if (field.path)
                                        text = field.modelData.value ?? "";
                                }
                                onBrowse: field.browse()
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

                                text: field.label
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
