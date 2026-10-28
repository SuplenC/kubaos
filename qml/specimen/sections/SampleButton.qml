import QtQuick
import KubaOS.Components as K

// A label button of `kind` and `size` in the `state`th of `Samples.stateNames`.
K.Button {
    property int shownState: 0

    text: kind === "Filled" ? "Continue" : kind === "Tonal" ? "Save log" : kind === "Text" ? "Back" : "Erase disk"
    icon: kind === "Filled" ? "arrow_forward" : kind === "Tonal" ? "save" : kind === "Text" ? "arrow_back" : "warning"
    trailing: kind === "Filled"
    preview: shownState === 1 ? "hovered" : shownState === 2 ? "pressed" : ""
    focused: shownState === 3
    enabled: shownState !== 4
}
