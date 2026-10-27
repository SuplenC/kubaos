import QtQuick
import Quickshell
import Quickshell.Wayland
import Quickshell.Services.Polkit
import KubaOS.Components as K
import qs.services

// Polkit authentication agent and its dialog. Loaded only when
// Settings.polkitAgent is true; otherwise hyprpolkitagent handles requests.
// The keyboard starts in the field; Enter answers, Esc (or Cancel) cancels.
Scope {
    id: root

    readonly property AuthFlow flow: agent.flow
    // What PAM asks for, without the colon it ends its prompt with: "Password".
    readonly property string asked: (root.flow?.inputPrompt ?? "").replace(/:\s*$/, "") || "Password"
    readonly property string who: root.flow?.selectedIdentity?.displayName ?? ""
    // "Password for Kuba" reads; "Verification code for Kuba" doesn't, so any other prompt shows as
    // PAM wrote it, and who it is for goes under the body.
    readonly property bool forWho: root.who !== "" && root.asked.toLowerCase() === "password"
    readonly property bool invalid: (root.flow?.failed ?? false) || (root.flow?.supplementaryIsError ?? false)

    function respond(): void {
        if (!root.flow?.isResponseRequired)
            return;
        root.flow.submit(field.text);
        field.text = "";
        // Authenticate pressed from the keyboard holds the focus; a retry is typed in the field.
        field.forceActiveFocus();
    }

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
            field.forceActiveFocus();
        }

        K.Scrim {
            anchors.fill: parent

            K.Dialog {
                icon: "admin_panel_settings"
                title: "Authentication required"
                body: {
                    const message = root.flow?.message ?? "";
                    return root.forWho || root.who === "" ? message : `${message}\nAs ${root.who}`;
                }
                onDismissed: root.flow?.cancelAuthenticationRequest()
                dismiss: K.Button {
                    text: "Cancel"
                    kind: "Text"
                    onActivated: root.flow?.cancelAuthenticationRequest()
                }
                actions: [
                    K.Button {
                        text: "Authenticate"
                        enabled: root.flow?.isResponseRequired ?? false
                        onActivated: root.respond()
                    }
                ]

                K.Labelled {
                    label: root.forWho ? `${root.asked} for ${root.who}` : root.asked
                    note: root.flow?.failed ? "That didn't work. Fat fingers? Try again." : root.flow?.supplementaryMessage ?? ""
                    invalid: root.invalid

                    K.Field {
                        id: field

                        icon: "key"
                        secret: !(root.flow?.responseVisible ?? false)
                        status: root.invalid ? "Invalid" : "Normal"
                        onAccepted: root.respond()
                    }
                }
            }
        }
    }
}
