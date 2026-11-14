// The Toast page: `docs/design/components/toast.md`. One line saying what just happened, in the
// inverse colours at the bottom of the surface it confirms in: it rises in, stays, and fades out.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
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
  const inside = stack("s8", "start", actions);
  inside.classList.add("k-toast-inside");
  surface.append(inside, room);
  return surface;
}

const TOAST = "a toast";

const ARRIVES: Move = {
  name: "Arriving",
  what: "It fades in over `short` on `emphasized_decel`, rising `space.s32` into place; no rise with reduced motion.",
  tracks: [
    { part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "its rise", on: TOAST, target: ".k-toast", property: "--toast-rise", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => over(COPIED),
};

const LEAVES: Move = {
  name: "Leaving",
  what: "After `duration.toast`, held under the pointer, it fades out where it is over `medium` on `standard_accel`.",
  tracks: [{ part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 1, to: 0, duration: "medium", curve: "standard_accel" }],
  reduced: [{ part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  back: "cuts",
  sample: () => over(COPIED),
};

const REPLACED: Move = {
  name: "Another takes its place",
  what: "One at a time: the one showing fades over `micro` as the new one arrives over `short`.",
  tracks: [
    { part: "the one showing", on: "two toasts", target: ".k-toast:first-child", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard_accel" },
    { part: "the new one's fade", on: "two toasts", target: ".k-toast:last-child", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "the new one's rise", on: "two toasts", target: ".k-toast:last-child", property: "--toast-rise", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the one showing", on: "two toasts", target: ".k-toast:first-child", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard_accel" },
    { part: "the new one's fade", on: "two toasts", target: ".k-toast:last-child", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
  ],
  back: "cuts",
  sample: () => over(COPIED, FORGOT),
};

export default page({
  level: "molecules",
  slug: "toast",
  title: "Toast",
  lede: "One line saying what just happened, at the bottom of the surface it happened in: it rises in and goes by itself.",
  code: { rust: "toast::toast, toast::toast_at", qml: "Toast", unbuilt: "rise, held stay: not built yet" },
  stage: { live, hint: "press an action; Ctrl+Z after Forget", daylight: true },
  variants: [
    { name: "Words", for: "what just happened, alone", sample: () => over(PLAIN) },
    { name: "With a symbol", for: "a symbol says it faster: Copied", sample: () => over(COPIED) },
    { name: "With the keys that undo it", for: "it can be undone: the surface's key as hints", sample: () => over(FORGOT) },
  ],
  sizes: doesntApply("one size: its line and padding, as wide as what it says."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "the pointer only holds it",
      pressed: "a press does nothing",
      disabled: "there or not",
      selected: "it holds no choice",
      loading: "work going on is a bar or spinner",
      empty: "with nothing to say it isn't shown",
      error: "it confirms: errors show inline or in a notification",
    },
    rows: [
      { name: "Words", cell: () => over(PLAIN) },
      { name: "With a symbol", cell: () => over(COPIED) },
      { name: "With keys", cell: () => over(FORGOT) },
    ],
    looks: {
      rest: "inverse: `surface` on `on_surface`, step 2's shadow and rim",
    },
  },
  focus: doesntApply("a toast never takes the keyboard and isn't neon: no line, no glow."),
  content: {
    text: [{ name: "What it says", words: SAID, sample: (words: string) => over({ words }) }],
    missingPicture: doesntApply("its symbol is optional; without one it is its words alone."),
    counts: [
      { name: "None", for: "no toast, nothing kept for it", sample: () => onSurface(WIDE, SHORT) },
      { name: "One", for: "what just happened", sample: () => over(COPIED) },
      { name: "Another while one shows", for: "only the last thing done is said", sample: () => over(FORGOT) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [LEAVES],
    between: [REPLACED],
  },
  keyboard: {
    keys: [{ keys: ["Ctrl+Z"], does: "its surface's key: undoes it, and says so in a toast" }],
    order: "No stop: a toast never takes the keyboard.",
    lands: "Nowhere: the keyboard stays on what was done.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a symbol", for: "padding, symbol and gap, its place", sample: () => over(COPIED) },
      { name: "With keys", for: "the key hints after the words", sample: () => over(FORGOT) },
    ]),
    measures: [
      { part: "Words → key hints", token: "space-s16" },
      { part: "Padding, symbol → words", token: "space-s12" },
      { part: "Words' line", token: "type-body-line" },
      { part: "Symbol", token: "type-body-icon" },
      { part: "Keycap", token: "size-keycap" },
      { part: "Corners", token: "radius-control" },
      { part: "Rim", token: "stroke-hairline_device_px" },
      { part: "Shadow contact blur", token: "elevation-2-contact-blur" },
      { part: "Shadow ambient drop", token: "elevation-2-ambient-y" },
      { part: "Shadow ambient blur", token: "elevation-2-ambient-blur" },
      { part: "Above the edge, side room", token: "space-s24" },
      { part: "Rise as it arrives", token: "space-s32" },
    ],
    reflow: {
      note: "As wide as what it says, up to its surface less `space.s24` a side, centred; past that its words wrap.",
      sample: () => within("100%", onSurface("100%", SHORT, toast({ words: shown(SAID), icon: "check" }))),
    },
  },
});
