.pragma library

// What the components page's sections share, as the iced page's modules share it
// (`crates/kubaos-specimen/src/pages/components/buttons.rs` and `media.rs`).

// The states a button is shown in, left to right.
var stateNames = ["Resting", "Hovered", "Pressed", "Focused", "Disabled"];
var kinds = ["Filled", "Tonal", "Text", "Danger"];
var sizes = ["Sm", "Md", "Lg"];
// The columns of the button tables: the kinds' names, then one per state.
var columns = [1, 2, 2, 2, 2, 2];

// A button size's name over its table, then its height from `metrics`, `K.ButtonMetrics`, which a
// library can't import.
function sizeName(size, metrics) {
    return `${size} · ${metrics.of(size).height}`;
}

// The picture the media samples load: `from` fading into `to`, the accent into the cold accent, as
// the iced page draws it.
function picture(from, to) {
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 10"><defs><linearGradient id="g"><stop offset="0" stop-color="${from.toString()}"/><stop offset="1" stop-color="${to.toString()}"/></linearGradient></defs><rect width="16" height="10" fill="url(#g)"/></svg>`;
    return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}
