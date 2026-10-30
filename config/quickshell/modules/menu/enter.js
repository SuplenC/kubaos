.pragma library

// What Enter runs on a row while the row can't show an Action the query chose. A verb in the query
// (`remove chrome`, `wifi off`) chooses one of the Result's Actions, and kotodama-qml makes it the
// row's primary: the first entry of `KotodamaSearch.actions`, the Action `run` runs and the one
// `hints` names. Until the row shows that Action, the menu keeps to the Item's own first Action
// wherever it runs or lists one, so nothing runs that the row didn't show. `chosen` is the row's
// `primaryChosen`; with it false, everything is as kotodama-qml gives it.

// The place of the Item's own first Action in `actions`, the row's Action panel as
// `KotodamaSearch.actions` lists it; -1 when the Item has none. kotodama-qml lists the chosen Action
// first and the Item's own Actions next, in their order.
function usual(actions, chosen) {
    if (!chosen)
        return actions.length > 0 ? 0 : -1;
    return actions.findIndex((action, n) => n > 0 && action.group === "");
}

// The Action panel with the Item's own first Action first, as it is without a chosen Action. The
// chosen one follows it, or goes last with the other destructive Actions, as the panel keeps them.
function unchosen(actions, chosen) {
    const first = usual(actions, chosen);
    if (!chosen || first < 0)
        return actions;
    const rest = actions.filter((_, n) => n !== 0 && n !== first);
    const picked = actions[0];
    if (picked.role === "destructive")
        return [actions[first], ...rest, picked];
    return [actions[first], picked, ...rest];
}

// The row's key hints, `↵ Remove · ⌃K Actions` as `KotodamaSearch.hints` writes them, with ↵
// naming the Item's own first Action in place of the chosen one.
function hints(text, actions, chosen) {
    const first = usual(actions, chosen);
    if (!chosen || first < 0)
        return text;
    const named = `↵ ${actions[0].title}`;
    if (!text.startsWith(named))
        return text;
    return `↵ ${actions[first].title}${text.slice(named.length)}`;
}
