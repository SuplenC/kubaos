import QtQuick
import qs.theme
import qs.components
import qs.services

// The system monitor in the bar: the metrics the settings pick (CPU and RAM
// by default) as numbers, tiny bars or sparklines, amber close to a limit and
// red past it, with a red drive when a disk is nearly full. Click opens the
// monitor's popup; right-click offers Hide.
Pill {
    id: root

    // Hide is offered in place of the metrics, until the pointer leaves.
    property bool offering: false
    readonly property int graphWidth: Theme.bar.pillHeight + Theme.spacing.sm
    readonly property int graphHeight: Theme.bar.icon - 2

    function tint(level: int): color {
        return level === 2 ? Theme.m3error : level === 1 ? Theme.warning : Theme.m3onSurfaceVariant;
    }

    visible: Sysmon.available && !!Sysmon.settings.pill
    interactive: true
    spacing: Theme.spacing.md
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

    Row {
        visible: root.offering
        anchors.verticalCenter: parent?.verticalCenter
        spacing: Theme.spacing.xs

        Icon {
            anchors.verticalCenter: parent.verticalCenter
            name: "visibility_off"
            size: Theme.bar.icon
            color: Theme.m3onSurfaceVariant
        }

        StyledText {
            anchors.verticalCenter: parent.verticalCenter
            text: "Hide"
            font.weight: Font.Medium
        }
    }

    Repeater {
        model: root.offering ? [] : Sysmon.settings.metrics ?? []

        Row {
            id: entry

            required property string modelData
            readonly property var metric: Sysmon.metric(modelData)
            readonly property color tint: root.tint(metric?.level ?? 0)

            anchors.verticalCenter: parent?.verticalCenter
            spacing: Theme.spacing.xs
            visible: metric !== null

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: entry.metric?.icon ?? ""
                size: Theme.bar.icon
                color: entry.tint
            }

            Meter {
                visible: Sysmon.settings.style === "bars" && entry.metric?.max > 0
                anchors.verticalCenter: parent.verticalCenter
                width: root.graphWidth
                parts: [{ value: entry.metric?.fill ?? 0, color: entry.metric?.level > 0 ? entry.tint : Theme.m3primary }]
            }

            Graph {
                visible: Sysmon.settings.style === "sparklines"
                anchors.verticalCenter: parent.verticalCenter
                width: root.graphWidth
                height: root.graphHeight
                points: Sysmon.history[entry.metric?.series] ?? []
                revision: Sysmon.revision
                max: entry.metric?.max ?? 0
                color: entry.metric?.level > 0 ? entry.tint : Theme.m3primary
                lineWidth: 1
                fillOpacity: 0.25
            }

            StyledText {
                visible: Sysmon.settings.style === "numbers" || entry.metric?.max === 0 && Sysmon.settings.style === "bars"
                anchors.verticalCenter: parent.verticalCenter
                // As wide as "100%" at least, so the bar's neighbours stay put
                // as the numbers change.
                width: Math.max(implicitWidth, widest.advanceWidth)
                horizontalAlignment: Text.AlignRight
                text: entry.metric?.text ?? ""
                color: entry.metric?.level > 0 ? entry.tint : Theme.m3onSurface
                font.weight: Font.Medium
                font.features: Theme.font.tabular
            }
        }
    }

    TextMetrics {
        id: widest

        font.family: Theme.font.sans
        font.pixelSize: Theme.font.md
        font.weight: Font.Medium
        font.features: Theme.font.tabular
        text: "100%"
    }

    Icon {
        visible: !root.offering && Sysmon.fullDisks.length > 0
        anchors.verticalCenter: parent?.verticalCenter
        name: "hard_drive"
        size: Theme.bar.icon
        color: Theme.m3error
    }
}
