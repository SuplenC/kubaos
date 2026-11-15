// Kōbō's first boot: one window on the new desktop, floated in the middle of the screen at the size
// `config/hypr/rules.lua` gives it, and in it either a scene (the arrival, the ending) or the step
// rail of the cards. Live, it is the whole first boot: Let's go deals the cards, each card's work
// runs and finishes in place, the last one hands over to the ending, and the ending to the desktop.
//
// Unlike the installer it follows the desktop's mode: it is the first window of the installed
// system, which has a day as well as a night.
//
// Every move is a recipe (`Track`s) the pages' players show and the live window plays.

import "./kobo.css";
import "./setup.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import blurred from "../../../../../crates/kubaos-kobo/assets/wallpaper-blur.jpg?url";
import markSvg from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-mark.svg?raw";
import wordmark from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-wordmark.svg?raw";
import { backdrop, glassPanel } from "../../lib/backdrop.ts";
import { button } from "../../lib/button.ts";
import { focusRing, windowRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { type KeyHint, keyHints } from "../../lib/keycap.ts";
import { gap, row, stack } from "../../lib/layout.ts";
import { nested, role } from "../../lib/radius.ts";
import { spinner } from "../../lib/spinner.ts";
import { koboBrand, RECHECK, stepRailPage } from "../../lib/step-rail.ts";
import { text, wrapped } from "../../lib/text.ts";
import type { Duration, Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { milliseconds } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import { RELEASE } from "./kobo-data.ts";
import { hanko, neonMark, neonSign, play, sealed, writing } from "./kobo-room.ts";
import { CARD_IDS, type CardId, CARDS, firstViews, open, running, type Views } from "./setup-cards.ts";
import { cardBody, cardPage } from "./setup-parts.ts";

/** What the arrival waits on or finds: detection running, no network, nothing left, a reboot behind it. */
export type Moment = "rest" | "detecting" | "offline" | "settled" | "rebooted";

/** Where first boot is. */
export type Place =
  | { readonly at: "arrival"; readonly moment?: Moment }
  | { readonly at: "card"; readonly card: CardId }
  | { readonly at: "ending"; readonly going?: boolean }
  | { readonly at: "desktop" };

export interface SetupOptions {
  readonly place: Place;
  readonly views?: Partial<Views>;
  /** Cards passed by with Skip while they wanted something. */
  readonly skipped?: readonly CardId[];
  /** The desktop round the window, as the person meets it. */
  readonly desk?: boolean;
  /** It takes the keyboard and the pointer and goes on through first boot. */
  readonly live?: boolean;
  /** Under the hood open on the card shown. */
  readonly hood?: boolean;
  /** A move's first state: the place it leaves, drawn over the one it arrives at. */
  readonly leaving?: Place;
  /** The arrival's hero line, at the toolbar's length. */
  readonly hero?: string;
  /** The place it opens on plays its arrival as it is drawn. */
  readonly opening?: boolean;
}

/** A laptop's screen, and the window Hyprland floats on it: 80 % of its width, 85 % of its height. */
export const DESK = { width: "calc(1280 * var(--px))", height: "calc(800 * var(--px))" };
export const WINDOW = { width: "calc(1024 * var(--px))", height: "calc(680 * var(--px))" };

export const HERO = { short: "You're in.", long: "You're in, and the hard part is done." };

/** The keys the rail lists at its foot: what nothing on the card shows. */
const HINTS: readonly KeyHint[] = [
  ["↑ ↓", "pick a card"],
  ["Space", "open it"],
  ["Tab", "next field"],
];

// ---------------------------------------------------------------- Playing --

function ms(token: Duration): number {
  return milliseconds(token) / settings().speed;
}

/** Which of a move's track sets plays now: reduced, the flicker off, or its own. */
function tracksNow(move: Move): readonly Track[] {
  const { reduced, flicker } = settings();
  if (reduced) return move.reduced === "same" ? move.tracks : move.reduced;
  if (!flicker && move.unflickered) return move.unflickered;
  return move.tracks;
}

/** A part rising `space.s24` into place as it fades in, over `long`, from `cue`. */
function rising(target: string, name: string, on: string, cue?: Duration | NonEmpty<Duration>): Track<"emphasized_decel">[] {
  const timing = cue === undefined ? {} : { cue };
  return [
    { part: `${name}'s fade`, on, target, property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel", ...timing },
    { part: `${name}'s rise`, on, target, property: "--kobo-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel", ...timing },
  ];
}

function fading(target: string, name: string, on: string, from = 0, to = 1): Track<"standard"> {
  return { part: `${name}'s fade`, on, target, property: "opacity", from, to, duration: "short", curve: "standard" };
}

// ----------------------------------------------------------------- Scenes --

function part<E extends HTMLElement>(element: E, name: string): E {
  element.classList.add("kobo-part", `setup-${name}`);
  return element;
}

/** Key hints that keep a keycap's line of room with none to show, so nothing under them moves. */
function keys(hints: readonly KeyHint[]): HTMLElement {
  const line = keyHints(hints);
  line.classList.add("kobo-keys");
  return line;
}

/** A line under a scene's actions: a symbol and its words. */
function promise(glyph: string, words: string, tone = "on_surface_variant"): HTMLElement {
  return row("s8", "centre", icon(glyph, "var(--type-small-icon)", `var(--color-${tone})`), wrapped(words, "small", tone));
}

/** How many cards want the person, said as a count. */
function wanted(views: Views): string {
  const left = CARD_IDS.filter((id) => open(CARDS[id].views[views[id]]!)).length;
  const words = ["None", "One", "Two", "Three", "Four", "Five", "Six"];
  return `${words[left] ?? left} ${left === 1 ? "thing" : "things"}`;
}

/** The arrival's words for what it found. */
function arrivalWords(moment: Moment, views: Views): { hero?: string; lede: string } {
  switch (moment) {
    case "settled":
      return { hero: "Nothing to do.", lede: "Every card is done or not on this machine. Setup checks again each time it opens, so done stays done." };
    case "rebooted":
      return { hero: "Back again.", lede: `The TUXEDO drivers loaded, which is what this was waiting for. ${wanted(views)} left, if you like.` };
    case "detecting":
      return { lede: "KubaOS is on the disk and running. Checking what this machine has: the reader, the keys, the vault." };
    default:
      return { lede: `KubaOS is on the disk and running. ${wanted(views)} need your fingers, your keys or your phone, so the install left them to you. A minute each; skip any, it'll wait.` };
  }
}

/** The arrival: the first frame of KubaOS proper, the neon sign struck as on Kōbō's welcome, and one button. */
export function arrivalColumn(views: Views, moment: Moment = "rest", hero = HERO.short): HTMLElement {
  const words = arrivalWords(moment, views);
  const detecting = moment === "detecting";
  const settled = moment === "settled";
  const start = button({ label: settled ? "Go to the desktop" : "Let's go", size: "Lg", icon: "arrow_forward", trailing: true, ...(detecting ? { disabled: true } : {}) });
  start.classList.add("setup-start");
  const later = button({ label: "Later", kind: "Text", size: "Lg" });
  later.classList.add("setup-not-now");
  const actions = settled ? row("s8", "centre", start) : row("s8", "centre", start, later);
  if (!settled) actions.dataset.hangs = "";
  const hints: readonly KeyHint[] = detecting ? [] : settled ? [["Enter", "the desktop"]] : [["Enter", "start"], ["Esc", "later"]];
  const foot = detecting
    ? row("s8", "centre", spinner("s20"), text("Checking what this machine has…", "small", "on_surface_variant"))
    : moment === "offline"
      ? promise("wifi_off", "Offline: drivers, fprintd and Bitwarden need the internet. The rest works without it.", "warning")
      : row("s8", "centre", icon("info", "var(--type-small-icon)", "var(--color-on_surface_variant)"), text("All of it stays in the Command menu:", "small", "on_surface_variant"), keyHints([["Super+Space", "then “First boot”"]]));
  const box = stack(
    0,
    "centre",
    part(text("初期設定", "accent", "primary"), "accent"),
    gap("s8"),
    neonSign(),
    gap("s20"),
    part(wrapped(words.hero ?? hero, "hero"), "hero"),
    gap("s16"),
    part(wrapped(words.lede, "body_large", "on_surface_variant"), "lede"),
    gap("s40"),
    part(stack("s12", "centre", actions, keys(hints)), "act"),
    gap("s24"),
    part(foot, "promise"),
  );
  box.classList.add("kobo-part", "setup-column", "setup-arrival");
  return box;
}

/** The titles of the cards `test` picks, as a list in words. */
function named(test: (id: CardId) => boolean): string {
  const titles = CARD_IDS.filter(test).map((id) => CARDS[id].title);
  return titles.length < 2 ? (titles[0] ?? "") : `${titles.slice(0, -1).join(", ")} and ${titles.at(-1)}`;
}

/** The ending's lede: what got done, what waits and where, or the reboot it needs. */
function endingLede(views: Views, skipped: ReadonlySet<CardId>): string {
  const state = (id: CardId) => CARDS[id].views[views[id]]!;
  const done = CARD_IDS.filter((id) => state(id).state === "done").length;
  const isLeft = (id: CardId) => open(state(id)) || skipped.has(id);
  const left = named(isLeft);
  const many = CARD_IDS.filter(isLeft).length > 1;
  const reboot = CARD_IDS.some((id) => state(id).state === "reboot");
  const tally = `${done} done.`;
  const waits = left ? ` ${left} ${many ? "wait" : "waits"} in the Command menu; I won't bring ${many ? "them" : "it"} up again.` : "";
  if (reboot) return `${tally}${waits} The TUXEDO drivers load after a reboot.`;
  if (left) return `${tally}${waits}`;
  return `${tally} Your machine, your rules. Go break something.`;
}

/** The ending, as Kōbō's done screen: クバ writes itself in, 完了 stamped beside it, the headline word by word. */
export function endingColumn(views: Views, skipped: ReadonlySet<CardId>, going = false): HTMLElement {
  const reboot = CARD_IDS.some((id) => CARDS[id].views[views[id]]!.state === "reboot");
  const drawn = neonMark();
  const seal = part(hanko(), "stamp");
  seal.classList.add("kobo-stamp");
  const mark = row("s8", "start", drawn, seal);
  mark.classList.add("kobo-sealed");
  sealed(mark, drawn);
  const words = "That's it.".split(" ").map((word, index) => part(text(index === 0 ? word : ` ${word}`, "hero"), `word-${index}`));
  const headline = row(0, "end", ...words);
  headline.classList.add("setup-headline");
  const desktop = button({ label: "Go to the desktop", size: "Lg", kind: reboot ? "Text" : "Filled", ...(reboot ? {} : { icon: "arrow_forward", trailing: true }) });
  desktop.classList.add("setup-desktop");
  const restart = button({ label: "Reboot now", size: "Lg", icon: "restart_alt" });
  restart.classList.add("setup-reboot");
  const actions = going ? row("s8", "centre", spinner("s20"), text("Restarting…", "body", "on_surface_variant")) : reboot ? row("s8", "centre", restart, desktop) : row("s8", "centre", desktop);
  if (reboot && !going) actions.dataset.hangs = "";
  actions.classList.add("setup-actions");
  const hints: readonly KeyHint[] = going ? [] : reboot ? [["Enter", "reboot"], ["Tab", "move"]] : [["Enter", "the desktop"]];
  const parts: HTMLElement[] = [
    mark,
    gap("s24"),
    headline,
    gap("s16"),
    part(wrapped(endingLede(views, skipped), "body_large", "on_surface_variant"), "lede"),
    gap("s40"),
    part(stack("s12", "centre", actions, keys(hints)), "act"),
  ];
  // The blue screen comes right after this reboot, so its steps are said here, where they are needed.
  const queued = CARD_IDS.some((id) => id === "tuxedo" && views[id] === "queued");
  if (queued) parts.push(gap("s24"), part(promise("key", "On the blue screen after it: Enroll MOK → Continue → Yes → your one-time password → Reboot."), "promise"));
  const box = stack(0, "centre", ...parts);
  box.classList.add("kobo-part", "setup-column", "setup-ending");
  return box;
}

// ----------------------------------------------------------------- The room --

/** The first open card, or the first that is there at all. */
function firstCard(views: Views): CardId {
  return CARD_IDS.find((id) => open(CARDS[id].views[views[id]]!)) ?? CARD_IDS.find((id) => CARDS[id].views[views[id]]!.state !== "absent") ?? "git";
}

/** How far the window lies from the desktop's edges: Hyprland centres it. */
const AWAY = { left: "calc((1280 - 1024) / 2 * var(--px))", top: "calc((800 - 680) / 2 * var(--px))" };

/**
 * The window: Kōbō's glass panel, the wallpaper's blurred copy lying where the wallpaper lies
 * under it, its scenes and its rail on the glass, one over another as they hand over. On the
 * desktop it is the active window, its corner the frame's less the gap, its tube lit, as
 * `config/hypr/looks.lua` and `focus.lua` draw it; drawn alone it is the window's own size.
 */
function windowBox(desk: boolean): { win: HTMLElement; body: HTMLElement } {
  const win = document.createElement("span");
  win.className = "setup-window";
  win.style.setProperty("--setup-window-width", WINDOW.width);
  win.style.setProperty("--setup-window-height", WINDOW.height);
  win.style.setProperty("--panel-left", desk ? AWAY.left : "0px");
  win.style.setProperty("--panel-margin", desk ? AWAY.top : "0px");
  const corner = nested(role("surface"), "s8");
  win.style.setProperty("--corner", corner.css);
  const glass = glassPanel(blurred);
  win.append(glass);
  if (desk) {
    windowRing(focusRing(win, "surface", "accent", false), "stroke-thin", corner.css);
    win.dataset.pinned = "lit";
  }
  return { win, body: glass.querySelector<HTMLElement>(".k-glass-body")! };
}

export function setupRoom(options: SetupOptions): HTMLElement {
  const views: Record<CardId, string> = { ...firstViews(), ...options.views };
  const skipped = new Set(options.skipped ?? []);
  const opened = new Set<CardId>(options.hood && options.place.at === "card" ? [options.place.card] : []);
  const { win, body } = windowBox(options.desk ?? false);
  win.tabIndex = -1;
  const size = options.desk ? DESK : WINDOW;
  const element = backdrop({ width: size.width, height: size.height, picture: wallpaper, scrim: "band", content: win, dark: 0 });
  element.classList.add("setup-room");
  element.dataset.whole = "";
  let place: Place = options.place;
  const timers = new Set<number>();
  // The cards' bodies shown now, each redrawn in place as its card's work moves on.
  const holders = new Map<CardId, HTMLElement>();

  const drawCard = (id: CardId) => {
    const holder = holders.get(id) ?? document.createElement("span");
    holder.className = "setup-card";
    holder.replaceChildren(cardBody(id, views[id], opened.has(id)));
    holders.set(id, holder);
    return holder;
  };

  /** A card's view changed: its body is drawn again in place, its band and its step follow. */
  const change = (id: CardId, to: string) => {
    const from = CARDS[id].views[views[id]]!;
    views[id] = to;
    const holder = holders.get(id);
    if (!holder?.isConnected) return;
    drawCard(id);
    holder.dispatchEvent(new Event(RECHECK, { bubbles: true }));
    const now = CARDS[id].views[to]!;
    const move = now.failed ? FAILS : now.prompt?.count && from.prompt?.count ? TOUCH : !running(now) && running(from) ? finishing(CARD_IDS.indexOf(id)) : null;
    if (move) void play(element, tracksNow(move));
    const next = CARDS[id].auto[to];
    if (next) {
      const timer = window.setTimeout(() => {
        timers.delete(timer);
        if (views[id] === to) change(id, next[0]);
      }, ms(next[1]));
      timers.add(timer);
    }
  };

  const act = (id: CardId) => (): boolean => {
    const at = views[id];
    if (CARDS[id].action(at)?.label === "Reboot now") {
      go({ at: "ending", going: true });
      return true;
    }
    const to = CARDS[id].act[at];
    if (!to) return false;
    change(id, to);
    return true;
  };

  const railLayer = (id: CardId): HTMLElement => {
    const pages = CARD_IDS.map((each) => ({ ...cardPage(each, () => views, () => drawCard(each), act(each), () => skipped.has(each)), skipped: () => skipped.add(each) }));
    const rail = stepRailPage({
      host: "window",
      brand: koboBrand(markSvg, wordmark, `First boot · ${RELEASE.codename} ${RELEASE.version}`),
      pages,
      current: CARD_IDS.indexOf(id),
      free: true,
      hints: () => HINTS,
      ...(options.live ? { onExit: (forward: boolean) => go(forward ? { at: "ending" } : { at: "arrival" }) } : {}),
    });
    rail.classList.add("setup-layer", "setup-rail");
    return rail;
  };

  const sceneLayer = (at: Place): HTMLElement => {
    const layer = document.createElement("span");
    layer.className = "setup-layer setup-scene";
    const column = at.at === "arrival" ? arrivalColumn(views, at.moment ?? "rest", options.hero) : endingColumn(views, skipped, at.at === "ending" && (at.going ?? false));
    layer.append(column);
    return layer;
  };

  const layerOf = (at: Place): HTMLElement | null => (at.at === "card" ? railLayer(at.card) : at.at === "desktop" ? null : sceneLayer(at));

  /** Draws `at`'s layer, over `leaving`'s as a move's first state, and returns it. */
  const draw = (at: Place, leaving?: Place): HTMLElement | null => {
    const arriving = layerOf(at);
    const layers = [leaving ? layerOf(leaving) : null, arriving].filter((each): each is HTMLElement => each !== null);
    // A scene lies over the rail it hands over to or from.
    layers.sort((a, b) => Number(a.classList.contains("setup-scene")) - Number(b.classList.contains("setup-scene")));
    // The window's tube stays: only its layers change.
    for (const old of body.querySelectorAll(":scope > .setup-layer")) old.remove();
    body.prepend(...layers);
    win.toggleAttribute("data-left", at.at === "desktop" && !leaving);
    return arriving;
  };

  const between = (from: Place, to: Place): Move | null => {
    if (from.at === "arrival" && to.at === "card") return CARDS_IN;
    if (from.at === "card" && to.at === "arrival") return BACK;
    if (from.at === "card" && to.at === "ending") return to.going ? null : ENDS;
    if (from.at === "ending" && to.at === "desktop") return DESKTOP;
    if (to.at === "ending" && to.going) return REBOOT;
    return null;
  };

  function go(to: Place): void {
    const from = place;
    place = to;
    for (const timer of timers) window.clearTimeout(timer);
    timers.clear();
    const move = between(from, to);
    if (to.at === "ending" && to.going && from.at === "ending") {
      win.querySelector(".setup-ending .setup-actions")?.replaceWith(row("s8", "centre", spinner("s20"), text("Restarting…", "body", "on_surface_variant")));
      win.querySelector(".setup-ending .kobo-keys")?.replaceChildren();
    }
    const kept = to.at === "ending" && to.going && from.at === "ending" ? body.querySelector(":scope > .setup-layer") : to.at === "desktop" ? null : draw(to, move ? from : undefined);
    // The move's end is the new place drawn: only the layer it left goes.
    const settle = () => {
      if (place !== to) return;
      for (const old of body.querySelectorAll(":scope > .setup-layer")) if (old !== kept) old.remove();
      win.toggleAttribute("data-left", to.at === "desktop");
      (win.querySelector<HTMLElement>(".setup-start, .setup-reboot, .setup-desktop") ?? win.querySelector<HTMLElement>(".k-step-room:not([data-kept]) [tabindex='0'], .k-step-room:not([data-kept]) button:not(:disabled)") ?? win).focus({ preventScroll: true });
    };
    if (!move) return settle();
    void play(element, tracksNow(move)).then(settle);
  }

  draw(place, options.leaving);

  if (options.live) {
    element.addEventListener("click", (event) => {
      const target = event.target as HTMLElement;
      if (target.closest(".setup-start")) go(place.at === "arrival" && place.moment === "settled" ? { at: "desktop" } : { at: "card", card: firstCard(views) });
      else if (target.closest(".setup-not-now")) go({ at: "desktop" });
      else if (target.closest(".setup-desktop")) go({ at: "desktop" });
      else if (target.closest(".setup-reboot")) go({ at: "ending", going: true });
      else if (target.closest(".k-collapsible-header") && place.at === "card") {
        const id: CardId = place.card;
        if (opened.has(id)) opened.delete(id);
        else opened.add(id);
      }
    });
    element.addEventListener("keydown", (event) => {
      if (event.defaultPrevented || event.repeat || place.at === "card") return;
      if (event.key === "Enter" && place.at === "arrival" && place.moment !== "detecting") go(place.moment === "settled" ? { at: "desktop" } : { at: "card", card: firstCard(views) });
      else if (event.key === "Escape" && place.at === "arrival") go({ at: "desktop" });
      else if (event.key === "Enter" && place.at === "ending" && !place.going) {
        const reboot = CARD_IDS.some((id) => CARDS[id].views[views[id]]!.state === "reboot");
        go(reboot ? { at: "ending", going: true } : { at: "desktop" });
      } else return;
      event.preventDefault();
    });
  }
  if (options.opening) requestAnimationFrame(() => void play(element, tracksNow(place.at === "arrival" ? OPENS : place.at === "ending" ? ARRIVES : OPENS_CARD)));
  return element;
}

// ---------------------------------------------------------------- The moves --

const ARRIVAL = "the arrival";

/** `cue` after `first`: the sum of both, or `first` alone, or nothing. */
function after(first: readonly Duration[], cue?: Duration | NonEmpty<Duration>): { cue?: NonEmpty<Duration> } {
  const all = [...first, ...(cue === undefined ? [] : Array.isArray(cue) ? cue : [cue])];
  return all.length > 0 ? { cue: all as unknown as NonEmpty<Duration> } : {};
}

const OPENING_TRACKS = (strike: "neon.strike" | "standard"): Track[] => [
  ...rising(".setup-arrival > .setup-accent", "the accent", ARRIVAL),
  { part: "the sign", on: ARRIVAL, target: ".setup-arrival .kobo-sign", property: "--kobo-sign", from: 0, to: 1, duration: "medium", cue: "short", curve: strike },
  ...rising(".setup-arrival > .setup-hero", "the hero line", ARRIVAL, ["short", "medium"]),
  ...rising(".setup-arrival > .setup-lede", "the lede", ARRIVAL, ["short", "medium", "micro"]),
  ...rising(".setup-arrival > .setup-act", "the actions", ARRIVAL, ["short", "medium", "short"]),
  ...rising(".setup-arrival > .setup-promise", "the promise", ARRIVAL, ["short", "medium", "medium"]),
];

/** The window opens on the desktop: the sign strikes as on Kōbō's welcome, the words rise in. */
export const OPENS: Move = {
  name: "Setup opens",
  what: "The accent rises in; the sign strikes as Kōbō's welcome's does; the words rise in one after another.",
  tracks: OPENING_TRACKS("neon.strike") as unknown as NonEmpty<Track>,
  unflickered: OPENING_TRACKS("standard") as unknown as NonEmpty<Track<Easing>>,
  reduced: [fading(".setup-arrival", "the arrival", ARRIVAL)],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "arrival" }, desk: true }),
};

/** Opened on a card from the Command menu: the rail page arrives as the template's pages do. */
const OPENS_CARD: Move = {
  name: "Opened on a card",
  what: "The card's page slides in and fades in over `long`.",
  tracks: [
    { part: "slide", on: "the page", target: ".k-step-room:not([data-kept])", property: "--step-slide", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "fade", on: "the page", target: ".k-step-room:not([data-kept])", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [fading(".k-step-room:not([data-kept])", "the page", "the page")],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "fingerprint" } }),
};

/** Let's go: the arrival's words rise off and the rail comes in on the first open card. */
export const CARDS_IN: Move = {
  name: "Let's go: the cards",
  what: "The arrival rises off and fades over `medium`; then, on the glass alone, the rail fades in, its first card sliding in.",
  tracks: [
    { part: "the arrival's rise", on: ARRIVAL, target: ".setup-scene .setup-arrival", property: "--kobo-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the arrival's fade", on: ARRIVAL, target: ".setup-scene", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the rail's fade", on: "the rail", target: ".setup-rail", property: "opacity", from: 0, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the card's slide", on: "the card", target: ".k-step-room:not([data-kept])", property: "--step-slide", from: 1, to: 0, duration: "long", cue: "medium", curve: "emphasized_decel" },
  ],
  reduced: [fading(".setup-scene", "the arrival", ARRIVAL, 1, 0), fading(".setup-rail", "the rail", "the rail")],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "tuxedo" }, leaving: { at: "arrival" } }),
};

/** Esc on the first card: the rail fades and the arrival comes back down. */
export const BACK: Move = {
  name: "Back to the arrival",
  what: "Esc or Back on the first card: the rail fades over `medium`; then the arrival settles back down.",
  tracks: [
    { part: "the rail's fade", on: "the rail", target: ".setup-rail", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the arrival's fade", on: ARRIVAL, target: ".setup-scene", property: "opacity", from: 0, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the arrival's fall", on: ARRIVAL, target: ".setup-scene .setup-arrival", property: "--kobo-rise", from: -1, to: 0, duration: "long", cue: "medium", curve: "emphasized_decel" },
  ],
  reduced: [fading(".setup-rail", "the rail", "the rail", 1, 0), fading(".setup-scene", "the arrival", ARRIVAL)],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "arrival" }, leaving: { at: "card", card: "tuxedo" } }),
};

/** Continue or Skip: the next card slides in from the right as the wire's light grows to it. */
export function onward(sample: () => HTMLElement): Move {
  return {
    name: "Continue: the next card",
    what: "It slides `space.s32` in and fades in over `long`; the wire stays: its light is what's done.",
    tracks: [
      { part: "slide", on: "the card", target: ".k-step-room:not([data-kept])", property: "--step-slide", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
      { part: "fade", on: "the card", target: ".k-step-room:not([data-kept])", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    ],
    reduced: [fading(".k-step-room:not([data-kept])", "the card", "the card")],
    back: "cuts",
    sample,
  };
}

/** A card's work finished: its status settles in and the rail's step takes its check. */
export function finishing(index: number, sample: () => HTMLElement = () => setupRoom({ place: { at: "card", card: CARD_IDS[index]! }, views: { [CARD_IDS[index]!]: "done", [CARD_IDS[index === 0 ? 1 : index - 1]!]: "done" } })): Move {
  return {
    name: "Done: the card settles",
    what: "The prompt gives way at once; the status, the rail's check and the wire it joins fade in over `short`.",
    tracks: [
      fading(".k-step-room:not([data-kept]) .setup-status", "the status", "the card"),
      fading(`.k-rail-steps > .k-step:nth-child(${index + 1}) > * > :last-child`, "the check", "the rail"),
      fading(`.k-rail-wire-lit[data-steps~="${index}"]`, "the wire's light", "the rail"),
    ],
    reduced: "same",
    back: "cuts",
    sample,
  };
}

/** A touch counted: the bar fills to it, the count rolls, the next words fade in. */
export const TOUCH: Move = {
  name: "A touch counted",
  what: "The bar fills to the new count over `short`; the count rolls; the next words fade in.",
  tracks: [
    { part: "the bar", on: "the prompt", target: ".setup-prompt .k-progress", property: "--progress", from: 2 / 8, to: 3 / 8, duration: "short", curve: "standard_decel" },
    fading(".setup-prompt .setup-prompt-words", "the words", "the prompt"),
  ],
  reduced: "same",
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "fingerprint" }, views: { fingerprint: "touching" } }),
};

/** Waiting on a hand or a device: the prompt's symbol hums until it comes. */
export const HUM: Move = {
  name: "Waiting on a hand",
  what: "The symbol hums down to `opacity.hum` and back over `hum`, until the touch comes; reduced, it holds still.",
  tracks: [{ part: "the symbol", on: "the prompt", target: ".setup-prompt .setup-hum", property: "opacity", from: 1, to: { opacity: "hum" }, duration: "hum", curve: "standard" }],
  reduced: [{ part: "the symbol", on: "the prompt", target: ".setup-prompt .setup-hum", property: "opacity", from: 1, to: 1, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: () => setupRoom({ place: { at: "card", card: "fingerprint" }, views: { fingerprint: "touch" } }),
};

/** A run fails: the prompt gives way and the error block fades in, its rail lit. */
export const FAILS: Move = {
  name: "A run fails",
  what: "The prompt gives way at once; the error block fades in over `short`; Under the hood opens.",
  tracks: [fading(".k-step-room:not([data-kept]) .k-error-block", "the error block", "the card")],
  reduced: "same",
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "fingerprint" }, views: { fingerprint: "failed" } }),
};

const ENDING = "the ending";

const ENDING_TRACKS = (first: Duration[], strike: "neon.strike" | "standard"): Track[] => [
  ...writing().map((track) => ({ ...track, on: ENDING, target: `.setup-ending ${track.target}`, ...after(first, track.cue) })),
  { part: "the mark, struck", on: ENDING, target: ".setup-ending .kobo-mark", property: "--kobo-burn", from: 0, to: 1, duration: "medium", ...after(first, "scene"), curve: strike },
  { part: "the hanko", on: ENDING, target: ".setup-ending .kobo-stamp", property: "--kobo-stamp", from: 0, to: 1, duration: "short", ...after(first, ["scene", "medium"]), curve: "pop" },
  ...rising(".setup-ending .setup-word-0", "“That's”", ENDING, after(first, ["scene", "medium"]).cue),
  ...rising(".setup-ending .setup-word-1", "“it.”", ENDING, after(first, ["scene", "medium", "micro"]).cue),
  ...rising(".setup-ending > .setup-lede", "the lede", ENDING, after(first, ["scene", "medium", "short"]).cue),
  ...rising(".setup-ending > .setup-act", "the actions", ENDING, after(first, ["scene", "medium", "medium"]).cue),
];

/** The ending shown on its own, opened on it. */
const ARRIVES: Move = {
  name: "That's it",
  what: "クバ writes itself in and strikes; 完了 is stamped beside it; the headline rises word by word.",
  tracks: ENDING_TRACKS([], "neon.strike") as unknown as NonEmpty<Track>,
  unflickered: ENDING_TRACKS([], "standard") as unknown as NonEmpty<Track<Easing>>,
  reduced: [fading(".setup-ending", "the ending", ENDING)],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "ending" }, views: { tuxedo: "done", fingerprint: "done", yubikey: "done", git: "done" }, skipped: ["bitwarden", "phone"], desk: true }),
};

/** Past the last card: the rail fades off and the ending arrives over it. */
export const ENDS: Move = {
  name: "Past the last card: That's it",
  what: "The rail fades over `medium`; クバ writes itself in and strikes, 完了 stamped beside it; the headline rises.",
  tracks: [{ part: "the rail's fade", on: "the rail", target: ".setup-rail", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }, ...ENDING_TRACKS(["medium"], "neon.strike")] as unknown as NonEmpty<Track>,
  unflickered: [{ part: "the rail's fade", on: "the rail", target: ".setup-rail", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }, ...ENDING_TRACKS(["medium"], "standard")] as unknown as NonEmpty<Track<Easing>>,
  reduced: [fading(".setup-rail", "the rail", "the rail", 1, 0), fading(".setup-scene", "the ending", ENDING)],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "ending" }, views: { tuxedo: "done", fingerprint: "done", yubikey: "done", git: "done" }, skipped: ["bitwarden", "phone"], leaving: { at: "card", card: "phone" } }),
};

/** Go to the desktop: the window steps back and is gone; the desktop is what's left. */
export const DESKTOP: Move = {
  name: "Go to the desktop",
  what: "The window shrinks to `scale.behind` round its middle as it fades over `medium`: the desktop is yours.",
  tracks: [
    { part: "the window's scale", on: "the window", target: ".setup-window", property: "scale", from: 1, to: { scale: "behind" }, duration: "medium", curve: "emphasized_accel" },
    { part: "the window's fade", on: "the window", target: ".setup-window", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "the window's fade", on: "the window", target: ".setup-window", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "ending" }, views: { tuxedo: "done", fingerprint: "done", yubikey: "done", git: "done" }, skipped: ["bitwarden", "phone"], desk: true }),
};

/** Reboot now: Restarting…, and the desktop goes dark with the window. */
export const REBOOT: Move = {
  name: "Reboot: the lights go out",
  what: "Restarting… in the actions' place; the desktop and the window go dark over `slow`.",
  tracks: [
    { part: "the window's fade", on: "the window", target: ".setup-window", property: "opacity", from: 1, to: 0, duration: "slow", cue: "medium", curve: "standard_accel" },
    { part: "the dark", on: "the desktop", target: ".k-backdrop-dark", property: "opacity", from: 0, to: 1, duration: "slow", cue: "medium", curve: "standard_accel" },
  ],
  reduced: [
    { part: "the window's fade", on: "the window", target: ".setup-window", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the dark", on: "the desktop", target: ".k-backdrop-dark", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "ending", going: true }, views: { tuxedo: "queued", fingerprint: "done", git: "done" }, skipped: ["yubikey", "bitwarden", "phone"], desk: true }),
};
