// The Toast page: `docs/design/components/toast.md`. One line saying what just happened, in the
// inverse colours at the bottom of the surface it confirms in: it rises in, stays, and fades out.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { onSurface, type ToastOptions, toast, toastRoom } from "../../lib/toast.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const SAID: Words = { short: "Copied.", long: "Copied the path of the file you saved to Downloads this morning." };

/** A window's content, where a toast shows: a column wide, a few rows tall. */
const WIDE = "var(--size-column)";
const TALL = "calc(3 * var(--space-s64))";
/** A sample's surface: a toast's place and a little of what it shows over. */
const SHORT = "calc(var(--space-s64) + var(--space-s32))";

const COPIED: ToastOptions = { words: "Copied. Evidence secured.", icon: "check" };
const FORGOT: ToastOptions = { words: "Forgot Home 5G.", undo: ["Ctrl+Z", "Undo"] };
const PLAIN: ToastOptions = { words: "Saved." };

function over(...options: ToastOptions[]): HTMLElement {
  return onSurface(WIDE, SHORT, ...options.map(toast));
}

/** A window with two actions that confirm themselves in a toast, and the key that undoes one. */
function live(): HTMLElement {
  const { room, say } = toastRoom();
  let forgotten = false;
  const actions = row(
    "s8",
    "centre",
    button({ label: "Copy the path", kind: "Tonal", size: "Sm" }),
    button({ label: "Forget Home 5G", kind: "Text", size: "Sm" }),
  );
  const [copy, forget] = actions.children as unknown as [HTMLElement, HTMLElement];
  copy.addEventListener("click", () => say(COPIED));
  forget.addEventListener("click", () => {
    forgotten = true;
    say(FORGOT);
  });
  const surface = onSurface(WIDE, TALL);
  surface.tabIndex = -1;
  surface.addEventListener("keydown", (event) => {
    if (!(event.ctrlKey && event.key.toLowerCase() === "z") || !forgotten) return;
    event.preventDefault();
    forgotten = false;
    say({ words: "Home 5G is back." });
  });
  const inside = stack("s8", "start", actions, text("Press one; Ctrl+Z brings Home 5G back.", "small", "on_surface_variant"));
  inside.classList.add("k-toast-inside");
  surface.append(inside, room);
  return surface;
}

const TOAST = "a toast";

const ARRIVES: Move = {
  name: "Arriving",
  what: "It fades in over `short` on `emphasized_decel` as it rises `space.s32` into its place, a whole device pixel at a time, so its edges stay sharp. A toast isn't a light: it never flickers. With reduced motion it fades in where it stays, with no rise.",
  tracks: [
    { part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "its rise", on: TOAST, target: ".k-toast", property: "--rise", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => over(COPIED),
};

const LEAVES: Move = {
  name: "Leaving",
  what: "After `duration.toast` (3.6 s, long enough to read its line twice; held while the pointer is on it, and counted again once it leaves), it fades out where it is over `medium` on `standard_accel`, something small leaving for good. With reduced motion the fade is capped at `short`; the stay isn't, since the words must still be read.",
  tracks: [{ part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 1, to: 0, duration: "medium", curve: "standard_accel" }],
  reduced: [{ part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  back: "cuts",
  sample: () => over(COPIED),
};

const REPLACED: Move = {
  name: "Another takes its place",
  what: "One shows at a time. A new one while one shows: the one showing fades out over `micro` on `standard_accel` where it is, as the new one arrives, fading and rising over `short`. With reduced motion the new one fades in with no rise.",
  tracks: [
    { part: "the one showing", on: "two toasts", target: ".k-toast:first-child", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard_accel" },
    { part: "the new one's fade", on: "two toasts", target: ".k-toast:last-child", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "the new one's rise", on: "two toasts", target: ".k-toast:last-child", property: "--rise", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the one showing", on: "two toasts", target: ".k-toast:first-child", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard_accel" },
    { part: "the new one's fade", on: "two toasts", target: ".k-toast:last-child", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
  ],
  back: "cuts",
  sample: () => over(COPIED, FORGOT),
};

const DESCRIBED =
  "One `body` line saying what just happened, in `surface` on `on_surface`, the inverse colours, so it reads over anything; an optional symbol (`body`'s 20) `space.s12` before the words; `space.s16` by `space.s12` inside; the surface radius (`radius.surface`, 24), which on one line is a pill. It is as wide as what it says, rounded up to a device pixel, up to its room less `space.s24` at either side; past that its words wrap and its corners stay the surface's.";

export default page({
  level: "molecules",
  slug: "toast",
  title: "Toast",
  summary: "One line saying what just happened, at the bottom of the surface it happened in: it rises in, stays, and goes by itself.",
  live,
  description: [
    "Rust `toast::toast` and `toast::toast_at`, QML `Toast`. Press an action: its toast rises in at the bottom, stays, and fades; press the other while it shows and the new one takes its place. After Forget, Ctrl+Z with the keyboard in the window brings Home 5G back.",
    DESCRIBED,
    "It confirms something just done that left no other trace on screen (`docs/DESIGN.md`, \"Notify or show inline\"): Copied, Forgot. It shows at the bottom of the surface it happened in (Kōbō's window, the Command menu's list), centred across it on a whole device pixel, `space.s24` above the edge, over what is there, moving nothing. It never asks for anything and never takes the keyboard. Where what it says can be undone, the surface's own key for it shows as key hints `space.s16` after the words, in `surface`: the keys as keycaps, Undo in `caption`; the toast itself takes no key.",
    "It arrives over `short`, fading in as it rises `space.s32` into place (`emphasized_decel`), stays `duration.toast` (3.6 s), and leaves over `medium`, fading where it is (`standard_accel`). While the pointer is on it, it stays, and counts its stay again once the pointer leaves. One shows at a time: a new one takes the place of the one showing, which fades out over `micro`.",
  ],
  variants: [
    { name: "Words", for: "What just happened, alone.", sample: () => over(PLAIN) },
    { name: "With a symbol", for: "Where a symbol says it faster: a tick on Copied.", sample: () => over(COPIED) },
    { name: "With the keys that undo it", for: "What it says can be undone: the surface's key for it, as key hints after the words.", sample: () => over(FORGOT) },
  ],
  sizes: doesntApply("a toast has one size: its words' line and its padding; it is as wide as what it says, up to its room."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "it takes no focus, and the pointer on it only holds it, with no change of look.",
      pressed: "a press on it does nothing: it goes by itself.",
      disabled: "a toast is there or not.",
      selected: "it holds no choice.",
      loading: "it says what is done; work still going on is a progress bar or a spinner where it happens.",
      empty: "a toast with nothing to say isn't shown.",
      error: "a toast confirms: what went wrong shows inline where it happened, or in a notification when you are elsewhere.",
    },
    rows: [
      { name: "Words", cell: () => over(PLAIN) },
      { name: "With a symbol", cell: () => over(COPIED) },
      { name: "With keys", cell: () => over(FORGOT) },
    ],
    described: {
      rest: "Showing: `on_surface` behind its words, symbol and key hints' words in `surface`, the keycaps their own colours. By day the same roles in daylight colours: a dark pill on a light surface, where at night it is a light one on a dark surface.",
    },
  },
  focus: doesntApply("a toast never takes the keyboard and isn't neon: it has no line and no glow."),
  content: {
    text: [{ name: "What it says", words: SAID, sample: (words: string) => over({ words }) }],
    missingPicture: doesntApply("its symbol is optional and one of KubaOS Symbols; a toast without one is its words alone."),
    counts: [
      { name: "None", for: "Nothing just happened: no toast, nothing kept for it.", sample: () => onSurface(WIDE, SHORT) },
      { name: "One", for: "What just happened.", sample: () => over(COPIED) },
      { name: "Another while one shows", for: "The new one takes the place of the one showing, which fades out over `micro`: only the last thing done is said.", sample: () => over(FORGOT) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [LEAVES],
    between: [REPLACED],
  },
  keyboard: {
    keys: [{ keys: ["Ctrl+Z"], does: "Its surface's own key, shown on a toast whose words can be undone: it undoes them, and its own toast says so, taking this one's place." }],
    order: "No stop: a toast never takes the keyboard, which stays where it was.",
    lands: "Nowhere: the keyboard stays on what was done, so the next key goes on from there.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a symbol", for: "Its padding, the symbol and its gap, and its place in the surface.", sample: () => over(COPIED) },
      { name: "With keys", for: "The key hints after the words.", sample: () => over(FORGOT) },
    ]),
    measures: [
      { what: "Inside its sides, and between the words and the key hints.", token: "space-s16" },
      { what: "Above and below its words, and between the symbol and the words.", token: "space-s12" },
      { what: "Its words, `body`'s line.", token: "type-body-line" },
      { what: "The symbol, `body`'s.", token: "type-body-icon" },
      { what: "A keycap.", token: "size-keycap" },
      { what: "Its corners.", token: "radius-surface" },
      { what: "Above the surface's bottom edge, and the least room at either side.", token: "space-s24" },
      { what: "How far under its place it starts as it arrives.", token: "space-s32" },
    ],
    reflow: {
      described: "A toast is as wide as what it says, up to its surface less `space.s24` at either side, and centred across it at every window size; past that its words wrap onto more lines, the pill becoming a rounded box.",
      sample: () => within("100%", stack("s8", "start", onSurface("100%", SHORT, toast({ words: shown(SAID), icon: "check" })), text("A toast in a window as wide as this one.", "small", "on_surface_variant"))),
    },
  },
});
