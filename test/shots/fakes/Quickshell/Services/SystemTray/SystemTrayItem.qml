import QtQuick

QtObject {
    property string id
    property string icon
    property string title
    property string tooltipTitle
    property string tooltipDescription
    property bool onlyMenu: false
    property bool hasMenu: true
    // The app's menu: a stand-in with `entries`.
    property QtObject menu: null
    property int status: 1
    property int category: 0

    function activate(): void {
    }

    function secondaryActivate(): void {
    }

    function scroll(delta: real, horizontal: bool): void {
    }

    function display(window: var, x: real, y: real): void {
    }
}
