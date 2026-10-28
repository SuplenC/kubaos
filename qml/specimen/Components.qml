import QtQuick
import QtQuick.Shapes
import KubaOS.Components as K

// The specimen's components page, drawn with `KubaOS.Components`: the same page
// `kubaos-specimen` draws with `kubaos-ui` (`crates/kubaos-specimen/src/pages/components.rs` and
// the bar above it), box for box, so the cross-toolkit test can hold the two to the same frame and
// their screenshots can be laid side by side.
K.Stack {
    id: root

    // The scale the page is shown at, named in the bar.
    property real shownScale: 1

    // The states a button is shown in, left to right.
    readonly property list<string> stateNames: ["Resting", "Hovered", "Pressed", "Focused", "Disabled"]
    readonly property list<string> kinds: ["Filled", "Tonal", "Text", "Danger"]
    readonly property list<string> sizes: ["Sm", "Md", "Lg"]
    // The columns of the button tables: the kinds' names, then one per state.
    readonly property list<int> columns: [1, 2, 2, 2, 2, 2]
    // The picture the media samples load: the accent fading into the cold accent, as the iced
    // page draws it.
    readonly property string picture: {
        const from = K.Tokens.color.primary.toString();
        const to = K.Tokens.color.cold.toString();
        const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 10"><defs><linearGradient id="g"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect width="16" height="10" fill="url(#g)"/></svg>`;
        return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
    }

    // The specimen's October 2026, weeks from Monday, as the shots' clock has it: Tuesday the 27th
    // is today, and All Saints' Day falls in the next month's first week, as the iced page has it.
    readonly property var october: {
        const pad = n => String(n).padStart(2, "0");
        const weeks = [];
        for (let week = 0; week < 6; week++) {
            const days = [];
            for (let weekday = 0; weekday < 7; weekday++) {
                const index = week * 7 + weekday;
                const [month, day] = index <= 2 ? [9, 28 + index] : index <= 33 ? [10, index - 2] : [11, index - 33];
                days.push({
                    date: `2026-${pad(month)}-${pad(day)}`,
                    day: day,
                    inMonth: month === 10,
                    today: month === 10 && day === 27,
                    weekend: weekday >= 5,
                    holiday: month === 11 && day === 1 ? "All Saints' Day" : ""
                });
            }
            weeks.push({
                number: 40 + week,
                days: days
            });
        }
        return {
            year: 2026,
            month: 10,
            title: "October 2026",
            weekdays: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            weeks: weeks
        };
    }

    fill: true

    // A minute of a reading, a sample every two seconds, as [t, value]: the wave the iced page
    // draws.
    function wave(level: real, swing: real, phase: real): var {
        const points = [];
        for (let i = 0; i < 30; i++)
            points.push([i * 2000, level + swing * Math.sin(i * 0.4 + phase) + swing * 0.4 * Math.sin(i * 1.3 + phase)]);
        return points;
    }

    // A 24 symbol in a small control's box, a badge over its top right corner, as the iced page
    // stacks them.
    component Badged: Item {
        id: badged

        property string icon
        property int count: 0
        readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

        implicitWidth: K.Tokens.size.control_sm
        implicitHeight: K.Tokens.size.control_sm

        K.Icon {
            x: K.Look.snap((badged.width - width) / 2, badged.dpr)
            y: K.Look.snap((badged.height - height) / 2, badged.dpr)
            name: badged.icon
            size: K.Tokens.icon.s24
            color: K.Tokens.color.on_surface_variant
        }

        K.Badge {
            x: badged.width - width
            count: badged.count
        }
    }

    function sizeName(size: string): string {
        return `${size} · ${K.ButtonMetrics.of(size).height}`;
    }

    // The bar above the page: the pages, then the mode and the scale it is shown at. A toolbar
    // band: the container padding at the sides, as the page under it, and a small control's room
    // above and below.
    K.Container {
        K.Row {
            gap: K.Tokens.space.s16
            align: "centre"
            fill: true
            topPadding: K.Tokens.space.s16
            bottomPadding: K.Tokens.space.s16

            K.Text {
                text: "KubaOS design specimen"
                style: K.Tokens.type.subheading
            }

            K.Row {
                gap: K.Tokens.space.s4
                align: "centre"

                Repeater {
                    model: ["Type", "Colour", "Space and shape", "Components", "Icons", "Motion"]

                    K.Button {
                        required property string modelData

                        text: modelData
                        kind: modelData === "Components" ? "Tonal" : "Text"
                        size: "Sm"
                    }
                }
            }

            K.Fill {
            }

            K.KeyHints {
                hints: [["Tab", "next page"], ["← →", "pages"], ["M", "mode"]]
            }

            K.Chip {
                text: `${root.shownScale}×`
            }

            K.IconButton {
                icon: K.Tokens.mode === "dark" ? "light_mode" : "dark_mode"
                label: K.Tokens.mode === "dark" ? "Light mode" : "Dark mode"
                kind: "Text"
                size: "Sm"
            }
        }
    }

    K.Divider {
    }

    K.Page {
        K.Stack {
            gap: K.Tokens.space.s40

            K.Header {
                accent: "部品"
                title: "Components"
                lede: "The closed set: every kind and size, resting, hovered, pressed, focused and disabled."
            }

            K.Stack {
                gap: K.Tokens.space.s48

                K.Section {
                    title: "Buttons"
                    description: "Filled, Tonal, Text and Danger, in small, regular and large, each resting, hovered, pressed, focused and disabled."

                    K.Stack {
                        gap: K.Tokens.space.s32

                        Repeater {
                            model: root.sizes

                            K.Stack {
                                id: buttonSize

                                required property string modelData

                                gap: K.Tokens.space.s12

                                K.Text {
                                    text: root.sizeName(buttonSize.modelData)
                                    style: K.Tokens.type.subheading
                                }

                                ButtonTable {
                                    size: buttonSize.modelData
                                }
                            }
                        }
                    }
                }

                K.Section {
                    title: "Icon buttons"
                    description: "Circles as tall as the label buttons of each size. The label is required: it shows in a hint on hover and while focused."

                    K.Stack {
                        gap: K.Tokens.space.s32

                        Repeater {
                            model: root.sizes

                            K.Stack {
                                id: iconSize

                                required property string modelData

                                gap: K.Tokens.space.s12

                                K.Text {
                                    text: root.sizeName(iconSize.modelData)
                                    style: K.Tokens.type.subheading
                                }

                                // Rows 40 apart, so a focused button's hint has room under it.
                                K.Stack {
                                    gap: K.Tokens.space.s40

                                    K.Grid {
                                        weights: root.columns
                                        gap: K.Tokens.space.s16
                                        centred: true

                                        StateHeads {
                                        }
                                    }

                                    Repeater {
                                        model: root.kinds

                                        IconButtonRow {
                                            required property string modelData

                                            kind: modelData
                                            size: iconSize.modelData
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                K.Section {
                    title: "Fields"
                    description: "As tall as a regular button. The label's row fits a small button, so fields side by side line up."

                    K.Grid {
                        columns: 3
                        gap: K.Tokens.space.s24

                        K.Labelled {
                            label: "Full name"

                            K.Field {
                                placeholder: "Your name"
                            }
                        }

                        K.Labelled {
                            label: "Hostname"
                            note: "Letters, digits and dashes."

                            K.Field {
                                text: "tuxedo"
                                icon: "computer"
                            }
                        }

                        K.Labelled {
                            label: "Search"
                            note: "Focused: the caret, and the tube on its own edge."

                            K.Field {
                                text: "Shinj"
                                placeholder: "Search"
                                icon: "search"
                                focused: true
                            }
                        }

                        K.Labelled {
                            label: "Password"
                            action: Component {
                                K.Button {
                                    text: "Show"
                                    kind: "Text"
                                    size: "Sm"
                                    icon: "visibility"
                                }
                            }

                            K.Field {
                                text: "hunter2"
                                secret: true
                            }
                        }

                        K.Labelled {
                            label: "Email for git"
                            note: "That's not an email. Typo?"
                            invalid: true

                            K.Field {
                                text: "not-an-email"
                                icon: "mail"
                                status: "Invalid"
                            }
                        }

                        K.Labelled {
                            label: "User name"
                            note: "Set when you installed; read-only here."

                            K.Field {
                                text: "kuba"
                                icon: "lock"
                                status: "Locked"
                            }
                        }
                    }
                }

                K.Section {
                    title: "Switches, checkboxes and radio buttons"

                    K.Stack {
                        gap: K.Tokens.space.s24

                        K.Row {
                            gap: K.Tokens.space.s32
                            align: "centre"

                            Repeater {
                                model: [
                                    {
                                        label: "Off",
                                        on: false,
                                        position: 0,
                                        focused: false,
                                        enabled: true
                                    },
                                    {
                                        label: "On",
                                        on: true,
                                        position: 1,
                                        focused: false,
                                        enabled: true
                                    },
                                    {
                                        label: "Halfway",
                                        on: true,
                                        position: 0.5,
                                        focused: false,
                                        enabled: true
                                    },
                                    {
                                        label: "Focused",
                                        on: true,
                                        position: 1,
                                        focused: true,
                                        enabled: true
                                    },
                                    {
                                        label: "Disabled off",
                                        on: false,
                                        position: 0,
                                        focused: false,
                                        enabled: false
                                    },
                                    {
                                        label: "Disabled on",
                                        on: true,
                                        position: 1,
                                        focused: false,
                                        enabled: false
                                    }
                                ]

                                K.Stack {
                                    id: sample

                                    required property var modelData

                                    gap: K.Tokens.space.s8

                                    K.Switch {
                                        checked: sample.modelData.on
                                        position: sample.modelData.position
                                        focused: sample.modelData.focused
                                        enabled: sample.modelData.enabled
                                    }

                                    K.Text {
                                        text: sample.modelData.label
                                        style: K.Tokens.type.small
                                        color: K.Tokens.color.on_surface_variant
                                    }
                                }
                            }
                        }

                        K.Row {
                            gap: K.Tokens.space.s32
                            align: "centre"

                            K.Checkbox {
                                text: "Bluetooth"
                            }

                            K.Checkbox {
                                text: "Some wallpapers"
                                tick: "Partial"
                            }

                            K.Checkbox {
                                text: "Encrypt the disk"
                                tick: "Full"
                            }

                            K.Checkbox {
                                text: "Focused"
                                tick: "Full"
                                focused: true
                            }

                            K.Checkbox {
                                text: "Disabled"
                                tick: "Full"
                                enabled: false
                            }
                        }

                        K.Row {
                            gap: K.Tokens.space.s32
                            align: "centre"

                            K.Radio {
                                text: "Keep Windows"
                            }

                            K.Radio {
                                text: "Erase everything"
                                checked: true
                            }

                            K.Radio {
                                text: "Focused"
                                focused: true
                            }

                            K.Radio {
                                text: "Disabled"
                                checked: true
                                enabled: false
                            }
                        }
                    }
                }

                K.Section {
                    title: "Sliders"
                    description: "A pill whose fill is the value: its round end marks it. Steps show on both sides, and the mark the value rests on turns solid. Regular above, compact below."

                    K.Stack {
                        gap: K.Tokens.space.s32

                        Repeater {
                            model: ["Regular", "Compact"]

                            K.Grid {
                                id: sizes

                                required property string modelData

                                columns: 3
                                gap: K.Tokens.space.s32

                                Repeater {
                                    model: [
                                        {
                                            label: "Volume, 62%",
                                            icon: "volume_up",
                                            value: 0.62
                                        },
                                        {
                                            label: "Under the pointer",
                                            icon: "volume_up",
                                            value: 0.62,
                                            preview: "hovered"
                                        },
                                        {
                                            label: "Held: the fill lights up",
                                            icon: "volume_up",
                                            value: 0.62,
                                            preview: "pressed"
                                        },
                                        {
                                            label: "Muted: the icon mutes",
                                            icon: "volume_off",
                                            value: 0,
                                            words: "Muted"
                                        },
                                        {
                                            label: "Auto: the sensor's level",
                                            icon: "motion_photos_auto",
                                            value: 0.55,
                                            words: "Auto · 55%",
                                            soft: true
                                        },
                                        {
                                            label: "Five steps, resting on one",
                                            icon: "balance",
                                            stops: 5,
                                            index: 2,
                                            names: ["Super Saver", "Power Saver", "Balanced", "Performance", "Max Performance"]
                                        },
                                        {
                                            label: "Sticky at 100% of 150%",
                                            icon: "volume_up",
                                            value: 2 / 3,
                                            sticky: 2 / 3,
                                            number: "100%",
                                            widest: "150%"
                                        },
                                        {
                                            label: "Focused",
                                            icon: "volume_up",
                                            value: 0.3,
                                            focused: true
                                        },
                                        {
                                            label: "Disabled",
                                            icon: "volume_up",
                                            value: 0.3,
                                            enabled: false
                                        }
                                    ]

                                    K.Stack {
                                        id: slid

                                        required property var modelData

                                        gap: K.Tokens.space.s8

                                        K.Slider {
                                            size: sizes.modelData
                                            icon: slid.modelData.icon
                                            value: slid.modelData.value ?? 0
                                            stops: slid.modelData.stops ?? 0
                                            index: slid.modelData.index ?? 0
                                            names: slid.modelData.names ?? []
                                            words: slid.modelData.words ?? ""
                                            number: slid.modelData.number ?? ""
                                            widest: slid.modelData.widest ?? "100%"
                                            soft: slid.modelData.soft ?? false
                                            sticky: slid.modelData.sticky ?? -1
                                            iconButton: true
                                            focused: slid.modelData.focused ?? false
                                            enabled: slid.modelData.enabled ?? true
                                            preview: slid.modelData.preview ?? ""
                                        }

                                        K.Text {
                                            text: slid.modelData.label
                                            style: K.Tokens.type.small
                                            color: K.Tokens.color.on_surface_variant
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                K.Section {
                    title: "Tabs"
                    description: "One of a few views or values, picked from small buttons: the picked one tonal. The same row is a setting's segmented control."

                    K.Row {
                        gap: K.Tokens.space.s32
                        align: "centre"

                        K.Tabs {
                            labels: ["Wi-Fi", "Bluetooth", "Output", "Keyboard"]
                            currentIndex: 0
                        }

                        K.Tabs {
                            labels: ["Timer", "Alarm"]
                            icons: ["timer", "alarm"]
                            currentIndex: 1
                            focused: true
                        }

                        K.Tabs {
                            labels: ["Always", "On AC", "Never"]
                            currentIndex: 1
                        }
                    }
                }

                K.Section {
                    title: "Tiles"
                    description: "Quick toggles as tall as a two-line row, filled with the accent while on. The detail arrow stays quiet: no fill, a state layer under the pointer."

                    K.Grid {
                        columns: 4
                        gap: K.Tokens.space.s16

                        K.Tile {
                            icon: "signal_wifi_4_bar"
                            title: "Wi-Fi"
                            subtitle: "Shinjuku-5G"
                            checked: true
                        }

                        K.Tile {
                            icon: "bluetooth"
                            title: "Bluetooth"
                            subtitle: "Off"
                        }

                        K.Tile {
                            icon: "keyboard"
                            title: "Keyboard"
                            subtitle: "US · 1 of 2"
                            checked: true
                            expandable: true
                        }

                        K.Tile {
                            icon: "key"
                            title: "Proxy"
                            subtitle: "Off"
                            expandable: true
                            focused: true
                        }
                    }
                }

                K.Section {
                    title: "Icon tiles and badges"
                    description: "Every row's icon on one tile, a symbol until a picture loads, so rows read the same size: small and regular, an app's icon, a photo, a critical one. Then badges, a count or a dot over an icon's corner."

                    K.Row {
                        gap: K.Tokens.space.s24
                        align: "centre"

                        K.IconTile {
                            icon: "terminal"
                            size: "Sm"
                        }

                        K.IconTile {
                            icon: "terminal"
                        }

                        K.IconTile {
                            photo: root.picture
                        }

                        K.IconTile {
                            icon: "warning"
                            danger: true
                        }

                        K.Badge {
                            count: 3
                        }

                        K.Badge {
                            count: 128
                        }

                        K.Badge {
                        }

                        Badged {
                            icon: "notifications"
                            count: 3
                        }

                        Badged {
                            icon: "system_update_alt"
                        }
                    }
                }

                K.Section {
                    title: "Skeletons"
                    description: "The shape of what is loading, in the placeholder colour, so what arrives moves nothing: a row being scanned, a thumbnail being decoded."

                    K.Grid {
                        columns: 2
                        gap: K.Tokens.space.s24

                        K.Row {
                            gap: K.Tokens.space.s12
                            align: "centre"

                            K.Skeleton {
                                bone: "circle"
                                size: K.Tokens.icon.s24
                            }

                            K.Stack {
                                fill: true
                                gap: K.Tokens.space.s4

                                K.Skeleton {
                                    style: K.Tokens.type.body_strong
                                }

                                K.Skeleton {
                                    style: K.Tokens.type.small
                                }
                            }
                        }

                        K.Skeleton {
                            bone: "block"
                            size: K.Tokens.size.control_lg
                        }
                    }
                }

                K.Section {
                    title: "Lists"
                    description: "Rows in a well: a surface, 24, with 12 inside, so the rows take 24 − 12 = 12, the control radius. The focus tube lights the well's edge; the row shows where the keyboard is."

                    K.Grid {
                        columns: 3
                        gap: K.Tokens.space.s24

                        K.List {
                            K.ListRow {
                                title: "English (US)"
                                chips: [{ kind: "Accent", text: "Default" }]
                                detail: "us"
                                checkable: true
                                checked: true
                            }

                            K.ListRow {
                                title: "Italian"
                                detail: "it"
                                checkable: true
                                selected: true
                            }

                            K.ListRow {
                                title: "German"
                                detail: "de"
                                checkable: true
                            }
                        }

                        K.List {
                            focused: true

                            K.ListRow {
                                title: "Home-5G"
                                subtitle: "Secured · strong"
                                icon: "wifi"
                                selected: true
                            }

                            K.ListRow {
                                title: "FRITZ!Box 7590 XL Gästenetz OG"
                                subtitle: "Secured · fair"
                                icon: "wifi"
                            }

                            K.ListRow {
                                title: "Samsung SSD 990 PRO"
                                subtitle: "nvme0n1 · 2 TB · Linux"
                                icon: "hard_drive"
                                detail: "2 TB"
                            }
                        }

                        K.EmptyList {
                            text: "Nothing matches. Typo?"
                        }

                        K.List {
                            K.ListRow {
                                title: "Reboot"
                                icon: "restart_alt"
                            }

                            K.ListRow {
                                title: "Shut down"
                                icon: "power_settings_new"
                                danger: true
                                selected: true
                            }
                        }

                        K.List {
                            K.ListRow {
                                title: "English (US)"
                                subtitle: "us"
                                icon: "star_filled"
                                tile: true
                                accent: true
                                checkable: true
                            }

                            K.ListRow {
                                title: "Italian"
                                subtitle: "it"
                                icon: "keyboard"
                                tile: true
                                checkable: true
                                checked: true
                                selected: true
                            }
                        }
                    }
                }

                K.Section {
                    title: "Chips"

                    K.Row {
                        gap: K.Tokens.space.s8
                        align: "centre"

                        K.Chip {
                            text: "2 TB"
                        }

                        K.Chip {
                            text: "English (US)"
                            kind: "Selected"
                        }

                        K.Chip {
                            text: "Recommended"
                            kind: "Accent"
                        }

                        K.Chip {
                            text: "Demo"
                            kind: "Cold"
                        }

                        K.Chip {
                            text: "Reboot"
                            kind: "Filled"
                        }

                        K.Chip {
                            text: "Default"
                            kind: "Selected"
                            icon: "check"
                        }

                        K.Chip {
                            text: "Demo"
                            kind: "Cold"
                            icon: "bolt"
                        }

                        K.Chip {
                            text: "Italian"
                            pressable: true
                            removable: true
                        }

                        K.Chip {
                            text: "English (US)"
                            kind: "Selected"
                            icon: "check"
                            removable: true
                        }
                    }
                }

                K.Section {
                    title: "Keycaps and key hints"
                    description: "Keys drawn as keys, as tall as a line of body text so they sit inline with it, then what each does."

                    K.Stack {
                        gap: K.Tokens.space.s16

                        K.Row {
                            gap: K.Tokens.space.s8
                            align: "centre"

                            Repeater {
                                model: ["Esc", "Tab", "Enter", "↵", "K", "Ctrl", "← →", "+"]

                                K.Keycap {
                                    required property string modelData

                                    key: modelData
                                }
                            }
                        }

                        K.KeyHints {
                            hints: [["Enter", "continue"], ["Ctrl+K", "actions"], ["Esc", "back"]]
                        }

                        K.KeyHints {
                            text: "↵ Open · Tab Enter app · Ctrl+K Actions"
                        }

                        K.Row {
                            gap: K.Tokens.space.s4
                            align: "centre"

                            K.Text {
                                text: "Press"
                                style: K.Tokens.type.body
                            }

                            K.Keycap {
                                key: "Space"
                            }

                            K.Text {
                                text: "to pick a layout, or"
                                style: K.Tokens.type.body
                            }

                            K.Keycap {
                                key: "Del"
                            }

                            K.Text {
                                text: "to drop it."
                                style: K.Tokens.type.body
                            }
                        }
                    }
                }

                K.Section {
                    title: "Tooltips"
                    description: "What a control does, on a popover 8 above it while the pointer or the keyboard is on it; drawn here in place, over the chip it explains."

                    K.Stack {
                        gap: K.Tokens.space.s8

                        K.Tooltip {
                            floating: false
                            title: "Mac accents on Left Alt"
                            body: "Type accents as a Mac does: hold Left Alt with the accent's key, then type the letter."

                            K.Stack {
                                gap: K.Tokens.space.s8

                                K.KeyHints {
                                    hints: [["Left Alt+E", "then"], ["E", "é"]]
                                }

                                K.KeyHints {
                                    hints: [["Left Alt+U", "then"], ["O", "ö"]]
                                }
                            }
                        }

                        K.Chip {
                            text: "Mac accents"
                            kind: "Selected"
                            icon: "keyboard_option_key"
                            pressable: true
                        }
                    }
                }

                K.Section {
                    title: "Cards"
                    description: "Picked cards take the accent, so their focus tube is the cold one."

                    K.Grid {
                        columns: 4
                        gap: K.Tokens.space.s24

                        K.Card {
                            CardText {
                                title: "Plain card"
                                body: "The well's surface and a soft hairline, 16 inside."
                            }
                        }

                        K.ChoiceCard {
                            checked: true

                            CardText {
                                title: "Erase the disk"
                                body: "Everything on it goes."
                            }
                        }

                        K.ChoiceCard {
                            CardText {
                                title: "Install alongside"
                                body: "Keep Windows, shrink it."
                            }
                        }

                        K.ChoiceCard {
                            checked: true
                            focused: true

                            CardText {
                                title: "Focused, picked"
                                body: "The cold tube."
                            }
                        }
                    }
                }

                K.Section {
                    title: "Sheets"
                    description: "A panel growing out of the frame's edge, one surface with it: concave fillets where it meets the edge, the surface radius at its far corners."

                    Item {
                        id: stage

                        readonly property bool fillWidth: true
                        readonly property color frameColour: K.Look.alpha(K.Tokens.color.surface, K.Tokens.opacity.frame)

                        implicitHeight: K.Tokens.space.s8 + sheet.contentHeight + K.Tokens.space.s32

                        Rectangle {
                            anchors.fill: parent
                            radius: K.Tokens.radius.surface
                            color: K.Look.alpha(K.Tokens.color.surface_container_highest, K.Tokens.opacity.well)
                        }

                        // The frame's top edge, which the sheet hangs from.
                        Rectangle {
                            width: stage.width
                            height: K.Tokens.space.s8
                            topLeftRadius: K.Tokens.radius.surface
                            topRightRadius: K.Tokens.radius.surface
                            color: stage.frameColour
                        }

                        Shape {
                            anchors.fill: parent
                            preferredRendererType: Shape.CurveRenderer

                            ShapePath {
                                fillColor: stage.frameColour
                                // design-allow(size): -1 is ShapePath's "no stroke", not a width.
                                strokeWidth: -1

                                PathSvg {
                                    path: sheet.path
                                }
                            }
                        }

                        K.Sheet {
                            id: sheet

                            frame: QtObject {
                                readonly property real l: 0
                                readonly property real t: K.Tokens.space.s8
                                readonly property real r: stage.width
                                readonly property real b: stage.height
                            }
                            // Shown open, without the motion that opens it.
                            progress: 1
                            takesFocus: false
                            animateSize: false
                            contentWidth: sheetBody.implicitWidth
                            contentHeight: sheetBody.implicitHeight

                            K.Stack {
                                id: sheetBody

                                gap: K.Tokens.space.s4
                                leftPadding: K.Tokens.space.s16
                                rightPadding: K.Tokens.space.s16
                                topPadding: K.Tokens.space.s16
                                bottomPadding: K.Tokens.space.s16

                                K.Text {
                                    text: "Control center"
                                    style: K.Tokens.type.subheading
                                }

                                K.Text {
                                    text: "It grows out of the frame's edge."
                                    style: K.Tokens.type.small
                                    color: K.Tokens.color.on_surface_variant
                                }
                            }
                        }
                    }
                }

                K.Section {
                    title: "Calendars"
                    description: "A month as six weeks led by their week numbers, today filled and the picked day ringed; the title opens the year."

                    K.Row {
                        gap: K.Tokens.space.s48

                        K.Calendar {
                            month: root.october
                            selected: "2026-10-30"
                        }

                        K.Calendar {
                            month: root.october
                            yearView: true
                        }
                    }
                }

                K.Section {
                    title: "Media"
                    description: "Pictures in a box of a fixed aspect, reserved before they load, so nothing moves when they arrive: the placeholder, the picture, then two pictures to pick, picked with its name under it and focused with its name over its foot."

                    K.Grid {
                        columns: 4
                        gap: K.Tokens.space.s16

                        K.Media {
                        }

                        K.Media {
                            source: root.picture
                        }

                        K.PictureChoice {
                            source: root.picture
                            name: "Shinjuku, night"
                            picked: true
                            markIcon: "check"
                            markText: "Current"
                        }

                        K.PictureChoice {
                            source: root.picture
                            name: "Golden Gai"
                            namePlace: "over"
                            badgeIcon: "play_arrow"
                            badgeText: "Video"
                            focused: true
                        }
                    }
                }

                K.Section {
                    title: "Banners"
                    description: "A message that matters on the page itself, its actions inside: info, a warning, and danger, really red, holding the button that destroys."

                    K.Stack {
                        fill: true
                        gap: K.Tokens.space.s16

                        K.Banner {
                            tone: "Info"
                            icon: "storage"
                            title: "KubaOS goes into the free space on Samsung SSD 990 PRO."
                            body: "Windows stays where it is."
                        }

                        K.Banner {
                            tone: "Warning"
                            icon: "warning"
                            title: "12% left. Plug the damn thing in."
                            body: "An install on battery may not finish."
                        }

                        K.Banner {
                            tone: "Danger"
                            icon: "warning"
                            title: "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased."
                            body: "All 4 partitions, Windows included. There's no undo."
                            foot: [
                                K.ProgressBar {
                                    value: 0.4
                                    danger: true
                                }
                            ]

                            K.Button {
                                text: "Back"
                                kind: "Text"
                                icon: "arrow_back"
                            }

                            K.Stack {
                                gap: K.Tokens.space.s8

                                K.Button {
                                    text: "Hold to erase and install"
                                    kind: "Danger"
                                    icon: "download"
                                }

                                K.Row {
                                    gap: K.Tokens.space.s4
                                    align: "centre"

                                    K.Text {
                                        text: "or hold"
                                        style: K.Tokens.type.caption
                                        color: K.Look.alpha(K.Tokens.color.on_error_container, K.Tokens.opacity.subdued_text)
                                    }

                                    K.Keycap {
                                        key: "Enter"
                                    }
                                }
                            }
                        }
                    }
                }

                K.Section {
                    title: "Steps"
                    description: "To do, done and waiting on an action each look their own, and the action carries the accent. A description wraps; a rail marks the state instead."

                    K.Grid {
                        weights: [2, 1]
                        gap: K.Tokens.space.s24

                        K.List {
                            focused: true

                            K.Step {
                                title: "TUXEDO drivers"
                                subtitle: "Key queued. Reboot and enroll it on the blue MokManager screen, before KubaOS starts."
                                icon: "computer"
                                status: "action"
                                actionLabel: "Reboot"
                                actionIcon: "restart_alt"
                                selected: true
                            }

                            K.Step {
                                title: "Fingerprint"
                                subtitle: "One finger: right index. sudo and password dialogs take it."
                                icon: "fingerprint"
                                status: "done"
                            }

                            K.Step {
                                title: "Bitwarden"
                                subtitle: "Not logged in."
                                icon: "lock"
                            }
                        }

                        K.Stack {
                            K.Step {
                                title: "Language"
                                icon: "language"
                                status: "done"
                                mark: true
                            }

                            K.Step {
                                title: "Disk"
                                icon: "hard_drive"
                                status: "current"
                                mark: true
                                enabled: false
                            }

                            K.Step {
                                title: "Git"
                                icon: "code"
                                mark: true
                                enabled: false
                            }

                            K.Step {
                                title: "Reboot"
                                icon: "computer"
                                status: "action"
                                actionLabel: "Reboot"
                                actionIcon: "restart_alt"
                                mark: true
                                enabled: false
                            }
                        }
                    }
                }

                K.Section {
                    title: "Description lists"
                    description: "Facts beside their labels, a quarter of the width; a path or an email gets a mono line of its own."

                    K.Grid {
                        columns: 2
                        gap: K.Tokens.space.s24

                        K.Card {
                            K.DescriptionList {
                                facts: [
                                    {
                                        label: "Name",
                                        value: "Kuba Example"
                                    },
                                    {
                                        label: "Login",
                                        value: "kuba@tuxedo"
                                    },
                                    {
                                        label: "Git",
                                        value: "Kuba Example",
                                        sub: "kuba@example.com"
                                    }
                                ]
                            }
                        }

                        K.Card {
                            K.DescriptionList {
                                facts: [
                                    {
                                        label: "Disk",
                                        value: "Samsung SSD 990 PRO 2TB",
                                        sub: "/dev/nvme0n1 · 2.0 TB"
                                    },
                                    {
                                        label: "Mode",
                                        value: "Erase everything"
                                    },
                                    {
                                        label: "Encryption",
                                        value: "LUKS2, account password"
                                    }
                                ]
                            }
                        }
                    }
                }

                K.Section {
                    title: "Dialogs"
                    description: "A question, the confirmation before something is destroyed, and a prompt for what only you can type."

                    K.Grid {
                        columns: 2
                        gap: K.Tokens.space.s24

                        K.Scrim {
                            implicitHeight: K.Tokens.size.column / 2 + K.Tokens.space.s64

                            K.Dialog {
                                // A picture of a dialog: it leaves the keyboard where it is.
                                takesFocus: false
                                icon: "power_settings_new"
                                title: "Restart now?"
                                body: "Everything open closes. I'll be back in a few seconds."
                                dismiss: K.Button {
                                    text: "Cancel"
                                    kind: "Text"
                                }
                                actions: [
                                    K.Button {
                                        text: "Shut down"
                                        kind: "Tonal"
                                        icon: "power_settings_new"
                                    },
                                    K.Button {
                                        text: "Restart"
                                        icon: "restart_alt"
                                        focused: true
                                    }
                                ]
                            }
                        }

                        K.Scrim {
                            implicitHeight: K.Tokens.size.column / 2 + K.Tokens.space.s64

                            K.Dialog {
                                // A picture of a dialog: it leaves the keyboard where it is.
                                takesFocus: false
                                tone: "Danger"
                                icon: "warning"
                                title: "Last chance"
                                body: "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased. There's no undo."
                                dismiss: K.Button {
                                    text: "Cancel"
                                    kind: "Text"
                                    focused: true
                                }
                                actions: [
                                    K.Button {
                                        text: "Erase and install"
                                        kind: "Danger"
                                        icon: "warning"
                                    }
                                ]
                            }
                        }

                        K.Scrim {
                            implicitHeight: K.Tokens.size.column / 2 + K.Tokens.space.s64

                            K.Dialog {
                                // A picture of a dialog: it leaves the keyboard where it is.
                                takesFocus: false
                                icon: "admin_panel_settings"
                                title: "Authentication required"
                                body: "Installing btop needs your password."
                                dismiss: K.Button {
                                    text: "Cancel"
                                    kind: "Text"
                                }
                                actions: [
                                    K.Button {
                                        text: "Authenticate"
                                    }
                                ]

                                K.Labelled {
                                    label: "Password for Kuba"
                                    note: "That didn't work. Fat fingers? Try again."
                                    invalid: true

                                    K.Field {
                                        text: "hunter"
                                        icon: "key"
                                        secret: true
                                        status: "Invalid"
                                    }
                                }
                            }
                        }
                    }
                }

                K.Section {
                    title: "Toasts and progress"

                    K.Stack {
                        gap: K.Tokens.space.s32

                        K.Row {
                            gap: K.Tokens.space.s16
                            align: "centre"

                            K.Toast {
                                text: "Copied. Evidence secured."
                            }

                            K.Toast {
                                text: "Replay saved."
                                icon: "check_circle"
                            }
                        }

                        K.Row {
                            gap: K.Tokens.space.s48

                            K.Stack {
                                fill: true
                                gap: K.Tokens.space.s16

                                K.Stack {
                                    gap: K.Tokens.space.s16

                                    K.ProgressBar {
                                        value: 0
                                    }

                                    K.ProgressBar {
                                        value: 0.42
                                    }

                                    K.ProgressBar {
                                        value: 1
                                    }
                                }

                                K.Row {
                                    gap: K.Tokens.space.s24
                                    align: "centre"

                                    Repeater {
                                        model: [K.Tokens.icon.s16, K.Tokens.icon.s20, K.Tokens.icon.s24]

                                        K.Spinner {
                                            required property int modelData

                                            size: modelData
                                            running: false
                                            seconds: 0.7
                                        }
                                    }

                                    K.Text {
                                        text: "Poking at the hardware…"
                                        color: K.Tokens.color.on_surface_variant
                                    }
                                }
                            }

                            K.Stack {
                                fill: true

                                K.Stack {
                                    gap: K.Tokens.space.s12

                                    K.Row {
                                        gap: K.Tokens.space.s12
                                        align: "end"

                                        K.Text {
                                            text: "Installing"
                                            style: K.Tokens.type.title
                                        }

                                        K.Fill {
                                        }

                                        K.Text {
                                            text: "42%"
                                            style: K.Tokens.type.hero_number
                                        }
                                    }

                                    K.ProgressBar {
                                        value: 0.42
                                    }

                                    K.Row {
                                        gap: K.Tokens.space.s12
                                        align: "centre"

                                        K.Text {
                                            text: "Unpacking the system"
                                            style: K.Tokens.type.body_strong
                                        }

                                        K.Text {
                                            text: "1.8 / 4.2 GB"
                                            style: K.Tokens.type.mono
                                            color: K.Tokens.color.primary
                                        }

                                        K.Fill {
                                        }

                                        K.Text {
                                            text: "01:42"
                                            style: K.Tokens.type.mono
                                            color: K.Tokens.color.on_surface_variant
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                K.Section {
                    title: "Meters and sparklines"
                    description: "Readings at a glance: a reading in parts on a progress bar's track, and the last minute of one as a line."

                    K.Row {
                        gap: K.Tokens.space.s48

                        K.Stack {
                            fill: true
                            gap: K.Tokens.space.s16

                            K.Meter {
                                parts: [
                                    {
                                        value: 0.55,
                                        color: K.Tokens.color.primary
                                    },
                                    {
                                        value: 0.15,
                                        color: K.Tokens.color.tertiary
                                    }
                                ]
                            }

                            K.Meter {
                                parts: [
                                    {
                                        value: 0.92,
                                        color: K.Tokens.color.warning
                                    }
                                ]
                            }

                            K.Meter {
                                parts: [
                                    {
                                        value: 0.3,
                                        color: K.Tokens.color.secondary
                                    }
                                ]
                            }
                        }

                        K.Stack {
                            fill: true
                            gap: K.Tokens.space.s16

                            // The waves' 30 samples, the first at the left edge and the last at
                            // the right, as the iced page spaces them.
                            K.Sparkline {
                                points: root.wave(40, 20, 0)
                                span: 58000
                            }

                            K.Sparkline {
                                points: root.wave(1200, 600, 1)
                                second: root.wave(300, 150, 2)
                                span: 58000
                                max: 0
                            }
                        }
                    }
                }

                K.Section {
                    title: "Empty states"

                    K.Grid {
                        columns: 2
                        gap: K.Tokens.space.s24

                        K.Card {
                            K.EmptyState {
                                icon: "wifi"
                                title: "No networks"
                                body: "Nothing in range. Plug in a cable, or skip it."
                                action: Component {
                                    K.Button {
                                        text: "Skip"
                                        kind: "Tonal"
                                    }
                                }
                            }
                        }

                        K.Card {
                            K.EmptyState {
                                icon: "search"
                                title: "Nothing matches"
                                body: "Typo? Try fewer letters."
                            }
                        }
                    }
                }

                K.Section {
                    title: "Selects and menus"
                    description: "A popover floats over the page: the surface radius, 12 inside, its items small controls. A select drops one from a field-like control."

                    K.Grid {
                        columns: 3
                        gap: K.Tokens.space.s24

                        K.Stack {
                            fill: true
                            gap: K.Tokens.space.s8

                            K.Select {
                                options: ["None", "Germany", "Italy", "Poland"]
                                currentIndex: 3
                                icon: "flag"
                                open: true
                            }

                            K.Popover {
                                fillWidth: true

                                K.Menu {
                                    currentIndex: 2
                                    entries: ["None", "Germany", "Italy", "Poland"].map(name => ({
                                                text: name,
                                                checkable: true,
                                                checked: name === "Poland"
                                            }))
                                }
                            }
                        }

                        K.Popover {
                            fixedWidth: K.Tokens.size.menu

                            K.Menu {
                                currentIndex: 0
                                entries: [
                                    {
                                        text: "Open",
                                        icon: "folder_open",
                                        shortcut: ["Enter"]
                                    },
                                    {
                                        text: "Copy path",
                                        icon: "content_paste",
                                        shortcut: ["Ctrl", "C"]
                                    },
                                    {
                                        separator: true
                                    },
                                    {
                                        text: "Pin to the top",
                                        icon: "push_pin"
                                    },
                                    {
                                        text: "Forget",
                                        icon: "close",
                                        enabled: false
                                    },
                                    {
                                        separator: true
                                    },
                                    {
                                        text: "Quit",
                                        icon: "logout",
                                        danger: true
                                    }
                                ]
                            }
                        }

                        K.Stack {
                            fill: true
                            gap: K.Tokens.space.s16

                            K.Select {
                                options: ["Every 5 minutes"]
                                currentIndex: 0
                            }

                            K.Select {
                                options: ["Every 5 minutes"]
                                currentIndex: 0
                                focused: true
                            }

                            K.Select {
                                options: ["Unavailable"]
                                currentIndex: 0
                                enabled: false
                            }
                        }
                    }
                }

                K.Section {
                    title: "Tables"
                    description: "Readings in columns, a line of text with 4 above and below a row; the whole row takes the pointer, numbers line up at their column's right edge."

                    K.Table {
                        columns: [
                            {
                                title: "Name",
                                weight: 4
                            },
                            {
                                title: "CPU",
                                weight: 1,
                                numeric: true
                            },
                            {
                                title: "Memory",
                                weight: 1,
                                numeric: true
                            }
                        ]
                        rows: [
                            {
                                cells: ["firefox", "38.4%", "2.1 GB"]
                            },
                            {
                                cells: ["Hyprland", "12.1%", "384 MB"],
                                selected: true
                            },
                            {
                                cells: ["rust-analyzer", "9.7%", "1.8 GB"]
                            },
                            {
                                cells: ["quickshell", "4.2%", "256 MB"]
                            }
                        ]
                    }
                }

                K.Section {
                    title: "Collapsibles"
                    description: "A titled part that opens to show more."

                    K.Grid {
                        columns: 2
                        gap: K.Tokens.space.s24

                        K.Collapsible {
                            title: "firefox · pid 4127"
                            open: true

                            K.Stack {
                                gap: K.Tokens.space.s4

                                K.Text {
                                    text: "/usr/lib/firefox/firefox -contentproc"
                                    style: K.Tokens.type.mono
                                    color: K.Tokens.color.on_surface_variant
                                }

                                K.Text {
                                    text: "kuba · sleeping · 92 threads · 2.1 GB"
                                    style: K.Tokens.type.small
                                    color: K.Tokens.color.on_surface_variant
                                }
                            }
                        }

                        K.Collapsible {
                            title: "Hyprland · pid 1203"
                            focused: true

                            K.Text {
                                text: "Hidden while closed."
                                style: K.Tokens.type.small
                                color: K.Tokens.color.on_surface_variant
                            }
                        }
                    }
                }

                K.Section {
                    title: "Errors"
                    description: "What broke, what to do next, and the log behind it."

                    K.Stack {
                        gap: K.Tokens.space.s16

                        K.ErrorBlock {
                            message: "efibootmgr failed to register the boot entry: Operation not permitted."
                            hint: "The firmware refused a new boot entry. Turn off Secure Boot's \"lock boot order\" in the setup menu and try again."
                        }

                        K.ErrorBlock {
                            message: "grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=ubuntu failed with exit status 1."
                            hint: "A message this long drops to body size."
                        }

                        K.Log {
                            lines: [
                                {
                                    level: "step",
                                    text: "==> Bootloader"
                                },
                                {
                                    level: "info",
                                    text: "Installing for x86_64-efi platform."
                                },
                                {
                                    level: "debug",
                                    text: "efibootmgr -c -d /dev/nvme0n1 -p 1 -L ubuntu"
                                },
                                {
                                    level: "warn",
                                    text: "warning: the firmware reports Secure Boot's boot order as locked"
                                },
                                {
                                    level: "error",
                                    text: "grub-install: error: efibootmgr failed to register the boot entry: Operation not permitted."
                                }
                            ]
                        }

                        K.Row {
                            gap: K.Tokens.space.s12
                            align: "centre"

                            K.Button {
                                text: "Back to the review"
                                kind: "Text"
                                icon: "arrow_back"
                            }

                            K.Button {
                                text: "Hide log"
                                kind: "Text"
                                icon: "terminal"
                            }

                            K.Fill {
                            }

                            K.Button {
                                text: "Save log"
                                kind: "Tonal"
                                icon: "save"
                            }

                            K.Button {
                                text: "Try again"
                                icon: "restart_alt"
                                focused: true
                            }
                        }
                    }
                }
            }
        }
    }

    // A button table's first row: an empty corner, then the states' names.
    component StateHeads: Repeater {
        model: [""].concat(root.stateNames)

        K.Text {
            required property string modelData

            text: modelData
            style: K.Tokens.type.label
            color: K.Tokens.color.on_surface_variant
        }
    }

    // A label button of `kind` and `size` in the `state`th of `root.stateNames`.
    component SampleButton: K.Button {
        property int shownState: 0

        text: kind === "Filled" ? "Continue" : kind === "Tonal" ? "Save log" : kind === "Text" ? "Back" : "Erase disk"
        icon: kind === "Filled" ? "arrow_forward" : kind === "Tonal" ? "save" : kind === "Text" ? "arrow_back" : "warning"
        trailing: kind === "Filled"
        preview: shownState === 1 ? "hovered" : shownState === 2 ? "pressed" : ""
        focused: shownState === 3
        enabled: shownState !== 4
    }

    // Every kind of label button of `size`, in every state.
    component ButtonTable: K.Grid {
        id: table

        property string size

        weights: root.columns
        gap: K.Tokens.space.s16
        centred: true

        StateHeads {
        }

        Repeater {
            model: root.kinds.length * (root.stateNames.length + 1)

            Loader {
                required property int index

                readonly property string kind: root.kinds[Math.floor(index / (root.stateNames.length + 1))]
                readonly property int shownState: index % (root.stateNames.length + 1) - 1

                sourceComponent: shownState < 0 ? kindName : sample

                Component {
                    id: kindName

                    K.Text {
                        text: kind
                        style: K.Tokens.type.label
                        color: K.Tokens.color.on_surface_variant
                    }
                }

                Component {
                    id: sample

                    SampleButton {
                        kind: parent.kind
                        size: table.size
                        shownState: parent.shownState
                    }
                }
            }
        }
    }

    // One kind of icon button of `size`, in every state, under the label buttons' columns.
    component IconButtonRow: K.Grid {
        id: line

        property string kind
        property string size

        weights: root.columns
        gap: K.Tokens.space.s16
        centred: true

        K.Text {
            text: line.kind
            style: K.Tokens.type.label
            color: K.Tokens.color.on_surface_variant
        }

        Repeater {
            model: root.stateNames.length

            K.IconButton {
                required property int index

                icon: line.kind === "Filled" ? "arrow_forward" : line.kind === "Tonal" ? "settings" : line.kind === "Text" ? "close" : "power_settings_new"
                label: line.kind === "Filled" ? "Continue" : line.kind === "Tonal" ? "Settings" : line.kind === "Text" ? "Close" : "Shut down"
                kind: line.kind
                size: line.size
                preview: index === 1 ? "hovered" : index === 2 ? "pressed" : ""
                focused: index === 3
                enabled: index !== 4
            }
        }
    }

    // A card's title over a line under it.
    component CardText: K.Stack {
        id: words

        property string title
        property string body

        gap: K.Tokens.space.s4

        K.Text {
            text: words.title
            style: K.Tokens.type.subheading
        }

        K.Text {
            text: words.body
            style: K.Tokens.type.small
            color: K.Tokens.color.on_surface_variant
        }
    }
}
