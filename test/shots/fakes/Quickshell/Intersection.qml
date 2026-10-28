pragma Singleton
import QtQuick

QtObject {
    enum Kind {
        Combine,
        Subtract,
        Intersect,
        Xor
    }
}
