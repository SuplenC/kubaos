import QtQuick

// Stand-in for FileView: every file reads as missing, so nothing on the machine reaches a shot.
QtObject {
    property string path
    property bool watchChanges
    property bool printErrors
    property bool blockLoading
    property bool blockWrites
    property bool atomicWrites
    property bool preload
    readonly property bool loaded: false

    signal fileChanged
    signal loadFailed(var error)
    signal saved
    signal saveFailed(var error)

    function reload(): void {
    }

    function text(): string {
        return "";
    }

    function data(): var {
        return null;
    }

    function setText(text: string): void {
    }

    function waitForJob(): bool {
        return true;
    }
}
