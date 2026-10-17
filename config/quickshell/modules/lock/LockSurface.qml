import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import Quickshell
import Quickshell.Wayland
import qs.theme
import qs.settings
import qs.components
import qs.services

// What each screen shows while locked: the wallpaper (or a moving loop)
// blurring in, a large clock with a soft glow in the accent color, the date
// (with its Japanese weekday), and the password field. Typing goes straight to the field;
// a security key (or a fingerprint reader, if enabled) works in parallel.
WlSessionLockSurface {
    id: root

    required property Auth auth
    property bool unlocking: false
    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]
    // 0 -> 1 when the surface appears, back to 0 on unlock.
    property real enter: 0

    color: Theme.m3surfaceContainerLowest
    Component.onCompleted: enter = 1
    onUnlockingChanged: if (unlocking)
        enter = 0

    Behavior on enter {
        Anim {
            duration: root.unlocking ? Theme.anim.normal : Theme.anim.slow
            easing.bezierCurve: root.unlocking ? Theme.anim.emphasizedAccel : Theme.anim.emphasizedDecel
        }
    }

    SystemClock {
        id: clock

        precision: SystemClock.Minutes
    }

    // The current wallpaper if it moves, else the lock loop from Settings
    // (or the still, when that is ""). Its poster shows at once and the video
    // fades in over it; unlocking never waits for either. A ukiyo scene
    // shows its poster here, and the frame below covers it.
    readonly property string scene: {
        const wall = Wallpapers.current;
        if (Wallpapers.isVideo(wall) || Wallpapers.isScene(wall) || !Settings.lockLoop)
            return wall;
        return `${Settings.wallpaperRoot}/${Settings.lockLoop}`;
    }

    // ukiyo under the lock: the session lock covers the background layer, so
    // the scene can't show through. The moment it's told to go still
    // (Ukiyo.qml, on lock), ukiyo saves the frame that's on this screen and
    // stops drawing. That frame, blurred like any wallpaper, is the
    // background: zero GPU while locked, and it's exactly what was on screen,
    // so the blur-in is seamless. (A second live renderer in QML would mean
    // the scene twice, in another shader language, burning battery behind a
    // lock.) The poster stands in until the frame is written; the serial in
    // the URL makes Image reload it for every lock.
    readonly property string stillFrame: Wallpapers.isScene(Wallpapers.current) ? Ukiyo.stillFor(screen?.name ?? "") : ""

    // Blurred anyway: drawn at half size (the blur and the upscale are
    // cheap there), then stretched back over the screen by the effect.
    WallpaperView {
        id: wallpaper

        width: Math.ceil(root.width / 2)
        height: Math.ceil(root.height / 2)
        path: root.scene
        sourceSize: Qt.size(width, height)
        playing: Motion.lock && !root.unlocking
        visible: false
        layer.enabled: true

        Image {
            id: frame

            anchors.fill: parent
            source: root.stillFrame ? `file://${root.stillFrame}?${Ukiyo.stillSerial}` : ""
            fillMode: Image.PreserveAspectCrop
            sourceSize: Qt.size(parent.width, parent.height)
            asynchronous: true
            cache: false
            retainWhileLoading: true
            opacity: status === Image.Ready ? 1 : 0

            Behavior on opacity {
                Anim {
                    duration: Theme.anim.normal
                }
            }
        }
    }

    MultiEffect {
        anchors.fill: parent
        source: wallpaper
        blurEnabled: true
        blur: root.enter
        blurMax: 48
        brightness: -0.22 * root.enter
        saturation: 0.15 * root.enter
        opacity: wallpaper.ready ? 1 : 0

        Behavior on opacity {
            Anim {
                duration: Theme.anim.slow
            }
        }
    }

    // Vignette toward the bottom so the field always reads.
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop {
                position: 0.45
                color: "transparent"
            }
            GradientStop {
                position: 1
                color: Theme.alpha(Theme.m3scrim, 0.55)
            }
        }
        opacity: root.enter
    }

    ColumnLayout {
        id: content

        anchors.centerIn: parent
        anchors.verticalCenterOffset: -root.height * 0.06 + (1 - root.enter) * 40
        spacing: 0
        opacity: root.enter

        Text {
            Layout.alignment: Qt.AlignHCenter
            text: `東京  ·  ${root.kanji[clock.date.getDay()]}曜日`
            color: Theme.m3tertiary
            font.family: Theme.font.jp
            font.pixelSize: Theme.font.lg
            font.weight: Font.Medium
            font.letterSpacing: 3
            renderType: Text.NativeRendering
        }

        Item {
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: time.implicitWidth
            implicitHeight: time.implicitHeight * 0.86

            Text {
                id: time

                anchors.centerIn: parent
                text: Qt.formatDateTime(clock.date, "HH:mm")
                color: Theme.m3onSurface
                font.family: Theme.font.sans
                font.pixelSize: Theme.font.display
                font.weight: Font.Light
                font.features: Theme.font.tabular
                font.letterSpacing: -4
                renderType: Text.CurveRendering
                layer.enabled: true
                layer.effect: MultiEffect {
                    shadowEnabled: true
                    shadowColor: Theme.alpha(Theme.m3primary, 0.55)
                    shadowBlur: 1
                    blurMax: 40
                    shadowHorizontalOffset: 0
                    shadowVerticalOffset: 0
                }
            }
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: Qt.formatDateTime(clock.date, "dddd, d MMMM")
            font.pixelSize: Theme.font.xl
            color: Theme.m3onSurfaceVariant
        }

        SearchField {
            id: field

            Layout.topMargin: Theme.spacing.xxxl + Theme.spacing.lg
            Layout.preferredWidth: 340
            Layout.alignment: Qt.AlignHCenter
            implicitHeight: 52
            color: Theme.alpha(Theme.m3surfaceContainerHigh, 0.82)
            icon: root.auth.busy ? "hourglass_top" : "lock"
            iconColor: Theme.m3primary
            echoMode: TextInput.Password
            placeholder: root.auth.busy ? "Checking…" : "Password"
            catchEscape: true
            onEscaped: text = ""
            onAccepted: root.auth.submit(text)
            Component.onCompleted: focusInput()

            transform: Translate {
                id: offset
            }

            SequentialAnimation {
                id: shake

                Anim {
                    target: offset
                    property: "x"
                    to: 14
                    duration: 60
                }

                Anim {
                    target: offset
                    property: "x"
                    to: -10
                    duration: 90
                }

                Anim {
                    target: offset
                    property: "x"
                    to: 6
                    duration: 80
                }

                Anim {
                    target: offset
                    property: "x"
                    to: 0
                    duration: Theme.anim.normal
                    easing.bezierCurve: Theme.anim.spatial
                }
            }
        }

        Item {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: Theme.spacing.md
            implicitWidth: Math.max(error.implicitWidth, hint.implicitWidth)
            implicitHeight: 24

            StyledText {
                id: error

                anchors.centerIn: parent
                text: root.auth.message
                color: Theme.m3error
                font.weight: Font.Medium
                opacity: text !== "" ? 1 : 0

                Behavior on opacity {
                    Anim {
                        duration: Theme.anim.small
                    }
                }
            }

            Row {
                id: hint

                // The key's hint wins when both are on. Auth.keyHint follows
                // the key half: waiting for a finger, failed, locked, PIN.
                readonly property bool key: root.auth.keyAvailable
                readonly property bool keyWaiting: root.auth.keyState === "waiting"

                anchors.centerIn: parent
                spacing: Theme.spacing.sm
                opacity: root.auth.message === "" && (key || root.auth.fingerprintActive) ? 1 : 0

                Behavior on opacity {
                    Anim {
                        duration: Theme.anim.small
                    }
                }

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: hint.key && root.auth.keyState === "" ? "passkey" : "fingerprint"
                    size: 20
                    color: Theme.m3primary
                }

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: hint.key ? root.auth.keyHint : (root.auth.fingerprintMessage || "Or touch the fingerprint reader")
                    color: hint.key && root.auth.keyHintIsError ? Theme.m3error : hint.key && hint.keyWaiting ? Theme.m3primary : Theme.m3onSurfaceVariant
                }
            }
        }
    }

    Connections {
        target: root.auth

        function onFailed() {
            field.text = "";
            shake.restart();
        }
    }
}
