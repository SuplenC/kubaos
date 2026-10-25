import QtQuick
import qs.theme
import qs.components
import qs.services
import qs.settings

// The popup's settings page: holidays, first weekday, analog face, the bar's
// countdown and the extra clocks. Each change goes through the engine into
// ~/.config/kubaos/desktop/clock/clock.toml, comments kept, and the state it answers
// with redraws the popup.
Column {
    id: root

    property string error: ""
    readonly property var settings: Time.settings
    readonly property var weekdays: ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"]

    spacing: Theme.spacing.lg

    function set(key: string, value: string): void {
        Time.change("set", { key: key, value: value }, error => root.error = error);
    }

    Section {
        title: "Holidays"
        note: Time.state.country ? `Marked in the calendar and left out of working days: ${Time.state.country}` : "None marked"

        Flow {
            width: parent.width
            spacing: Theme.spacing.xs

            Chip {
                label: "None"
                selected: (root.settings.holidays ?? "") === ""
                onClicked: root.set("holidays", "none")
            }

            Repeater {
                model: Time.state.countries ?? []

                Chip {
                    required property var modelData

                    label: modelData.code.toUpperCase()
                    tooltip: modelData.name
                    selected: root.settings.holidays === modelData.code
                    onClicked: root.set("holidays", modelData.code)
                }
            }
        }
    }

    Section {
        title: "First day of the week"

        Row {
            spacing: Theme.spacing.xs

            Repeater {
                model: root.weekdays

                Chip {
                    required property string modelData
                    required property int index

                    label: modelData.slice(0, 3).replace(/^./, c => c.toUpperCase())
                    selected: root.settings.firstWeekday === index + 1
                    onClicked: root.set("first-weekday", modelData)
                }
            }
        }
    }

    Section {
        title: "Analog face"

        Row {
            spacing: Theme.spacing.xs

            Chip {
                label: "Off"
                selected: !root.settings.analog
                onClicked: root.set("analog", "off")
            }

            Chip {
                label: "On"
                selected: !!root.settings.analog
                onClicked: root.set("analog", "on")
            }
        }
    }

    Section {
        title: "Countdown in the bar"

        Flow {
            width: parent.width
            spacing: Theme.spacing.xs

            Chip {
                label: "None"
                selected: !root.settings.barCountdown
                onClicked: root.set("bar-countdown", "none")
            }

            Repeater {
                model: Time.state.countdowns ?? []

                Chip {
                    required property var modelData

                    label: modelData.name
                    selected: root.settings.barCountdown === modelData.name
                    onClicked: root.set("bar-countdown", modelData.name)
                }
            }
        }
    }

    Section {
        title: "Extra clocks"
        note: "Up to four, by city or IANA name"

        Repeater {
            model: Time.state.clocks ?? []

            Item {
                required property var modelData

                width: parent.width
                height: 28

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: `${modelData.label}  ·  ${modelData.name}`
                }

                IconButton {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    icon: "close"
                    size: 26
                    tooltip: "Remove"
                    onClicked: Time.change("zone-remove", { name: modelData.name }, error => root.error = error)
                }
            }
        }

        SearchField {
            visible: (Time.state.clocks ?? []).length < 4
            width: parent.width
            icon: "add_location_alt"
            placeholder: "Tokyo, New York, America/Chicago"
            catchEscape: true
            onEscaped: text = ""
            onAccepted: Time.change("zone-add", { name: text }, error => {
                root.error = error;
                if (error === "")
                    text = "";
            })
        }
    }

    StyledText {
        width: parent.width
        text: root.error !== "" ? root.error : Time.problem !== "" ? Time.problem : `Kept in ${UserLayer.shown("clock", "clock.toml")}`
        color: root.error !== "" || Time.problem !== "" ? Theme.m3error : Theme.m3outline
        font.pixelSize: Theme.font.sm
        wrapMode: Text.Wrap
    }

    component Section: Column {
        property string title
        property string note

        width: root.width
        spacing: Theme.spacing.sm

        StyledText {
            text: parent.title
            font.weight: Font.DemiBold
            color: Theme.m3onSurface
        }

        StyledText {
            visible: parent.note !== ""
            width: parent.width
            text: parent.note
            font.pixelSize: Theme.font.sm
            color: Theme.m3onSurfaceVariant
        }
    }
}
