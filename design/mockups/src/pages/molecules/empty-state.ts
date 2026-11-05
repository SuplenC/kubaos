// The EmptyState page: `docs/design/components/empty-state.md`. A place with nothing in it that
// isn't a list: a symbol, what is missing, what to do, and an action.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type EmptyStateOptions, emptyState } from "../../lib/empty-state.ts";
import { stack, within } from "../../lib/layout.ts";
import { emptyList } from "../../lib/list.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
import { WELL_ROOM } from "./list.ts";

const ROOM = "var(--size-column)";

const TITLE: Words = { short: "Nothing here yet", long: "Nothing here yet: this folder holds no pictures or videos to pick from" };
const BODY: Words = { short: "Drop some images in.", long: "Drop some images or videos in ~/Pictures/Wallpapers, or pick another folder above." };

const BLUETOOTH: EmptyStateOptions = { icon: "bluetooth_disabled", title: "Bluetooth is off", body: "Turn it on to find devices." };
const WALLPAPERS: EmptyStateOptions = { icon: "wallpaper", title: "Nothing here yet", body: "Drop some images or videos in ~/Pictures/Wallpapers." };
const RESULTS: EmptyStateOptions = { icon: "search_off", title: "No results", body: "Nothing matches that. Fewer letters, or check the spelling." };

/** `options` with its action, a Tonal button. */
function withAction(options: EmptyStateOptions, label: string): EmptyStateOptions {
  return { ...options, action: button({ label, kind: "Tonal" }) };
}

function inRoom(options: EmptyStateOptions): HTMLElement {
  return within(ROOM, emptyState(options));
}

const DESCRIBED =
  "Centred across its room: a 32 symbol (`icon.s32`, a symbol above text takes the size its place names) in `on_surface_variant`, `space.s12` under it the title in `subheading`, `space.s4` under that a `body` line in `on_surface_variant` saying what to do, and `space.s16` under that an optional action. Its words wrap, centred, in a column `size.column` wide at most.";

export default page<"empty">({
  level: "molecules",
  slug: "empty-state",
  title: "EmptyState",
  summary: "A place with nothing in it: a symbol, what is missing, what to do about it, and an action.",
  live: () => stack("s48", "centre", inRoom(withAction(BLUETOOTH, "Turn on")), inRoom(RESULTS)),
  description: [
    "Rust `empty::empty_state(look, glyph, title, body, action)`, QML `EmptyState` (`icon`, `title`, `body`, `action`). The control center's Bluetooth and Wi-Fi tabs with their radio off, the wallpaper picker with no pictures, the Command menu with nothing matching, its detail pane with nothing picked.",
    DESCRIBED,
    "It is for a place that isn't a list (a page, a panel, a picker, the Command menu's results), and says what is missing and what to do: never a bare \"No items\". Where something can fix it at once, the action is that thing, a Tonal button; the screen's one Filled action stays the screen's. A list's well with nothing in it is an EmptyList instead: one line in the well, no symbol, since the list round it already says what it holds.",
    "It isn't a stop for Tab; its action is, as any button. It sits in the page, so it isn't raised and has no box of its own.",
  ],
  variants: [
    { name: "With an action", for: "Something fixes it at once: Bluetooth turned on.", sample: () => inRoom(withAction(BLUETOOTH, "Turn on")) },
    { name: "Words alone", for: "Nothing here can fix it: a search with no results says how to search again.", sample: () => inRoom(RESULTS) },
    { name: "Where", for: "Where what is missing goes: the wallpaper picker names the folder to fill.", sample: () => inRoom(WALLPAPERS) },
    { name: "EmptyList instead", for: "A list's well with nothing in it: one line in the well, no symbol, no action.", sample: () => within(WELL_ROOM, emptyList("No devices found")) },
  ],
  sizes: doesntApply("one size: its symbol's 32 and its words' styles; its width is its room's, its column `size.column` at most."),
  states: {
    shown: ["empty"],
    absent: {
      rest: "with something in it, the place shows that instead.",
      lit: "it takes no pointer and no focus; its action lights as a button.",
      pressed: "its action takes the press.",
      disabled: "an empty place is never disabled.",
      selected: "it holds no choice.",
      loading: "a place still filling shows its skeletons or a spinner; it says it is empty only once it knows.",
      error: "a place that couldn't be filled shows an ErrorBlock: what broke and what to do.",
    },
    rows: [
      { name: "With an action", cell: () => inRoom(withAction(BLUETOOTH, "Turn on")) },
      { name: "Words alone", cell: () => inRoom(RESULTS) },
    ],
    described: {
      empty: "The symbol and the body in `on_surface_variant`, the title in `on_surface`, the action a Tonal button in its own states. By day the same roles in daylight colours.",
    },
  },
  focus: doesntApply("it takes no focus of its own; its action is a button, lit as every button is."),
  content: {
    text: [
      { name: "Its title", words: TITLE, sample: (words: string) => inRoom({ ...WALLPAPERS, title: words }) },
      { name: "What to do", words: BODY, sample: (words: string) => inRoom({ ...WALLPAPERS, body: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, named by its place: Rust names it by its enum, so it can't be missing."),
    counts: doesntApply("it is the place with none; one or more is the place itself."),
  },
  motion: {
    arrives: doesntApply("it shows in place at once when its place turns out empty, as what it replaces would have."),
    leaves: doesntApply("what arrives takes its place at once."),
    between: powerMoves(() => inRoom(withAction(BLUETOOTH, "Turn on"))),
  },
  keyboard: {
    keys: [{ keys: ["Enter", "Space"], does: "Presses its action, with the keyboard on it." }],
    order: "Its action is a stop for Tab in reading order; the rest of it isn't.",
    lands: "Where its place puts the keyboard: on the action when it is the place's first thing to do.",
  },
  measures: {
    samples: nonEmpty([{ name: "With an action", for: "The symbol, its gaps, the words and the action.", sample: () => inRoom(withAction(BLUETOOTH, "Turn on")) }]),
    measures: [
      { what: "The symbol.", token: "icon-s32" },
      { what: "Between the symbol and the title.", token: "space-s12" },
      { what: "Between the title and the line.", token: "space-s4" },
      { what: "Between the words and the action.", token: "space-s16" },
      { what: "The title's line, `subheading`'s.", token: "type-subheading-line" },
      { what: "The line's, `body`'s.", token: "type-body-line" },
      { what: "Its words' column at most.", token: "size-column" },
    ],
    reflow: {
      described: "It is centred in its room at every window size; its words wrap in a column `size.column` wide at most, or the room's width below it.",
      sample: () => within("100%", stack("s8", "start", emptyState({ ...WALLPAPERS, title: shown(TITLE), body: shown(BODY), action: button({ label: "Pick another folder", kind: "Tonal" }) }), text("An empty state in a room as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
