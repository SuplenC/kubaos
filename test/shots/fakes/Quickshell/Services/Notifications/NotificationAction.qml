import QtQuick

QtObject {
    property string identifier
    property string text

    function invoke(): void {
        console.log(`notification: invoke ${identifier}`);
    }
}
