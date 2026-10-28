import QtQuick
import KubaOS.Components as K
import qs.services
import qs.settings

// The popup's settings page: holidays, first weekday, time format, date order,
// analog face, the bar's countdown and the extra clocks. Each change goes through the engine into
// ~/.config/kubaos/desktop/clock/clock.toml, comments kept, and the state it answers
// with redraws the popup.
K.Stack {
    id: root

    property string error: ""
    readonly property var settings: Time.settings
    readonly property var weekdays: ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"]

    fill: true
    gap: K.Tokens.space.s16

    function set(key: string, value: string): void {
        Time.change("set", { key: key, value: value }, error => root.error = error);
    }

    // One country of many: a select, where every country's chip made a wall.
    Section {
        title: "Holidays"
        note: "Marked in the calendar and left out of working days"

        K.Select {
            readonly property var countries: Time.state.countries ?? []

            options: ["None", ...countries.map(country => country.name)]
            currentIndex: (root.settings.holidays ?? "") === "" ? 0 : countries.findIndex(country => country.code === root.settings.holidays) + 1
            icon: "flag"
            onPicked: index => root.set("holidays", index === 0 ? "none" : countries[index - 1].code)
        }
    }

    Section {
        title: "First day of the week"

        K.Row {
            gap: K.Tokens.space.s4

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
        title: "Time format"
        note: "How the bar, the lock screen and this popup write the time"

        K.Row {
            gap: K.Tokens.space.s4

            Chip {
                label: "Locale"
                tooltip: `Your locale's: ${Time.twelveHourByLocale ? "12-hour" : "24-hour"}`
                selected: !root.settings.timeFormat
                onClicked: root.set("time-format", "locale")
            }

            Chip {
                label: "24-hour"
                selected: root.settings.timeFormat === "24h"
                onClicked: root.set("time-format", "24h")
            }

            Chip {
                label: "12-hour"
                selected: root.settings.timeFormat === "12h"
                onClicked: root.set("time-format", "12h")
            }
        }
    }

    Section {
        title: "Date order"

        K.Row {
            gap: K.Tokens.space.s4

            Chip {
                label: "Locale"
                tooltip: `Your locale's: ${Time.dayFirstByLocale ? "day first" : "month first"}`
                selected: !root.settings.dateOrder
                onClicked: root.set("date-order", "locale")
            }

            Chip {
                label: "Day first"
                selected: root.settings.dateOrder === "day-month"
                onClicked: root.set("date-order", "day-month")
            }

            Chip {
                label: "Month first"
                selected: root.settings.dateOrder === "month-day"
                onClicked: root.set("date-order", "month-day")
            }
        }
    }

    Section {
        title: "Analog face"

        K.Row {
            gap: K.Tokens.space.s4

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
            readonly property bool fillWidth: true

            spacing: K.Tokens.space.s4

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

            K.Row {
                required property var modelData

                fill: true
                align: "centre"

                K.Text {
                    text: `${modelData.label}  ·  ${modelData.name}`
                    style: K.Tokens.type.body
                }

                K.Fill {}

                K.IconButton {
                    icon: "close"
                    label: qsTr("Remove")
                    kind: "Text"
                    size: "Sm"
                    onActivated: Time.change("zone-remove", {
                        name: modelData.name
                    }, error => root.error = error)
                }
            }
        }

        K.Field {
            id: zoneField

            visible: (Time.state.clocks ?? []).length < 4
            icon: "add_location_alt"
            placeholder: "Tokyo, New York, America/Chicago"
            Keys.onEscapePressed: zoneField.text = ""
            Keys.onReturnPressed: root.addZone()
            Keys.onEnterPressed: root.addZone()
        }
    }

    K.Text {
        text: root.error !== "" ? root.error : Time.problem !== "" ? Time.problem : `Kept in ${UserLayer.shown("clock", "clock.toml")}`
        style: K.Tokens.type.small
        color: root.error !== "" || Time.problem !== "" ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
    }

    function addZone(): void {
        Time.change("zone-add", {
            name: zoneField.text
        }, error => {
            root.error = error;
            if (error === "")
                zoneField.text = "";
        });
    }

    // A setting: its title, an optional note under it, and its choices.
    component Section: K.Stack {
        id: section

        property string title
        property string note
        default property alias choices: choices.data

        fill: true
        gap: K.Tokens.space.s8

        K.Stack {
            fill: true
            gap: K.Tokens.space.s4

            K.Text {
                text: section.title
                style: K.Tokens.type.body_strong
            }

            K.Text {
                visible: section.note !== ""
                text: section.note
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }
        }

        K.Stack {
            id: choices

            fill: true
            gap: K.Tokens.space.s8
        }
    }
}
