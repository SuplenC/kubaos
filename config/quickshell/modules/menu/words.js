.pragma library

// What the menu writes about an Item that its Extension doesn't: the symbol standing for it, its
// Extension as a label, and its Actions' keys as Kotodama's key hints write them.

// The KubaOS Symbol standing for an Extension's Items when they have no icon of their own.
const glyphs = {
    apps: "desktop_windows",
    calculator: "calculate",
    clipboard: "content_paste",
    emoji: "mood",
    folders: "folder",
    hyprland: "keyboard_command_key",
    system: "settings_power",
    "web-search": "travel_explore"
};

function glyph(extension) {
    return glyphs[extension] ?? "bolt";
}

// An Extension's name as a label: `web-search` reads "Web search".
function kindLabel(extension) {
    const words = extension.replace(/[-_.]/g, " ");
    return words.charAt(0).toUpperCase() + words.slice(1);
}

const keys = {
    backspace: "⌫",
    delete: "⌦",
    return: "↵",
    enter: "↵",
    escape: "Esc",
    esc: "Esc",
    tab: "Tab",
    space: "Space",
    up: "↑",
    down: "↓",
    left: "←",
    right: "→"
};

const modifiers = {
    ctrl: "⌃",
    control: "⌃",
    shift: "⇧",
    alt: "⌥",
    super: "Super",
    meta: "Super"
};

// An Action's shortcut as its keys, one keycap each, written as Kotodama's key hints write them:
// `ctrl+l` is `⌃` and `L`, as `⌃K Actions` is, and `ctrl+shift+backspace` `⌃`, `⇧` and `⌫`.
function shortcutKeys(shortcut) {
    if (shortcut === "")
        return [];
    return shortcut.split("+").map(part => {
        const name = part.toLowerCase();
        const known = modifiers[name] ?? keys[name];
        if (known !== undefined)
            return known;
        return part.length === 1 ? part.toUpperCase() : part.charAt(0).toUpperCase() + part.slice(1);
    });
}
