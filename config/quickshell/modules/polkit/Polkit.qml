import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Services.Polkit
import qs.theme
import qs.components
import qs.services

// Polkit authentication agent and its dialog. Loaded only when
// Settings.polkitAgent is true; otherwise hyprpolkitagent handles requests.
Scope {
    id: root

    readonly property AuthFlow flow: agent.flow

    PolkitAgent {
        id: agent
    }

    PanelWindow {
        screen: Hypr.focusedScreen
        visible: agent.isActive
        color: "transparent"
        exclusionMode: ExclusionMode.Ignore
        anchors {
            top: true
            bottom: true
            left: true
            right: true
        }
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
        WlrLayershell.namespace: "qs-polkit"
        onVisibleChanged: if (visible) {
            field.text = "";
            field.focusInput();
        }

        Rectangle {
            anchors.fill: parent
            color: Theme.m3scrim
            opacity: 0.45
        }

        Elevation {
            target: dialog
            level: 4
        }

        PanelSurface {
            id: dialog

            anchors.centerIn: parent
            width: 440
            height: column.implicitHeight + Theme.spacing.xl * 2

            ColumnLayout {
                id: column

                anchors.fill: parent
                anchors.margins: Theme.spacing.xl
                spacing: Theme.spacing.md

                Icon {
                    name: "admin_panel_settings"
                    size: 32
                    color: Theme.m3primary
                }

                StyledText {
                    text: "Authentication required"
                    font.pixelSize: Theme.font.xl
                }

                StyledText {
                    Layout.fillWidth: true
                    text: root.flow?.message ?? ""
                    wrapMode: Text.Wrap
                    color: Theme.m3onSurfaceVariant
                }

                StyledText {
                    text: `As ${root.flow?.selectedIdentity?.displayName ?? ""}`
                    font.pixelSize: Theme.font.sm
                    color: Theme.m3outline
                }

                SearchField {
                    id: field

                    Layout.fillWidth: true
                    icon: "key"
                    echoMode: root.flow?.responseVisible ? TextInput.Normal : TextInput.Password
                    placeholder: root.flow?.inputPrompt || "Password"
                    catchEscape: true
                    onEscaped: root.flow?.cancelAuthenticationRequest()
                    onAccepted: {
                        if (!root.flow?.isResponseRequired)
                            return;
                        root.flow.submit(text);
                        text = "";
                    }
                }

                StyledText {
                    Layout.fillWidth: true
                    visible: text !== ""
                    text: root.flow?.failed ? "That didn't work. Fat fingers? Try again" : root.flow?.supplementaryMessage ?? ""
                    wrapMode: Text.Wrap
                    color: root.flow?.failed || root.flow?.supplementaryIsError ? Theme.m3error : Theme.m3onSurfaceVariant
                }

                StyledText {
                    Layout.alignment: Qt.AlignRight
                    text: "Enter authenticate · Esc cancel"
                    color: Theme.m3outline
                    font.pixelSize: Theme.font.sm
                }
            }
        }
    }
}
