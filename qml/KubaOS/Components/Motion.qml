pragma Singleton

import QtQuick

// How the user wants things to move: `kubaos-ui`'s `motion::Motion`. The host sets these from its
// settings, as it sets `Tokens.mode`; a host that sets nothing gets the defaults.
QtObject {
    // The focus tube strikes on as focus arrives, as the neon sign does. On by default.
    property bool stutter: true
    // TODO(#154): the shell's reduce-motion setting sets this; until it exists nothing does.
    // Moves become short fades, or nothing, and no tube strikes.
    property bool reduced: false
    // Whether a tube strikes on: the stutter is on and motion isn't reduced.
    readonly property bool strikes: stutter && !reduced
}
