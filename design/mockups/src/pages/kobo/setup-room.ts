// Kōbō's first boot: one full-screen place over the new desktop, as the installer is over its
// street. Its wallpaper lies under everything; on it the arrival or the ending, words over the
// picture, or the cards, the wallpaper gone out of focus behind glass with the step rail on it.
// Live, it is the whole first boot: Let's go deals the cards, each card's work runs and finishes in
// place, the last one hands over to the ending, and the ending to the desktop.
//
// It is a night place, as the installer is: the sign and the street are lit tubes in the dark, and
// a day band would wash them out. Opened from the Command menu on one card, over a desktop in use,
// it is a window instead, in the desktop's mode: a card is a tool you came to use, beside what you
// were doing.
//
// It is the installer's room in its parts and its moves: the same Backdrop scrims, StepRailPage's
// Kōbō host, the welcome's sign and the done screen's ending (`kobo-room.ts`), each move a recipe
// (`Track`s) the pages' players show and the live room plays.

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
import type { KeyHint } from "../../lib/keycap.ts";
import { gap, row, stack } from "../../lib/layout.ts";
import { nested, role } from "../../lib/radius.ts";
import { spinner } from "../../lib/spinner.ts";
import { koboBrand, RECHECK, stepRailPage } from "../../lib/step-rail.ts";
import { text, wrapped } from "../../lib/text.ts";
import type { Duration, Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { milliseconds } from "../../site/recipe.ts";
import { paint, palette } from "../../site/recolour.ts";
import { settings } from "../../site/settings.ts";
import { RELEASE } from "./kobo-data.ts";
import { TURNING_REDUCED, turning } from "./kobo-pages.ts";
import { after, arrival, column, ENTERING, endingColumn, GOING, keys, LEAVING, neonSign, OPENING, part, play } from "./kobo-room.ts";
import { CARD_IDS, type CardId, CARDS, firstViews, open, running, type Views } from "./setup-cards.ts";
import { cardBody, cardPage } from "./setup-parts.ts";
import { apps, installing, yoursCard } from "./setup-yours-card.ts";

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
  /** The screen's size: a laptop's unless given. */
  readonly width?: string;
  readonly height?: string;
  /** It takes the keyboard and the pointer and goes on through first boot. */
  readonly live?: boolean;
  /** Under the hood open on the card shown. */
  readonly hood?: boolean;
  /** A move's first state: the place it leaves, drawn with the one it arrives at. */
  readonly leaving?: Place;
  /** The card the rail turned from, drawn leaving over the one shown: a page turn's first state. */
  readonly turnedFrom?: CardId;
  /** The arrival's hero line, at the toolbar's length. */
  readonly hero?: string;
  /** The place it opens on plays its arrival as it is drawn. */
  readonly opening?: boolean;
  /** Opened from the Command menu on its card alone: a window on the desktop, the rail that card. */
  readonly alone?: boolean;
}

/** A laptop's screen, where the sample machine shows first boot. */
export const SCREEN = { width: "calc(1280 * var(--px))", height: "calc(800 * var(--px))" };

/** The window Hyprland floats a card opened alone in: 80 % of the screen's width, 85 % of its height. */
export const WINDOW = { width: "calc(1024 * var(--px))", height: "calc(680 * var(--px))" };

export const HERO = { short: "You're in.", long: "You're in, and the hard part is done." };

/** The ending's headline. */
const HEADLINE = "That's it.";

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

function fading(target: string, name: string, on: string, from = 0, to = 1): Track<"standard"> {
  return { part: `${name}'s fade`, on, target, property: "opacity", from, to, duration: "short", curve: "standard" };
}

// ----------------------------------------------------------------- Scenes --

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
      return { hero: "Back again.", lede: `The drivers loaded, which is what this was waiting for. ${wanted(views)} left, if you like.` };
    case "detecting":
      return { lede: "KubaOS is on the disk and running. Checking what this machine has: the maker, the reader, the keys." };
    default:
      return { lede: `KubaOS is on the disk and running. ${wanted(views)} to go: what you use it for, your name for Git, then what needs your hardware, your fingers or your phone. A minute each; skip any, it'll wait.` };
  }
}

/** The arrival: the first frame of KubaOS proper, laid out and struck as Kōbō's welcome is. */
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
      ? promise("wifi_off", "Offline: apps, drivers and fprintd need the internet. The rest works without it.", "warning")
      : row("s8", "centre", icon("info", "var(--type-small-icon)", "var(--color-on_surface_variant)"), text("All of it stays in the Command menu: Super, then “First boot”.", "small", "on_surface_variant"));
  // It is Kōbō's welcome, on the installed system: the welcome's column, moved as the welcome is.
  return column(
    "welcome",
    part(text("初期設定", "accent", "primary"), "accent"),
    gap("s8"),
    neonSign(),
    gap("s24"),
    part(wrapped(words.hero ?? hero, "hero"), "hero"),
    gap("s16"),
    part(wrapped(words.lede, "body_large", "on_surface_variant"), "lede"),
    gap("s40"),
    // The action and its keys are one thing, close; the line under them is a statement of its own.
    part(stack(0, "centre", actions, gap("s12"), keys(hints), gap("s24"), foot), "act"),
  );
}

/** The titles of the cards `test` picks, as a list in words. */
function named(test: (id: CardId) => boolean): string {
  const titles = CARD_IDS.filter(test).map((id) => CARDS[id].title);
  return titles.length < 2 ? (titles[0] ?? "") : `${titles.slice(0, -1).join(", ")} and ${titles.at(-1)}`;
}

function capitalised(words: string): string {
  return words.charAt(0).toUpperCase() + words.slice(1);
}

/** The ending's lede: what got done, what waits and where, the apps still coming, the reboot. */
function endingLede(views: Views, skipped: ReadonlySet<CardId>): string {
  const state = (id: CardId) => CARDS[id].views[views[id]]!;
  // Apps still installing aren't left to do: they finish on their own.
  const coming = views.yours === "installing";
  const done = CARD_IDS.filter((id) => state(id).state === "done").length;
  const isLeft = (id: CardId) => !(id === "yours" && coming) && (open(state(id)) || skipped.has(id));
  const left = named(isLeft);
  const many = CARD_IDS.filter(isLeft).length > 1;
  const reboot = CARD_IDS.some((id) => state(id).state === "reboot");
  const tally = done === 0 && !coming ? "Nothing done. Fair enough." : `${done} done.`;
  const waits = left ? ` ${left} ${many ? "wait" : "waits"} in the Command menu; I won't bring ${many ? "them" : "it"} up again.` : "";
  const landing = coming ? ` ${capitalised(apps(installing("installing")))} still installing; they show up as they land.` : "";
  if (reboot) return `${tally}${waits}${landing} The drivers load after a reboot.`;
  if (left || coming) return `${tally}${waits}${landing}`;
  return `${tally} Your machine, your rules. Go break something.`;
}

/** What the ending's actions say once Reboot now is taken. */
function restarting(): HTMLElement {
  return row("s8", "centre", spinner("s20"), text("Restarting…", "body", "on_surface_variant"));
}

/** The ending: the installer's done screen with first boot's words, the desktop handed over. */
export function endingScene(views: Views, skipped: ReadonlySet<CardId>, going = false): HTMLElement {
  const reboot = CARD_IDS.some((id) => CARDS[id].views[views[id]]!.state === "reboot");
  const desktop = button({ label: "Go to the desktop", size: "Lg", kind: reboot ? "Text" : "Filled", ...(reboot ? {} : { icon: "arrow_forward", trailing: true }) });
  desktop.classList.add("setup-desktop");
  const restart = button({ label: "Reboot now", size: "Lg", icon: "restart_alt" });
  restart.classList.add("setup-reboot");
  const actions = going ? restarting() : reboot ? row("s8", "centre", restart, desktop) : row("s8", "centre", desktop);
  if (reboot && !going) actions.dataset.hangs = "";
  const hints: readonly KeyHint[] = going ? [] : reboot ? [["Enter", "reboot"], ["Tab", "move"]] : [["Enter", "the desktop"]];
  // The blue screen comes right after this reboot, so its steps are said here, where they are needed.
  const queued = views.drivers === "queued";
  return endingColumn({
    headline: HEADLINE,
    lede: endingLede(views, skipped),
    actions,
    hints,
    ...(queued ? { promise: promise("key", "On the blue screen after it: Enroll MOK → Continue → Yes → your one-time password → Reboot.") } : {}),
  });
}

// ----------------------------------------------------------------- The room --

/** The first open card, or the first that is there at all. */
function firstCard(views: Views): CardId {
  return CARD_IDS.find((id) => open(CARDS[id].views[views[id]]!)) ?? CARD_IDS.find((id) => CARDS[id].views[views[id]]!.state !== "absent") ?? "yours";
}

/** How far the window lies from the screen's edges: Hyprland centres it. */
const AWAY = { left: "calc((1280 - 1024) / 2 * var(--px))", top: "calc((800 - 680) / 2 * var(--px))" };

/**
 * A card opened alone: a window on the desktop, Kōbō's glass panel at the window's size and corner,
 * the wallpaper's blurred copy lying where the wallpaper lies under it, its tube lit as the active
 * window's, as `config/hypr/looks.lua` and `focus.lua` draw it.
 */
function windowBox(): { win: HTMLElement; body: HTMLElement } {
  const win = document.createElement("span");
  win.className = "setup-window";
  win.style.setProperty("--setup-window-width", WINDOW.width);
  win.style.setProperty("--setup-window-height", WINDOW.height);
  win.style.setProperty("--panel-left", AWAY.left);
  win.style.setProperty("--panel-margin", AWAY.top);
  const corner = nested(role("surface"), "s8");
  win.style.setProperty("--corner", corner.css);
  const glass = glassPanel(blurred);
  win.append(glass);
  windowRing(focusRing(win, "surface", "accent", false), "stroke-thin", corner.css);
  win.dataset.pinned = "lit";
  return { win, body: glass.querySelector<HTMLElement>(".k-glass-body")! };
}

export function setupRoom(options: SetupOptions): HTMLElement {
  const views: Record<CardId, string> = { ...firstViews(), ...options.views };
  const skipped = new Set(options.skipped ?? []);
  const opened = new Set<CardId>(options.hood && options.place.at === "card" ? [options.place.card] : []);
  const width = options.width ?? SCREEN.width;
  const height = options.height ?? SCREEN.height;
  const element = document.createElement("span");
  element.className = "kobo-room setup-room";
  // A whole screen: a motion player draws it as it is laid out.
  element.dataset.whole = "";
  element.tabIndex = -1;
  element.style.setProperty("--kobo-width", width);
  element.style.setProperty("--kobo-height", height);
  if (!options.alone) {
    element.dataset.mode = "dark";
    paint(element, palette(settings().theme, "dark"));
  }
  let place: Place = options.place;
  const timers = new Set<number>();
  // The cards' bodies shown now, each redrawn in place as its card's work moves on; Make it yours
  // is drawn once and changes itself.
  const holders = new Map<CardId, HTMLElement>();
  const yours = yoursCard(views.yours, { hood: opened.has("yours") });
  yours.onFinished = (failed) => {
    views.yours = failed ? "failed" : "done";
    void play(element, tracksNow(failed ? FAILS : finishing(CARD_IDS.indexOf("yours"))));
  };

  const drawCard = (id: CardId) => {
    const holder = holders.get(id) ?? document.createElement("span");
    holder.className = "setup-card";
    holder.replaceChildren(id === "yours" ? yours.body() : cardBody(id, views[id], opened.has(id)));
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
    if (id === "yours") {
      const started = yours.install();
      if (started) views.yours = "installing";
      return started;
    }
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

  /** The step rail of the cards, at `id`: every card's page, or `id`'s alone in a window. */
  const rail = (id: CardId): HTMLElement => {
    const shown = options.alone ? [id] : CARD_IDS;
    const pages = shown.map((each) => {
      const page = each === "yours" ? yours.page(act(each), () => skipped.has(each)) : cardPage(each, () => views, () => drawCard(each), act(each), () => skipped.has(each));
      // Alone, Back closes the window, but for Make it yours's later steps, where it goes back a step.
      const closes = each === "yours" ? () => yours.state.step === 0 || yours.state.phase !== "choosing" : true;
      return { ...page, body: () => drawCard(each), skipped: () => skipped.add(each), ...(options.alone ? { closes } : {}) };
    });
    return stepRailPage({
      host: options.alone ? "window" : "kobo",
      brand: koboBrand(markSvg, wordmark, options.alone ? `${CARDS[id].title} · ${RELEASE.codename} ${RELEASE.version}` : `First boot · ${RELEASE.codename} ${RELEASE.version}`),
      pages,
      current: shown.indexOf(id),
      free: true,
      // Make it yours says what its keys do on the step it shows; the other cards share the rail's.
      hints: (page) => (shown[page] === "yours" ? yours.hints() : options.alone ? HINTS.slice(2) : HINTS),
      ...(options.turnedFrom && !options.alone ? { leaving: CARD_IDS.indexOf(options.turnedFrom) } : {}),
      ...(options.live ? { onExit: (forward: boolean) => go(options.alone ? { at: "desktop" } : forward ? { at: "ending" } : { at: "arrival" }) } : {}),
    });
  };

  /** A place's layer: words over the wallpaper, or the cards on glass over it out of focus. */
  const layerOf = (at: Place): HTMLElement => {
    if (at.at === "card") {
      const layer = backdrop({ width, height, picture: blurred, scrim: "glass", content: rail(at.card) });
      layer.classList.add("kobo-panel");
      return layer;
    }
    const content = at.at === "arrival" ? arrivalColumn(views, at.moment ?? "rest", options.hero) : at.at === "ending" ? endingScene(views, skipped, at.going ?? false) : document.createElement("span");
    // The ending's desktop has come close; handed over, it is the desktop as it is.
    const push = at.at === "ending" ? 1 : 0;
    const layer = backdrop({ width, height, picture: wallpaper, scrim: "band", content, push, dark: at.at === "ending" && at.going ? 1 : 0 });
    layer.classList.add("kobo-scene");
    layer.dataset.scene = at.at;
    return layer;
  };

  /** Draws `at`'s layer, with `leaving`'s as a move's first state: a scene over the cards. */
  const draw = (at: Place, leaving?: Place): HTMLElement => {
    const arriving = layerOf(at);
    const layers = leaving ? [layerOf(leaving), arriving] : [arriving];
    layers.sort((a, b) => Number(a.classList.contains("kobo-scene")) - Number(b.classList.contains("kobo-scene")));
    element.replaceChildren(...layers);
    return arriving;
  };

  /** Builds `at`'s layer beside the one shown, as a move's first state: a scene over what it covers. */
  const arrive = (at: Place): HTMLElement => {
    const arriving = layerOf(at);
    if (at.at === "card") element.prepend(arriving);
    else element.append(arriving);
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
    let kept: HTMLElement;
    if (to.at === "ending" && to.going && from.at === "ending") {
      // Reboot taken on the ending: it stays, its actions giving way to what it is doing.
      kept = element.querySelector<HTMLElement>(".kobo-scene") ?? draw(to);
      kept.querySelector(".kobo-done .kobo-status")?.replaceWith(restarting());
      kept.querySelector(".kobo-done .kobo-keys")?.replaceChildren();
    } else if (to.at === "desktop" && move) {
      // The ending lifts off the desktop it lies on: the desktop is drawn once it has gone.
      kept = element.querySelector<HTMLElement>(".kobo-scene") ?? draw(to);
    } else kept = move ? arrive(to) : draw(to);
    // The move's end is the new place drawn: only the layers it left go.
    const settle = () => {
      if (place !== to) return;
      if (to.at === "desktop" && kept.dataset.scene !== "desktop") kept = draw(to);
      for (const child of [...element.children]) if (child !== kept) child.remove();
      (element.querySelector<HTMLElement>(".setup-start, .setup-reboot, .setup-desktop") ?? element.querySelector<HTMLElement>(".k-step-room:not([data-kept]) [tabindex='0'], .k-step-room:not([data-kept]) button:not(:disabled)") ?? element).focus({ preventScroll: true });
    };
    if (!move) return settle();
    void play(element, tracksNow(move)).then(settle);
  }

  if (options.alone) {
    // A card opened alone: the desktop as it is, the window in its middle.
    const { win, body } = windowBox();
    const layer = rail(options.place.at === "card" ? options.place.card : "yours");
    layer.classList.add("setup-layer");
    body.append(layer);
    const desk = backdrop({ width, height, picture: wallpaper, scrim: "band", content: win });
    desk.classList.add("kobo-scene", "setup-desk");
    element.append(desk);
  } else draw(place, options.leaving);

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

/**
 * One of the installer's moves played in first boot's room: the same tracks, less those on what
 * first boot hasn't got (the installer's foot, with its power button and battery).
 */
function inFirstBoot(move: Move, name: string, what: string, sample: () => HTMLElement): Move {
  // The tracks kept are some of the move's own, so their curves are the move's: a cast cannot
  // widen them, and a move keeps at least its tracks on the screen's words.
  const ours = <T extends Track>(tracks: readonly T[]): NonEmpty<T> => tracks.filter((track) => !track.target.includes("kobo-foot")) as unknown as NonEmpty<T>;
  const { unflickered, reduced } = move;
  return {
    ...move,
    name,
    what,
    tracks: ours(move.tracks),
    ...(unflickered ? { unflickered: ours(unflickered) } : {}),
    reduced: reduced === "same" ? "same" : ours(reduced),
    sample,
  } as Move;
}

/** First boot opens: as Kōbō's welcome does, the street lighting up, the sign striking, the words rising. */
export const OPENS: Move = inFirstBoot(OPENING, "Setup opens", "As Kōbō's welcome: the desktop lights up and settles; the sign strikes; the words rise in.", () => setupRoom({ place: { at: "arrival" } }));

/** Opened on a card from the Command menu: the window's page arrives as the template's pages do. */
const OPENS_CARD: Move = {
  name: "Opened on a card",
  what: "The card's page slides in and fades in over `long`.",
  tracks: [
    { part: "slide", on: "the page", target: ".k-step-room:not([data-kept])", property: "--step-slide", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "fade", on: "the page", target: ".k-step-room:not([data-kept])", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [fading(".k-step-room:not([data-kept])", "the page", "the page")],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "drivers" }, alone: true }),
};

/** Let's go: the arrival lifts off, the desktop goes out of focus behind glass, the cards rise in. */
export const CARDS_IN: Move = inFirstBoot(
  ENTERING,
  "Let's go: the cards",
  "As the installer's Let's go: the words rise off; the desktop goes out of focus behind glass; the cards rise in.",
  () => setupRoom({ place: { at: "card", card: "yours" }, leaving: { at: "arrival" } }),
);

/** Back from the first card: the cards sink off and the arrival comes back. */
export const BACK: Move = inFirstBoot(LEAVING, "Back to the arrival", "Esc or Back on the first card: the cards sink off, the desktop comes into focus, the words come back.", () =>
  setupRoom({ place: { at: "arrival" }, leaving: { at: "card", card: "yours" } }),
);

/** Continue or Skip onto a card: the card left slides off, the next slides in, its band after it. */
export function onward(sample: () => HTMLElement): Move {
  return {
    name: "Continue: the next card",
    what: "As the installer's pages turn: the card left slides off; the next slides in, its band after it.",
    tracks: turning(true),
    reduced: TURNING_REDUCED,
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
const SOME: Partial<Views> = { yours: "done", drivers: "done", fingerprint: "done" };

/** The ending shown on its own: the installer's done screen's arrival, with first boot's headline. */
const ARRIVES: Move = {
  name: "That's it",
  what: "As the installer's done screen: the desktop goes dark as クバ writes itself in; it strikes, the desktop lights up and comes close; the headline rises.",
  tracks: arrival("neon.strike", HEADLINE) as unknown as NonEmpty<Track>,
  unflickered: arrival("standard", HEADLINE) as unknown as NonEmpty<Track<Easing>>,
  reduced: [fading(".kobo-done", "the ending", ENDING)],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "ending" }, views: SOME, skipped: ["yubikey", "phone"] }),
};

/** The cards leaving for the ending: they rise off the glass, and the ending's band comes over the desktop. */
const LEAVE_CARDS = (): Track[] => [
  { part: "the cards", on: "the cards", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
  { part: "their rise", on: "the cards", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
  { part: "the ending's band", on: ENDING, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
];

/** Past the last card: the cards lift off, and the ending's arrival plays as the installer's does. */
export const ENDS: Move = {
  name: "Past the last card: That's it",
  what: "The cards rise off over `medium`; then the done screen's arrival: the desktop dark, クバ written and struck, the desktop lit and close.",
  tracks: [...LEAVE_CARDS(), ...after("medium", arrival("neon.strike", HEADLINE))] as unknown as NonEmpty<Track>,
  unflickered: [...LEAVE_CARDS(), ...after("medium", arrival("standard", HEADLINE))] as unknown as NonEmpty<Track<Easing>>,
  // The cards go, not the desktop under them, which stays until the ending lies over it: no frame
  // between them is empty.
  reduced: [fading(".kobo-panel > .k-backdrop-thing", "the cards", "the cards", 1, 0), { ...fading(".kobo-scene", "the ending", ENDING), cue: "short" }],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "ending" }, views: SOME, skipped: ["yubikey", "phone"], leaving: { at: "card", card: "phone" } }),
};

/** Go to the desktop: the words lift off and the desktop settles back to itself, yours. */
export const DESKTOP: Move = {
  name: "Go to the desktop",
  what: "The ending's words rise off over `medium`; its band fades; the desktop settles back from close to its own size.",
  tracks: [
    { part: "the words", on: ENDING, target: ".kobo-scene[data-scene='ending'] > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "their rise", on: ENDING, target: ".kobo-scene[data-scene='ending'] > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: ENDING, target: ".kobo-scene[data-scene='ending'] > .k-backdrop-layers > .k-backdrop-scrim", property: "opacity", from: 1, to: 0, duration: "long", curve: "emphasized_accel" },
    { part: "the desktop settling", on: "the desktop", target: ".kobo-scene[data-scene='ending'] .k-backdrop-picture", property: "scale", from: { scale: "push" }, to: 1, duration: "scene", curve: "emphasized_decel" },
  ],
  reduced: [fading(".kobo-scene[data-scene='ending'] > .k-backdrop-thing", "the words", ENDING, 1, 0)],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "ending" }, views: SOME, skipped: ["yubikey", "phone"] }),
};

/** Reboot now: the installer's lights going out, over the desktop. */
export const REBOOT: Move = inFirstBoot(GOING, "Reboot: the lights go out", "As the installer's: Restarting…; クバ switches off; the desktop and the words go dark over `slow`.", () =>
  setupRoom({ place: { at: "ending", going: true }, views: { yours: "done", drivers: "queued", fingerprint: "done" }, skipped: ["yubikey", "phone"] }),
);
