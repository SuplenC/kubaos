import QtQuick
import KubaOS.Components as K

// A job followed to its end, as Kōbō's install screen shows one: its state as the panel's one big
// symbol, its title, a bar while it knows how far it is, and what it last said, in a column centred
// in the panel's room. The Command menu's progress View.
Item {
    id: root

    // `{title, state, progress, message}`, as `KotodamaSearch.jobs` lists them.
    property var job: ({})
    // Whether things may move: off, the spinner stands still.
    property bool moves: true
    readonly property bool failed: job.state === "failed"
    readonly property bool succeeded: job.state === "succeeded"
    readonly property bool known: root.succeeded || (job.progress ?? -1) >= 0
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    K.Stack {
        id: column

        x: K.Look.snap((root.width - width) / 2, root.dpr)
        y: K.Look.snap((root.height - height) / 2, root.dpr)
        width: Math.min(root.width, K.Tokens.size.column)
        fill: true
        centred: true
        gap: K.Tokens.space.s16

        // Only a job that can't tell how far it has got spins: a running animation makes every
        // frame wait on the screen's refresh.
        K.Spinner {
            visible: !root.failed && !root.known
            running: root.moves && root.visible
            size: K.Tokens.icon.s48
        }

        K.Icon {
            visible: root.failed || root.known
            name: root.failed ? "error" : root.succeeded ? "check_circle" : "hourglass_top"
            size: K.Tokens.icon.s48
            color: root.failed ? K.Tokens.color.error : K.Tokens.color.primary
        }

        K.Text {
            text: root.job.title ?? ""
            style: K.Tokens.type.headline
            horizontalAlignment: Text.AlignHCenter
        }

        K.ProgressBar {
            visible: root.known && !root.failed
            value: root.succeeded ? 1 : root.job.progress ?? 0
        }

        K.Text {
            text: {
                const job = root.job;
                const state = root.succeeded ? "Done" : root.failed ? "Failed" : job.progress >= 0 ? `${Math.round(job.progress * 100)}%` : "Running";
                return job.message ? `${state} · ${job.message}` : state;
            }
            style: K.Tokens.type.body
            horizontalAlignment: Text.AlignHCenter
            color: root.failed ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
        }
    }
}
