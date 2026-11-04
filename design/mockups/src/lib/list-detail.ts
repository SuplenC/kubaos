// The ListDetail template: a list beside the detail of its selected row
// (`docs/design/components/list-detail.md`), as its two builds draw it today: the Command menu
// (`config/quickshell/modules/menu/Menu.qml`), a panel of the shell sized by its screen, and Kōbō's
// first-boot setup (`crates/kubaos-kobo/src/setup/view.rs`), an app's window. Its look is
// `list-detail.css`.

import "./list-detail.css";
import { button } from "./button.ts";
import { hangs } from "./button-bar.ts";
import { card } from "./card.ts";
import { chip, type ChipKind } from "./chip.ts";
import { type KeyHint, keycap, keyHints, keys } from "./keycap.ts";
import { emptyState } from "./empty-state.ts";
import { field } from "./field.ts";
import { icon } from "./icon.ts";
import { divider, fill, filling, fillingHeight, flow, gap, header, page, row, stack } from "./layout.ts";
import { type ListRowSpec, scrolled, scrollingList } from "./list.ts";
import { probe, workedOut } from "./probe.ts";
import { scroll } from "./scroll.ts";
import { step, type StepOptions } from "./step.ts";
import { clamped, text, wrapped } from "./text.ts";
import { cubic, length, raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** A duration token in milliseconds at the speed picked, capped at `short` with reduced motion. */
function timed(token: string): number {
  const ms = (name: string) => {
    const value = raw(`duration-${name}`);
    return Number.parseFloat(value) * (value.endsWith("ms") ? 1 : 1000);
  };
  const { reduced, speed } = settings();
  return (reduced ? Math.min(ms(token), ms("short")) : ms(token)) / speed;
}

function curve(token: string): string {
  return `cubic-bezier(${cubic(`easing-${token}`).join(", ")})`;
}

/** A logical length as CSS: whole device pixels, as the panel's measures are. */
function px(logical: number): string {
  return `calc(${logical} * var(--px))`;
}

/** Floored onto the 4 px grid, as the Command menu puts every measure of its panel. */
function onGrid(value: number): number {
  const grid = length("space-s4");
  return Math.floor(value / grid) * grid;
}

// ---------------------------------------------------------------- the Command menu --

/** One Result: its row, and what the pane says of it. */
export interface Result {
  readonly row: ListRowSpec;
  /** What kind of Item it is, and the Alias the query named it by: Application · ff. */
  readonly kind: string;
  /** Its Action panel: the first runs with Enter, the rest by their keys. */
  readonly actions: readonly { readonly title: string; readonly keys: string }[];
  /** The keys the row takes, at the pane's foot. */
  readonly hints: readonly KeyHint[];
}

/** The Command menu's measures on a screen `width` × `height` logical pixels big. */
export interface MenuMeasures {
  readonly width: number;
  readonly height: number;
  readonly rows: number;
  readonly body: number;
  /** The pane's width, or 0 where the panel is too narrow for one. */
  readonly pane: number;
  /** The panel's top, two fifths of the room it leaves free. */
  readonly top: number;
}

/**
 * The Command menu's panel on a screen: half the screen's width, from one column to two; rows for
 * about two thirds of its height, eight to fourteen, each `size.row_two_line` and `space.s4`; the
 * pane where the panel is as wide as a dialog beside a rail, two fifths of the room and a rail at
 * the least. `fit` lets it take fewer than eight rows where eight would run past the screen.
 */
export function menuMeasures(screenWidth: number, screenHeight: number, fit = false): MenuMeasures {
  const space = length("space-s12");
  const column = length("size-column");
  const width = Math.max(column, Math.min(column * 2, onGrid(screenWidth / 2)));
  const pane = width >= length("size-dialog") + length("size-rail") ? Math.max(length("size-rail"), onGrid((width - space * 3) * 0.4)) : 0;
  const pitch = length("size-row_two_line") + length("space-s4");
  const chrome = space * 5 + length("size-control_md") + length("size-control_sm");
  const wanted = Math.floor((screenHeight * 0.64 - chrome) / pitch);
  const least = fit ? Math.max(1, Math.min(8, Math.floor((screenHeight - chrome + length("space-s4")) / pitch))) : 8;
  const rows = Math.max(least, Math.min(14, wanted));
  const body = rows * pitch - length("space-s4");
  const height = chrome + body;
  return { width, height, rows, body, pane, top: onGrid((screenHeight - height) * 0.4) };
}

/** What the pane shows of a Result: its symbol, words and kind, its Actions, and its keys. */
function paneContent(result: Result): HTMLElement {
  const glyph = result.row.icon ?? "search";
  const top = stack(
    0,
    "start",
    icon(glyph, "var(--icon-s48)", "var(--color-primary)"),
    gap("s16"),
    clamped(result.row.title, "heading", "on_surface", 3),
    gap("s4"),
    ...(result.row.subtitle ? [clamped(result.row.subtitle, "body", "on_surface_variant", 4)] : []),
    gap("s8"),
    text(result.kind, "label", "on_surface_variant"),
  );
  const actions = result.actions.map((action, index) =>
    row("s8", "centre", filling(clamped(action.title, "body", index === 0 ? "on_surface" : "on_surface_variant")), row("s4", "centre", ...keys(action.keys).map(keycap))),
  );
  const list = stack("s8", "start", text("Actions", "label", "on_surface_variant"), ...actions.map(filling));
  const content = stack(0, "start", filling(top), gap("s24"), filling(list), fillingHeight(document.createElement("span")), gap("s16"), keyHints(result.hints));
  content.classList.add("k-menu-pane-content");
  return content;
}

/**
 * The pane: the selected Result on a card. A move of the selection crossfades it over `short` on
 * `standard`; nothing selected, its empty state.
 */
function pane(width: number): { box: HTMLElement; show: (result: Result | null, fade: boolean) => void } {
  const room = document.createElement("span");
  room.className = "k-menu-pane-room";
  const box = card({ body: [room] });
  box.classList.add("k-menu-pane");
  workedOut(box, "size-rail").style.width = px(width);
  let shown: HTMLElement | null = null;
  const show = (result: Result | null, fade: boolean) => {
    const next = result ? paneContent(result) : emptyPane();
    room.append(next);
    const old = shown;
    shown = next;
    if (!old) return;
    if (!fade) {
      old.remove();
      return;
    }
    const timing = { duration: timed("short"), easing: curve("standard") };
    next.animate([{ opacity: 0 }, { opacity: 1 }], timing);
    old.animate([{ opacity: 1 }, { opacity: 0 }], timing).finished.then(() => old.remove(), () => old.remove());
  };
  return { box, show };
}

function emptyPane(): HTMLElement {
  const words = stack("s4", "centre", text("Nothing selected", "subheading"), wrapped("Pick a row and it shows up here.", "body", "on_surface_variant"));
  const content = stack("s16", "centre", icon("manage_search", "var(--icon-s32)", "var(--color-on_surface_variant)"), words);
  content.classList.add("k-menu-pane-content");
  content.dataset.empty = "";
  return content;
}

/** The list's room with no Results: its empty state, centred down it. */
function noResults(body: number, query: string): HTMLElement {
  const typed = query !== "";
  const room = document.createElement("span");
  room.className = "k-menu-empty";
  room.style.height = px(body);
  room.append(
    emptyState({
      icon: "search_off",
      title: typed ? "No results" : "Nothing here yet",
      body: typed ? "Nothing matches that. Fewer letters, or check the spelling." : "This list is empty. Come back when it isn't.",
    }),
  );
  return room;
}

export interface MenuOptions {
  readonly results: readonly Result[];
  /** The screen it opens on, in logical pixels; the toolbar's window, where none is given. */
  readonly screen?: { readonly width: number; readonly height: number };
  /** Rows shown, in place of what the screen gives: a sample of a few, on a panel without the pane. */
  readonly rows?: number;
  /** What is typed in the field. */
  readonly query?: string;
  /** The row selected, by its index. */
  readonly current?: number;
}

/**
 * The Command menu's panel: the frame's glass in the soft hairline at the surface radius,
 * `space.s12` inside and between its parts: the search field, the body (the Results beside the
 * pane where it fits), and under a hairline across the panel the footer, a small control tall,
 * holding the keys. The field keeps the keyboard and hands ↑ ↓, Page Up and Down, Ctrl+N/P and
 * Enter to the list.
 */
export function commandMenu(options: MenuOptions, measures: MenuMeasures): HTMLElement {
  // A sample of a few rows only where the pane doesn't show: beside it the body keeps the screen's
  // rows, eight at the least, which the pane's symbol, words, Actions and keys are drawn to fit.
  const few = measures.pane > 0 ? undefined : options.rows;
  const pitch = length("size-row_two_line") + length("space-s4");
  const body = few ? few * pitch - length("space-s4") : measures.body;
  const search = field({ placeholder: "Search apps, projects and windows. Sums work too", icon: "search", ...(options.query ? { value: options.query } : {}), pinned: "lit" });
  const list = scrollingList(
    options.results.map((each) => each.row),
    { height: px(body), slides: true, current: options.current ?? 0, fieldKeys: true, vimKeys: false },
  );
  const parts: Element[] = [filling(options.results.length > 0 ? list : noResults(body, options.query ?? ""))];
  const shownPane = measures.pane > 0 ? pane(measures.pane) : null;
  if (shownPane) parts.push(shownPane.box);
  const middle = row("s12", "start", ...parts);
  middle.classList.add("k-menu-body");
  const hints: readonly KeyHint[] = shownPane
    ? [["↑↓", "Select"], ["⌃K", "Actions"], ["Esc", "Close"]]
    : (options.results[options.current ?? 0]?.hints ?? [["Esc", "Close"]]);
  const rule = document.createElement("span");
  rule.className = "k-menu-rule";
  rule.append(divider());
  const foot = row(0, "centre", keyHints(hints));
  foot.classList.add("k-menu-footer");
  const panel = document.createElement("span");
  panel.className = "k-menu";
  panel.tabIndex = 0;
  workedOut(panel, "size-column").style.width = px(measures.width);
  panel.append(stack("s12", "start", filling(search), filling(middle), rule, filling(foot)));
  if (shownPane) {
    let at = options.current ?? 0;
    shownPane.show(options.results[at] ?? null, false);
    new MutationObserver(() => {
      const index = [...list.querySelectorAll(".k-list-row")].findIndex((each) => each.hasAttribute("data-current"));
      if (index < 0 || index === at) return;
      at = index;
      shownPane.show(options.results[at] ?? null, true);
    }).observe(list, { attributes: true, attributeFilter: ["data-current"], subtree: true });
  }
  panel.addEventListener("keydown", (event) => {
    if (event.target !== panel) return;
    event.preventDefault();
    list.dispatchEvent(new KeyboardEvent("keydown", { key: event.key, ctrlKey: event.ctrlKey, repeat: event.repeat }));
  });
  return probe(panel, { role: "component", name: "ListDetail", variant: "menu", fixed: { width: true, height: false, holds: true } });
}

/**
 * The Command menu on a screen of the toolbar's window size: its panel's measures follow the screen
 * as it resizes, centred across it and two fifths of the way down the room it leaves.
 */
export function menuOnScreen(options: MenuOptions): HTMLElement {
  const screen = document.createElement("span");
  screen.className = "k-menu-screen";
  let drawn = "";
  new ResizeObserver(() => {
    const scale = Number.parseFloat(getComputedStyle(screen).getPropertyValue("--scale")) || settings().scale;
    const width = screen.clientWidth / scale;
    const height = screen.clientHeight / scale;
    const measures = menuMeasures(width, height, true);
    const key = JSON.stringify(measures);
    if (key === drawn) return;
    drawn = key;
    const panel = commandMenu(options, measures);
    panel.style.left = `${Math.round(((width - measures.width) / 2) * scale)}px`;
    panel.style.top = `${Math.round(measures.top * scale)}px`;
    screen.replaceChildren(panel);
  }).observe(screen);
  return screen;
}

// ---------------------------------------------------------------- Kōbō's setup --

/** One card of first boot: its step in the list and its page in the detail. */
export interface SetupCard {
  readonly step: StepOptions;
  /** Its state as the detail's chip says it. */
  readonly chip: { readonly kind: ChipKind; readonly label: string; readonly icon?: string };
  /** What the detail says of it, a paragraph each, or a part of its own (a banner, a form). */
  readonly explain: readonly (string | (() => HTMLElement))[];
  /** Its action, Filled, on the right of its footer; and a Tonal one before it. */
  readonly primary?: string;
  readonly secondary?: string;
  /** The way past it: Skip while it is open, Next once done. */
  readonly skip: string;
}

export interface SetupOptions {
  readonly cards: readonly SetupCard[];
  /** The card selected, by its index. */
  readonly selected?: number;
  /** The machine is online. */
  readonly online?: boolean;
  /** The detail rising in, held this far, as a still sample of it. */
  readonly rising?: number;
}

/** The card's page: its symbol, title and chip, its words in a scroll, and its actions. */
function setupDetail(spec: SetupCard): HTMLElement {
  const title = row("s12", "centre", text(spec.step.title, "title"), chip({ label: spec.chip.label, kind: spec.chip.kind, ...(spec.chip.icon ? { icon: spec.chip.icon } : {}) }));
  const head = row("s16", "centre", icon(spec.step.icon ?? "info", "var(--icon-s48)", "var(--color-primary)"), filling(stack("s4", "start", title, wrapped(spec.step.subtitle ?? "", "body", "on_surface_variant"))));
  const parts = spec.explain.map((each) => (typeof each === "string" ? wrapped(each, "body", "on_surface_variant") : each()));
  const words = stack("s16", "start", ...parts.map(filling));
  words.classList.add("k-setup-words");
  const { box } = scroll(words, { height: "100%", above: "s24", below: "s24" });
  const actions: Element[] = [button({ label: spec.skip, kind: "Text", icon: "arrow_forward", trailing: true }), fill()];
  if (spec.secondary) actions.push(button({ label: spec.secondary, kind: "Tonal" }));
  if (spec.primary) actions.push(button({ label: spec.primary, kind: "Filled" }));
  const content = stack("s24", "start", filling(head), fillingHeight(filling(box)), filling(hangs(row("s12", "centre", ...actions))));
  return page(fillingHeight(content), { clear: true });
}

/**
 * The detail arriving for a newly picked card: it rises `space.s16` into place and fades in over
 * `long` on `emphasized_decel`, its offset on whole device pixels; with reduced motion it fades.
 */
export function rise(room: HTMLElement): void {
  const frames: Keyframe[] = settings().reduced ? [{ opacity: 0 }, { opacity: 1 }] : [{ opacity: 0, "--rise": 1 }, { opacity: 1, "--rise": 0 }];
  room.animate(frames, { duration: timed("long"), easing: curve("emphasized_decel") });
}

/**
 * The side's width: a third of the window, so the card's page keeps the room, never wider than a
 * column of text, on a whole device pixel. It follows the window's own width, which CSS can't
 * compare with a length, so script sets it.
 */
function sideWidth(side: HTMLElement, window: HTMLElement): void {
  workedOut(side, "size-column");
  new ResizeObserver(() => {
    side.style.width = `${Math.round(Math.min(window.clientWidth / 3, length("size-column") * (Number.parseFloat(getComputedStyle(window).getPropertyValue("--scale")) || settings().scale)))}px`;
  }).observe(window);
}

/**
 * Kōbō's setup window: the side, a third of the window on the well's fill, holding the header, the
 * cards in a scrolling list, whether the machine is online, the way out and the keys; beside it the
 * selected card's page. ↑ ↓ (j k) in the list pick a card, and the page rises in.
 */
export function setupWindow(options: SetupOptions): HTMLElement {
  let selected = options.selected ?? 0;
  const window = document.createElement("span");
  window.className = "k-setup";
  const steps = options.cards.map((each, index) => step({ ...each.step, selected: index === selected, pressable: true }));
  for (const each of steps) each.tabIndex = -1;
  const rows = stack("s4", "start", ...steps);
  rows.classList.add("k-list-rows");
  const { box: list } = scrolled("ScrollingList", rows, { height: "100%" });
  list.tabIndex = 0;
  const online = options.online ?? true;
  const status = online
    ? chip({ label: "Online", kind: "Neutral", icon: "wifi" })
    : chip({ label: "Offline: drivers, fprintd and Bitwarden need the internet", kind: "Neutral", icon: "wifi_off" });
  const hints: readonly KeyHint[] = [
    ["↑ ↓", "pick a card"],
    ["Tab", "next control"],
    ["Enter", "open, press"],
    ["Esc", "back to the cards"],
  ];
  const sideContent = stack(
    0,
    "start",
    filling(header({ accent: "初期設定", title: "First boot", subtitle: "The things the install couldn't do without you or the hardware. Each one checks itself, so done stays done." })),
    gap("s20"),
    fillingHeight(filling(list)),
    gap("s12"),
    gap("s12"),
    status,
    gap("s16"),
    button({ label: "Done for now", kind: "Tonal", icon: "check_circle" }),
    gap("s16"),
    filling(flow("s16", ...hints.map((hint) => keyHints([hint])))),
  );
  const side = document.createElement("span");
  side.className = "k-setup-side";
  side.append(page(fillingHeight(sideContent), { clear: true }));
  const detail = document.createElement("span");
  detail.className = "k-setup-detail";
  const show = (animate: boolean) => {
    const spec = options.cards[selected];
    if (!spec) return;
    const room = document.createElement("span");
    room.className = "k-setup-room";
    room.append(setupDetail(spec));
    detail.replaceChildren(room);
    if (animate) rise(room);
    if (options.rising !== undefined) {
      room.style.opacity = String(options.rising);
      room.style.setProperty("--rise", String(1 - options.rising));
    }
  };
  const pick = (index: number) => {
    if (index < 0 || index >= steps.length || index === selected) return;
    steps[selected]?.toggleAttribute("data-selected", false);
    selected = index;
    steps[selected]?.toggleAttribute("data-selected", true);
    show(true);
  };
  steps.forEach((each, index) => each.addEventListener("click", () => pick(index)));
  list.addEventListener("keydown", (event) => {
    const by = event.key === "ArrowDown" || event.key === "j" ? 1 : event.key === "ArrowUp" || event.key === "k" ? -1 : 0;
    if (by === 0) return;
    event.preventDefault();
    pick(selected + by);
  });
  window.append(side, detail);
  sideWidth(side, window);
  show(false);
  return probe(window, { role: "component", name: "ListDetail", variant: "setup", fixed: { width: false, height: false, holds: true } });
}
