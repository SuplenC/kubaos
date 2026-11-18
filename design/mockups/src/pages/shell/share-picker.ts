// Picking what to share: the system's share prompt, which an app's request to share opens in place
// of the portal's own. No dialog: every screen dims a little and holds a bar at its bottom left
// (who asks, Share this screen, Region, Cancel), a window is picked by pointing at it, and a region
// is drawn after Region or by a right-button drag anywhere. What lights while picking is the mark
// it keeps on air. Its parts are `sharing-parts.ts`'.

import type { Words } from "../../lib/content.ts";
import { type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Strike, type Track } from "../../site/page.ts";
import { HOLD, keyframes, loop, milliseconds } from "../../site/recipe.ts";
import { ALL } from "../../site/settings.ts";
import {
  DISCORD,
  DP10,
  EDP1,
  DP10_THIRD,
  LONG_ASKER,
  type Lit,
  MEET,
  MONITORS,
  NARROW,
  NOTHING,
  pickBar,
  picking,
  type PickingOptions,
  REGION,
} from "./sharing-parts.ts";

/** Picking on both screens, as `options` say. */
const both = (options: PickingOptions = {}) => picking(options);
/** Picking on DP-10 alone, for the samples that compare states. */
const one = (options: PickingOptions = {}) => picking({ monitors: [DP10], ...options });

/** Every place Tab stops on, in order: each screen's windows, then its bar's Share this screen. */
function stops(): Lit[] {
  return MONITORS.flatMap((monitor, screenAt) => [
    ...monitor.windows.map((_, index): Lit => ({ kind: "window", screen: screenAt, index, keyboard: true })),
    { kind: "screen", screen: screenAt, keyboard: true } as Lit,
  ]);
}

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
 * The stage, picking live: point at a window and click it, point at Share this screen and click it,
 * Region or a right-button drag to draw, or the keys. Shared, the move to sharing plays as its
 * recipe says; cancelled, the desktop is left as it was. A double-click on the desktop asks again.
 */
function live(): HTMLElement {
  const holder = document.createElement("span");
  holder.style.display = "contents";
  let lit: Lit = NOTHING;
  let at = -1;

  const again = (shown: HTMLElement) => {
    shown.addEventListener("dblclick", () => {
      lit = NOTHING;
      at = -1;
      draw();
    });
    holder.replaceChildren(shown);
  };

  const light = (next: Lit) => {
    if (JSON.stringify(next) === JSON.stringify(lit)) return;
    lit = next;
    draw();
  };

  const share = (picked: Lit) => {
    if (picked.kind === "none") return;
    const shown = picking({ lit: picked, pill: true });
    again(shown);
    if (!ALL) playShared(shown);
  };

  const cancel = () => again(picking({ phase: "shared" }));

  const keys = (event: KeyboardEvent) => {
    const screenAt = lit.kind === "none" ? 0 : lit.screen;
    if (event.key === "Tab") {
      const all = stops();
      at = (at + (event.shiftKey ? -1 : 1) + all.length) % all.length;
      light(all[at]!);
    } else if (event.key.toLowerCase() === "s") light({ kind: "screen", screen: screenAt, keyboard: true });
    else if (event.key.toLowerCase() === "r") light({ kind: "region", screen: screenAt });
    else if (event.key === "Enter" && (lit.kind === "window" || lit.kind === "screen")) share(lit);
    else if (event.key === "Escape") {
      if (lit.kind === "region") light(NOTHING);
      else cancel();
    } else return;
    event.preventDefault();
    event.stopPropagation();
  };

  function draw(): void {
    const shown = picking({ lit, handlers: { onLight: light, onShare: share, onCancel: cancel } });
    shown.tabIndex = 0;
    shown.addEventListener("keydown", keys);
    holder.replaceChildren(shown);
    if (ALL) return;
    const drawing = shown.querySelector<HTMLElement>(".share-draw");
    requestAnimationFrame(() => (drawing ?? shown).focus({ preventScroll: true }));
  }
  draw();
  return holder;
}

const DIM = "the dim";
const BARS = "the bars";

function asking(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "long";
  return [
    { part: DIM, on: "every screen", target: ".k-screen-dim", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
    { part: BARS, on: "every screen", target: ".share-pick-bar", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
  ];
}

function cancelling(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "medium";
  return [
    { part: DIM, on: "every screen", target: ".k-screen-dim", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
    { part: BARS, on: "every screen", target: ".share-pick-bar", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
  ];
}

/** Picking giving way to the share: the label, the bars and the dim go, the mark stays, the pill arrives. */
function onAir(power: Strike | Easing, reduced: boolean): NonEmpty<Track> {
  const pill = ".shell-on-air";
  if (reduced) {
    return [
      { part: "the label", on: "the window", target: ".share-pick-label", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
      { part: BARS, on: "every screen", target: ".share-pick-bar", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
      { part: DIM, on: "every screen", target: ".k-screen-dim", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
      { part: "the pill", on: "every bar", target: pill, property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    ];
  }
  return [
    { part: "the label", on: "the window", target: ".share-pick-label", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard_accel" },
    { part: BARS, on: "every screen", target: ".share-pick-bar", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: DIM, on: "every screen", target: ".k-screen-dim", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the pill's power", on: "every bar", target: pill, property: "opacity", from: 0, to: 1, duration: "medium", cue: "medium", curve: power },
    { part: "the pill's beat lit", on: "every bar", target: pill, property: "--lit", from: 0, to: 1, duration: "short", cue: ["medium", "medium"], curve: "standard" },
    { part: "the pill's beat lit", on: "every bar", target: pill, property: "--lit", from: 1, to: 0, duration: "long", cue: ["medium", "medium", "short", "scene"], curve: "standard" },
  ];
}

/** Picking with a window lit and the pill in every bar: what the move to sharing plays on, the pill out at its start. */
function aboutToShare(): HTMLElement {
  return both({ lit: { kind: "window", screen: 0, index: 1 }, pill: true });
}

const SHARED: Move = {
  name: "Picked, then on air",
  what: "A click: the label, bars and dim go; the violet edge stays; the pill strikes on and burns a beat.",
  tracks: onAir("neon.strike", false),
  reduced: onAir("standard", true) as NonEmpty<Track<Easing>>,
  unflickered: onAir("standard", false) as NonEmpty<Track<Easing>>,
  back: "cuts",
  sample: screen(aboutToShare),
};

type Shown = "rest" | "lit";

const SITE: Words = { short: "meet.google.com", long: LONG_ASKER.site! };

export default page<Shown>({
  level: "shell",
  slug: "share-picker",
  title: "Share picker",
  lede: "Picking what to share on the desktop itself: each screen's bar, a window pointed at, a region drawn.",
  code: { qml: "SharePicker", unbuilt: "Not built yet" },
  from: ["popover", "button", "icon-button", "tooltip", "keycap", "banner", "chip"],
  stage: { live, hint: "point and click; S, R, Tab, Enter, Esc; right-drag", daylight: true },
  variants: [
    { name: "Picking, at rest", for: "both screens dimmed, a bar on each", sample: screen(() => both()) },
    { name: "A window pointed at", for: "out of the dim, its on-air edge, the label", sample: screen(() => both({ lit: { kind: "window", screen: 0, index: 1 } })) },
    { name: "This screen pointed at", for: "out of the dim, its corner marks", sample: screen(() => both({ lit: { kind: "screen", screen: 0 } })) },
    { name: "A region being drawn", for: "clear inside its on-air line; its size", sample: screen(() => both({ lit: { kind: "region", screen: 0, rect: REGION } })) },
    { name: "The keyboard on a window", for: "lit as pointed; the label says Enter", sample: screen(() => both({ lit: { kind: "window", screen: 1, index: 0, keyboard: true } })) },
    { name: "An app asks", for: "the app's name in each bar", sample: screen(() => both({ asker: DISCORD })) },
    { name: "Without the tip", for: "the bar's row alone, were the tip dropped", sample: screen(() => both({ tip: false })) },
    { name: "Another workspace", for: "switched while picking: dim and bar follow", sample: screen(() => both({ monitors: [DP10_THIRD, EDP1] })) },
    { name: "Shared", for: "the dim and the bars gone; mark and pill stay", sample: screen(() => both({ lit: { kind: "window", screen: 0, index: 1 }, phase: "shared" })) },
  ],
  sizes: [
    { name: "Two screens, two sizes", for: "DP-10 1600 by 1000, eDP-1 1440 by 900", sample: screen(() => both()) },
    { name: "The narrowest screen", for: "720 by 480: who asks gives way", sample: screen(() => picking({ monitors: [NARROW], asker: LONG_ASKER })) },
    { name: "The bar alone", for: "as wide as its row", sample: () => pickBar(MEET, { tip: true }) },
  ],
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "a click shares at once",
      disabled: "picking is up or gone",
      selected: "what is lit is the pick",
      loading: "the screens are there as it opens",
      empty: "a screen with no window still has its bar",
      error: "a failed share ends; a notification says so",
    },
    rows: [
      { name: "A window", cell: (state: Shown) => one(state === "lit" ? { lit: { kind: "window", screen: 0, index: 1 } } : {}) },
      { name: "This screen", cell: (state: Shown) => one(state === "lit" ? { lit: { kind: "screen", screen: 0 } } : {}) },
    ],
    looks: {
      rest: "dimmed a little, the bar at the bottom left",
      lit: "out of the dim, wearing the mark it keeps on air",
    },
  },
  focus: {
    tubes: [
      { name: "A window lit", tone: "sharing", filled: false, content: false },
      { name: "The bar's buttons", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the window's violet border; a button's tube and glow", signsOff: "the border stays; a button's tube as glass" },
    signsOff: (index: number) => (index === 0 ? one({ lit: { kind: "window", screen: 0, index: 1 } }) : pickBar(MEET, { lit: "screen" })),
  },
  content: {
    text: [{ name: "Who asks", words: SITE, sample: (words: string) => pickBar({ ...MEET, site: words }) }],
    missingPicture: [{ name: "A window's app with no icon", for: "its initial on the window's tile", sample: screen(() => both({ lit: { kind: "window", screen: 1, index: 1 } })) }],
    counts: [
      { name: "One screen", for: "one bar", sample: screen(() => one()) },
      { name: "Two screens", for: "a bar on each", sample: screen(() => both()) },
      { name: "A screen with no window", for: "only its bar and the wallpaper", sample: screen(() => picking({ monitors: [DP10, { ...EDP1, windows: [] }] })) },
    ],
  },
  motion: {
    arrives: [{ name: "Asked", what: "An app asks: every screen dims a little and each bar fades in.", tracks: asking(false), reduced: asking(true), back: "cuts", sample: screen(() => both()) }],
    leaves: [
      { name: "Cancelled", what: "Esc or ×: the bars and the dim fade off every screen.", tracks: cancelling(false), reduced: cancelling(true), back: "cuts", sample: screen(() => both()) },
      SHARED,
    ],
    between: [
      {
        name: "A window lit",
        what: "Pointed at: its violet edge and its label fade in over `short`; a light, not a power change.",
        tracks: [
          { part: "the edge", on: "the window", target: ".share-window[data-lifted] .share-border", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
          { part: "the label", on: "the window", target: ".share-pick-label", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: screen(() => one({ lit: { kind: "window", screen: 0, index: 1 } })),
      },
      {
        name: "This screen lit",
        what: "Share this screen pointed at: the corner marks fade in over `short`.",
        tracks: [{ part: "the marks", on: "the screen", target: ".share-corner", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: screen(() => one({ lit: { kind: "screen", screen: 0 } })),
      },
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "the next or last window, then the screen's bar, round" },
      { keys: ["S"], does: "lights this screen; Enter shares it" },
      { keys: ["R"], does: "draws a region on this screen" },
      { keys: ["Enter"], does: "shares what is lit, or the region drawn" },
      { keys: ["Esc"], does: "leaves drawing; else cancels" },
      { keys: ["Right-button drag"], does: "draws a region at once, anywhere" },
    ],
    order: "Tab walks a screen's windows, then its bar's Share this screen, screen by screen, round.",
    lands: "Opening, nothing lit, the keys on the pointer's screen; Esc from drawing, back to picking.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A window lit", for: "the bar, the label, the edge", sample: screen(() => one({ lit: { kind: "window", screen: 0, index: 1 } })) },
      { name: "The bar alone", for: "its row and its tip", sample: () => pickBar(MEET, { tip: true }) },
    ]),
    measures: [
      { part: "Bar from the frame", token: "space-s16" },
      { part: "Bar's padding", token: "space-s12" },
      { part: "Between the bar's parts", token: "space-s8" },
      { part: "Buttons", token: "size-control_sm" },
      { part: "Label's width", token: "size-menu" },
      { part: "Dim", token: "opacity-panel_scrim" },
      { part: "A window's edge", token: "stroke-thin" },
      { part: "Corner mark", token: "size-bar" },
    ],
    reflow: { note: "The bar keeps to the bottom left; on a narrow screen who asks is cut first.", sample: screen(() => picking({ monitors: [NARROW], asker: LONG_ASKER })) },
  },
});
