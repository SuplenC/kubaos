import QtQuick
import QtQuick.Layouts
import qs.theme
import qs.components
import qs.services

// The wallpaper settings, the picker's last tab. Motion's mode on top, then
// every option ukiyo has, generated from `ukiyo config schema --json`: one
// row per option, grouped by `group`, the scene's groups first. Nothing here
// knows a single option by name; a new knob in ukiyo or a new scene param
// shows up on its own.
//
// Bool: a switch. Float: a slider with the value and its unit. Choice:
// segments. Color: a swatch and a hex field. String and list: read-only.
// Changes apply live (ukiyo config set, into config.toml); a slider sends at
// most ~10 values a second and the last one when you let go. "yours" marks
// what you changed; Delete puts it back to KubaOS's default (defaults.toml).
//
// Keys: Tab/Shift+Tab or Up/Down move between rows, Left/Right change
// (Shift: bigger steps), Home/End to the ends, Space toggles, Enter
// activates or edits a colour, Delete resets, Esc goes back to the grid.
FocusScope {
    id: root

    // Esc on a row (not while editing a colour): back to the wallpapers.
    signal back

    // The rows, in order: { kind: "motion" | "scene" | "group" | "option", ... }.
    readonly property var rows: {
        const out = [{
                kind: "motion"
            }];
        const schema = Ukiyo.schema;
        const order = [];
        const byGroup = {};
        for (const o of schema) {
            if (!byGroup[o.group]) {
                byGroup[o.group] = [];
                order.push(o.group);
            }
            byGroup[o.group].push(o);
        }
        const sceneGroups = order.filter(g => byGroup[g][0].key.startsWith("scenes."));
        const globalGroups = order.filter(g => !byGroup[g][0].key.startsWith("scenes."));
        let lastScene = "";
        for (const g of sceneGroups) {
            const id = byGroup[g][0].key.split(".")[1];
            if (id !== lastScene) {
                lastScene = id;
                const name = Ukiyo.name(id);
                out.push({
                    kind: "scene",
                    id: id,
                    name: name,
                    yours: schema.filter(o => o.key.startsWith(`scenes.${id}.`) && o.source === "user").length
                });
            }
            // "Kabukichō Gate: Weather" -> "Weather" under the scene's header.
            const name = Ukiyo.name(id);
            out.push({
                kind: "group",
                title: g.startsWith(`${name}: `) ? g.slice(name.length + 2) : g
            });
            for (const o of byGroup[g])
                out.push({
                    kind: "option",
                    opt: o
                });
        }
        for (const g of globalGroups) {
            out.push({
                kind: "group",
                title: g
            });
            for (const o of byGroup[g])
                out.push({
                    kind: "option",
                    opt: o
                });
        }
        return out;
    }
    readonly property var focusable: rows.map((r, i) => r.kind === "group" ? -1 : i).filter(i => i >= 0)
    property int current: 0
    // Values being dragged or stepped, shown before ukiyo confirms them.
    property var pending: ({})

    function move(delta: int): void {
        const at = focusable.indexOf(current);
        const n = focusable.length;
        if (n === 0)
            return;
        current = focusable[((at < 0 ? 0 : at + delta) % n + n) % n];
    }

    function rowItem(i: int): Item {
        return repeater.count > i ? repeater.itemAt(i)?.item ?? null : null;
    }

    // Keep the focused row on screen, with a row of air around it.
    function reveal(): void {
        const it = repeater.itemAt(current);
        if (!it)
            return;
        const top = it.y - Theme.spacing.xxl;
        const bottom = it.y + it.height + Theme.spacing.xxl;
        if (top < flick.contentY)
            flick.contentY = Math.max(0, top);
        else if (bottom > flick.contentY + flick.height)
            flick.contentY = Math.min(flick.contentHeight - flick.height, bottom - flick.height);
    }

    function setPending(key: string, value: var): void {
        const p = Object.assign({}, pending);
        if (value === undefined)
            delete p[key];
        else
            p[key] = value;
        pending = p;
    }

    onCurrentChanged: Qt.callLater(reveal)
    onActiveFocusChanged: if (activeFocus && focusable.indexOf(current) < 0)
        current = focusable[0] ?? 0
    // ukiyo confirmed: drop what's pending once the schema has the value.
    Connections {
        target: Ukiyo

        function onSchemaChanged(): void {
            const p = {};
            for (const k in root.pending) {
                const o = Ukiyo.option(k);
                if (o && o.value !== root.pending[k])
                    p[k] = root.pending[k];
            }
            root.pending = p;
        }
    }

    Keys.onPressed: event => {
        const k = event.key;
        if (k === Qt.Key_Tab || k === Qt.Key_Down || k === Qt.Key_J)
            move(1);
        else if (k === Qt.Key_Backtab || k === Qt.Key_Up || k === Qt.Key_K)
            move(-1);
        else if (k === Qt.Key_Escape)
            root.back();
        else if (k === Qt.Key_PageDown)
            flick.contentY = Math.min(flick.contentHeight - flick.height, flick.contentY + flick.height * 0.8);
        else if (k === Qt.Key_PageUp)
            flick.contentY = Math.max(0, flick.contentY - flick.height * 0.8);
        else {
            const it = rowItem(current);
            if (!it || !it.act(k, (event.modifiers & Qt.ShiftModifier) !== 0))
                return;
        }
        event.accepted = true;
    }

    Flickable {
        id: flick

        anchors.fill: parent
        contentWidth: width
        contentHeight: column.implicitHeight + Theme.spacing.lg
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Behavior on contentY {
            enabled: !flick.moving

            Anim {
                duration: Theme.anim.normal
                easing.bezierCurve: Theme.anim.standard
            }
        }

        Column {
            id: column

            width: flick.width - Theme.spacing.md
            spacing: Theme.spacing.xxs

            Repeater {
                id: repeater

                model: root.rows

                delegate: Loader {
                    id: slot

                    required property var modelData
                    required property int index

                    width: column.width
                    sourceComponent: ({
                            motion: motionRow,
                            scene: sceneRow,
                            group: groupRow,
                            option: optionRow
                        })[modelData.kind]
                }
            }

            StyledText {
                visible: !Ukiyo.available || !Ukiyo.schemaLoaded
                width: column.width
                topPadding: Theme.spacing.xl
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WordWrap
                text: !Ukiyo.listed ? "Asking ukiyo what it's got…" : !Ukiyo.available ? "ukiyo isn't installed, so there's nothing live to tune. Motion above still runs the loops." : "ukiyo didn't answer. Is it installed right? `ukiyo config schema` in a terminal says why."
                color: Theme.m3onSurfaceVariant
            }
        }
    }

    // --- rows ---------------------------------------------------------------

    // A focusable row: label and help on the left, the control on the right,
    // a focus ring while it's the current one.
    component SettingRow: Rectangle {
        id: row

        required property int rowIndex
        property string label
        property string help
        default property alias control: controlSlot.data
        readonly property bool current: root.current === rowIndex
        readonly property bool focused: current && root.activeFocus

        // Key -> handled? Overridden per kind.
        property var handler: (key, shift) => false
        function act(key: int, shift: bool): bool {
            return handler(key, shift);
        }

        implicitHeight: Math.max(52, texts.implicitHeight + Theme.spacing.md * 2)
        radius: Theme.rounding.md
        color: current ? Theme.m3surfaceContainerHigh : "transparent"

        Behavior on color {
            CAnim {}
        }

        FocusRing {
            shown: row.focused
        }

        MouseArea {
            anchors.fill: parent
            onPressed: {
                root.current = row.rowIndex;
                root.forceActiveFocus();
            }
        }

        Column {
            id: texts

            anchors.left: parent.left
            anchors.leftMargin: Theme.spacing.lg
            anchors.right: controlSlot.left
            anchors.rightMargin: Theme.spacing.lg
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.spacing.xxs

            StyledText {
                width: parent.width
                text: row.label
                font.weight: Font.Medium
                color: Theme.m3onSurface
            }

            StyledText {
                width: parent.width
                visible: text !== ""
                text: row.help
                font.pixelSize: Theme.font.sm
                color: Theme.m3onSurfaceVariant
                elide: row.current ? Text.ElideNone : Text.ElideRight
                wrapMode: row.current ? Text.WordWrap : Text.NoWrap
                maximumLineCount: row.current ? 3 : 1
            }
        }

        Item {
            id: controlSlot

            anchors.right: parent.right
            anchors.rightMargin: Theme.spacing.md
            anchors.verticalCenter: parent.verticalCenter
            width: Math.round(row.width * 0.5)
            height: childrenRect.height
        }
    }

    Component {
        id: motionRow

        SettingRow {
            rowIndex: parent?.index ?? -1
            label: "Motion"
            help: Motion.mode === "never" ? "Never. Posters only, the scene holds one frame." : Motion.mode === "always" ? "Always, battery be damned. Loops and scenes keep moving off the charger." : "On AC. Plugged in it moves; on battery it holds still."
            handler: (key, shift) => {
                const order = ["always", "ac", "never"];
                const i = order.indexOf(Motion.mode);
                if (key === Qt.Key_Left || key === Qt.Key_H)
                    Motion.setMode(order[(i + 2) % 3]);
                else if (key === Qt.Key_Right || key === Qt.Key_L || key === Qt.Key_Space || key === Qt.Key_Return || key === Qt.Key_Enter)
                    Motion.setMode(order[(i + 1) % 3]);
                else
                    return false;
                return true;
            }

            Segmented {
                anchors.right: parent.right
                options: ["always", "ac", "never"]
                labels: ["Always", "On AC", "Never"]
                value: Motion.mode
                onPicked: v => Motion.setMode(v)
            }
        }
    }

    Component {
        id: sceneRow

        SettingRow {
            id: sr

            readonly property var entry: parent?.modelData ?? ({})

            rowIndex: parent?.index ?? -1
            label: entry.name ?? ""
            help: entry.yours === 1 ? "One of this scene's settings is yours. Delete or Enter puts it back." : entry.yours ? `${entry.yours} of this scene's settings are yours. Delete or Enter puts them all back.` : "Everything below is the scene's own, as it ships."
            handler: (key, shift) => {
                if (key !== Qt.Key_Delete && key !== Qt.Key_Backspace && key !== Qt.Key_Return && key !== Qt.Key_Enter && key !== Qt.Key_Space)
                    return false;
                if (sr.entry.yours)
                    Ukiyo.configReset(`scenes.${sr.entry.id}`);
                return true;
            }
            color: current ? Theme.m3surfaceContainerHigh : Theme.alpha(Theme.m3surfaceContainerHigh, 0.5)

            Chip {
                anchors.right: parent.right
                icon: "restart_alt"
                text: "Reset scene"
                enabled: sr.entry.yours > 0
                onClicked: Ukiyo.configReset(`scenes.${sr.entry.id}`)
            }
        }
    }

    Component {
        id: groupRow

        Item {
            implicitHeight: 34

            StyledText {
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacing.lg
                anchors.bottom: parent.bottom
                anchors.bottomMargin: Theme.spacing.xs
                text: (parent.parent?.modelData?.title ?? "").toUpperCase()
                font.pixelSize: Theme.font.xs
                font.weight: Font.DemiBold
                font.letterSpacing: 1.2
                color: Theme.m3primary
            }
        }
    }

    Component {
        id: optionRow

        SettingRow {
            id: orow

            readonly property var opt: parent?.modelData?.opt ?? ({})
            readonly property var value: root.pending[opt.key] !== undefined ? root.pending[opt.key] : opt.value
            readonly property bool yours: opt.source === "user"
            readonly property bool editable: ["bool", "float", "choice", "color"].includes(opt.type)

            rowIndex: parent?.index ?? -1
            label: opt.label ?? ""
            help: opt.help ?? ""
            handler: (key, shift) => {
                if (key === Qt.Key_Delete || key === Qt.Key_Backspace) {
                    if (orow.yours) {
                        root.setPending(orow.opt.key, undefined);
                        Ukiyo.configReset(orow.opt.key);
                    }
                    return true;
                }
                return control.item?.act(key, shift) ?? false;
            }

            RowLayout {
                anchors.right: parent.right
                width: parent.width
                spacing: Theme.spacing.sm

                Loader {
                    id: control

                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignVCenter
                    sourceComponent: ({
                            bool: boolControl,
                            float: floatControl,
                            choice: choiceControl,
                            color: colorControl
                        })[orow.opt.type] ?? textControl

                    // What every control gets.
                    property var opt: orow.opt
                    property var value: orow.value
                    property bool current: orow.current
                    property int rowIndex: orow.rowIndex
                }

                // Where the value comes from, and the way back.
                Item {
                    Layout.preferredWidth: 78
                    Layout.preferredHeight: 26
                    Layout.alignment: Qt.AlignVCenter

                    Rectangle {
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        width: source.implicitWidth + (orow.yours ? 26 : 0) + Theme.spacing.md
                        height: 22
                        radius: 11
                        color: orow.yours ? Theme.m3secondaryContainer : "transparent"
                        border.width: orow.yours ? 0 : 1
                        border.color: Theme.m3outlineVariant

                        Row {
                            anchors.centerIn: parent
                            spacing: Theme.spacing.xxs

                            StyledText {
                                id: source

                                anchors.verticalCenter: parent.verticalCenter
                                text: orow.yours ? "yours" : orow.opt.source === "defaults" ? "KubaOS" : "default"
                                font.pixelSize: Theme.font.xs
                                font.weight: Font.Medium
                                color: orow.yours ? Theme.m3onSecondaryContainer : Theme.m3outline
                            }

                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: orow.yours
                                name: "undo"
                                size: 16
                                color: Theme.m3onSecondaryContainer
                            }
                        }

                        StateLayer {
                            enabled: orow.yours
                            color: Theme.m3onSecondaryContainer
                            onClicked: {
                                root.setPending(orow.opt.key, undefined);
                                Ukiyo.configReset(orow.opt.key);
                            }
                        }
                    }
                }
            }
        }
    }

    // --- controls (inside an option row's Loader: opt, value, current) -------

    Component {
        id: boolControl

        Item {
            id: c

            readonly property var opt: parent.opt
            readonly property bool on: parent.value === true

            function act(key: int, shift: bool): bool {
                if (![Qt.Key_Space, Qt.Key_Return, Qt.Key_Enter, Qt.Key_Left, Qt.Key_Right, Qt.Key_H, Qt.Key_L].includes(key))
                    return false;
                const want = key === Qt.Key_Left || key === Qt.Key_H ? false : key === Qt.Key_Right || key === Qt.Key_L ? true : !on;
                if (want !== on)
                    set(want);
                return true;
            }
            function set(v: bool): void {
                root.setPending(opt.key, v);
                Ukiyo.configSet(opt.key, v);
            }

            implicitHeight: 28

            Rectangle {
                id: track

                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: 46
                height: 26
                radius: 13
                color: c.on ? Theme.m3primary : Theme.m3surfaceContainerHighest
                border.width: c.on ? 0 : 2
                border.color: Theme.m3outline

                Behavior on color {
                    CAnim {}
                }

                Rectangle {
                    width: c.on ? 18 : 12
                    height: width
                    radius: width / 2
                    anchors.verticalCenter: parent.verticalCenter
                    x: c.on ? parent.width - width - 4 : 7
                    color: c.on ? Theme.m3onPrimary : Theme.m3outline

                    Behavior on x {
                        Anim {
                            duration: Theme.anim.small
                            easing.bezierCurve: Theme.anim.spatial
                        }
                    }

                    Behavior on width {
                        Anim {
                            duration: Theme.anim.small
                        }
                    }
                }

                StateLayer {
                    radius: 13
                    onClicked: c.set(!c.on)
                }
            }

            StyledText {
                anchors.right: track.left
                anchors.rightMargin: Theme.spacing.md
                anchors.verticalCenter: parent.verticalCenter
                text: c.on ? "On" : "Off"
                color: Theme.m3onSurfaceVariant
            }
        }
    }

    Component {
        id: floatControl

        Item {
            id: c

            readonly property var opt: parent.opt
            readonly property real min: opt.min
            readonly property real max: opt.max
            readonly property real step: opt.step > 0 ? opt.step : (max - min) / 100
            readonly property real v: Number(parent.value)
            readonly property real frac: max > min ? (v - min) / (max - min) : 0
            // Last value sent; a drag sends at most every 100 ms.
            property real sent: NaN
            property real want: NaN

            function snap(x: real): real {
                const s = Math.round((Math.max(min, Math.min(max, x)) - min) / step) * step + min;
                return Number(s.toFixed(6));
            }
            function request(x: real, now: bool): void {
                want = snap(x);
                root.setPending(opt.key, want);
                if (now) {
                    throttle.stop();
                    send();
                } else if (!throttle.running) {
                    throttle.start();
                }
            }
            function send(): void {
                if (isNaN(want) || want === sent)
                    return;
                sent = want;
                Ukiyo.configSet(opt.key, want);
            }
            function act(key: int, shift: bool): bool {
                const big = shift ? 10 : 1;
                if (key === Qt.Key_Left || key === Qt.Key_H)
                    request(v - step * big, false);
                else if (key === Qt.Key_Right || key === Qt.Key_L)
                    request(v + step * big, false);
                else if (key === Qt.Key_Home)
                    request(min, false);
                else if (key === Qt.Key_End)
                    request(max, false);
                else
                    return false;
                return true;
            }
            // "1.25×", "600 s", "22:15" for hours, "60 fps".
            function format(x: real): string {
                const unit = opt.unit ?? "";
                if (unit === "h") {
                    const m = Math.round(x * 60);
                    return `${Math.floor(m / 60)}:${String(m % 60).padStart(2, "0")}`;
                }
                const d = Math.max(0, Math.min(2, -Math.floor(Math.log10(step) + 1e-9)));
                const n = x.toFixed(d);
                return unit === "" ? n : unit === "×" ? `${n}×` : `${n} ${unit}`;
            }

            implicitHeight: 28

            Timer {
                id: throttle

                interval: 100
                onTriggered: c.send()
            }

            StyledText {
                id: readout

                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: 64
                horizontalAlignment: Text.AlignRight
                text: c.format(c.v)
                font.features: Theme.font.tabular
                color: c.parent.current ? Theme.m3onSurface : Theme.m3onSurfaceVariant
            }

            Rectangle {
                id: bar

                anchors.left: parent.left
                anchors.right: readout.left
                anchors.rightMargin: Theme.spacing.md
                anchors.verticalCenter: parent.verticalCenter
                height: 8
                radius: 4
                color: Theme.m3surfaceContainerHighest

                Rectangle {
                    width: Math.max(height, parent.width * Math.max(0, Math.min(1, c.frac)))
                    height: parent.height
                    radius: 4
                    color: c.parent.current ? Theme.m3primary : Theme.alpha(Theme.m3primary, 0.7)

                    Behavior on width {
                        enabled: !drag.pressed

                        Anim {
                            duration: Theme.anim.small
                            easing.bezierCurve: Theme.anim.emphasizedDecel
                        }
                    }
                }

                Rectangle {
                    x: Math.max(0, Math.min(parent.width, parent.width * c.frac)) - width / 2
                    anchors.verticalCenter: parent.verticalCenter
                    width: drag.pressed ? 6 : 4
                    height: 20
                    radius: 2
                    color: Theme.m3primary
                }

                MouseArea {
                    id: drag

                    anchors.fill: parent
                    anchors.margins: -8
                    cursorShape: Qt.PointingHandCursor
                    onPressed: mouse => {
                        root.current = c.parent.rowIndex;
                        root.forceActiveFocus();
                        c.request(c.min + (mouse.x - 8) / bar.width * (c.max - c.min), false);
                    }
                    onPositionChanged: mouse => c.request(c.min + (mouse.x - 8) / bar.width * (c.max - c.min), false)
                    onReleased: c.request(c.want, true)
                    onWheel: wheel => c.request(c.v + (wheel.angleDelta.y > 0 ? c.step : -c.step), false)
                }
            }
        }
    }

    Component {
        id: choiceControl

        Item {
            id: c

            readonly property var opt: parent.opt

            function act(key: int, shift: bool): bool {
                const o = opt.options;
                const i = o.indexOf(parent.value);
                let next;
                if (key === Qt.Key_Left || key === Qt.Key_H)
                    next = o[(i - 1 + o.length) % o.length];
                else if ([Qt.Key_Right, Qt.Key_L, Qt.Key_Space, Qt.Key_Return, Qt.Key_Enter].includes(key))
                    next = o[(i + 1) % o.length];
                else
                    return false;
                set(next);
                return true;
            }
            function set(v: string): void {
                root.setPending(opt.key, v);
                Ukiyo.configSet(opt.key, v);
            }

            implicitHeight: seg.implicitHeight

            Segmented {
                id: seg

                anchors.right: parent.right
                options: c.opt.options
                labels: c.opt.options
                value: c.parent.value
                onPicked: v => c.set(v)
            }
        }
    }

    Component {
        id: colorControl

        Item {
            id: c

            readonly property var opt: parent.opt
            readonly property string hex: String(parent.value)
            readonly property bool valid: /^#[0-9a-fA-F]{6}$/.test(field.text)

            function act(key: int, shift: bool): bool {
                if (key !== Qt.Key_Return && key !== Qt.Key_Enter)
                    return false;
                field.text = hex;
                field.forceActiveFocus();
                field.selectAll();
                return true;
            }
            function apply(): void {
                if (valid && field.text.toLowerCase() !== hex.toLowerCase()) {
                    root.setPending(opt.key, field.text.toLowerCase());
                    Ukiyo.configSet(opt.key, field.text.toLowerCase());
                }
                root.forceActiveFocus();
            }

            implicitHeight: 30

            Rectangle {
                id: box

                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: 132
                height: 30
                radius: Theme.rounding.sm
                color: Theme.m3surfaceContainerHighest
                border.width: field.activeFocus ? 2 : 0
                border.color: c.valid ? Theme.m3primary : Theme.m3error

                Rectangle {
                    id: swatch

                    anchors.left: parent.left
                    anchors.leftMargin: 5
                    anchors.verticalCenter: parent.verticalCenter
                    width: 20
                    height: 20
                    radius: 5
                    color: field.activeFocus && c.valid ? field.text : c.hex
                    border.width: 1
                    border.color: Theme.alpha(Theme.m3onSurface, 0.2)
                }

                TextInput {
                    id: field

                    anchors.left: swatch.right
                    anchors.leftMargin: Theme.spacing.sm
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: c.hex
                    font.family: Theme.font.mono
                    font.pixelSize: Theme.font.md
                    color: Theme.m3onSurface
                    selectionColor: Theme.m3secondaryContainer
                    selectedTextColor: Theme.m3onSecondaryContainer
                    maximumLength: 7
                    readOnly: !activeFocus
                    activeFocusOnTab: false
                    onAccepted: c.apply()
                    onActiveFocusChanged: if (!activeFocus)
                        text = Qt.binding(() => c.hex)
                    Keys.onEscapePressed: event => {
                        text = c.hex;
                        root.forceActiveFocus();
                        event.accepted = true;
                    }
                    Keys.onTabPressed: event => {
                        c.apply();
                        event.accepted = true;
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    visible: !field.activeFocus
                    cursorShape: Qt.IBeamCursor
                    onClicked: c.act(Qt.Key_Return, false)
                }
            }

            StyledText {
                anchors.right: box.left
                anchors.rightMargin: Theme.spacing.md
                anchors.verticalCenter: parent.verticalCenter
                visible: c.parent.current && !field.activeFocus
                text: "Enter to edit"
                font.pixelSize: Theme.font.sm
                color: Theme.m3outline
            }
        }
    }

    Component {
        id: textControl

        Item {
            function act(key: int, shift: bool): bool {
                return false;
            }

            implicitHeight: 24

            StyledText {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width
                horizontalAlignment: Text.AlignRight
                text: {
                    const v = parent.parent.value;
                    const s = Array.isArray(v) ? v.join(", ") : String(v ?? "");
                    return s === "" ? "—" : s;
                }
                font.family: Theme.font.mono
                font.pixelSize: Theme.font.sm
                color: Theme.m3onSurfaceVariant
            }
        }
    }

    // Pills side by side, the selected one filled. Clicks pick; keys are
    // the row's.
    component Segmented: Row {
        id: seg

        property list<string> options
        property list<string> labels
        property string value

        signal picked(string value)

        spacing: Theme.spacing.xxs

        Repeater {
            model: seg.options

            delegate: Rectangle {
                id: pill

                required property string modelData
                required property int index
                readonly property bool selected: seg.value === modelData

                implicitWidth: label.implicitWidth + Theme.spacing.lg * 2
                implicitHeight: 28
                radius: 14
                color: selected ? Theme.m3secondaryContainer : Theme.m3surfaceContainerHighest

                Behavior on color {
                    CAnim {}
                }

                StyledText {
                    id: label

                    anchors.centerIn: parent
                    text: seg.labels[pill.index] ?? pill.modelData
                    font.pixelSize: Theme.font.sm
                    font.weight: pill.selected ? Font.DemiBold : Font.Normal
                    color: pill.selected ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
                }

                StateLayer {
                    onClicked: seg.picked(pill.modelData)
                }
            }
        }
    }

    // A small action pill with an icon.
    component Chip: Rectangle {
        id: chip

        property string icon
        property string text

        signal clicked

        implicitWidth: chipRow.implicitWidth + Theme.spacing.md * 2
        implicitHeight: 28
        radius: 14
        color: Theme.m3surfaceContainerHighest
        opacity: enabled ? 1 : 0.45

        Row {
            id: chipRow

            anchors.centerIn: parent
            spacing: Theme.spacing.xs

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: chip.icon
                size: 16
                color: Theme.m3onSurfaceVariant
            }

            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                text: chip.text
                font.pixelSize: Theme.font.sm
                color: Theme.m3onSurfaceVariant
            }
        }

        StateLayer {
            enabled: chip.enabled
            onClicked: chip.clicked()
        }
    }
}
