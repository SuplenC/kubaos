pragma Singleton
import QtQuick

QtObject {
    enum State {
        Unknown,
        Disconnected,
        Connecting,
        Connected,
        Disconnecting
    }

    function toString(state: int): string {
        return ["Unknown", "Disconnected", "Connecting", "Connected", "Disconnecting"][state] ?? "Unknown";
    }
}
