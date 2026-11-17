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

// An Item's icon that is a KubaOS Symbol rather than a picture: `symbol:<name>`, as the KubaOS
// Extension gives an app with no logo.
const symbolPrefix = "symbol:";

// The symbol an Item's tile draws: the one its icon names, else its Extension's.
function itemGlyph(icon, extension) {
    const named = icon.startsWith(symbolPrefix) ? icon.slice(symbolPrefix.length) : "";
    return named !== "" ? named : glyph(extension);
}

// An Item's icon that is an app's initial: `initial:<name>`, as the KubaOS Extension gives an app
// of the catalog with no logo and no symbol, its Item's title being "Install <name>".
const initialPrefix = "initial:";

// The symbol a Result row's tile draws, as `itemGlyph`, but none for an app that names none: a
// desktop app, or one of the catalog whose icon is its initial. Its tile shows the initial of
// `rowName` instead, its picture loading too.
function tileGlyph(icon, extension) {
    if (icon.startsWith(initialPrefix) || (extension === "apps" && !icon.startsWith(symbolPrefix)))
        return "";
    return itemGlyph(icon, extension);
}

// The name a Result row's tile takes its initial from: the one an `initial:` icon carries, else
// the Item's title, a desktop app's name.
function rowName(icon, title) {
    return icon.startsWith(initialPrefix) ? icon.slice(initialPrefix.length) : title;
}

// Where an Item's picture is: a path as a file URL, an icon theme's name looked up; none for no
// icon, a symbol or an initial.
function iconSource(icon, lookUp) {
    if (icon === "" || icon.startsWith(symbolPrefix) || icon.startsWith(initialPrefix))
        return "";
    return icon.startsWith("/") ? `file://${icon}` : lookUp(icon);
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
