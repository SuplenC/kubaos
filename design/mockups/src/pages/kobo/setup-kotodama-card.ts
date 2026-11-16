// First boot's Kotodama card, in four steps of its own on a Stepper under its title, as Make it
// yours's are. What: the one key, what to type and Enter, beside the Command menu playing three asks
// by itself. Why: one question asked of Start, Spotlight and Kotodama at the same moment, raced,
// each clock running until its answer lands. Yours: the keys that open it, what it finds, the web's
// place, beside a preview that follows each change. Try it: three tries that tick off, beside the
// menu itself, the real Command menu's parts (`lib/list-detail.ts`).
//
// Wherever it is, Super tapped alone, as for Start, brings up the other menu for a moment, stamped
// "Nope.": the old habit met where it happens.
//
// A draft: its own parts (the big keys, the examples, the lanes, the parodies, the ad, the clock,
// the tries, the stamp) live here, each marked, until they are promoted to the library.

import { button } from "../../lib/button.ts";
import { choiceCard, choiceGroup } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { collapsible } from "../../lib/collapsible.ts";
import { field, setValue } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { type KeyHint, keycap, keyHints, keys } from "../../lib/keycap.ts";
import { fill, filling, fillingHeight, flow, gap, row, splitColumns, stack } from "../../lib/layout.ts";
import { commandMenu, type MenuMeasures } from "../../lib/list-detail.ts";
import { list, type ListRowSpec, listLabel } from "../../lib/list.ts";
import { spinner } from "../../lib/spinner.ts";
import { RECHECK, type RailPage, type RailStatus } from "../../lib/step-rail.ts";
import { setStep, stepper } from "../../lib/stepper.ts";
import { switchControl } from "../../lib/switch.ts";
import { tabs } from "../../lib/tabs.ts";
import { text, wrapped } from "../../lib/text.ts";
import { toastRoom } from "../../lib/toast.ts";
import { length } from "../../lib/tokens.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import { body } from "./kobo-parts.ts";
import { CARDS } from "./setup-cards.ts";
import { AD, defaultSettings, ENGINES, type Found, LANDS, type MenuSettings, OPENS_WITH, OTHERS, QUESTION, search } from "./setup-kotodama-data.ts";

/** The card's steps, by their index. */
export const STEPS = ["What", "Why", "Yours", "Try it"] as const;
export type StepIndex = 0 | 1 | 2 | 3;

/** The three tries: what to do, the words typed if any, and what it shows. */
const TRIES = [
  { id: "open", before: "Open it", typed: "", after: "", line: "Super + Space" },
  { id: "wifi", before: "Try", typed: "wifi", after: ", then Enter", line: "A setting, no Settings app" },
  { id: "sum", before: "Try", typed: "12*7", after: "", line: "Sums answer as you type" },
] as const;

type TryId = (typeof TRIES)[number]["id"];

/** Where the card is: its step, its settings, what was tried, and the menu open or shut. */
export interface Kotodama {
  step: StepIndex;
  /** A step turn's first state: the step left, drawn leaving over this one. */
  readonly from?: StepIndex;
  readonly settings: MenuSettings;
  readonly tried: Set<TryId>;
  open: boolean;
  /** What is typed in the menu on Try it. */
  query: string;
  /** Yours's advanced options open. */
  more: boolean;
  /** The other menu over the page, stamped: a sample of the gag, held. */
  gag: boolean;
  passed: boolean;
}

/** Where a view of the card starts. A view `name<N` is `name` reached from step N. */
export function kotodamaAt(view: string): Kotodama {
  const [name, from] = view.split("<") as [string, string | undefined];
  const base = (step: StepIndex, more: Partial<Kotodama> = {}): Kotodama => ({
    step,
    settings: defaultSettings(),
    tried: new Set(),
    open: false,
    query: "",
    more: false,
    gag: false,
    passed: false,
    ...(from === undefined ? {} : { from: Number(from) as StepIndex }),
    ...more,
  });
  switch (name) {
    case "why":
      return base(1);
    case "yours":
      return base(2);
    case "yours-more":
      return base(2, { more: true });
    case "yours-changed": {
      const state = base(2);
      state.settings.key = "alt-space";
      state.settings.web = false;
      return state;
    }
    case "try":
      return base(3);
    case "try-open":
      return base(3, { open: true, tried: new Set(["open"]) });
    case "try-typed":
      return base(3, { open: true, query: "wifi", tried: new Set(["open"]) });
    case "try-ticked":
      return base(3, { open: true, query: "12*7", tried: new Set(["open", "wifi", "sum"]) });
    case "gag":
      return base(0, { gag: true });
    case "done":
      return base(3, { passed: true, tried: new Set(["open", "wifi", "sum"]) });
    default:
      return base(0);
  }
}

// ----------------------------------------------------------------- Moves --

function ms(token: Parameters<typeof milliseconds>[0]): number {
  return milliseconds(token) / settings().speed;
}

function curve(name: Parameters<typeof bezier>[0]): string {
  return `cubic-bezier(${bezier(name).join(", ")})`;
}

/**
 * A step turning inside the card, as Make it yours's turn: the step left slides `space.s24` off
 * and fades over `short`; the next slides in from the other side over `long`, a `short` after.
 * Reduced, the step left goes at once and the next fades in.
 */
function turn(leaving: HTMLElement | null, arriving: HTMLElement, forward: boolean): void {
  const { reduced } = settings();
  const toward = forward ? -1 : 1;
  if (leaving) {
    leaving.dataset.leaving = "";
    const gone = leaving.animate(reduced ? [{ opacity: 0 }, { opacity: 0 }] : [{ opacity: 1, "--kotodama-slide": 0 }, { opacity: 0, "--kotodama-slide": toward }], { duration: reduced ? 0 : ms("short"), easing: curve("emphasized_accel"), fill: "forwards" });
    void gone.finished.then(() => leaving.remove(), () => leaving.remove());
  }
  if (reduced) arriving.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("short"), easing: curve("standard"), fill: "backwards" });
  else arriving.animate([{ opacity: 0, "--kotodama-slide": -toward }, { opacity: 1, "--kotodama-slide": 0 }], { duration: ms("long"), delay: ms("short"), easing: curve("emphasized_decel"), fill: "backwards" });
}

const wait = (milliseconds: number) => new Promise<void>((done) => window.setTimeout(done, milliseconds));

/** A clock's seconds, as the locale writes them, a tenth at a time. */
function seconds(milliseconds: number): string {
  return `${(milliseconds / 1000).toLocaleString("en-GB", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} s`;
}

// ------------------------------------------------------------ The stand-in --

/** The scale a box is drawn at: the page's, or its sample's. */
function scaleOf(element: HTMLElement): number {
  return Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || settings().scale;
}

/**
 * The Command menu's measures in a room `width` × `height` logical pixels big: the panel the
 * room's width, as many rows as its height holds, the pane where the panel is as wide as a dialog
 * beside a rail, as the menu has on a screen (`menuMeasures`).
 */
function measuresIn(width: number, height: number): MenuMeasures {
  const space = length("space-s12");
  const pitch = length("size-row_two_line") + length("space-s4");
  const chrome = space * 5 + length("size-control_md") + length("size-control_sm");
  const grid = length("space-s4");
  const panel = Math.floor(width / grid) * grid;
  const rows = Math.max(1, Math.floor((height - chrome + length("space-s4")) / pitch));
  const pane = panel >= length("size-dialog") + length("size-rail") ? Math.max(length("size-rail"), Math.floor(((panel - space * 3) * 0.4) / grid) * grid) : 0;
  const body = rows * pitch - length("space-s4");
  return { width: panel, height: chrome + body, rows, body, pane, top: 0 };
}

interface StandIn {
  readonly box: HTMLElement;
  /** Types `words` into it, `micro` a key (at once with reduced motion), then leaves them there. */
  readonly type: (words: string) => Promise<void>;
  readonly set: (words: string) => void;
  readonly redraw: () => void;
  readonly focus: () => void;
  /** Runs the Result selected, as Enter does. */
  readonly run: () => void;
  readonly query: () => string;
}

/**
 * The Command menu filling `box`: its field, its Results for what is typed, the pane where it has
 * room and its footer. Live, the field takes typing and the list ↑ ↓ and Enter; still, it shows
 * what it is told. `ran` hears each Result run; the Toast it says rises from the panel's foot.
 */
function standIn(prefs: MenuSettings, options: { readonly live: boolean; readonly query?: string; readonly ran?: (found: Found, typed: string) => void; readonly onEscape?: () => void }): StandIn {
  const box = document.createElement("span");
  box.className = "setup-kotodama-menu";
  const { room, say } = toastRoom();
  room.classList.add("setup-kotodama-toasts");
  let typed = options.query ?? "";
  let found: Found[] = [];
  let selected = 0;
  let measures: MenuMeasures | null = null;
  const run = (at: number) => {
    const each = found[at];
    if (!each) return;
    say({ words: each.said, icon: each.result.row.icon ?? "check" });
    options.ran?.(each, typed);
  };
  const head = field({
    placeholder: "Search apps, projects and windows. Sums work too",
    icon: "search",
    ...(typed ? { value: typed } : {}),
    pinned: "lit",
    ...(options.live
      ? {
          onInput: (value: string) => {
            typed = value;
            selected = 0;
            draw();
          },
        }
      : {}),
  });
  if (options.live)
    head.addEventListener(
      "keydown",
      (event) => {
        if (event.key !== "Escape") return;
        event.preventDefault();
        event.stopPropagation();
        if (typed) set("");
        else options.onEscape?.();
      },
      true,
    );
  const draw = () => {
    if (!measures) return;
    found = search(typed, prefs, measures.rows);
    const took = text(typed ? "2 ms" : "", "mono", "cold");
    const foot = row("s8", "centre", keyHints([["↑↓", "Select"], ["Enter", "Run"], ["Esc", "Clear"]]), fill(), took);
    const panel = commandMenu({ results: found.map((each) => each.result), query: typed, head, current: Math.min(selected, Math.max(0, found.length - 1)), onTake: run, footer: filling(foot) }, measures);
    panel.querySelector(".k-list")?.addEventListener("click", () => queueMicrotask(() => head.focus({ preventScroll: true })));
    box.replaceChildren(panel, room);
  };
  new ResizeObserver(() => {
    const scale = scaleOf(box);
    const next = measuresIn(box.clientWidth / scale, box.clientHeight / scale);
    if (measures && next.width === measures.width && next.rows === measures.rows) return;
    measures = next;
    draw();
  }).observe(box);
  const set = (words: string) => {
    typed = words;
    selected = 0;
    setValue(head, words);
    draw();
  };
  return {
    box,
    set,
    redraw: draw,
    query: () => typed,
    focus: () => head.focus({ preventScroll: true }),
    run: () => {
      const current = [...box.querySelectorAll(".k-list-row")].findIndex((each) => each.hasAttribute("data-current"));
      run(Math.max(0, current));
    },
    type: async (words) => {
      if (settings().reduced) return set(words);
      set("");
      for (const each of [...words]) {
        await wait(ms("micro"));
        set(typed + each);
      }
    },
  };
}

/** A key combination as keycaps, one a key. */
function caps(combination: string): HTMLElement {
  return row("s4", "centre", ...keys(combination).map(keycap));
}

/**
 * Draft part, to be promoted: the big keys, a combination as large lit keycaps, `size.control_lg`
 * tall in `button_large`, each in the primary's tube, a plus between.
 */
function bigKeys(combination: string): HTMLElement {
  const parts = keys(combination).flatMap((key, at) => {
    const cap = document.createElement("span");
    cap.className = "setup-kotodama-key";
    cap.append(text(key, "button_large"));
    return at === 0 ? [cap] : [text("+", "subheading", "on_surface_variant"), cap];
  });
  const line = row("s12", "centre", ...parts);
  line.classList.add("setup-kotodama-keys");
  return line;
}

// ---------------------------------------------------------------- The others --

interface Answer {
  readonly title: string;
  readonly subtitle?: string;
  readonly icon: string;
  readonly ad?: boolean;
}

/** A row of the others': a symbol and its words, nothing to take. */
function answer(each: Answer): ListRowSpec {
  return { title: each.title, ...(each.subtitle ? { subtitle: each.subtitle } : {}), icon: each.icon, ...(each.ad ? { chips: [{ kind: "Neutral" as const, label: "Ad" }] } : {}) };
}

/**
 * Draft part, to be promoted: the other menu, drained of colour: its field with a search's bar
 * crawling under it, its answers in groups as they land, and the line at its foot that never ends.
 */
function otherMenu(q: string, groups: readonly (readonly [string, readonly Answer[]])[], foot: string): { box: HTMLElement; groups: HTMLElement[]; answers: HTMLElement } {
  const words = field({ value: q, icon: "search", pointer: false });
  const bar = document.createElement("span");
  bar.className = "setup-kotodama-crawl";
  const lists = groups.map(([label, rows]) => list([listLabel(label, false), ...rows.map(answer)]));
  const answers = stack("s4", "start", ...lists.map(filling));
  answers.classList.add("setup-kotodama-answers");
  const status = row("s8", "centre", spinner("s16", "on_surface_variant"), filling(wrapped(foot, "small", "on_surface_variant")));
  status.classList.add("setup-kotodama-status");
  const inner = stack("s8", "start", filling(stack(0, "start", filling(words), filling(bar))), fillingHeight(filling(answers)), filling(status));
  inner.classList.add("setup-kotodama-sad");
  const box = document.createElement("span");
  box.className = "setup-kotodama-other";
  box.append(inner);
  return { box, groups: lists, answers };
}

/** Start's answers, by group, in the order they land. */
function startGroups(q: string): (readonly [string, readonly Answer[]])[] {
  return [
    ["Best match", [{ title: q, subtitle: "See web results", icon: "world" }]],
    ["Search the web", [{ title: `${q} not working`, icon: "search" }, { title: `${q} near me`, icon: "search" }]],
    ["Recommended", [{ title: "A game you never installed", subtitle: "Installing anyway", icon: "gamepad", ad: true }]],
    ["Settings (1)", [{ title: "Show 1 more", subtitle: "What you came for is in here", icon: "expand_more" }]],
    ["Weather", [{ title: "14° and cloudy", subtitle: "In a town you left in 2019", icon: "thermostat" }]],
  ];
}

function spotlightGroups(q: string): (readonly [string, readonly Answer[]])[] {
  return [
    ["Suggested website", [{ title: `${q}.com`, subtitle: "You didn't ask, it suggested", icon: "public" }]],
    ["Definition", [{ title: `${q} (noun)`, subtitle: "A thing you were trying to switch on", icon: "books" }, { title: `Search the web for “${q}”`, icon: "search" }]],
  ];
}

/**
 * Draft part, to be promoted: the ad Start raises over its answers, loud in the warning and error
 * tones, an invented cleaner for a PC that is fine.
 */
function fakeAd(): HTMLElement {
  const tag = text(AD.tag, "label", "on_burn_warning");
  tag.classList.add("setup-kotodama-ad-tag");
  const close = icon("close", "var(--type-small-icon)", "var(--color-on_burn_warning)");
  const top = row("s8", "centre", tag, filling(text(AD.sponsor, "caption", "on_burn_warning")), close);
  const tile = document.createElement("span");
  tile.className = "setup-kotodama-ad-tile";
  tile.append(icon("rocket_launch", "var(--type-heading-icon)", "var(--color-on_error)"));
  const product = row("s12", "start", tile, filling(stack(0, "start", text(AD.product, "subheading", "on_burn_warning"), filling(wrapped(AD.maker, "caption", "on_burn_warning")))));
  const stars = row("s4", "centre", ...Array.from({ length: 5 }, () => icon("star_filled", "var(--type-small-icon)", "var(--color-error)")), text(AD.rating, "caption", "on_burn_warning"));
  const go = button({ label: AD.action, kind: "Danger", icon: "arrow_forward", trailing: true });
  const ad = stack("s8", "start", filling(top), filling(wrapped(AD.headline, "heading", "on_burn_warning")), filling(product), filling(stars), filling(go), filling(wrapped(AD.fine, "caption", "on_burn_warning")));
  ad.classList.add("setup-kotodama-ad");
  return ad;
}

/**
 * Draft part, to be promoted: a lane's clock, a chip with a timer, counting in tenths of a second
 * until its answer lands; Kotodama's in the cold.
 */
function clockChip(words: string, ours: boolean): HTMLElement {
  return chip({ label: words, icon: "timer", kind: ours ? "Cold" : "Neutral" });
}

/**
 * The gag: the other menu rising from the screen's corner as Super alone opens it, its answers
 * landed, stamped "Nope." in the lit primary, then going back down. Reduced, it shows still.
 */
export function gag(held = false): HTMLElement {
  const other = otherMenu(QUESTION, startGroups(QUESTION), OTHERS.start.searching);
  // Draft part, to be promoted: the stamp.
  const stamp = text("Nope.", "display", "primary_lit");
  stamp.classList.add("setup-kotodama-stamp");
  const over = document.createElement("span");
  over.className = "setup-kotodama-gag";
  over.append(other.box, stamp);
  if (held || settings().reduced) return over;
  over.animate([{ opacity: 0, translate: "0 var(--space-s32)" }, { opacity: 1, translate: "0 0" }], { duration: ms("long"), easing: curve("emphasized_decel"), fill: "backwards" });
  stamp.animate([{ opacity: 0, scale: 1.4 }, { opacity: 1, scale: 1 }], { duration: ms("medium"), delay: ms("slow"), easing: curve("pop"), fill: "backwards" });
  return over;
}

// ----------------------------------------------------------------- The card --

export interface KotodamaCard {
  readonly state: Kotodama;
  readonly body: () => HTMLElement;
  readonly page: (skipped: () => boolean) => RailPage;
  /** The keys the rail lists at its foot for the step shown. */
  readonly hints: () => readonly KeyHint[];
}

const WORDS: Record<StepIndex, string> = {
  0: "The Command menu. One key, wherever you are: type what you want, Enter does it.",
  1: "One key for everything on this machine, and nothing off it. No ads, no news, no weather nobody asked for.",
  2: "It works as it is. Change what you like; the menu's “Settings: Menu” has the rest.",
  3: "Your turn. Three tries, or none: Next works either way.",
};

/** What's examples: what to type, what it is, and its symbol. */
const EXAMPLES = [
  { typed: "firefox", is: "an app", icon: "public" },
  { typed: "wifi", is: "a setting", icon: "wifi" },
  { typed: "12*7", is: "a sum", icon: "calculate" },
] as const;

/** A label over a part of Yours, `subheading`. */
const heading = (words: string) => text(words, "subheading");

/** A switch with its words, the setting's name over what it does. */
function switchRow(title: string, line: string, on: boolean, turned: (on: boolean) => void): HTMLElement {
  const control = switchControl({ on });
  control.addEventListener("click", () => queueMicrotask(() => turned(control.getAttribute("aria-checked") === "true")));
  return row("s12", "centre", filling(stack(0, "start", text(title, "body"), filling(wrapped(line, "small", "on_surface_variant")))), control);
}

export function kotodamaCard(view: string): KotodamaCard {
  const state = kotodamaAt(view);
  const prefs = state.settings;
  const spec = CARDS.kotodama;
  let element: HTMLElement | null = null;
  let line: HTMLElement | null = null;
  let room: HTMLElement | null = null;
  let shown: HTMLElement | null = null;
  /** Stops what the step shown is playing: the demo's loop, the race's clocks. */
  let stop = () => undefined as void;
  /** Races the three again, on Why. */
  let raceAgain = () => undefined as void;
  const recheck = () => element?.dispatchEvent(new Event(RECHECK, { bubbles: true }));

  // ----------------------------------------------------------------- What --

  const what = (): HTMLElement => {
    const examples = EXAMPLES.map((each) => {
      // Draft part, to be promoted: an example, its symbol on a tile, what to type and what it is.
      const tile = document.createElement("span");
      tile.className = "setup-kotodama-tile";
      tile.append(icon(each.icon, "var(--type-body-icon)", "currentColor"));
      const line = row("s12", "centre", tile, filling(text(each.typed, "mono_large")), text(each.is, "mono", "on_surface_variant"));
      line.classList.add("setup-kotodama-example");
      return line;
    });
    const menu = standIn(prefs, { live: false });
    menu.box.inert = true;
    let playing = true;
    const quiet = () => {
      playing = false;
    };
    stop = () => {
      quiet();
      window.removeEventListener("keydown", quiet);
    };
    window.addEventListener("keydown", quiet, { once: true });
    const lit = (at: number) => examples.forEach((each, index) => each.toggleAttribute("data-lit", index === at));
    const play = async () => {
      if (settings().reduced) {
        lit(1);
        menu.set("wifi");
        return;
      }
      for (let round = 0; playing; round = (round + 1) % EXAMPLES.length) {
        lit(round);
        await menu.type(EXAMPLES[round]!.typed);
        // It holds on the answer as long as a reader takes over it: two holds.
        await wait(ms("hold") * 2);
        if (!playing) return;
      }
    };
    requestAnimationFrame(() => void play());
    const left = stack(
      0,
      "start",
      bigKeys("Super+Space"),
      gap("s32"),
      text("Type what you want. Enter.", "headline"),
      gap("s8"),
      filling(wrapped("Apps, settings, projects, windows and sums, in one place. It opens over whatever you're doing, on the screen you're on.", "body", "on_surface_variant")),
      gap("s32"),
      filling(stack("s8", "start", ...examples.map(filling))),
    );
    left.classList.add("setup-kotodama-what");
    return splitColumns([1, 1], "s40", [filling(left), filling(fillingHeight(menu.box))], { fillHeight: true });
  };

  // ------------------------------------------------------------------ Why --

  const why = (): HTMLElement => {
    const lanes = document.createElement("span");
    lanes.className = "setup-kotodama-lanes";
    let timers: number[] = [];
    let frame = 0;
    stop = () => {
      timers.forEach(window.clearTimeout);
      timers = [];
      cancelAnimationFrame(frame);
    };

    /** The three, asked the question at the same moment, each answering in its own time. */
    const race = () => {
      stop();
      const reduced = settings().reduced;
      const start = otherMenu(QUESTION, startGroups(QUESTION), OTHERS.start.searching);
      const ad = fakeAd();
      start.box.append(ad);
      const spotlight = otherMenu(QUESTION, spotlightGroups(QUESTION), OTHERS.spotlight.indexing);
      const ours = standIn(prefs, { live: false, query: QUESTION });
      ours.box.inert = true;
      const clocks = { start: document.createElement("span"), spotlight: document.createElement("span"), kotodama: document.createElement("span") };
      const read = (who: keyof typeof clocks, words: string) => clocks[who].replaceChildren(clockChip(words, who === "kotodama"));
      const roasts = { start: wrapped(OTHERS.start.still, "small", "on_surface_variant"), spotlight: wrapped(OTHERS.spotlight.still, "small", "on_surface_variant") };
      const lane = (who: keyof typeof clocks, box: HTMLElement, roast: HTMLElement) => {
        const each = stack("s12", "start", filling(row("s8", "centre", text(OTHERS[who].name, "heading"), fill(), clocks[who])), filling(fillingHeight(box)), filling(roast));
        each.classList.add("setup-kotodama-lane");
        each.dataset.lane = who;
        return filling(fillingHeight(each));
      };
      const oursBox = document.createElement("span");
      oursBox.className = "setup-kotodama-ours";
      oursBox.append(ours.box);
      lanes.replaceChildren(
        splitColumns([1, 1, 1], "s16", [lane("start", start.box, roasts.start), lane("spotlight", spotlight.box, roasts.spotlight), lane("kotodama", oursBox, wrapped(OTHERS.kotodama.roast, "small", "on_surface_variant"))], { fillHeight: true }),
      );
      read("kotodama", OTHERS.kotodama.stopped);
      const settle = () => {
        read("start", OTHERS.start.stopped(seconds(LANDS.start.done)));
        read("spotlight", OTHERS.spotlight.stopped(seconds(LANDS.spotlight.done)));
        roasts.start.replaceWith((roasts.start = wrapped(OTHERS.start.roast(QUESTION), "small", "on_surface_variant")));
        roasts.spotlight.replaceWith((roasts.spotlight = wrapped(OTHERS.spotlight.roast, "small", "on_surface_variant")));
      };
      if (reduced) {
        settle();
        return;
      }
      read("start", seconds(0));
      read("spotlight", seconds(0));
      const [web, search, recommended, settingsGroup, weather] = start.groups;
      const [site, definition] = spotlight.groups;
      for (const part of [web, search, recommended, settingsGroup, weather, ad, site, definition]) if (part) part.style.opacity = "0";
      const speed = settings().speed;
      const land = (parts: readonly (HTMLElement | undefined)[], after: number, from = "0 var(--space-s16)") =>
        timers.push(
          window.setTimeout(() => {
            for (const part of parts) {
              if (!part) continue;
              part.style.opacity = "";
              part.animate([{ opacity: 0, translate: from }, { opacity: 1, translate: "0 0" }], { duration: ms("long"), easing: curve("emphasized_decel") });
            }
          }, after / speed),
        );
      land([web, search], LANDS.start.web);
      land([recommended, settingsGroup], LANDS.start.recommended);
      land([ad], LANDS.start.ad, "0 var(--space-s48)");
      land([weather], LANDS.start.weather);
      land([site], LANDS.spotlight.site);
      land([definition], LANDS.spotlight.web);
      const from = performance.now();
      const tick = () => {
        const spent = (performance.now() - from) * speed;
        read("start", spent >= LANDS.start.done ? OTHERS.start.stopped(seconds(LANDS.start.done)) : seconds(spent));
        read("spotlight", spent >= LANDS.spotlight.done ? OTHERS.spotlight.stopped(seconds(LANDS.spotlight.done)) : seconds(spent));
        if (spent >= LANDS.spotlight.done && roasts.spotlight.textContent === OTHERS.spotlight.still) roasts.spotlight.replaceWith((roasts.spotlight = wrapped(OTHERS.spotlight.roast, "small", "on_surface_variant")));
        if (spent < LANDS.start.done) frame = requestAnimationFrame(tick);
        else settle();
      };
      frame = requestAnimationFrame(tick);
    };
    raceAgain = race;
    race();
    return stack(0, "start", fillingHeight(filling(lanes)));
  };

  // ---------------------------------------------------------------- Yours --

  const yours = (): HTMLElement => {
    const preview = standIn(prefs, { live: true, query: "3/2" });
    const show = (words: string) => preview.set(words);
    const cards = OPENS_WITH.map((each) => {
      const card = choiceCard({ title: each.title, subtitle: each.line, picks: "one", picked: prefs.key === each.id });
      card.querySelector(".k-card-header, .k-card-title-line")?.parentElement?.prepend(caps(each.keys));
      return card;
    });
    const keysGroup = choiceGroup("one", 3, cards);
    keysGroup.addEventListener("click", () =>
      queueMicrotask(() => {
        const at = cards.findIndex((each) => each.getAttribute("aria-checked") === "true");
        if (at >= 0) prefs.key = OPENS_WITH[at]!.id;
      }),
    );
    const finds = (
      [
        ["apps", "Apps", "fi"],
        ["actions", "Settings and actions", "wifi"],
        ["projects", "Projects and scripts", "kubaos"],
        ["windows", "Windows and key binds", "move"],
        ["clipboard", "Clipboard history", "clip"],
        ["emoji", "Emoji", "emoji"],
        ["calculator", "Sums", "3/2"],
      ] as const
    ).map(([id, label, shows]) => {
      const on = id === "apps" ? prefs.apps : prefs.finds[id];
      const each = chip({ label, press: "toggle", on, kind: "Accent", icon: "check" });
      each.addEventListener("click", () =>
        queueMicrotask(() => {
          const now = each.getAttribute("aria-pressed") === "true";
          if (id === "apps") prefs.apps = now;
          else prefs.finds[id] = now;
          show(shows);
        }),
      );
      return each;
    });
    const engine = tabs({ choices: ENGINES.map((label) => ({ label })), picked: prefs.engine, onPick: (at) => ((prefs.engine = at), show("zzz")) });
    const motion = tabs({ choices: [{ label: "Full" }, { label: "Fades" }], picked: prefs.slides ? 0 : 1, onPick: (at) => ((prefs.slides = at === 0), preview.box.animate(prefs.slides ? [{ opacity: 0, translate: "0 calc(-1 * var(--space-s8))" }, { opacity: 1, translate: "0 0" }] : [{ opacity: 0 }, { opacity: 1 }], { duration: ms("medium"), easing: curve("emphasized_decel") })) });
    const mark = tabs({ choices: [{ label: "1.5" }, { label: "1,5" }], picked: prefs.comma ? 1 : 0, onPick: (at) => ((prefs.comma = at === 1), show(prefs.comma ? "3/2*1" : "3/2")) });
    const web = stack(
      "s12",
      "start",
      heading("When nothing here matches"),
      filling(switchRow("Offer a web search", "Rows at the bottom, never a guess", prefs.web, (on) => ((prefs.web = on), show("zzz")))),
      filling(stack(0, "start", text("First with", "body"), filling(wrapped("The engine on the first web row", "small", "on_surface_variant")))),
      engine,
    );
    const typing = stack(
      "s12",
      "start",
      heading("Typing and motion"),
      filling(switchRow("Forgive typos", "“fierfox” finds Firefox", prefs.typos, (on) => ((prefs.typos = on), show("fierfox")))),
      filling(row("s12", "centre", filling(text("Motion", "body")), motion)),
    );
    const files = button({ label: "Open in Files", kind: "Text", size: "Sm" });
    const more = collapsible({
      title: "More",
      summary: "Decimal mark, aliases, the file",
      open: state.more,
      content: splitColumns([1, 1], "s24", [
        filling(row("s12", "centre", filling(stack(0, "start", text("Decimal mark", "body"), filling(wrapped("How sums write a half", "small", "on_surface_variant")))), mark)),
        filling(stack("s4", "start", text("Aliases, quicklinks, your own actions", "body"), filling(wrapped("All in one file: user.toml", "small", "on_surface_variant")), files)),
      ]),
    });
    const left = stack(
      0,
      "start",
      heading("Opens with"),
      gap("s8"),
      filling(keysGroup),
      gap("s16"),
      heading("It finds"),
      gap("s8"),
      filling(flow("s8", ...finds)),
      gap("s16"),
      filling(splitColumns([1, 1], "s24", [filling(web), filling(typing)])),
      gap("s8"),
      filling(more),
    );
    const right = stack("s8", "start", text("As it will look", "label", "on_surface_variant"), fillingHeight(filling(preview.box)));
    return splitColumns([2, 1], "s24", [filling(left), filling(fillingHeight(right))], { fillHeight: true });
  };

  // --------------------------------------------------------------- Try it --

  let openMenu = () => undefined as void;

  const tryIt = (): HTMLElement => {
    const cardsBy = new Map<TryId, HTMLElement>();
    const tryCard = (each: (typeof TRIES)[number]) => {
      // Draft part, to be promoted: a try, its round tick lit once done, the words to type in mono.
      const mark = document.createElement("span");
      mark.className = "setup-kotodama-tick";
      mark.append(icon("check", "var(--type-small-icon)", "currentColor"));
      const title = row(0, "centre", text(each.before, "body_strong"), ...(each.typed ? [gap("s4"), text(each.typed, "mono_large", "primary")] : []), ...(each.after ? [text(each.after, "body_strong")] : []));
      const card = row("s12", "centre", mark, filling(stack(0, "start", filling(title), text(each.line, "small", "on_surface_variant"))));
      card.className += " setup-kotodama-try-card";
      card.toggleAttribute("data-done", state.tried.has(each.id));
      card.addEventListener("click", () => {
        openMenu();
        if (each.typed) void menu.type(each.typed);
      });
      cardsBy.set(each.id, card);
      return filling(card);
    };
    const tick = (id: TryId) => {
      if (state.tried.has(id)) return;
      state.tried.add(id);
      const card = cardsBy.get(id);
      card?.setAttribute("data-done", "");
      if (card && !settings().reduced) card.querySelector(".setup-kotodama-tick")?.animate([{ scale: 0.4 }, { scale: 1 }], { duration: ms("medium"), easing: curve("pop") });
      recheck();
    };
    const menu = standIn(prefs, {
      live: true,
      query: state.query,
      ran: (each, typed) => {
        if (each.item?.id === "wifi") tick("wifi");
        if (each.answer && typed.replace(/\s/g, "") === "12*7") tick("sum");
      },
      onEscape: () => shut(),
    });
    const holder = document.createElement("span");
    holder.className = "setup-kotodama-try";
    // Draft part, to be promoted: the shut menu's room, its keys big, a click opening it this once.
    const invitation = stack("s16", "centre", bigKeys(KEY_OF[prefs.key]), text("Press it. Or click here, this once.", "small", "on_surface_variant"));
    invitation.classList.add("setup-kotodama-invitation");
    invitation.addEventListener("click", () => openMenu());
    const shut = () => {
      state.open = false;
      holder.replaceChildren(invitation);
    };
    openMenu = () => {
      if (!state.open) {
        state.open = true;
        holder.replaceChildren(menu.box);
        if (!settings().reduced) menu.box.animate(prefs.slides ? [{ opacity: 0, translate: "0 calc(-1 * var(--space-s8))" }, { opacity: 1, translate: "0 0" }] : [{ opacity: 0 }, { opacity: 1 }], { duration: ms("medium"), easing: curve("emphasized_decel") });
      }
      tick("open");
      requestAnimationFrame(() => menu.focus());
    };
    holder.append(state.open ? menu.box : invitation);
    const tip = row("s8", "start", icon("info", "var(--type-small-icon)", "var(--color-on_surface_variant)"), filling(wrapped("The menu shows up wherever you are: on top of any app, on the screen you're on.", "small", "on_surface_variant")));
    const left = stack("s12", "start", ...TRIES.map(tryCard), fillingHeight(gap("s4")), filling(tip));
    return splitColumns([2, 3], "s24", [filling(left), filling(fillingHeight(holder))], { fillHeight: true });
  };

  // ----------------------------------------------------------------- Steps --

  const stepView = (at: StepIndex): HTMLElement => {
    stop();
    stop = () => undefined;
    const view = at === 0 ? what() : at === 1 ? why() : at === 2 ? yours() : tryIt();
    const holder = document.createElement("span");
    holder.className = "setup-kotodama-step";
    holder.dataset.step = String(at);
    holder.append(view);
    return holder;
  };

  const headerFor = (at: StepIndex): HTMLElement => body(spec.accent, spec.title, WORDS[at], document.createElement("span"), false).querySelector<HTMLElement>(".k-header")!;

  const show = (to: StepIndex) => {
    if (!room) return;
    const forward = to > state.step;
    state.step = to;
    const next = stepView(to);
    const left = shown;
    room.append(next);
    shown = next;
    if (line) setStep(line, to, backTo);
    element?.querySelector(".k-header")?.replaceWith(headerFor(to));
    turn(left, next, forward);
    recheck();
  };
  const backTo = (to: number) => show(to as StepIndex);

  /** The old habit: Super tapped alone shows the other menu over the room for a moment. */
  const habit = () => {
    const place = element?.closest<HTMLElement>(".kobo-room");
    if (!place || element?.closest("[data-kept]")) return;
    place.querySelector(".setup-kotodama-gag")?.remove();
    const over = gag();
    place.append(over);
    window.setTimeout(() => {
      if (settings().reduced) return over.remove();
      void over.animate([{ opacity: 1 }, { opacity: 0, translate: "0 var(--space-s16)" }], { duration: ms("medium"), easing: curve("emphasized_accel"), fill: "forwards" }).finished.then(() => over.remove());
    }, ms("hold") * 3);
  };

  const listen = () => {
    let alone = false;
    const here = () => element?.isConnected && !element.closest("[data-kept]");
    window.addEventListener("keydown", (event) => {
      if (!here()) return;
      if (event.key === "Meta" || event.key === "OS") {
        alone = true;
        return;
      }
      alone = false;
      const space = event.code === "Space";
      const opens = (prefs.key === "super-space" && space && event.metaKey) || (prefs.key === "alt-space" && space && event.altKey);
      if (opens && state.step === 3) {
        event.preventDefault();
        openMenu();
      }
    });
    window.addEventListener("keyup", (event) => {
      if (!here() || (event.key !== "Meta" && event.key !== "OS") || !alone) return;
      alone = false;
      if (prefs.key === "super" && state.step === 3) openMenu();
      else habit();
    });
  };

  const draw = (): HTMLElement => {
    if (element) return element;
    line = stepper({ steps: [...STEPS], current: state.step, onBack: backTo });
    room = document.createElement("span");
    room.className = "setup-kotodama-room";
    if (state.from !== undefined) {
      const left = stepView(state.from);
      left.dataset.leaving = "";
      room.append(left);
    }
    shown = stepView(state.step);
    room.append(shown);
    const content = stack("s24", "start", filling(line), fillingHeight(filling(room)));
    element = body(spec.accent, spec.title, WORDS[state.step], content, false);
    element.classList.add("setup-kotodama");
    // On Why, Enter races the three again; the band's Next still goes on.
    element.addEventListener("keydown", (event) => {
      if (state.step !== 1 || event.key !== "Enter" || (event.target as Element).closest("button")) return;
      event.preventDefault();
      raceAgain();
    });
    if (state.gag) element.append(gag(true));
    listen();
    return element;
  };

  const status = (): RailStatus => ({ state: { is: state.passed ? "done" : "todo" } });

  const NEXT: Record<StepIndex, string> = { 0: "Next: why", 1: "Next: yours", 2: "Next: try it", 3: "Next: Git" };
  const NOTE: Record<StepIndex, () => string> = {
    0: () => "Nothing here is required",
    1: () => "Enter races them again",
    2: () => "Defaults are fine. Really",
    3: () => (state.tried.size === 3 ? "3 of 3 tried. That's the whole trick" : `${state.tried.size} of 3 tried`),
  };

  return {
    state,
    body: draw,
    hints: (): readonly KeyHint[] => {
      if (state.step === 1) return [["Enter", "race again"], ["Esc", "back a step"]];
      if (state.step === 2) return [["← →", "pick a key"], ["Space", "switch it"], ["Tab", "next field"]];
      if (state.step === 3) return [[KEY_OF[prefs.key], "open it"], ["Enter", "run it"], ["Esc", "close it"]];
      return [[KEY_OF[prefs.key], "open it"], ["↑ ↓", "pick a Result"], ["Enter", "run it"]];
    },
    page: (skipped) => ({
      step: { title: spec.title, icon: spec.icon },
      body: draw,
      status,
      next: () => NEXT[state.step],
      skip: () => (state.passed ? undefined : skipped() ? "Later" : "Skip"),
      note: () => NOTE[state.step](),
      back: () => {
        if (state.step === 0) return false;
        show((state.step - 1) as StepIndex);
        return true;
      },
      act: () => {
        if (state.step < 3) {
          show((state.step + 1) as StepIndex);
          return true;
        }
        state.passed = true;
        recheck();
        return false;
      },
    }),
  };
}

/** The keys each choice of Opens with names. */
const KEY_OF: Record<MenuSettings["key"], string> = { "super-space": "Super+Space", "alt-space": "Alt+Space", super: "Super" };
