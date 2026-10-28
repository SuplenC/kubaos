import QtQuick

// Stand-in for PamContext: it never starts, so no shot reaches PAM.
QtObject {
    property string config
    property string configDirectory
    property string user
    property bool active: false
    readonly property string message: ""
    readonly property bool messageIsError: false
    readonly property bool responseRequired: false
    readonly property bool responseVisible: false

    signal completed(int result)
    signal error(int error)
    signal pamMessage

    function start(): bool {
        return false;
    }

    function abort(): void {
    }

    function respond(text: string): void {
    }
}
