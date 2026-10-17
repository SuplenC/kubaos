import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// Paired and discovered Bluetooth devices. Enter connects, disconnects or pairs.
ColumnLayout {
    spacing: Theme.spacing.sm

    RowLayout {
        Layout.fillWidth: true

        StyledText {
            Layout.fillWidth: true
            text: Bt.scanning ? "Searching for devices…" : `${Bt.devices.length} devices`
            color: Theme.m3onSurfaceVariant
        }

        IconButton {
            icon: "radar"
            toggled: Bt.scanning
            enabled: Bt.enabled
            onClicked: Bt.setScanning(!Bt.scanning)
        }
    }

    NavList {
        id: list

        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: count > 0
        model: ScriptModel {
            values: Bt.devices
        }
        highlightShown: activeFocus
        onActivated: index => Bt.activate(Bt.devices[index])

        delegate: ListRow {
            required property var modelData
            required property int index

            width: ListView.view.width
            icon: modelData.connected ? "bluetooth_connected" : "bluetooth"
            title: modelData.name
            subtitle: Bt.stateText(modelData)
            active: modelData.connected
            current: ListView.isCurrentItem && ListView.view.activeFocus
            onClicked: Bt.activate(modelData)
        }
    }

    Item {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: list.count === 0

        EmptyState {
            anchors.centerIn: parent
            icon: Bt.enabled ? "bluetooth_searching" : "bluetooth_disabled"
            text: !Bt.available ? "No Bluetooth adapter" : Bt.enabled ? "No devices" : "Bluetooth is off"
        }
    }
}
