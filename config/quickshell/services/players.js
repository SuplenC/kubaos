.pragma library

// The logic behind `Players.qml` that needs no MPRIS: which player is the active one, and which
// window belongs to a player. Players are keyed by their D-Bus name.

// The active player: the one picked or that played last, while it is there; else one that
// plays, else the first.
function active(players, key) {
    return players.find(p => p.dbusName === key) ?? players.find(p => p.isPlaying) ?? players[0] ?? null;
}

// The key after a player starts or stops playing: a player that starts takes over; one that
// stops stays active, so its controls stay on the bar.
function played(key, player) {
    return player.isPlaying ? player.dbusName : key;
}

// The window of a player's app, by its class against the player's names (its desktop entry, the
// entry's id and StartupWMClass, its identity), ignoring case. Either side may be a reverse-DNS id, so
// each also matches by its last dotted part, as `org.mozilla.firefox` is `firefox`.
function windowOf(windows, names) {
    const lowered = names.filter(Boolean).map(n => n.toLowerCase());
    const wanted = [...lowered, ...lowered.map(n => n.split(".").pop())];
    return windows.find(w => {
        const cls = (w.appClass ?? "").toLowerCase();
        return cls !== "" && (wanted.includes(cls) || wanted.includes(cls.split(".").pop()));
    }) ?? null;
}

// The desktop entry a player names only by its identity: the entry of that name, else one whose
// name ends with it, as "Google Chrome" is "Chrome". Ignores case.
function entryNamed(entries, identity) {
    const wanted = (identity ?? "").toLowerCase();
    if (wanted === "")
        return null;
    return entries.find(e => e.name.toLowerCase() === wanted) ?? entries.find(e => e.name.toLowerCase().endsWith(" " + wanted)) ?? null;
}
