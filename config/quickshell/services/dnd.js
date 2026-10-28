.pragma library

// Do not disturb as a state: on by hand (`manual`), or by itself while the screen is shared or a
// window is fullscreen (`sharing`, `fullscreen`, what Hyprland says), unless the user turned it
// off during that occurrence (`overridden`). An occurrence runs from the first reason starting to
// the last one ending, so a second reason during it turns nothing on again. Every function returns
// a new state and never changes the one it is given.
//
//   off --toggle--> manual ------------------------toggle--> off
//   off --share or fullscreen--> auto --both end--> off
//   auto --toggle--> overridden --toggle--> auto
//   overridden --both end--> off

function initial() {
    return {
        manual: false,
        overridden: false,
        sharing: false,
        fullscreen: false
    };
}

function automatic(state) {
    return state.sharing || state.fullscreen;
}

function on(state) {
    return state.manual || (automatic(state) && !state.overridden);
}

// Why it is on: "manual", "sharing" or "fullscreen"; "overridden" while it is off during an
// occurrence; "" while it is simply off.
function reason(state) {
    if (state.manual)
        return "manual";
    if (automatic(state))
        return state.overridden ? "overridden" : state.sharing ? "sharing" : "fullscreen";
    return "";
}

// What Hyprland says now. The override lasts until every reason has ended.
function sense(state, reasons) {
    const next = Object.assign({}, state, {
        sharing: reasons.sharing === true,
        fullscreen: reasons.fullscreen === true
    });
    if (!automatic(next))
        next.overridden = false;
    return next;
}

// The user's switch: off if on, on if off. Off clears the hand's on and, during an occurrence,
// overrides it; on during an occurrence goes back to it, so it ends with it.
function toggle(state) {
    const next = Object.assign({}, state);
    if (on(state)) {
        next.manual = false;
        next.overridden = automatic(state);
    } else if (automatic(state)) {
        next.overridden = false;
    } else {
        next.manual = true;
    }
    return next;
}

function set(state, wanted) {
    return on(state) === wanted ? state : toggle(state);
}

// The state file (`$XDG_RUNTIME_DIR/kubaos-dnd.json`): whether it is on and why, which the Command
// menu reads, and the hand's on, which a restarted shell takes back.
function saved(state) {
    return JSON.stringify({
        on: on(state),
        manual: state.manual,
        reason: reason(state)
    });
}

// The state a restarted shell starts in: the hand's on from the file, the reasons sensed again.
function restore(text) {
    const state = initial();
    try {
        state.manual = JSON.parse(text).manual === true;
    } catch (e) {
        state.manual = false;
    }
    return state;
}
