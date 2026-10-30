import QtQuick
import KubaOS.Components as K
import "shell" as S

// The parts only `KubaOS.Components` has, each in its states, under the components page: the
// shell's own (the bar's pill, a ticking figure) and the ones `kubaos-ui` hasn't got yet. Kuba
// reviews them here as he does the rest. They have no iced twin to be held to, so `main.qml`
// lays this page out under the components page but leaves it out of the frame the parity check
// reads. A part `kubaos-ui` gains moves from here to a section of the components page, on both
// sides.
K.Page {
    id: root

    K.Stack {
        gap: K.Tokens.space.s40

        K.Header {
            accent: "見本"
            title: "Shell only"
            lede: "Parts only the shell's library has, in their states. The parity check leaves them out until kubaos-ui has them."
        }

        K.Stack {
            gap: K.Tokens.space.s48

            S.Pills {
            }

            S.BadgedIcons {
            }

            S.Figures {
            }

            S.HeaderFades {
            }

            S.ScrollingLists {
            }

            S.LiveLists {
            }

            S.LoadingRows {
            }

            S.ProgressPanels {
            }
        }
    }
}
