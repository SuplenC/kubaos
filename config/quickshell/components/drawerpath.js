.pragma library

// SVG path of a panel that grows out of one inner edge of the screen frame.
//
// The panel is described in edge coordinates: u runs along the edge
// (u0..u1), v grows away from it (0..depth). Where the panel meets the edge
// it curves into the frame with concave fillets of radius `flare`; its far
// corners are convex with radius `round`. If an end is flush with the
// perpendicular frame edge (flushStart/flushEnd) that end has no fillet at
// the base and instead curves into the perpendicular edge at the far side.
// Radii shrink with the depth, so a panel starting at depth 0 grows smoothly
// out of a flat edge. A few pixels of overlap (`ov`) tuck under the frame so
// the two fills never leave an antialiasing seam.
//
//   frame:  { l, t, r, b } inner edges of the frame in window coordinates
//   edge:   "top" | "bottom" | "left" | "right"
function build(frame, edge, u0, u1, depth, flare, round, flushStart, flushEnd) {
    if (depth < 0.5 || u1 - u0 < 1)
        return "";
    const ov = 3;
    const fr = Math.min(flare, depth / 2);          // base fillets
    const fs = Math.min(flare, depth);             // fillets into a perpendicular edge
    const cr = Math.min(round, depth / 2, (u1 - u0) / 2); // far corners

    // (u, v) -> window coordinates, and whether the mapping mirrors (flips arc sweep).
    let map, mirror = false, swap = false;
    if (edge === "top") {
        map = (u, v) => [u, frame.t + v];
    } else if (edge === "bottom") {
        map = (u, v) => [u, frame.b - v];
        mirror = true;
    } else if (edge === "left") {
        map = (u, v) => [frame.l + v, u];
        mirror = true;
        swap = true;
    } else {
        map = (u, v) => [frame.r - v, u];
        swap = true;
    }

    const p = [];
    const pt = (u, v) => map(u, v).map(n => n.toFixed(2)).join(" ");
    const M = (u, v) => p.push(`M ${pt(u, v)}`);
    const L = (u, v) => p.push(`L ${pt(u, v)}`);
    // Elliptic arc with radii ru (along u) and rv (along v); sweep in edge space.
    const A = (ru, rv, sweep, u, v) => {
        const [rx, ry] = swap ? [rv, ru] : [ru, rv];
        const s = mirror ? 1 - sweep : sweep;
        p.push(`A ${rx.toFixed(2)} ${ry.toFixed(2)} 0 0 ${s} ${pt(u, v)}`);
    };

    if (flushStart) {
        M(u0 - ov, -ov);
        L(u0 - ov, depth + fs);
        L(u0, depth + fs);
        A(flare, fs, 1, u0 + flare, depth);
    } else {
        M(u0 - flare, -ov);
        L(u0 - flare, 0);
        A(flare, fr, 1, u0, fr);
        L(u0, depth - cr);
        A(cr, cr, 0, u0 + cr, depth);
    }

    if (flushEnd) {
        L(u1 - flare, depth);
        A(flare, fs, 1, u1, depth + fs);
        L(u1 + ov, depth + fs);
        L(u1 + ov, -ov);
    } else {
        L(u1 - cr, depth);
        A(cr, cr, 0, u1, depth - cr);
        L(u1, fr);
        A(flare, fr, 1, u1 + flare, 0);
        L(u1 + flare, -ov);
    }
    p.push("Z");
    return p.join(" ");
}

// The frame itself: the window rectangle minus a rounded inner rectangle
// (fill with OddEvenFill).
function frame(width, height, f, radius) {
    const r = Math.max(0, Math.min(radius, (f.r - f.l) / 2, (f.b - f.t) / 2));
    const n = v => v.toFixed(2);
    return `M 0 0 H ${n(width)} V ${n(height)} H 0 Z ` + `M ${n(f.l + r)} ${n(f.t)} H ${n(f.r - r)} A ${n(r)} ${n(r)} 0 0 1 ${n(f.r)} ${n(f.t + r)} ` + `V ${n(f.b - r)} A ${n(r)} ${n(r)} 0 0 1 ${n(f.r - r)} ${n(f.b)} ` + `H ${n(f.l + r)} A ${n(r)} ${n(r)} 0 0 1 ${n(f.l)} ${n(f.b - r)} ` + `V ${n(f.t + r)} A ${n(r)} ${n(r)} 0 0 1 ${n(f.l + r)} ${n(f.t)} Z`;
}
