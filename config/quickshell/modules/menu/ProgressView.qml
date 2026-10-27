import QtQuick
import KubaOS.Components as K

// A progress View: the job it follows, as Kōbō's install screen shows one: its state as the page's
// one big symbol, its title, a bar while it knows how far it is, and what it last said.
Item {
    id: view

    // `{title, state, progress, message}`, as `KotodamaSearch.jobs` lists them.
    property var job: ({})
    // Whether things may move: `Settings.menuMotion`, as Menu.qml reads it.
    property bool moves: true
    readonly property bool failed: job.state === "failed"
    readonly property bool succeeded: job.state === "succeeded"
    readonly property bool known: view.succeeded || (job.progress ?? -1) >= 0
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    K.Stack {
        id: column

        x: K.Look.snap((view.width - width) / 2, view.dpr)
        y: K.Look.snap((view.height - height) / 2, view.dpr)
        width: Math.min(view.width, K.Tokens.size.column)
        fill: true
        centred: true
        gap: K.Tokens.space.s16

        // Only a job that can't tell how far it has got spins: a running animation makes every
        // frame wait on the screen's refresh.
        K.Spinner {
            visible: !view.failed && !view.known
            running: view.moves && view.visible
            size: K.Tokens.icon.s48
        }

        K.Icon {
            visible: view.failed || view.known
            name: view.failed ? "error" : view.succeeded ? "check_circle" : "hourglass_top"
            size: K.Tokens.icon.s48
            color: view.failed ? K.Tokens.color.error : K.Tokens.color.primary
        }

        K.Text {
            text: view.job.title ?? ""
            style: K.Tokens.type.headline
            horizontalAlignment: Text.AlignHCenter
        }

        K.ProgressBar {
            visible: view.known && !view.failed
            value: view.succeeded ? 1 : view.job.progress ?? 0
        }

        K.Text {
            text: {
                const job = view.job;
                const state = view.succeeded ? "Done" : view.failed ? "Failed" : job.progress >= 0 ? `${Math.round(job.progress * 100)}%` : "Running";
                return job.message ? `${state} · ${job.message}` : state;
            }
            style: K.Tokens.type.body
            horizontalAlignment: Text.AlignHCenter
            color: view.failed ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
        }
    }
}
