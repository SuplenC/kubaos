.pragma library

// Subsequence fuzzy match. Returns a score (higher is better), or -1 when the
// query letters don't all appear in order in the text. Prefix matches win;
// otherwise word starts and consecutive runs earn bonuses and gaps cost a bit.
function score(query, text) {
    if (!text)
        return -1;
    const q = query.toLowerCase();
    const t = text.toLowerCase();
    if (t.startsWith(q))
        return 1000 - t.length;

    let total = 0;
    let from = 0;
    let run = 0;
    for (const ch of q) {
        const at = t.indexOf(ch, from);
        if (at < 0)
            return -1;
        const wordStart = at === 0 || " -_.".includes(t[at - 1]);
        run = at === from ? run + 1 : 0;
        total += 10 + run * 5 + (wordStart ? 15 : 0) - Math.min(at - from, 10);
        from = at + 1;
    }
    return total - t.length * 0.1;
}

// Sorts desktop entries by best match over name (weighted), generic name, keywords and id.
// `nameOf(entry)` gives the displayed name.
function rank(entries, query, nameOf) {
    if (!query)
        return entries.slice().sort((a, b) => nameOf(a).localeCompare(nameOf(b)));

    const scored = [];
    for (const entry of entries) {
        const best = Math.max(score(query, nameOf(entry)) * 2, score(query, entry.genericName), score(query, entry.id), ...Array.from(entry.keywords, k => score(query, k)));
        if (best > 0)
            scored.push({ entry, best });
    }
    return scored.sort((a, b) => b.best - a.best).map(s => s.entry);
}
