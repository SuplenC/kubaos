import QtQuick
import KubaOS.Components as K

// The "Shell only" page's loading rows.
K.Section {
    title: "ListRow, loading"
    description: "A row still being found keeps the place of the tiled two-line row that will take it: its tile and its two lines as skeletons, the lines of differing lengths so rows one under another don't read as one block. It takes no press."

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        K.List {
            K.ListRow {
                title: "Shinjuku-5G"
                subtitle: "Connected"
                icon: "wifi"
                tile: true
            }

            K.ListRow {
                loading: true
                share: 0.7
            }

            K.ListRow {
                loading: true
                share: 0.45
            }
        }
    }
}
