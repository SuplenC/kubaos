.pragma library

// Keeps a ListModel in step with a list of rows by their `key`, with the fewest model changes, so a
// view animates only what really happened: a row that came is inserted, one that went is removed,
// one that changed place is moved, and the rest stay the same elements. Every row has the same
// roles, as a ListModel takes its roles' types from the first row it holds.

// The removes, then the inserts and moves, that turn the keys `from` into `to`, in the order a
// ListModel applies them: `{op: "remove", at}`, `{op: "insert", at, key}`, `{op: "move", from, to}`.
// The rows of the longest run already in order stay put; only the others move.
function diff(from, to) {
    const ops = [];
    const wanted = new Set(to);
    const cur = from.slice();
    for (let i = cur.length - 1; i >= 0; i--) {
        if (!wanted.has(cur[i])) {
            ops.push({
                op: "remove",
                at: i
            });
            cur.splice(i, 1);
        }
    }
    const target = new Map(to.map((key, i) => [key, i]));
    const still = steady(cur.map(key => target.get(key)));
    const stays = new Set(cur.filter((key, i) => still.has(i)));
    // Backwards from the end, each key goes just before the one after it in `to`.
    let anchor = cur.length;
    for (let i = to.length - 1; i >= 0; i--) {
        const key = to[i];
        const at = cur.indexOf(key);
        if (at < 0) {
            cur.splice(anchor, 0, key);
            ops.push({
                op: "insert",
                at: anchor,
                key: key
            });
        } else if (stays.has(key)) {
            anchor = at;
        } else {
            const dest = at < anchor ? anchor - 1 : anchor;
            cur.splice(dest, 0, cur.splice(at, 1)[0]);
            if (dest !== at)
                ops.push({
                    op: "move",
                    from: at,
                    to: dest
                });
            anchor = dest;
        }
    }
    return ops;
}

// The indexes of a longest increasing run in `values`: the rows already in their order.
function steady(values) {
    const tails = [];
    const before = new Array(values.length).fill(-1);
    for (let i = 0; i < values.length; i++) {
        let lo = 0;
        let hi = tails.length;
        while (lo < hi) {
            const mid = (lo + hi) >> 1;
            if (values[tails[mid]] < values[i])
                lo = mid + 1;
            else
                hi = mid;
        }
        if (lo > 0)
            before[i] = tails[lo - 1];
        tails[lo] = i;
    }
    const run = new Set();
    for (let i = tails.length > 0 ? tails[tails.length - 1] : -1; i >= 0; i = before[i])
        run.add(i);
    return run;
}

// Brings `model` to `rows`: diffs the keys, then sets the roles that changed on the rows it kept.
function apply(model, rows) {
    const keys = [];
    for (let i = 0; i < model.count; i++)
        keys.push(model.get(i).key);
    const byKey = new Map(rows.map(row => [row.key, row]));
    for (const op of diff(keys, rows.map(row => row.key))) {
        if (op.op === "remove")
            model.remove(op.at, 1);
        else if (op.op === "insert")
            model.insert(op.at, byKey.get(op.key));
        else
            model.move(op.from, op.to, 1);
    }
    for (let i = 0; i < rows.length; i++) {
        const element = model.get(i);
        const row = rows[i];
        for (const role in row) {
            if (element[role] !== row[role])
                model.setProperty(i, role, row[role]);
        }
    }
}
