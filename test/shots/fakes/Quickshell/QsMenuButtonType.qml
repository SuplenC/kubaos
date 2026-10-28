pragma Singleton
import QtQuick

// Stand-in for QsMenuButtonType: what a tray menu entry's button is.
QtObject {
    enum Type {
        None,
        CheckBox,
        RadioButton
    }
}
