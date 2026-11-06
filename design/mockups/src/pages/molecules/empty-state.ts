// The EmptyState page: `docs/design/components/empty-state.md`. A place with nothing in it that
// isn't a list: a symbol, what is missing, what to do, and an action.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type EmptyStateOptions, emptyState } from "../../lib/empty-state.ts";
import { filling, flow, within } from "../../lib/layout.ts";
import { emptyList } from "../../lib/list.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { WELL_ROOM } from "./list.ts";

/** The stage's pane, less the room a lit control's glow throws past its edge, which it would otherwise scale down for. */
const ACROSS = "calc(100% - var(--space-s24))";

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

export default page<"empty">({
  level: "molecules",
  slug: "empty-state",
  title: "EmptyState",
  lede: "A place with nothing in it that isn't a list: a symbol, what is missing, what to do, and an action.",
  code: { rust: "empty::empty_state", qml: "EmptyState" },
  stage: {
    live: () => within(ACROSS, filling(flow("s32", within(WELL_ROOM, emptyState(withAction(BLUETOOTH, "Turn on"))), within(WELL_ROOM, emptyState(RESULTS))))),
    hint: "Tab to its action, or press it",
    daylight: true,
  },
  variants: [
    { name: "With an action", for: "something fixes it at once, a Tonal button", sample: () => inRoom(withAction(BLUETOOTH, "Turn on")) },
    { name: "Words alone", for: "nothing here can fix it", sample: () => inRoom(RESULTS) },
    { name: "Where", for: "names where what is missing goes", sample: () => inRoom(WALLPAPERS) },
    { name: "EmptyList instead", for: "a list's well: one line, no symbol", sample: () => within(WELL_ROOM, emptyList("No devices found")) },
  ],
  sizes: doesntApply("one size; its words' column `size.column` at most."),
  states: {
    shown: ["empty"],
    absent: {
      rest: "with something in it, the place shows that",
      lit: "takes no pointer or focus; its action lights",
      pressed: "its action takes the press",
      disabled: "never disabled",
      selected: "holds no choice",
      loading: "skeletons or a spinner until it knows",
      error: "couldn't be filled: an `ErrorBlock`",
    },
    rows: [
      { name: "With an action", cell: () => inRoom(withAction(BLUETOOTH, "Turn on")) },
      { name: "Words alone", cell: () => inRoom(RESULTS) },
    ],
    looks: {
      empty: "symbol and line `on_surface_variant`, title `on_surface`",
    },
  },
  focus: doesntApply("no focus of its own; its action is a button."),
  content: {
    text: [
      { name: "Its title", words: TITLE, sample: (words: string) => inRoom({ ...WALLPAPERS, title: words }) },
      { name: "What to do", words: BODY, sample: (words: string) => inRoom({ ...WALLPAPERS, body: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, named by its place."),
    counts: doesntApply("it is the place with none; one or more is the place itself."),
  },
  motion: {
    arrives: doesntApply("it shows at once when its place turns out empty."),
    leaves: doesntApply("what arrives takes its place at once."),
    between: powerMoves(() => inRoom(withAction(BLUETOOTH, "Turn on"))),
  },
  keyboard: {
    keys: [{ keys: ["Enter", "Space"], does: "presses its action" }],
    order: "Its action is a stop in reading order; the rest isn't.",
    lands: "Where its place puts the keyboard.",
  },
  measures: {
    samples: nonEmpty([{ name: "With an action", for: "symbol, gaps, words, action", sample: () => inRoom(withAction(BLUETOOTH, "Turn on")) }]),
    measures: [
      { part: "Symbol", token: "icon-s32" },
      { part: "Symbol → title", token: "space-s12" },
      { part: "Title → line", token: "space-s4" },
      { part: "Words → action", token: "space-s16" },
      { part: "Title line", token: "type-subheading-line" },
      { part: "Body line", token: "type-body-line" },
      { part: "Words' column at most", token: "size-column" },
    ],
    reflow: {
      note: "Centred in its room; its words wrap in `size.column` at most.",
      sample: () => within("100%", emptyState({ ...WALLPAPERS, title: shown(TITLE), body: shown(BODY), action: button({ label: "Pick another folder", kind: "Tonal" }) })),
    },
  },
});
