import QtQuick
import QtMultimedia
import qs.theme
import qs.services

// One wallpaper, cropped to fill: a still, or a video looping over its
// poster frame. The poster (or the still) is what `ready` waits for, so a
// video never makes anyone wait: it fades in over the poster on its first
// decoded frame. While `playing` is false the video fades back to the poster
// and its decoder is released; turning it on again starts from the top.
// One player per view: Qt can't feed one decoder into two windows, so every
// screen decodes its own copy (cheap with VA-API).
Item {
    id: root

    property string path
    // Decode and loop the video (ignored for stills).
    property bool playing: false
    // Decode size of the poster or still, in device pixels.
    property size sourceSize
    readonly property bool video: Wallpapers.isVideo(path)
    // The poster or still has loaded: safe to show. A video without a
    // poster counts as ready too and simply fades in on its first frame.
    readonly property bool ready: still.status === Image.Ready || (video && Wallpapers.listed && still.status !== Image.Loading)
    readonly property int status: still.status
    // A video frame is on screen and the player runs.
    readonly property bool moving: loader.item?.moving ?? false

    // Live while playing, and for one fade after it stops.
    readonly property bool want: video && playing
    onWantChanged: {
        if (want) {
            release.stop();
            loader.active = true;
        } else {
            release.restart();
        }
    }
    Component.onCompleted: loader.active = want

    Image {
        id: still

        anchors.fill: parent
        source: root.path && Wallpapers.poster(root.path) ? `file://${Wallpapers.poster(root.path)}` : ""
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        retainWhileLoading: true
        sourceSize: root.sourceSize
    }

    Timer {
        id: release

        interval: Theme.anim.slow + 100
        onTriggered: loader.active = false
    }

    Loader {
        id: loader

        anchors.fill: parent
        active: false
        // A new path restarts the player from scratch.
        sourceComponent: root.video ? player : null
    }

    Component {
        id: player

        Item {
            id: film

            property bool framed: false
            readonly property bool moving: framed && media.playbackState === MediaPlayer.PlayingState

            opacity: framed && root.want ? 1 : 0
            clip: true

            Behavior on opacity {
                Anim {
                    duration: Theme.anim.slow
                    easing.bezierCurve: Theme.anim.effects
                }
            }

            // Cropped by hand: sized to the video's aspect, centred, clipped.
            // VideoOutput's own PreserveAspectCrop zooms by the device pixel
            // ratio once a second output has run on a scaled screen (Qt 6, seen
            // with the picker's thumbnails at scale 1.5).
            VideoOutput {
                id: output

                readonly property real aspect: sourceRect.width > 0 && sourceRect.height > 0 ? sourceRect.width / sourceRect.height : film.width / Math.max(1, film.height)
                readonly property bool wider: aspect > film.width / Math.max(1, film.height)

                anchors.centerIn: parent
                width: wider ? film.height * aspect : film.width
                height: wider ? film.height : film.width / aspect
                fillMode: VideoOutput.Stretch
            }

            MediaPlayer {
                id: media

                source: `file://${root.path}`
                videoOutput: output
                // No audioOutput: the loops have no sound, and none is wanted.
                loops: MediaPlayer.Infinite
                onSourceChanged: {
                    film.framed = false;
                    if (root.want)
                        play();
                }
                onErrorOccurred: (error, message) => console.warn(`wallpaper: can't play ${root.path}: ${message}`)
            }

            Connections {
                target: film.framed ? null : output.videoSink

                function onVideoFrameChanged(): void {
                    film.framed = true;
                }
            }

            // Follow `want`: play, or hold the current frame through the fade out.
            Connections {
                target: root

                function onWantChanged(): void {
                    root.want ? media.play() : media.pause();
                }
            }

            Component.onCompleted: if (root.want)
                media.play()
        }
    }
}
