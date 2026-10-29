import QtQuick
import KubaOS.Components as K
import qs.services

// The system monitor in the bar: the metrics the settings pick (CPU and RAM by default) as
// numbers, tiny bars or sparklines, the warning colour close to a limit and the error colour past
// it, with a red drive when a disk is nearly full. Click opens the monitor's panel; right-click
// offers Hide.
K.Pill {
    id: root

    // Hide is offered in place of the metrics, until the pointer leaves.
    property bool offering: false

    function tint(level: int): color {
        return level === 2 ? K.Tokens.color.error : level === 1 ? K.Tokens.color.warning : K.Tokens.color.on_surface_variant;
    }

    visible: Sysmon.available && !!Sysmon.settings.pill
    interactive: true
    spacing: K.Tokens.space.s12
    tooltip: offering ? "" : [Sysmon.sample?.cpu?.usage != null ? `CPU ${Math.round(Sysmon.sample.cpu.usage)}%` : "", Sysmon.sample?.cpu?.temp != null ? `${Math.round(Sysmon.sample.cpu.temp)} °C` : "", Sysmon.sample?.memory ? `RAM ${Sysmon.bytes(Sysmon.sample.memory.used)} of ${Sysmon.bytes(Sysmon.sample.memory.total)}` : "", ...Sysmon.fullDisks.map(d => `${d.label} ${Math.round(d.percent)}% full`)].filter(Boolean).join("  ·  ")
    onClicked: {
        if (offering) {
            offering = false;
            Sysmon.set("pill", "off", null);
        } else {
            Sysmon.toggle();
        }
    }
    onRightClicked: offering = !offering
    onHoveredChanged: if (!hovered)
        offering = false

    K.Row {
        visible: root.offering
        gap: K.Tokens.space.s4
        align: "centre"

        K.Icon {
            name: "visibility_off"
            size: K.Tokens.icon.s16
            color: K.Tokens.color.on_surface_variant
        }

        K.Text {
            text: "Hide"
            style: K.Tokens.type.label
            color: K.Tokens.color.on_surface
        }
    }

    Repeater {
        model: root.offering ? [] : Sysmon.settings.metrics ?? []

        K.Row {
            id: entry

            required property string modelData
            readonly property var metric: Sysmon.metric(modelData)
            readonly property color tint: root.tint(metric?.level ?? 0)
            readonly property color ink: metric?.level > 0 ? tint : K.Tokens.color.primary

            visible: metric !== null
            gap: K.Tokens.space.s4
            align: "centre"

            K.Icon {
                name: entry.metric?.icon ?? ""
                size: K.Tokens.icon.s16
                color: entry.tint
            }

            K.Meter {
                visible: Sysmon.settings.style === "bars" && entry.metric?.max > 0
                fillWidth: false
                implicitWidth: K.Tokens.size.control_sm
                parts: [
                    {
                        value: entry.metric?.fill ?? 0,
                        color: entry.ink
                    }
                ]
            }

            K.Sparkline {
                visible: Sysmon.settings.style === "sparklines"
                fillWidth: false
                implicitWidth: K.Tokens.size.control_sm
                implicitHeight: K.Tokens.icon.s16
                points: Sysmon.history[entry.metric?.series] ?? []
                revision: Sysmon.revision
                max: entry.metric?.max ?? 0
                color: entry.ink
            }

            // As wide as "100%" at least, so the bar's neighbours stay put as the numbers change.
            Item {
                visible: Sysmon.settings.style === "numbers" || entry.metric?.max === 0 && Sysmon.settings.style === "bars"
                implicitWidth: Math.max(reading.implicitWidth, widest.implicitWidth)
                implicitHeight: reading.implicitHeight

                K.Text {
                    id: reading

                    x: parent.width - reading.width
                    text: entry.metric?.text ?? ""
                    style: K.Tokens.type.mono
                    color: entry.metric?.level > 0 ? entry.tint : K.Tokens.color.on_surface
                }
            }
        }
    }

    K.Icon {
        visible: !root.offering && Sysmon.fullDisks.length > 0
        name: "hard_drive"
        size: K.Tokens.icon.s16
        color: K.Tokens.color.error
    }

    // The widest reading, measured and never drawn.
    K.Text {
        id: widest

        visible: false
        text: "100%"
        style: K.Tokens.type.mono
    }
}
