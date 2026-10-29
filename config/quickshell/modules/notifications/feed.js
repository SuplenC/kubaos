.pragma library

// What the notifications panel, the bar's bell and its peek show of the history (newest first, as
// `Notifs.history` is): the notifications of one app in a group, New (unread) and Earlier,
// and the peek's few cards.

// How many cards the peek shows at most.
const PEEK = 3;

// The peek's footer button, which marks every card it shows read: worded apart from each unread
// card's own "Mark read".
const MARK_ALL = "Mark all read";

// Who a notification is from, which its stack goes by.
function appOf(notification) {
    return notification.appName || notification.desktopEntry || "";
}

// `list` as stacks, one per app, each where its newest is, its notifications newest first.
function stacks(list) {
    const byApp = new Map();
    for (const notification of list) {
        const app = appOf(notification);
        const stack = byApp.get(app);
        if (stack)
            stack.items.push(notification);
        else
            byApp.set(app, {
                app: app,
                items: [notification]
            });
    }
    return Array.from(byApp.values());
}

function sections(history, unread) {
    return {
        fresh: stacks(history.filter(n => unread(n))),
        earlier: stacks(history.filter(n => !unread(n)))
    };
}

// The peek's cards: the most recent unread stacks, and how many unread notifications are past
// them, for "+N more".
function peek(history, unread) {
    const all = stacks(history.filter(n => unread(n)));
    const shown = all.slice(0, PEEK);
    const past = all.slice(PEEK).reduce((sum, stack) => sum + stack.items.length, 0);
    return {
        stacks: shown,
        more: past
    };
}

// Whether the bar's bell shows: while there is a notification, while Do not disturb is on (the
// crossed-out bell says why things are quiet), or always with the setting.
function bellShown(count, always, dnd) {
    return count > 0 || always || dnd === true;
}

// The panel's line under its title: why it is silenced, how many are new, or a word that there
// is nothing at all. There is always a line, so the title sits level with the header's buttons
// whatever the panel holds. `reason` is `Notifs.dndReason`.
function status(unread, count, reason) {
    const silenced = {
        manual: "Silenced until you turn it off",
        sharing: "Silenced while you share your screen",
        fullscreen: "Silenced while a window is fullscreen"
    }[reason];
    if (silenced)
        return silenced;
    if (unread > 0)
        return `${unread} new`;
    return count > 0 ? "All read" : "Silence is golden";
}

// The Do not disturb button's tooltip: whether it is on, and why or until when. `sharing` says
// which occurrence an override is for, a share or a fullscreen window.
function dndTip(reason, sharing) {
    const on = {
        manual: "Until you turn it off",
        sharing: "While you share your screen",
        fullscreen: "While a window is fullscreen"
    }[reason];
    if (on)
        return {
            title: "Do not disturb is on",
            body: on
        };
    if (reason === "overridden")
        return {
            title: "Do not disturb",
            body: sharing ? "Off for this share" : "Off while this window is fullscreen"
        };
    return {
        title: "Do not disturb",
        body: "Turns on by itself while you share your screen or a window is fullscreen"
    };
}

// The panel's rows: each non-empty section's label, then its groups, one row each however it is
// opened, keyed by section and app, so a group keeps its row as notifications come and go.
function rows(sections) {
    const out = [];
    for (const [name, text] of [["fresh", "New"], ["earlier", "Earlier"]]) {
        const list = sections[name];
        if (list.length === 0)
            continue;
        out.push({
            kind: "label",
            key: name,
            text: text
        });
        for (const stack of list)
            out.push({
                kind: "card",
                key: `${name}:${stack.app}`,
                items: stack.items,
                stacked: stack.items.length > 1
            });
    }
    return out;
}

// How opaque a row of a list is while a group's preview floats over it, `spread` (0 -> 1) the
// preview's `recede`: a row the preview covers any of, from `coverTop` to `coverBottom`, fades out
// whole, so no cut-off piece of it shows through the gaps between the fanned cards; the others sit
// back by `amount`, `opacity.recede`.
function behind(top, bottom, coverTop, coverBottom, spread, amount) {
    const covered = top < coverBottom && bottom > coverTop;
    return 1 - (covered ? 1 : amount) * spread;
}

// The popups' groups, one per app, keyed by it, newest first.
function groups(list) {
    return stacks(list).map(stack => ({
                key: stack.app,
                items: stack.items
            }));
}

// A card's buttons for a notification's `list` of actions: the main one, filled (its default
// action, else its first), and the others after it. Its default action's text is a word for
// people, "Open" when the sender gave none. An action saying what the main one says is left out:
// a sender often offers its default action again under the same name, for servers that show no
// default.
function actions(list) {
    const main = list.find(a => a.identifier === "default") ?? list[0] ?? null;
    if (!main)
        return {
            main: null,
            label: "",
            others: []
        };
    const text = main.text.trim();
    const label = text === "" || text.toLowerCase() === "default" ? "Open" : text;
    return {
        main: main,
        label: label,
        others: list.filter(a => a !== main && a.identifier !== "default" && a.text !== label)
    };
}
