import QtQuick
import qs.theme
import qs.components

// A progress View: the job it follows, as Kōbō's install screen shows one,
// a title over a bar and what the job last said.
Item {
    id: view

    property MenuLook look
    // `{title, state, progress, message}`, as `KotodamaSearch.jobs` lists them.
    property var job: ({})
    readonly property bool failed: job.state === "failed"
    readonly property bool succeeded: job.state === "succeeded"

    Column {
        width: Math.min(parent.width - 2 * view.look.pad, view.look.columnWidth)
        anchors.centerIn: parent
        spacing: view.look.inset

        Icon {
            anchors.horizontalCenter: parent.horizontalCenter
            name: view.failed ? "error" : view.succeeded ? "check_circle" : "hourglass_top"
            filled: view.succeeded
            size: view.look.progressIcon
            color: view.failed ? Theme.m3error : Theme.m3primary
        }

        StyledText {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            text: view.job.title ?? ""
            font.pixelSize: view.look.font.heading
            font.weight: Font.Medium
        }

        JobBar {
            width: parent.width
            look: view.look
            implicitHeight: view.look.progressBarHeight
            progress: view.succeeded ? 1 : view.job.progress ?? -1
            failed: view.failed
        }

        StyledText {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            text: {
                const job = view.job;
                const state = view.succeeded ? "Done" : view.failed ? "Failed" : job.progress >= 0 ? `${Math.round(job.progress * 100)}%` : "Running";
                return job.message ? `${state} · ${job.message}` : state;
            }
            color: view.failed ? Theme.m3error : Theme.m3onSurfaceVariant
            font.pixelSize: view.look.font.body
            font.features: Theme.font.tabular
        }
    }
}
