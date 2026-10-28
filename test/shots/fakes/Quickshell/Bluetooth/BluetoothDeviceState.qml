pragma Singleton
import QtQuick

QtObject {
    enum State {
        Disconnected,
        Connected,
        Disconnecting,
        Connecting
    }

    function toString(state: int): string {
        return ["Disconnected", "Connected", "Disconnecting", "Connecting"][state] ?? "Disconnected";
    }
}
