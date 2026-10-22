import QtQuick
import qs.theme
import qs.components
import qs.services

// The keyboard layout in use, as a short code ("US", "IT"), shown only with
// two layouts or more. Click switches to the next, as SUPER+SHIFT+Space does.
Pill {
    id: root

    visible: Keyboard.several
    interactive: true
    padding: Theme.spacing.sm
    tooltip: `Typing with ${Keyboard.code}. Click or SUPER+SHIFT+Space for the next layout`
    onClicked: Keyboard.next()

    StyledText {
        anchors.verticalCenter: parent.verticalCenter
        text: Keyboard.code
        font.weight: Font.DemiBold
    }
}
