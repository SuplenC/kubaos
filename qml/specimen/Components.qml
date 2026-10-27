import QtQuick
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

    fill: true

    function sizeName(size: string): string {
        return `${size} · ${K.ButtonMetrics.of(size).height}`;
    }

    // The bar above the page: the pages, then the mode and the scale it is shown at. A toolbar
    // band: its content and the container padding, as the page under it is padded.
    K.Container {
        K.Row {
            gap: K.Tokens.space.s16
            align: "centre"
            fill: true

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

            K.Fill {}

            K.KeyHints {
                hints: [["Tab", "pages"], ["M", "mode"]]
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

    K.Divider {}

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

                                        StateHeads {}
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
                            note: "Focused: the caret, the accent hairline and the ring."

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
                    description: "A progress bar's track filled up to the thumb. One with stops marks those ahead and snaps to them."

                    K.Grid {
                        columns: 5
                        gap: K.Tokens.space.s32

                        Repeater {
                            model: [
                                {
                                    label: "Volume, 62%",
                                    value: 0.62,
                                    stops: 0,
                                    index: 0,
                                    focused: false,
                                    enabled: true
                                },
                                {
                                    label: "Pressed: the fill lights up",
                                    value: 0.62,
                                    stops: 0,
                                    index: 0,
                                    focused: false,
                                    enabled: true,
                                    preview: "pressed"
                                },
                                {
                                    label: "Five stops",
                                    value: 0,
                                    stops: 5,
                                    index: 2,
                                    focused: false,
                                    enabled: true
                                },
                                {
                                    label: "Focused",
                                    value: 0.3,
                                    stops: 0,
                                    index: 0,
                                    focused: true,
                                    enabled: true
                                },
                                {
                                    label: "Disabled",
                                    value: 0.3,
                                    stops: 0,
                                    index: 0,
                                    focused: false,
                                    enabled: false
                                }
                            ]

                            K.Stack {
                                id: slid

                                required property var modelData

                                gap: K.Tokens.space.s8

                                K.Slider {
                                    value: slid.modelData.value
                                    stops: slid.modelData.stops
                                    index: slid.modelData.index
                                    focused: slid.modelData.focused
                                    enabled: slid.modelData.enabled
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

                K.Section {
                    title: "Lists"
                    description: "Rows in a well: a surface, 24, with 12 inside, so the rows take 24 − 12 = 12, the control radius. The ring goes round the well; the row shows where the keyboard is."

                    K.Grid {
                        columns: 3
                        gap: K.Tokens.space.s24

                        K.List {
                            K.ListRow {
                                title: "English (US)"
                                chips: [
                                    {
                                        kind: "Accent",
                                        text: "Default"
                                    }
                                ]
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
                    title: "Cards"
                    description: "Picked cards take the accent, so their focus ring is the cold one."

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
                                body: "The cold ring."
                            }
                        }
                    }
                }

                K.Section {
                    title: "Media"
                    description: "Pictures in a box of a fixed aspect, reserved before they load, so nothing moves when they arrive: the placeholder, then the picture."

                    K.Grid {
                        columns: 4
                        gap: K.Tokens.space.s16

                        K.Media {}

                        K.Media {
                            source: root.picture
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

                                        K.Fill {}

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

                                        K.Fill {}

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

                            K.Fill {}

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

        StateHeads {}

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
