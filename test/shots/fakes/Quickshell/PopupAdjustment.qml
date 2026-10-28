pragma Singleton
import QtQuick

QtObject {
    enum Adjustment {
        None = 0,
        Slide = 1,
        FlipY = 2,
        FlipX = 4
    }
}
