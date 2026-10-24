pragma Singleton

import Quickshell

// What the Kotodama Daemon asks of the Command menu through another of the
// shell's `KotodamaSearch`es. The Daemon sends `kotodama toggle` and `open` to
// one Front end it takes for the menu, and before the menu has ever been shown
// that may be a search embedded elsewhere (the Keyboard tab's "Add a
// language…"). Such a search hands them on here, and the menu
// (modules/menu/Menu.qml) acts on them as on its own.
Singleton {
    signal toggleRequested
    signal openRequested(string query)

    function toggle(): void {
        toggleRequested();
    }

    function open(query: string): void {
        openRequested(query);
    }
}
