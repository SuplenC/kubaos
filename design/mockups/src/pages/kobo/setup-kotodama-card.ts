// First boot's Kotodama card, as an organism of library parts, in four steps of its own on a
// Stepper under its title, as Make it yours's are. What: the one key, what to type, Enter, beside
// the Command menu playing by itself. Why: one question typed into Start, Spotlight and Kotodama at
// once, raced, each clock running until its answer lands. Yours: the keys that open it, what it
// finds and the web's place, beside a preview that shows each change. Try it: the menu itself, the
// real Command menu's parts (`lib/list-detail.ts`), and three things to try that tick off.
//
// Wherever it is, Super tapped alone, as for Start, brings up the other menu for a moment, stamped
// "Nope.": the old habit met where it happens.

import { button } from "../../lib/button.ts";
import { choiceCard, choiceGroup } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { collapsible } from "../../lib/collapsible.ts";
import { descriptionList } from "../../lib/description.ts";
import { field, setValue } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { type KeyHint, keycap, keys } from "../../lib/keycap.ts";
import { fill, filling, fillingHeight, gap, row, splitColumns, stack } from "../../lib/layout.ts";
import { commandMenu, type MenuMeasures } from "../../lib/list-detail.ts";
import { list, type ListRowSpec, listLabel, listWith } from "../../lib/list.ts";
import { progressBar } from "../../lib/progress.ts";
import { scroll } from "../../lib/scroll.ts";
import { select } from "../../lib/select.ts";
import { spinner } from "../../lib/spinner.ts";
import { RECHECK, type RailPage, type RailStatus } from "../../lib/step-rail.ts";
import { setStep, stepper } from "../../lib/stepper.ts";
import { text, wrapped } from "../../lib/text.ts";
import { toastRoom } from "../../lib/toast.ts";
import { length } from "../../lib/tokens.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import { body } from "./kobo-parts.ts";
import { CARDS } from "./setup-cards.ts";
import { AD, defaultSettings, ENGINES, figure, type Found, LANDS, type MenuSettings, OPENS_WITH, OTHERS, QUESTIONS, search } from "./setup-kotodama-data.ts";

/** The card's steps, by their index. */
export const STEPS = ["What", "Why", "Yours", "Try it"] as const;
export type StepIndex = 0 | 1 | 2 | 3;

/** What the three things to try are, and how each is done. */
const TRIES = [
  { id: "open", spec: (done: boolean): ListRowSpec => ({ title: "Open it", subtitle: "Anywhere, any time", icon: "keyboard_command_key", tile: true, keys: "Super+Space", box: true, checked: done, accent: done }) },
  { id: "wifi", spec: (done: boolean): ListRowSpec => ({ title: "Type “wifi”, then Enter", subtitle: "A setting, no Settings app", icon: "wifi", tile: true, box: true, checked: done, accent: done }) },
  { id: "sum", spec: (done: boolean): ListRowSpec => ({ title: "Type “12*7”", subtitle: "Sums answer as you type", icon: "calculate", tile: true, box: true, checked: done, accent: done }) },
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

/** Something arriving where it is: it rises `space.s8` and fades in over `long`; reduced, it is there. */
function arrive(element: Element, delay = 0): void {
  if (settings().reduced) return;
  element.animate([{ opacity: 0, translate: "0 var(--space-s8)" }, { opacity: 1, translate: "0 0" }], { duration: ms("long"), delay, easing: curve("emphasized_decel"), fill: "backwards" });
}

/** What changes under the keys (a tick) fades in over `short`. */
function changed(element: Element | null): void {
  if (!element || settings().reduced) return;
  element.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("short"), easing: curve("standard") });
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
    const panel = commandMenu({ results: found.map((each) => each.result), query: typed, head, current: Math.min(selected, Math.max(0, found.length - 1)), onTake: run }, measures);
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

// ---------------------------------------------------------------- The others --

/** A row of the others': a symbol on its tile and two lines, nothing to take. */
function otherRow(each: { readonly title: string; readonly subtitle: string; readonly icon: string }): ListRowSpec {
  return { title: each.title, subtitle: each.subtitle, icon: each.icon, tile: true, tall: true };
}

/**
 * The ad Start slides over its answers: a game nobody installed, installing anyway, in all the
 * colours an ad has, the one thing in its lane not drained of them.
 */
function fakeAd(): HTMLElement {
  const installing = progressBar({ fraction: 0.12 });
  const ad = stack(
    "s8",
    "start",
    text(AD.sponsor, "caption", "on_primary_fill"),
    wrapped(AD.title, "headline", "on_primary_fill"),
    filling(wrapped(AD.line, "small", "on_primary_fill")),
    gap("s4"),
    row("s12", "centre", button({ label: AD.action, size: "Sm", kind: "Tonal" }), filling(stack("s4", "start", text(AD.small, "caption", "on_primary_fill"), filling(installing)))),
    text(AD.fine, "caption", "on_primary_fill"),
  );
  ad.classList.add("setup-kotodama-ad");
  return ad;
}

/** What the other menu shows once it has answered `q`: the web's rows, the ad over them, the weather. */
function startLane(q: string): { box: HTMLElement; rows: HTMLElement; ad: HTMLElement; weather: HTMLElement; status: HTMLElement; field: HTMLElement } {
  const words = field({ value: q, icon: "search", pointer: false });
  const rows = list([listLabel("Best match", false), ...OTHERS.start.rows(q).map(otherRow), listLabel("Settings", true), otherRow({ title: "Show more", subtitle: "What you came for is in here", icon: "expand_more" })]);
  const weather = list([otherRow(OTHERS.start.weather)]);
  const ad = fakeAd();
  const status = row("s8", "centre", spinner("s16", "on_surface_variant"), filling(wrapped(OTHERS.start.searching, "caption", "on_surface_variant")));
  const answers = stack("s8", "start", filling(rows), filling(weather));
  answers.classList.add("setup-kotodama-answers");
  const base = stack("s12", "start", filling(words), fillingHeight(filling(answers)), filling(status));
  base.classList.add("setup-kotodama-sad");
  const box = document.createElement("span");
  box.className = "setup-kotodama-other";
  box.append(base, ad);
  return { box, rows, ad, weather, status, field: words };
}

function spotlightLane(q: string): { box: HTMLElement; rows: HTMLElement[]; field: HTMLElement } {
  const words = field({ value: q, icon: "search", pointer: false });
  const [site, definition] = OTHERS.spotlight.rows(q);
  const first = list([listLabel("Suggested website", false), otherRow(site!)]);
  const second = list([listLabel("Definition", false), otherRow(definition!)]);
  const status = row("s8", "centre", spinner("s16", "on_surface_variant"), filling(wrapped(OTHERS.spotlight.indexing, "caption", "on_surface_variant")));
  const answers = stack("s8", "start", filling(first), filling(second));
  answers.classList.add("setup-kotodama-answers");
  const base = stack("s12", "start", filling(words), fillingHeight(filling(answers)), filling(status));
  base.classList.add("setup-kotodama-sad");
  const box = document.createElement("span");
  box.className = "setup-kotodama-other";
  box.append(base);
  return { box, rows: [first, second], field: words };
}

/**
 * The gag: the other menu rising from the screen's corner as Super alone opens it, its ad landed,
 * stamped "Nope." in the lit primary, then going back down. Reduced, it shows still and goes.
 */
export function gag(held = false): HTMLElement {
  const lane = startLane("wifi");
  lane.status.replaceChildren(wrapped("Super alone does nothing here. Super + Space does everything.", "small", "on_surface"));
  const stamp = text("Nope.", "display", "primary_lit");
  stamp.classList.add("setup-kotodama-stamp");
  const over = document.createElement("span");
  over.className = "setup-kotodama-gag";
  over.append(lane.box, stamp);
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
  0: "The Command menu. Everything on this machine is in it: apps, settings, projects, windows, sums.",
  1: "One key for everything on this machine, and nothing off it. No ads, no news, no weather nobody asked for.",
  2: "It works as it is. Change what you like; “Settings: Menu” in the menu has the rest.",
  3: "Your turn. Nothing here is required: Next works either way.",
};

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
  const recheck = () => element?.dispatchEvent(new Event(RECHECK, { bubbles: true }));

  // ----------------------------------------------------------------- What --

  const what = (): HTMLElement => {
    const beats = list([
      { title: "Press it", subtitle: "Anywhere, over anything", icon: "keyboard_command_key", tile: true, keys: "Super+Space" },
      { title: "Type what you want", subtitle: "An app, a setting, a project, a sum", icon: "search", tile: true },
      { title: "Enter", subtitle: "It does it, and gets out of the way", icon: "check", tile: true, keys: "↵" },
    ]);
    beats.classList.add("setup-kotodama-beats");
    const menu = standIn(prefs, { live: false });
    menu.box.inert = true;
    let playing = true;
    stop = () => {
      playing = false;
    };
    const lit = (at: number) => beats.querySelectorAll<HTMLElement>(".k-list-row").forEach((each, index) => each.toggleAttribute("data-current", index === at));
    const play = async () => {
      if (settings().reduced) {
        menu.set("12*7");
        return;
      }
      for (let round = 0; playing; round++) {
        const asked = ["fire", "wifi", "12*7"][round % 3]!;
        lit(0);
        menu.set("");
        await wait(ms("hold"));
        lit(1);
        await menu.type(asked);
        await wait(ms("hold"));
        if (!playing) return;
        lit(2);
        menu.run();
        // It holds there as long as a Toast stays: two holds.
        await wait(ms("hold") * 2);
      }
    };
    requestAnimationFrame(() => void play());
    return splitColumns([2, 3], "s40", [stack(0, "start", filling(beats)), filling(fillingHeight(menu.box))], { fillHeight: true });
  };

  // ------------------------------------------------------------------ Why --

  const why = (): HTMLElement => {
    let racing = 0;
    const lanes = document.createElement("span");
    lanes.className = "setup-kotodama-lanes";
    const ask = field({
      value: "wifi",
      icon: "search",
      placeholder: "Ask all three",
      onInput: (value) => {
        window.clearTimeout(racing);
        racing = window.setTimeout(() => race(value.trim() || "wifi"), ms("long"));
      },
    });
    const questions = QUESTIONS.map((q) => {
      const each = chip({ label: q, press: "press" });
      each.addEventListener("click", () => {
        setValue(ask, q);
        race(q);
      });
      return each;
    });
    const replay = button({ label: "Race again", kind: "Text", size: "Sm", icon: "restart_alt" });
    replay.addEventListener("click", () => race((ask.querySelector(".k-field-value")?.textContent ?? "").trim() || "wifi"));
    const top = row("s12", "centre", filling(ask), row("s8", "centre", ...questions), fill(), replay);
    let timers: number[] = [];
    let frame = 0;
    stop = () => {
      timers.forEach(window.clearTimeout);
      timers = [];
      cancelAnimationFrame(frame);
    };

    /** The three, asked `q` at the same moment: typed together, then each in its own time. */
    const race = (q: string) => {
      stop();
      const reduced = settings().reduced;
      const start = startLane(q);
      const spotlight = spotlightLane(q);
      const ours = standIn(prefs, { live: false, query: reduced ? q : "" });
      ours.box.inert = true;
      const clock = (who: string) => {
        const shownClock = document.createElement("span");
        shownClock.className = "setup-kotodama-clock";
        shownClock.dataset.who = who;
        shownClock.append(text(seconds(0), "mono", "on_surface_variant"));
        return shownClock;
      };
      // A text's box is laid out for its words, so a clock's new reading is a text of its own.
      const read = (shownClock: HTMLElement, words: string) => {
        if (shownClock.textContent === words) return;
        shownClock.replaceChildren(text(words, "mono", shownClock.hasAttribute("data-landed") ? "cold" : "on_surface_variant"));
      };
      const clocks = { start: clock("start"), spotlight: clock("spotlight"), kotodama: clock("kotodama") };
      const lane = (name: string, shownClock: HTMLElement, box: HTMLElement, roast: string, ours = false) => {
        const each = stack("s12", "start", filling(row("s8", "centre", text(name, "subheading"), fill(), shownClock)), filling(fillingHeight(box)), filling(wrapped(roast, "small", "on_surface_variant")));
        each.classList.add("setup-kotodama-lane");
        each.dataset.lane = name.toLowerCase();
        if (ours) each.dataset.ours = "";
        return each;
      };
      lanes.replaceChildren(
        splitColumns(
          [1, 1, 1],
          "s16",
          [lane(OTHERS.start.name, clocks.start, start.box, OTHERS.start.roast(q)), lane(OTHERS.spotlight.name, clocks.spotlight, spotlight.box, OTHERS.spotlight.roast()), lane(OTHERS.kotodama.name, clocks.kotodama, ours.box, OTHERS.kotodama.roast(), true)].map((each) => filling(fillingHeight(each))),
          { fillHeight: true },
        ),
      );
      const settle = () => {
        read(clocks.start, OTHERS.start.settled(seconds(LANDS.start.done)));
        read(clocks.spotlight, OTHERS.spotlight.settled(seconds(LANDS.spotlight.done)));
        clocks.kotodama.dataset.landed = "";
        read(clocks.kotodama, OTHERS.kotodama.settled);
      };
      if (reduced) {
        settle();
        return;
      }
      // Until the key is let go, the others have nothing; Kotodama answers each key as it lands.
      for (const part of [start.rows, start.weather, start.ad, ...spotlight.rows]) part.style.opacity = "0";
      const typing = [...q].map((_, at) => at * ms("micro"));
      for (const [at, after] of typing.entries()) {
        timers.push(
          window.setTimeout(() => {
            const sofar = q.slice(0, at + 1);
            setValue(start.field, sofar);
            setValue(spotlight.field, sofar);
            ours.set(sofar);
          }, after),
        );
      }
      const letGo = typing.at(-1)! + ms("micro");
      const speed = settings().speed;
      const land = (part: HTMLElement, after: number, from = "0 var(--space-s16)") =>
        timers.push(
          window.setTimeout(() => {
            part.style.opacity = "";
            part.animate([{ opacity: 0, translate: from }, { opacity: 1, translate: "0 0" }], { duration: ms("long"), easing: curve("emphasized_decel") });
          }, letGo + after / speed),
        );
      land(start.rows, LANDS.start.web);
      land(start.ad, LANDS.start.ad, "0 100%");
      land(start.weather, LANDS.start.weather);
      land(spotlight.rows[0]!, LANDS.spotlight.site);
      land(spotlight.rows[1]!, LANDS.spotlight.definition);
      timers.push(
        window.setTimeout(() => {
          clocks.kotodama.dataset.landed = "";
          read(clocks.kotodama, OTHERS.kotodama.settled);
          const from = performance.now();
          const tickClocks = () => {
            const spent = (performance.now() - from) * speed;
            read(clocks.start, spent >= LANDS.start.done ? OTHERS.start.settled(seconds(LANDS.start.done)) : seconds(spent));
            read(clocks.spotlight, spent >= LANDS.spotlight.done ? OTHERS.spotlight.settled(seconds(LANDS.spotlight.done)) : seconds(spent));
            if (spent < LANDS.start.done) frame = requestAnimationFrame(tickClocks);
            else settle();
          };
          frame = requestAnimationFrame(tickClocks);
        }, letGo),
      );
    };
    race("wifi");
    return stack("s16", "start", filling(top), fillingHeight(filling(lanes)));
  };

  // ---------------------------------------------------------------- Yours --

  const yours = (): HTMLElement => {
    const preview = standIn(prefs, { live: true, query: "fi" });
    const show = (words: string) => preview.set(words);
    const cards = OPENS_WITH.map((each) => {
      const card = choiceCard({ title: each.title, subtitle: each.line, picks: "one", picked: prefs.key === each.id });
      card.querySelector(".k-card-title-line")?.parentElement?.append(caps(each.keys));
      return card;
    });
    const keysGroup = choiceGroup("one", 3, cards);
    keysGroup.addEventListener("click", () =>
      queueMicrotask(() => {
        const at = cards.findIndex((each) => each.getAttribute("aria-checked") === "true");
        if (at >= 0) prefs.key = OPENS_WITH[at]!.id;
      }),
    );
    type Switch = { readonly spec: ListRowSpec; readonly get: () => boolean; readonly set: (on: boolean) => void; readonly shows: string };
    const switches: Switch[] = [
      { spec: { title: "Web search when nothing here matches", subtitle: "Rows at the bottom you pick, never a guess", icon: "travel_explore" }, get: () => prefs.web, set: (on) => (prefs.web = on), shows: "zzz" },
      { spec: { title: "Slide as it opens", subtitle: "Off, it only fades", icon: "motion_photos_on" }, get: () => prefs.slides, set: (on) => (prefs.slides = on), shows: "" },
      { spec: { title: "Forgive typos", subtitle: "“fierfox” finds Firefox", icon: "keyboard" }, get: () => prefs.typos, set: (on) => (prefs.typos = on), shows: "fierfox" },
    ];
    const switchList = (each: readonly Switch[]) => {
      const made = listWith(
        each.map((one) => ({ ...one.spec, tile: Boolean(one.spec.icon), toggle: one.get() })),
        {
          onTake: (at) => {
            const one = each[at];
            if (!one) return;
            one.set(!one.get());
            if (one.shows === "" && !settings().reduced) preview.box.firstElementChild?.animate(prefs.slides ? [{ opacity: 0, translate: "0 calc(-1 * var(--space-s8))" }, { opacity: 1, translate: "0 0" }] : [{ opacity: 0 }, { opacity: 1 }], { duration: ms("medium"), easing: curve("emphasized_decel") });
            else show(one.shows);
          },
        },
      );
      return made.box;
    };
    const finds: Switch[] = (
      [
        ["actions", "Settings and KubaOS's actions", "settings", "wifi"],
        ["projects", "Projects and scripts", "folder_open", "kubaos"],
        ["windows", "Windows and key binds", "desktop_windows", "move"],
        ["clipboard", "Clipboard history", "content_paste", "clip"],
        ["emoji", "Emoji", "mood", "emoji"],
        ["calculator", "Sums", "calculate", "3/2"],
      ] as const
    ).map(([id, title, glyph, shows]) => ({ spec: { title, icon: glyph }, get: () => prefs.finds[id], set: (on: boolean) => (prefs.finds[id] = on), shows }));
    const engine = select({ options: [...ENGINES], picked: prefs.engine, onPick: (at) => ((prefs.engine = at), show("zzz")) });
    const mark = select({ options: [`${figure(1.5, false)} (point)`, `${figure(1.5, true)} (comma)`], picked: prefs.comma ? 1 : 0, onPick: (at) => ((prefs.comma = at === 1), show(prefs.comma ? "1,5*3" : "1.5*3")) });
    const rest = descriptionList([
      { label: "Aliases", value: "Your own short words: ff for Firefox" },
      { label: "Quicklinks", value: "gh opens GitHub; yt cats searches YouTube" },
      { label: "Your actions", value: "Any command, with a title and keywords" },
      { label: "All of it in", value: "~/.config/kubaos/menu/kotodama/user.toml" },
    ]);
    const advanced = collapsible({
      title: "What it finds, and the rest",
      summary: "Six switches, the engine, the mark",
      open: state.more,
      content: stack("s16", "start", filling(switchList(finds)), filling(splitColumns([1, 1], "s16", [filling(stack("s8", "start", text("Search the web with", "label", "on_surface_variant"), filling(engine))), filling(stack("s8", "start", text("Decimal mark", "label", "on_surface_variant"), filling(mark)))])), filling(stack("s8", "start", text("The rest, when you want it", "label", "on_surface_variant"), filling(rest)))),
    });
    const settingsColumn = stack("s16", "start", text("Opens with", "label", "on_surface_variant"), filling(keysGroup), filling(switchList(switches)), filling(advanced));
    // What scrolls stands the page's padding in from the scroll's end, its thumb in that padding,
    // as every Kōbō page's body does (`kobo.css`).
    const padded = stack(0, "start", filling(settingsColumn));
    padded.classList.add("kobo-scrolled");
    const { box: scrolls } = scroll(padded, { height: "100%", below: "s24", noGutter: true, fade: "s24", footFade: true });
    scrolls.classList.add("kobo-scrolls");
    const right = stack("s8", "start", text("As it will look", "label", "on_surface_variant"), fillingHeight(filling(preview.box)));
    return splitColumns([2, 1], "s24", [filling(fillingHeight(scrolls)), filling(fillingHeight(right))], { fillHeight: true });
  };

  // --------------------------------------------------------------- Try it --

  let openMenu = () => undefined as void;

  const tryIt = (): HTMLElement => {
    const tries = listWith(
      TRIES.map((each) => each.spec(state.tried.has(each.id))),
      {
        onTake: (at) => {
          const each = TRIES[at];
          if (!each) return;
          openMenu();
          if (each.id === "wifi") void menu.type("wifi");
          if (each.id === "sum") void menu.type("12*7");
        },
      },
    );
    const tick = (id: TryId) => {
      if (state.tried.has(id)) return;
      state.tried.add(id);
      const at = TRIES.findIndex((each) => each.id === id);
      const fresh = tries.replaceRow(at, TRIES[at]!.spec(true));
      changed(fresh.querySelector(".k-list-box .k-check-mark"));
      recheck();
    };
    const holder = document.createElement("span");
    holder.className = "setup-kotodama-try";
    const menu = standIn(prefs, {
      live: true,
      query: state.query,
      ran: (each, typed) => {
        if (each.item?.id === "wifi") tick("wifi");
        if (each.answer && typed.replace(/\s/g, "") === "12*7") tick("sum");
      },
      onEscape: () => shut(),
    });
    const invitation = stack("s16", "centre", caps(KEY_OF[prefs.key]), wrapped("Press it. Or this, just this once.", "body", "on_surface_variant"), button({ label: "Open it", kind: "Tonal", icon: "keyboard_command_key" }));
    invitation.classList.add("setup-kotodama-invitation");
    invitation.querySelector("button")?.addEventListener("click", () => openMenu());
    const showMenu = (opening: boolean) => {
      holder.replaceChildren(menu.box);
      if (!opening || settings().reduced) return;
      menu.box.animate(prefs.slides ? [{ opacity: 0, translate: "0 calc(-1 * var(--space-s8))" }, { opacity: 1, translate: "0 0" }] : [{ opacity: 0 }, { opacity: 1 }], { duration: ms("medium"), easing: curve("emphasized_decel") });
    };
    const shut = () => {
      state.open = false;
      holder.replaceChildren(invitation);
      arrive(invitation);
    };
    openMenu = () => {
      if (!state.open) {
        state.open = true;
        showMenu(true);
      }
      tick("open");
      requestAnimationFrame(() => menu.focus());
    };
    if (state.open) showMenu(false);
    else holder.append(invitation);
    const tip = row("s8", "start", icon("info", "var(--type-small-icon)", "var(--color-on_surface_variant)"), filling(wrapped("It opens wherever you are: over any app, on the screen you're on.", "small", "on_surface_variant")));
    return splitColumns([2, 3], "s24", [filling(stack("s16", "start", filling(tries.box), fillingHeight(gap("s4")), filling(tip))), filling(fillingHeight(holder))], { fillHeight: true });
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
    window.addEventListener("keydown", (event) => {
      if (!element?.isConnected || element.closest("[data-kept]")) return;
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
      if (!element?.isConnected || (event.key !== "Meta" && event.key !== "OS") || !alone) return;
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
    if (state.gag) element.append(gag(true));
    listen();
    return element;
  };

  const status = (): RailStatus => ({ state: { is: state.passed ? "done" : "todo" } });

  const NEXT: Record<StepIndex, string> = { 0: "Next: why", 1: "Next: yours", 2: "Next: try it", 3: "Next: Git" };

  return {
    state,
    body: draw,
    hints: (): readonly KeyHint[] => {
      if (state.step === 1) return [["Tab", "the question"], ["Enter", "next"], ["Esc", "back a step"]];
      if (state.step === 2) return [["← →", "pick the keys"], ["Space", "switch it"], ["Tab", "next field"]];
      if (state.step === 3) return [[KEY_OF[prefs.key], "open it"], ["↑ ↓", "pick a Result"], ["Esc", "clear, close"]];
      return [["Enter", "next"], ["Esc", "back a card"], ["Tab", "next field"]];
    },
    page: (skipped) => ({
      step: { title: spec.title, icon: spec.icon },
      body: draw,
      status,
      next: () => NEXT[state.step],
      skip: () => (state.passed ? undefined : skipped() ? "Later" : "Skip"),
      note: () => (state.step === 3 ? (state.tried.size === 3 ? "All three. That's the whole trick." : `${state.tried.size} of 3 tried`) : undefined),
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
