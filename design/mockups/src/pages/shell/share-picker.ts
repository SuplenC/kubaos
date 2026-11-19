// Picking what to share: the system's share prompt, which an app's request to share opens in place
// of the portal's own. No dialog, no dim: while the request waits the desktop works as usual, and
// a click on a window only gives it the keyboard. Each screen's frame grows a bar out of the middle
// of its bottom edge (who asked, Share this screen, Region, Cancel), on the Sheet the shell's
// attached surfaces use, the standing sign that a request waits. The window under the pointer
// offers a violet Share pill riding its top border. Pointing at a way to share previews the mark
// it keeps on air. Region alone is modal: it dims the screen for drawing. Its parts are
// `sharing-parts.ts`'.

import type { Words } from "../../lib/content.ts";
import { type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Strike, type Track } from "../../site/page.ts";
import { HOLD, keyframes, loop, milliseconds } from "../../site/recipe.ts";
import { ALL } from "../../site/settings.ts";
import {
  DISCORD,
  DP10,
  EDP1,
  DP10_THIRD,
  type Lit,
  type Monitor,
  MEET,
  NO_ICON,
  MONITORS,
  NARROW,
  NOTHING,
  pickBar,
  type Picking,
  picking,
  buildPicking,
  type PickingOptions,
  REGION,
  type WindowAt,
} from "./sharing-parts.ts";

/** Picking on both screens, as `options` say. */
const both = (options: PickingOptions = {}) => picking(options);
/** Picking on DP-10 alone, for the samples that compare states. */
const one = (options: PickingOptions = {}) => picking({ monitors: [DP10], ...options });

/** Firefox's window on DP-10, the one the samples point at. */
const FIREFOX_WINDOW: WindowAt = { screen: 0, index: 1 };

/** Plays the move to sharing on `root` from its start to its last state, as its player does. */
function playShared(root: HTMLElement): void {
  const { timed, total } = loop(SHARED);
  for (const each of timed) {
    for (const target of root.querySelectorAll<HTMLElement>(each.track.target)) {
      target.animate(keyframes(each, total), { duration: total, delay: -milliseconds(HOLD), fill: "forwards" });
    }
  }
}

/**
 * A request waiting live on `monitors`: a click on a window gives it the keyboard; its Share pill,
 * on the window under the pointer, shares it; Share this screen shares the screen; Region
 * draws, Enter sharing the region and Esc going back to waiting; × cancels. The screens are built
 * once as the request arrives and only change attributes after that; shared, the move to sharing
 * plays as its recipe says; cancelled, the desktop is left as it was. A double-click on the
 * desktop asks again.
 */
function livePicking(monitors: readonly Monitor[]): HTMLElement {
  const holder = document.createElement("span");
  // A box of its own, as wide as what it holds: the stage measures a sample by its box, and one
  // with none (`contents`) reads as reaching from the page's edge.
  holder.style.display = "inline-block";
  let lit: Lit = NOTHING;
  let focused: WindowAt = { screen: 0, index: 0 };
  let picker: Picking | null = null;

  /** The keyboard to the region being drawn, a frame later while it isn't on the page yet. */
  const toDrawing = () => {
    if (ALL || !picker) return;
    const shown = picker.element;
    const take = () => shown.querySelector<HTMLElement>(".share-draw")?.focus({ preventScroll: true });
    if (holder.isConnected) take();
    else requestAnimationFrame(take);
  };

  const light = (next: Lit) => {
    if (JSON.stringify(next) === JSON.stringify(lit)) return;
    lit = next;
    picker?.light(lit);
    // Drawing takes the keyboard; it goes back to the app as drawing ends.
    if (lit.kind === "region") toDrawing();
  };

  const settled = (shown: HTMLElement) => {
    picker = null;
    shown.addEventListener("dblclick", ask);
    holder.replaceChildren(shown);
  };

  const share = (picked: Lit) => {
    if (picked.kind === "none") return;
    const shown = picking({ monitors, lit: picked, pill: true, focused });
    settled(shown);
    if (!ALL) playShared(shown);
  };

  // Cancelled, each bar sinks back into its edge, as the Sheet leaves, before the desktop is left as it was.
  const cancel = () => {
    const leaving = picker;
    if (!leaving || ALL) return settled(picking({ monitors, phase: "shared", focused }));
    leaving.close();
    window.setTimeout(() => {
      if (picker === leaving) settled(picking({ monitors, phase: "shared", focused }));
    }, milliseconds("medium"));
  };

  const focus = (window: WindowAt) => {
    focused = window;
    picker?.focus(window);
  };

  function ask(): void {
    lit = NOTHING;
    picker = buildPicking({ monitors, arrive: !ALL, focused, handlers: { onLight: light, onShare: share, onFocus: focus, onCancel: cancel } });
    picker.light(lit);
    holder.replaceChildren(picker.element);
    // Built shut, it arrives on the next frame: each bar's Sheet opens.
    const arriving = picker;
    if (!ALL) requestAnimationFrame(() => arriving.open());
  }
  ask();
  return holder;
}

/**
 * The stage: a request waiting live on DP-10 alone, so the stage draws one screen large enough to
 * read; the bar is on every screen, as the two screens beside each other show live among the sizes.
 */
const live = () => livePicking([DP10]);

const BARS = "the bars";

/** The bars' Sheets: their depth, as every attached surface moves; under reduced motion, a fade. */
const SHEETS = ".k-sheet-part";

/** The bars rising out of their edges, or sinking back into them. */
function bars(on: boolean, reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : on ? "long" : "medium";
  const curve = on ? "emphasized_decel" : "emphasized_accel";
  const [from, to] = on ? [0, 1] : [1, 0];
  return [
    reduced
      ? { part: BARS, on: "every screen", target: SHEETS, property: "opacity", from, to, duration, curve }
      : { part: BARS, on: "every screen", target: SHEETS, property: "--reveal", from, to, duration, curve },
  ];
}

/** The request giving way to the share: the Share pill fades, the bars sink, the mark stays, the pill arrives. */
function onAir(power: Strike | Easing, reduced: boolean): NonEmpty<Track> {
  const pill = ".shell-on-air";
  const offer = ".share-pill";
  if (reduced) {
    return [
      { part: "the Share pill", on: "the window", target: offer, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
      { part: BARS, on: "every screen", target: SHEETS, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
      { part: "the pill", on: "every bar", target: pill, property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    ];
  }
  return [
    { part: "the Share pill", on: "the window", target: offer, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard_accel" },
    { part: BARS, on: "every screen", target: SHEETS, property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the pill's power", on: "every bar", target: pill, property: "opacity", from: 0, to: 1, duration: "medium", cue: "medium", curve: power },
    { part: "the pill's beat lit", on: "every bar", target: pill, property: "--lit", from: 0, to: 1, duration: "short", cue: ["medium", "medium"], curve: "standard" },
    { part: "the pill's beat lit", on: "every bar", target: pill, property: "--lit", from: 1, to: 0, duration: "long", cue: ["medium", "medium", "short", "scene"], curve: "standard" },
  ];
}

/** Firefox's window shared by its pill, the pill in every bar: what the move to sharing plays on, the pill out at its start. */
function aboutToShare(): HTMLElement {
  return both({ lit: { kind: "window", ...FIREFOX_WINDOW }, pill: true });
}

const SHARED: Move = {
  name: "Shared, then on air",
  what: "A click on Share: the Share pill fades, the bars sink; the line stays; the bar's pill strikes on.",
  tracks: onAir("neon.strike", false),
  reduced: onAir("standard", true) as NonEmpty<Track<Easing>>,
  unflickered: onAir("standard", false) as NonEmpty<Track<Easing>>,
  back: "cuts",
  sample: screen(aboutToShare),
};

type Shown = "rest" | "lit" | "pressed";

/** The asking app's name, as its icon's Tooltip says it. */
const ASKER_NAME: Words = { short: "Zoom", long: "Visual Studio Code - Insiders" };

/** The window the pointer is on, its pill at rest; the pill pointed at, lit; pressed, shared. */
function windowState(state: Shown): HTMLElement {
  if (state === "pressed") return one({ lit: { kind: "window", ...FIREFOX_WINDOW }, phase: "shared" });
  return one({ pointed: { ...FIREFOX_WINDOW, pill: state === "lit" } });
}

export default page<Shown>({
  level: "shell",
  slug: "share-picker",
  title: "Share picker",
  lede: "A share request on the live desktop: a bar on each screen, Share on the pointed window.",
  code: { qml: "SharePicker", unbuilt: "Not built yet" },
  from: ["icon-tile", "button", "icon-button", "tooltip", "chip"],
  stage: { live, hint: "point at a window, then its Share pill; a click focuses it", daylight: true },
  variants: [
    { name: "Waiting", for: "the desktop as usual, no dim; a bar on each screen", sample: screen(() => both()) },
    { name: "A window pointed at", for: "its Share pill riding its top border", sample: screen(() => both({ pointed: FIREFOX_WINDOW })) },
    { name: "Its Share pill pointed at", for: "filled violet; the line runs on behind it", sample: screen(() => both({ pointed: { ...FIREFOX_WINDOW, pill: true } })) },
    { name: "This screen pointed at", for: "its violet edge previewed", sample: screen(() => both({ lit: { kind: "screen", screen: 0 } })) },
    { name: "A region being drawn", for: "the one dim; clear inside its line; its size", sample: screen(() => both({ lit: { kind: "region", screen: 0, rect: REGION } })) },
    { name: "A window clicked", for: "it takes the keyboard; nothing is shared", sample: screen(() => both({ focused: FIREFOX_WINDOW })) },
    { name: "Pointed on the other screen", for: "the pill on the window under the pointer", sample: screen(() => both({ pointed: { screen: 1, index: 0 } })) },
    { name: "An app asks", for: "the app's icon in each bar", sample: screen(() => both({ asker: DISCORD })) },
    { name: "Who asked, its hint up", for: "the app's icon names it on hover", sample: screen(() => one({ hint: "app" })) },
    { name: "Region's hint up", for: "its name alone", sample: screen(() => one({ hint: "region" })) },
    { name: "Another workspace", for: "switched while waiting: the bar stays", sample: screen(() => both({ monitors: [DP10_THIRD, EDP1] })) },
    { name: "Shared", for: "the bars gone; mark and pill stay", sample: screen(() => both({ lit: { kind: "window", ...FIREFOX_WINDOW }, phase: "shared" })) },
  ],
  sizes: [
    { name: "Two screens, two sizes", for: "live: a bar on each; Share on either", sample: screen(() => livePicking(MONITORS)) },
    { name: "The narrowest screen", for: "720 by 480: the one row fits whole", sample: screen(() => picking({ monitors: [NARROW] })) },
    { name: "The bar alone", for: "one row, as wide as its content", sample: () => pickBar(MEET) },
  ],
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "a request waits or is gone",
      selected: "what is pointed at is the pick",
      loading: "the bars are there as it opens",
      empty: "a screen with no window still has its bar",
      error: "a failed share ends; a notification says so",
    },
    rows: [
      { name: "A window", cell: (state: Shown) => (state === "pressed" ? one({ focused: FIREFOX_WINDOW }) : one(state === "lit" ? { pointed: FIREFOX_WINDOW } : {})) },
      { name: "Its Share pill", cell: windowState },
      {
        name: "Share this screen",
        cell: (state: Shown) => (state === "pressed" ? one({ lit: { kind: "screen", screen: 0 }, phase: "shared" }) : one(state === "lit" ? { lit: { kind: "screen", screen: 0 } } : {})),
      },
      { name: "Region", cell: (state: Shown) => (state === "pressed" ? one({ lit: { kind: "region", screen: 0, rect: REGION } }) : one(state === "lit" ? { hint: "region" } : {})) },
    ],
    looks: {
      rest: "waiting: the desktop as usual, the bar out of its edge",
      lit: "pointed at: the Share pill shows, or a line previews",
      pressed: "a window focused; shared; or a region drawn",
    },
  },
  focus: {
    tubes: [
      { name: "The window with the keyboard", tone: "accent", filled: false, content: false },
      { name: "The bar's buttons", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "a window's tube; a button's tube and glow", signsOff: "the tubes as glass; the violet line stays" },
    signsOff: (index: number) => (index === 0 ? one({ focused: FIREFOX_WINDOW }) : pickBar(MEET, { lit: "screen" })),
  },
  content: {
    text: [{ name: "Who asked, in its hint", words: ASKER_NAME, sample: (words: string) => pickBar({ ...MEET, app: words }, { hint: "app" }) }],
    missingPicture: [
      { name: "An asking app with no icon", for: "its initial on the bar's tile", sample: () => pickBar(NO_ICON, { hint: "app" }) },
      { name: "A window's app with no icon", for: "its initial on the window's tile", sample: screen(() => both({ pointed: { screen: 1, index: 1 } })) },
    ],
    counts: [
      { name: "One screen", for: "one bar", sample: screen(() => one()) },
      { name: "Two screens", for: "a bar on each", sample: screen(() => both()) },
      { name: "A screen with no window", for: "only its bar and the wallpaper", sample: screen(() => picking({ monitors: [DP10, { ...EDP1, windows: [] }] })) },
    ],
  },
  motion: {
    arrives: [
      { name: "Asked", what: "An app asks: each bar rises out of its screen's bottom edge; nothing dims.", tracks: bars(true, false), reduced: bars(true, true), back: "cuts", sample: screen(() => both()) },
      {
        name: "Drawing a region",
        what: "Region: the screen's dim fades in over `short`, the one modal moment.",
        tracks: [{ part: "the dim", on: "the screen", target: ".share-draw", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: screen(() => one({ lit: { kind: "region", screen: 0 } })),
      },
    ],
    leaves: [{ name: "Cancelled", what: "×: each bar sinks back into its edge.", tracks: bars(false, false), reduced: bars(false, true), back: "cuts", sample: screen(() => both()) }, SHARED],
    between: [
      {
        name: "A window's pill",
        what: "The pointer onto a window: its Share pill fades in over `short`, and out as it leaves.",
        tracks: [{ part: "the pill", on: "the window", target: ".share-window[data-pointed] > .share-pill", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: screen(() => one({ pointed: FIREFOX_WINDOW })),
      },
      {
        name: "A window previewed",
        what: "Its Share pill pointed at: it fills violet and the window's line fades in over `short`.",
        tracks: [{ part: "the line", on: "the window", target: ".share-window[data-pointed] .share-border", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: screen(() => one({ pointed: { ...FIREFOX_WINDOW, pill: true } })),
      },
      {
        name: "This screen previewed",
        what: "Share this screen pointed at: its violet edge fades in over `short`.",
        tracks: [{ part: "the edge", on: "the screen", target: ".share-edge", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: screen(() => one({ lit: { kind: "screen", screen: 0 } })),
      },
      {
        name: "A window focused",
        what: "A click on a window: its tube fades in over `short`, the last one's out.",
        tracks: [{ part: "the tube", on: "the window", target: ".share-window[data-pinned] > .k-ring", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: screen(() => one({ focused: FIREFOX_WINDOW })),
      },
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "while drawing: shares the region" },
      { keys: ["Esc"], does: "while drawing: back to waiting" },
    ],
    order: "None while waiting: the keyboard stays with the apps.",
    lands: "Region hands the drawing the keyboard; Esc hands it back to the app.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Its Share pill pointed at", for: "the pill on the border, the line", sample: screen(() => one({ pointed: { ...FIREFOX_WINDOW, pill: true } })) },
      { name: "The bar alone", for: "the icon, the buttons, their gaps", sample: () => pickBar(MEET) },
    ]),
    measures: [
      { part: "Bar's padding, on its Sheet", token: "space-s16" },
      { part: "Sheet's fillets into the edge", token: "radius-surface" },
      { part: "Between the bar's parts", token: "space-s8" },
      { part: "Who asked, its tile", token: "size-control_sm" },
      { part: "Buttons", token: "size-control_sm" },
      { part: "Share pill's height", token: "size-control_sm" },
      { part: "Share pill's corners", token: "radius-full" },
      { part: "Share pill, past the corner", token: "space-s8" },
      { part: "Share pill's ends", token: "space-s12" },
      { part: "Share pill's symbol to word", token: "space-s4" },
      { part: "Drawing's dim", token: "opacity-panel_scrim" },
      { part: "A window's line", token: "stroke-thin" },
      { part: "A screen's edge", token: "stroke-thin" },
    ],
    reflow: { note: "The bar keeps to the middle of the bottom edge, one row at every width.", sample: screen(() => picking({ monitors: [NARROW] })) },
  },
});
