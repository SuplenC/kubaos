pragma Singleton
import QtQuick

QtObject {
    enum Reason {
        Unknown,
        NoSecrets,
        WifiClientDisconnected,
        WifiClientFailed,
        WifiAuthTimeout,
        WifiNetworkLost
    }
}
