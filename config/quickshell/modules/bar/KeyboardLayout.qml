import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// The keyboard layout in use, as a short code ("US", "IT"), shown only with
// two layouts or more. Click switches to the next, as SUPER+SHIFT+Space does.
Pill {
    id: root

    visible: Keyboard.several
    interactive: true
    tooltip: `Typing with ${Keyboard.code}. Click or SUPER+SHIFT+Space for the next layout`
    onClicked: Keyboard.next()

    K.Text {
        text: Keyboard.code
        style: K.Tokens.type.button_small
        color: K.Tokens.color.on_surface
    }
}
