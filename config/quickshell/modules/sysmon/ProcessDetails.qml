import QtQuick
import KubaOS.Components as K

// The process the system monitor shows, under its row: what it runs, whose it is, and End (it may
// save and quit) or Kill (it stops at once), each asking first. Its text sits 12 in, as the row's.
K.Stack {
    id: root

    // The processes whose shown process this is.
    required property Processes owner
    readonly property var process: root.owner.details

    fill: true
    gap: K.Tokens.space.s4
    leftPadding: K.Tokens.space.s12
    rightPadding: K.Tokens.space.s12
    topPadding: K.Tokens.space.s8
    bottomPadding: K.Tokens.space.s12

    K.Text {
        text: root.process?.command ?? ""
        style: K.Tokens.type.mono
        color: K.Tokens.color.on_surface_variant
        maximumLines: 2
    }

    K.Text {
        text: root.process ? [`pid ${root.process.pid}`, root.process.user, root.process.state, `${root.process.threads} threads`, root.process.started ? `since ${Qt.formatDateTime(new Date(root.process.started * 1000), "ddd HH:mm")}` : ""].filter(Boolean).join(" · ") : ""
        style: K.Tokens.type.small
        color: K.Tokens.color.on_surface_variant
    }

    K.Gap {
        space: K.Tokens.space.s4
    }

    // One row in both states: the button pressed turns into its confirmation where it is, so it
    // stays under the pointer, the other holds still and rests, and the question and Cancel come
    // in after them.
    K.Row {
        fill: true
        gap: K.Tokens.space.s8
        align: "centre"

        K.Button {
            text: "End"
            kind: root.owner.confirming === "term" ? "Filled" : "Tonal"
            size: "Sm"
            enabled: root.owner.confirming !== "kill"
            activeFocusOnTab: false
            onActivated: {
                if (root.owner.confirming === "term")
                    root.owner.stop();
                else
                    root.owner.confirming = "term";
            }

            K.Hint {
                text: "Ask it to quit; it may save its work first"
                shown: parent.pointerOver && root.owner.confirming === ""
            }
        }

        K.Button {
            text: "Kill"
            kind: root.owner.confirming === "kill" ? "Danger" : "Text"
            size: "Sm"
            enabled: root.owner.confirming !== "term"
            activeFocusOnTab: false
            onActivated: {
                if (root.owner.confirming === "kill")
                    root.owner.stop();
                else
                    root.owner.confirming = "kill";
            }

            K.Hint {
                text: "Stop it at once; unsaved work is lost"
                shown: parent.pointerOver && root.owner.confirming === ""
            }
        }

        K.Text {
            readonly property bool fillWidth: true

            visible: root.owner.confirming !== ""
            text: root.process ? `${root.owner.confirming === "kill" ? "Kill" : "End"} ${root.process.name}?` : ""
            style: K.Tokens.type.body_strong
            color: root.owner.confirming === "kill" ? K.Tokens.color.error : K.Tokens.color.on_surface
            maximumLines: 1
        }

        K.Button {
            visible: root.owner.confirming !== ""
            text: "Cancel"
            kind: "Text"
            size: "Sm"
            activeFocusOnTab: false
            onActivated: root.owner.confirming = ""
        }
    }

    K.Text {
        visible: root.owner.error !== ""
        text: root.owner.error
        style: K.Tokens.type.small
        color: K.Tokens.color.error
    }
}
