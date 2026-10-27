import QtQuick
import QtQuick.Effects
import Quickshell
import Quickshell.Wayland
import KubaOS.Components as K
import qs.settings
import qs.components
import qs.services

// What each screen shows while locked: the wallpaper (or a moving loop) blurring in, the clock
// under its Japanese accent (with the weekday), the date, and the password field. Typing goes
// straight to the field; a security key (or a fingerprint reader, if enabled) works in parallel.
//
// Everything here is drawn over the surface's own colour, so a part that fails to draw leaves the
// screen covered, never showing the desktop.
WlSessionLockSurface {
    id: root

    required property Auth auth
    property bool unlocking: false
    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]
    // 0 -> 1 when the surface appears, back to 0 on unlock.
    property real enter: 0
    // Which way the veil over the wallpaper turns it, so the text reads: darker under the dark
    // mode's light text, lighter under the light mode's dark text.
    readonly property real veil: K.Tokens.mode === "light" ? 1 : -1

    color: K.Tokens.color.surface_container_lowest
    Component.onCompleted: enter = 1
    onUnlockingChanged: if (unlocking)
        enter = 0

    // Leaving takes `medium`, as Lock's release waits: the session unlocks the moment it ends, so
    // a longer exit only keeps the desktop waiting.
    Behavior on enter {
        NumberAnimation {
            duration: root.unlocking ? K.Tokens.duration.medium : K.Tokens.duration.slow
            easing.type: Easing.BezierSpline
            easing.bezierCurve: root.unlocking ? K.Tokens.easing.emphasized_accel : K.Tokens.easing.emphasized_decel
        }
    }

    SystemClock {
        id: clock

        precision: SystemClock.Minutes
    }

    // The current wallpaper if it moves, else the lock loop from Settings
    // (or the still, when that is ""). Its poster shows at once and the video
    // fades in over it; unlocking never waits for either.
    readonly property string scene: {
        const wall = Wallpapers.current;
        if (Wallpapers.isVideo(wall) || !Settings.lockLoop)
            return wall;
        return Settings.lockLoop.startsWith("/") ? Settings.lockLoop : `${Settings.wallpaperSets}/${Settings.lockLoop}`;
    }

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
    }

    MultiEffect {
        anchors.fill: parent
        source: wallpaper
        blurEnabled: true
        blur: root.enter
        blurMax: 48
        brightness: 0.22 * root.veil * root.enter
        saturation: 0.15 * root.enter
        opacity: wallpaper.ready ? 1 : 0

        Behavior on opacity {
            NumberAnimation {
                duration: K.Tokens.duration.slow
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
    }

    // Vignette toward the bottom, in the surface the text is drawn for, so the field always reads.
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop {
                position: 0.45
                color: "transparent"
            }
            GradientStop {
                position: 1
                color: K.Look.alpha(K.Tokens.color.surface, K.Tokens.opacity.scrim)
            }
        }
        opacity: root.enter
    }

    // Centred a little above the middle, where the eye takes a screen's centre to be, and rising
    // into place as it arrives.
    K.Stack {
        id: content

        width: K.Tokens.size.dialog
        centred: true
        x: K.Look.snap((root.width - content.width) / 2, content.dpr)
        y: K.Look.snap((root.height - content.height) / 2 - K.Tokens.space.s64 + K.Tokens.space.s40 - root.enter * K.Tokens.space.s40, content.dpr)
        opacity: root.enter

        K.Text {
            // The katakana middle dot: M PLUS 1 draws the Latin one off its centre.
            text: `東京・${root.kanji[clock.date.getDay()]}曜日`
            style: K.Tokens.type.accent
            color: K.Tokens.color.primary
        }

        K.Text {
            text: Qt.formatDateTime(clock.date, Time.timeFormat)
            style: K.Tokens.type.hero_number
            color: K.Tokens.color.on_surface
        }

        K.Gap {
            space: K.Tokens.space.s8
        }

        K.Text {
            text: Qt.formatDateTime(clock.date, Time.dateFormat)
            style: K.Tokens.type.body_large
            color: K.Tokens.color.on_surface_variant
        }

        K.Gap {
            space: K.Tokens.space.s48
        }

        K.Field {
            id: field

            icon: root.auth.busy ? "hourglass_top" : "lock"
            placeholder: root.auth.busy ? "Checking…" : "Password"
            secret: true
            status: root.auth.message !== "" ? "Invalid" : "Normal"
            Component.onCompleted: field.forceActiveFocus()
            Keys.onEscapePressed: field.text = ""
            onAccepted: root.auth.submit(field.text)

            transform: Translate {
                id: offset
            }

            SequentialAnimation {
                id: shake

                Jolt {
                    target: offset
                    to: K.Tokens.space.s12
                }

                Jolt {
                    target: offset
                    to: -K.Tokens.space.s8
                }

                Jolt {
                    target: offset
                    to: K.Tokens.space.s4
                }

                NumberAnimation {
                    target: offset
                    property: "x"
                    to: 0
                    duration: K.Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard_decel
                }
            }
        }

        K.Gap {
            space: K.Tokens.space.s8
        }

        // PAM and the key write messages of any length; a long one wraps inside the column
        // instead of running off both sides. The error and the key's hint share the place, one
        // fading out as the other fades in.
        Item {
            id: status

            readonly property bool fillWidth: true

            implicitHeight: Math.max(K.Tokens.type.small.line, error.implicitHeight, hint.implicitHeight)

            K.Note {
                id: error

                room: status.width
                x: K.Look.snap((status.width - error.width) / 2, content.dpr)
                text: root.auth.message
                invalid: true
                centred: true
                opacity: root.auth.message !== "" ? 1 : 0

                Behavior on opacity {
                    Fade {}
                }
            }

            K.Row {
                id: hint

                // The key's hint wins when both are on. Auth.keyHint follows
                // the key half: waiting for a finger, failed, locked, PIN.
                readonly property bool key: root.auth.keyAvailable
                readonly property bool keyWaiting: root.auth.keyState === "waiting"

                room: status.width
                x: K.Look.snap((status.width - hint.width) / 2, content.dpr)
                gap: K.Tokens.space.s8
                align: "centre"
                opacity: root.auth.message === "" && (hint.key || root.auth.fingerprintActive) ? 1 : 0

                Behavior on opacity {
                    Fade {}
                }

                K.Icon {
                    name: hint.key && root.auth.keyState === "" ? "passkey" : "fingerprint"
                    size: K.Tokens.icon.s16
                    color: K.Tokens.color.primary
                }

                K.Text {
                    text: hint.key ? root.auth.keyHint : (root.auth.fingerprintMessage || "Or touch the fingerprint reader")
                    style: K.Tokens.type.small
                    color: hint.key && root.auth.keyHintIsError ? K.Tokens.color.error : hint.key && hint.keyWaiting ? K.Tokens.color.primary : K.Tokens.color.on_surface_variant
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

    // A jolt of the shake that answers a wrong password: quick, so the field is still at once.
    component Jolt: NumberAnimation {
        property: "x"
        duration: K.Tokens.duration.micro
        easing.type: Easing.BezierSpline
        easing.bezierCurve: K.Tokens.easing.standard
    }

    // The error and the hint taking turns under the field.
    component Fade: NumberAnimation {
        duration: K.Tokens.duration.short
        easing.type: Easing.BezierSpline
        easing.bezierCurve: K.Tokens.easing.standard
    }
}
