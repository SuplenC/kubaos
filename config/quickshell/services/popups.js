.pragma library

// The popups on screen as a state, against the notifications panel: opening the panel is seeing
// every popup, critical ones too, so they leave as it opens and don't come back when it closes;
// one that arrives while it is open is listed there and doesn't pop up, then or after. Every
// function returns a new state and never changes the one it is given.
//
// `popups` are the notifications on screen, newest first. `seen` are the transient ones seen in the
// panel instead, which go once the panel has gone (`gone`), as a transient popup goes when it hides.

function initial() {
    return {
        listing: false,
        popups: [],
        seen: []
    };
}

// The panel opens (`listing`) or closes.
function listed(state, listing) {
    if (!listing)
        return Object.assign({}, state, {
            listing: false
        });
    return {
        listing: true,
        popups: [],
        seen: state.seen.concat(state.popups.filter(n => n.transient))
    };
}

// `notification` arrives, popping up if `popsUp` (Do not disturb's say) while the panel is closed.
function arrived(state, notification, popsUp) {
    if (state.listing)
        return Object.assign({}, state, {
            seen: notification.transient ? state.seen.concat([notification]) : state.seen
        });
    if (!popsUp)
        return state;
    return Object.assign({}, state, {
        popups: [notification].concat(state.popups)
    });
}

// `notification` leaves the screen: its popup hid, it was closed, or its action ran.
function left(state, notification) {
    return Object.assign({}, state, {
        popups: state.popups.filter(n => n !== notification),
        seen: state.seen.filter(n => n !== notification)
    });
}

// The panel has gone: the state without the transient ones seen in it, which the caller lets go.
function gone(state) {
    return Object.assign({}, state, {
        seen: []
    });
}
