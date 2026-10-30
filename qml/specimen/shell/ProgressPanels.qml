import QtQuick
import KubaOS.Components as K

// The "Shell only" page's progress panels: one job in each of its states.
K.Section {
    title: "ProgressPanel"
    description: "A job followed to its end, as the Command menu's progress View shows it: running with no idea how far, part of the way, done, and failed with what it last said."

    K.Grid {
        columns: 4
        gap: K.Tokens.space.s24

        Job {
            job: ({
                    title: "Updating",
                    state: "running",
                    progress: -1
                })
        }

        Job {
            job: ({
                    title: "Updating",
                    state: "running",
                    progress: 0.42,
                    message: "Fetching packages"
                })
        }

        Job {
            job: ({
                    title: "Updated",
                    state: "succeeded",
                    progress: 1
                })
        }

        Job {
            job: ({
                    title: "Update failed",
                    state: "failed",
                    progress: -1,
                    message: "No network"
                })
        }
    }

    // One job in a panel's room, still, so the spinner shows the same in every shot. It takes its
    // cell's width, as a panel's room.
    component Job: K.ProgressPanel {
        readonly property bool fillWidth: true

        implicitHeight: K.Tokens.size.log
        moves: false
    }
}
