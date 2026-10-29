.pragma library

// What the system monitor's panel shows of the stream: the subtitle under its title, and the
// process rows, which take the panel's room left under the readings.

// The subtitle for the worst colour step (`Sysmon.worstLevel`): 0 fine, 1 close to a threshold,
// 2 over one.
function mood(level) {
    return level >= 2 ? "This is fine" : level === 1 ? "Breaking a sweat" : "Cool as a cucumber";
}

// How many rows the processes have: as many as fit in `room` at `row` each, never fewer than
// `least` nor more than `most`, the most the engine lists. The count is the room's, not the
// list's, so rows arriving or leaving move nothing.
function slots(room, row, least, most) {
    if (row <= 0)
        return least;
    return Math.min(most, Math.max(least, Math.floor(room / row)));
}

// What row `index` shows: `process` once the list has one for it, `placeholder` while the
// processes are still being counted, and `empty` past the end of a list that came in shorter.
function slot(index, counted, count) {
    if (!counted)
        return "placeholder";
    return index < count ? "process" : "empty";
}

// A process's row: its name, its CPU share (unknown for a process outside the list, which only
// its details tell) and its memory, already written as `memory`.
function cells(process, memory) {
    return [process.name, process.cpu == null ? "–" : `${process.cpu.toFixed(1)}%`, memory];
}

// How much of its column a placeholder's bar takes in row `index`, varied row to row so the bars
// don't read as one block. A number's column is narrow and its figures nearly fill it, so its bar
// takes more of it, and differs from the name's beside it.
function share(index, numeric) {
    const shares = [0.7, 0.45, 0.6, 0.35, 0.55];
    if (!numeric)
        return shares[index % shares.length];
    return shares[(index + 2) % shares.length] + 0.3;
}
