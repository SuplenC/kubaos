// First boot's Kotodama card, in four steps of its own on a Stepper under its title, as Make it
// yours's are. What: the one key, what to type and Enter, beside the Command menu playing three asks
// by itself. Why: one question asked of Start and Kotodama at the same moment, raced, each clock
// running until its answer lands. Yours: the keys that open it, what it finds, the web's
// place, each applied to the menu as it is made, its keys opening the menu from there. Try it: the
// keys picked, large, and two lines saying the menu is already set; its keys, or a click on them,
// open the menu over the whole screen as it opens on the desktop, the real Command menu's parts
// (`lib/list-detail.ts`).
//
// Its parts are the library's. What is its own is content: the examples' rows, the race's grid,
// the other menu and its ad (a parody, drawn from library parts), and Yours's two columns.

import { button } from "../../lib/button.ts";
import { choiceCard, choiceGroup } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { field, setValue } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { type KeyHint, keycap, keyHints, keyPrompt, keys, pressKeys } from "../../lib/keycap.ts";
import { divider, fill, filling, fillingHeight, gap, grid, row, splitColumns, stack } from "../../lib/layout.ts";
import { type ListRowSpec, listWith } from "../../lib/list.ts";
import { commandMenu, type MenuMeasures, menuMeasures } from "../../lib/list-detail.ts";
import { scroll } from "../../lib/scroll.ts";
import { spinner } from "../../lib/spinner.ts";
import { RECHECK, type RailPage, type RailStatus } from "../../lib/step-rail.ts";
import { setStep, stepper } from "../../lib/stepper.ts";
import { orderList } from "../../lib/order-list.ts";
import { tabs } from "../../lib/tabs.ts";
import { text, wrapped } from "../../lib/text.ts";
import { toastRoom } from "../../lib/toast.ts";
import { floatingTooltip, term, termed, tooltip } from "../../lib/tooltip.ts";
import { length } from "../../lib/tokens.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { CHROME, settings } from "../../site/settings.ts";
import { body } from "./kobo-parts.ts";
import { CARDS } from "./setup-cards.ts";
import { AD, defaultSettings, type Found, LANDS, type MenuSettings, OPENS_WITH, OTHERS, QUESTION, search } from "./setup-kotodama-data.ts";

/** The card's steps, by their index. */
export const STEPS = ["What", "Why", "Yours", "Try it"] as const;
export type StepIndex = 0 | 1 | 2 | 3;

/** Where the card is: its step, its settings, and the menu open or shut over the screen. */
export interface Kotodama {
  step: StepIndex;
  /** A step turn's first state: the step left, drawn leaving over this one. */
  readonly from?: StepIndex;
  readonly settings: MenuSettings;
  open: boolean;
  /** The race held this many milliseconds after the key: a sample of it mid-way. */
  readonly raceAt?: number;
  /** The ad's asked word with its answer shown, as under the pointer: a sample of the tip. */
  readonly tip?: boolean;
  /** The ad's button just pressed, its reply shown: a sample of it. */
  readonly reply?: boolean;
  passed: boolean;
}

/** Where a view of the card starts. A view `name<N` is `name` reached from step N. */
export function kotodamaAt(view: string): Kotodama {
  const [name, from] = view.split("<") as [string, string | undefined];
  const base = (step: StepIndex, more: Partial<Kotodama> = {}): Kotodama => ({
    step,
    settings: defaultSettings(),
    open: false,
    passed: false,
    ...(from === undefined ? {} : { from: Number(from) as StepIndex }),
    ...more,
  });
  switch (name) {
    case "why":
      return base(1);
    case "why-mid":
      return base(1, { raceAt: 3300 });
    case "why-tip":
      return base(1, { raceAt: 5800, tip: true });
    case "why-reply":
      return base(1, { raceAt: 5800, reply: true });
    case "yours":
      return base(2);
    case "yours-changed": {
      const state = base(2);
      state.settings.key = "alt-space";
      state.settings.web = false;
      return state;
    }
    case "try":
      return base(3);
    case "try-alt": {
      const state = base(3);
      state.settings.key = "alt-space";
      return state;
    }
    case "try-open":
      return base(3, { open: true });
    case "done":
      return base(3, { passed: true });
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
 * room's whole width and height, the Results as many rows as that holds, the pane where the panel
 * is as wide as a dialog beside a rail, as the menu has on a screen (`menuMeasures`). The body
 * takes all the height the rows leave, so the panel ends where its room does.
 */
function measuresIn(width: number, height: number, scale: number): MenuMeasures {
  const space = length("space-s12");
  const pitch = length("size-row_two_line") + length("space-s4");
  const chrome = space * 5 + length("size-control_md") + length("size-control_sm");
  const grid = length("space-s4");
  const panel = Math.floor(width / grid) * grid;
  const rows = Math.max(1, Math.floor((height - chrome + length("space-s4")) / pitch));
  const pane = panel >= length("size-dialog") + length("size-rail") ? Math.max(length("size-rail"), Math.floor(((panel - space * 3) * 0.4) / grid) * grid) : 0;
  // The body takes the height left, down to a whole device pixel, so the panel's foot is on one.
  const body = Math.max(rows * pitch - length("space-s4"), Math.floor((height - chrome) * scale) / scale);
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
function standIn(prefs: MenuSettings, options: { readonly live: boolean; readonly query?: string; readonly ran?: (found: Found, typed: string) => void; readonly onEscape?: () => void; readonly hints?: readonly KeyHint[]; readonly took?: string }): StandIn {
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
  });
  // The field is drawn lit with the keyboard in it, as the open menu's always is, so it edits
  // nothing itself: what is typed into it here goes to the search.
  if (options.live)
    head.addEventListener(
      "keydown",
      (event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          event.stopPropagation();
          if (typed) set("");
          else options.onEscape?.();
        } else if (event.key === "Backspace") {
          event.preventDefault();
          set([...typed].slice(0, -1).join(""));
        } else if ([...event.key].length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
          event.preventDefault();
          set(typed + event.key);
        }
      },
      true,
    );
  /** Focus asked for before the panel was first drawn, given once it is. */
  let focusWanted = false;
  const draw = () => {
    if (!measures) return;
    // Each draw moves the field into a new panel, which takes the focus from it: it is given back.
    const focused = document.activeElement === head;
    found = search(typed, prefs, measures.rows);
    const took = text(typed ? (options.took ?? "2 ms") : "", "mono", "cold");
    const foot = row("s8", "centre", keyHints(options.hints ?? [["↑↓", "Select"], ["Enter", "Run"], ["Esc", "Clear"]]), fill(), took);
    const panel = commandMenu({ results: found.map((each) => each.result), query: typed, head, current: Math.min(selected, Math.max(0, found.length - 1)), onTake: run, footer: filling(foot) }, measures);
    panel.querySelector(".k-list")?.addEventListener("click", () => queueMicrotask(() => head.focus({ preventScroll: true })));
    box.replaceChildren(panel, room);
    if ((focused || focusWanted) && head.isConnected) {
      focusWanted = false;
      head.focus({ preventScroll: true });
    }
  };
  new ResizeObserver(() => {
    const scale = scaleOf(box);
    const next = measuresIn(box.clientWidth / scale, box.clientHeight / scale, scale);
    if (measures && next.width === measures.width && next.body === measures.body) return;
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
    focus: () => {
      if (head.isConnected) head.focus({ preventScroll: true });
      else focusWanted = true;
    },
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

interface Answer {
  readonly title: string;
  /** A second thing it says, after the title in the quieter colour. */
  readonly aside?: string;
  readonly icon: string;
  readonly ad?: boolean;
  /** The row the other menu calls its best match, filled. */
  readonly best?: boolean;
}

/**
 * A row of the other menu, `size.control_sm` tall at the least: its
 * symbol, its words whole (they wrap rather than end in an ellipsis), an aside, an Ad chip.
 */
function parodyRow(each: Answer): HTMLElement {
  const line = row(
    "s8",
    "centre",
    icon(each.icon, "var(--type-body-icon)", "var(--color-on_surface_variant)"),
    filling(wrapped(each.title, "body")),
    ...(each.aside ? [text(each.aside, "small", "on_surface_variant")] : []),
    ...(each.ad ? [chip({ label: "Ad", kind: "Neutral" })] : []),
  );
  line.classList.add("setup-kotodama-prow");
  line.toggleAttribute("data-best", each.best ?? false);
  return line;
}

/** A group of the other menu's answers: its label, its rows, and the beat it is, if it can go. */
interface Group {
  readonly label: string;
  readonly rows: readonly Answer[];
  /** A beat a short lane leaves out: first the news, then the game. */
  readonly extra?: "stories" | "recommended";
}

/** What the other menu holds besides its groups: an ad over the first ones, one in the flow. */
interface OtherParts {
  /** Laid over the first `covers` groups, which take its height at the least. */
  readonly over?: HTMLElement;
  readonly covers?: number;
}

/**
 * The other menu, a parody drawn from library parts, drained of colour: its field with a search's bar
 * crawling under it, its answers in one flat list, a label over each group, landing group by
 * group, and the line at its foot that never ends while it searches.
 */
function otherMenu(q: string, groups: readonly Group[], foot: string, more: OtherParts = {}): { box: HTMLElement; groups: HTMLElement[]; answers: HTMLElement; settle: () => void } {
  const words = field({ value: q, icon: "search", pointer: false });
  const bar = document.createElement("span");
  bar.className = "setup-kotodama-crawl";
  const parts = groups.map((group) => {
    const head = text(group.label, "caption", "on_surface_variant");
    head.classList.add("setup-kotodama-plabel");
    const part = stack(0, "start", filling(head), ...group.rows.map((each) => filling(parodyRow(each))));
    if (group.extra) part.dataset.extra = group.extra;
    return part;
  });
  const covers = more.over ? (more.covers ?? 0) : 0;
  const flowing: HTMLElement[] = [];
  if (covers > 0) {
    // The ad and the groups it covers share one cell: the rows after them start below both.
    const under = stack("s4", "start", ...parts.slice(0, covers).map(filling));
    const cell = document.createElement("span");
    cell.className = "setup-kotodama-under";
    cell.append(under, more.over!);
    flowing.push(cell);
  }
  flowing.push(...parts.slice(covers));
  const answers = stack("s4", "start", ...flowing.map(filling));
  answers.classList.add("setup-kotodama-answers");
  const waiting = spinner("s16", "on_surface_variant");
  const status = row("s8", "centre", waiting, filling(wrapped(foot, "small", "on_surface_variant")));
  const inner = stack("s8", "start", filling(stack("s4", "start", filling(words), filling(bar))), fillingHeight(filling(answers)), divider(), filling(status));
  inner.classList.add("setup-kotodama-sad");
  const box = document.createElement("span");
  box.className = "setup-kotodama-other";
  box.append(inner);
  /** The search over: its bar goes and its spinner gives way to a still symbol, nothing moving. */
  const settle = () => {
    bar.dataset.done = "";
    if (waiting.parentElement) waiting.replaceWith(icon("hourglass_top", "var(--icon-s16)", "var(--color-on_surface_variant)"));
  };
  // The fullest the lane holds whole: every beat; then without the news; then without the game
  // and the groups' labels too. The ad is whole at every size.
  const fits = ["tall", "medium", "short"] as const;
  new ResizeObserver(() => {
    for (const fit of fits) {
      box.dataset.fit = fit;
      if (answers.scrollHeight <= answers.clientHeight + 1) return;
    }
  }).observe(answers);
  return { box, groups: parts, answers, settle };
}

/** Start's answers for `q`, by group, in the order they land: what you wanted comes last. */
function startGroups(q: string): Group[] {
  return [
    { label: "Best match", rows: [{ title: q, aside: "See web results", icon: "world", best: true }] },
    { label: "Search the web", rows: [{ title: `${q} not working`, icon: "search" }] },
    { label: "Top stories", rows: [{ title: `11 ${q} secrets your router doesn't want you to know`, icon: "notes" }], extra: "stories" },
    { label: "Recommended", rows: [{ title: "A game you never installed", aside: "Installing anyway", icon: "gamepad", ad: true }], extra: "recommended" },
    { label: "Weather", rows: [{ title: "14° and cloudy", aside: "In a town you left in 2019", icon: "thermostat" }] },
    { label: "Settings (1)", rows: [{ title: "Show 1 more", aside: "What you wanted is in here", icon: "expand_more" }] },
  ];
}

/**
 * The ad Start raises over its answers, loud in the warning and error
 * tones, an invented cleaner for a PC that is fine.
 */
function fakeAd(tipShown = false, replyShown = false): HTMLElement {
  const tag = document.createElement("span");
  tag.className = "setup-kotodama-ad-tag";
  tag.append(text(AD.tag, "label", "burn_warning"));
  const close = icon("close", "var(--type-small-icon)", "var(--color-on_burn_warning)");
  const top = row("s8", "centre", tag, filling(wrapped(AD.sponsor, "caption", "on_burn_warning")), close);
  const tile = document.createElement("span");
  tile.className = "setup-kotodama-ad-tile";
  tile.append(icon("rocket_launch", "var(--type-heading-icon)", "var(--color-on_error)"));
  const product = row("s12", "start", tile, filling(stack(0, "start", filling(wrapped(AD.product, "subheading", "on_burn_warning")), filling(wrapped(AD.maker, "caption", "on_burn_warning")))));
  const stars = row("s4", "centre", ...Array.from({ length: 5 }, () => icon("star_filled", "var(--type-small-icon)", "var(--color-burn_error)")), filling(wrapped(AD.rating, "caption", "on_burn_warning")));
  const go = button({ label: AD.action, kind: "Danger", size: "Sm", icon: "arrow_forward", trailing: true });
  // The fine print is one paragraph that wraps as any does; its one term is explained on a card.
  const asked = term(AD.fine[1]);
  const words = wrapped(AD.fine.join(" "), "caption", "on_burn_warning");
  words.replaceChildren(`${AD.fine[0]} `, asked, ` ${AD.fine[2]}`);
  const fine = termed(words, asked, tooltip(AD.fineAnswer, ""), { shown: tipShown });
  // The button answers back, and does nothing else.
  const answering = floatingTooltip(go, tooltip(AD.reply, ""), { side: "above", shown: replyShown, shows: "pressed" });
  answering.classList.add("setup-kotodama-ad-reply");
  const ad = stack("s8", "start", filling(top), filling(wrapped(AD.headline, "subheading", "on_burn_warning")), filling(product), filling(stars), filling(answering), filling(fine));
  ad.classList.add("setup-kotodama-ad");
  return ad;
}

/** A lane's clock: a Chip with a timer, counting until its answer lands; Kotodama's in the cold. */
function clockChip(words: string, ours: boolean): HTMLElement {
  return chip({ label: words, icon: "timer", kind: ours ? "Cold" : "Neutral" });
}

// ----------------------------------------------------------------- The card --

export interface KotodamaCard {
  readonly state: Kotodama;
  readonly body: () => HTMLElement;
  readonly page: (skipped: () => boolean) => RailPage;
  /** The keys the rail lists at its foot for the step shown. */
  readonly hints: () => readonly KeyHint[];
  /** Given the screen it is drawn on: a view that starts with the menu open lays it over it. */
  readonly over: (screen: Element) => void;
}

const WORDS: Record<StepIndex, string> = {
  0: "The Command menu. One key, wherever you are: type what you want, Enter does it.",
  1: "One key for everything on this machine, and nothing off it. No ads, no news, no weather nobody asked for.",
  2: "The keys that open it, what it finds, and what it does when nothing here matches.",
  3: "Nothing left to set.",
};

/** What's examples: what to type, what it is, and its symbol. */
const EXAMPLES = [
  { typed: "firefox", is: "an app", icon: "public" },
  { typed: "wifi", is: "a setting", icon: "wifi" },
  { typed: "12*7", is: "a sum", icon: "calculate" },
] as const;

/** A label over a part of Yours, `subheading`. */
const heading = (words: string) => text(words, "subheading");

/** What the menu searches, a row each on Yours: the Extensions KubaOS turns on, as they are. */
const SOURCES = [
  { id: "apps", title: "Apps", line: "Installed apps by name · firefox", icon: "desktop_windows" },
  { id: "actions", title: "Settings and actions", line: "Wi-Fi, volume, updates · wifi", icon: "settings" },
  { id: "projects", title: "Projects and scripts", line: "~/Projects and ~/Scripts · kubaos", icon: "folder_open" },
  { id: "windows", title: "Windows and key binds", line: "Open windows, every shortcut", icon: "keyboard_command_key" },
  { id: "system", title: "Power and session", line: "Lock, suspend, reboot · lock", icon: "power_settings_new" },
  { id: "clipboard", title: "Clipboard history", line: "What you copied this week · clip", icon: "content_paste" },
  { id: "emoji", title: "Emoji", line: "By name or keyword · fire", icon: "mood" },
  { id: "calculator", title: "Sums", line: "Units, money, dates · 5 km in mi", icon: "calculate" },
] as const;

/** Space on a list's current row switches it, as a click does. */
function spaceSwitches(list: { readonly box: HTMLElement; readonly rows: { readonly current: () => HTMLElement | null } }): void {
  list.box.addEventListener("keydown", (event) => {
    if (event.key !== " ") return;
    event.preventDefault();
    list.rows.current()?.click();
  });
}

/** An option: its title and note, and a switch it turns, or a control of its own. */
type Option = { readonly title: string; readonly subtitle: string } & ({ readonly get: () => boolean; readonly set: (on: boolean) => void } | { readonly control: HTMLElement });

/**
 * Options as the library's list rows, the same part as It finds's: the title over its note, the
 * control at the row's end. A switch row turns with a click or Space; a row with a control of its
 * own leaves the clicks to it.
 */
function optionList(options: readonly Option[]): HTMLElement {
  const spec = (each: Option): ListRowSpec => ({ title: each.title, subtitle: each.subtitle, ...("get" in each ? { toggle: each.get() } : {}) });
  const made = listWith(options.map(spec), {
    onTake: (at) => {
      const each = options[at];
      if (!each || !("get" in each)) return;
      each.set(!each.get());
      made.replaceRow(at, spec(each));
    },
  });
  spaceSwitches(made);
  made.box.querySelectorAll<HTMLElement>(".k-list-row").forEach((item, at) => {
    const each = options[at];
    if (!each || !("control" in each)) return;
    item.dataset.control = "";
    item.firstElementChild?.append(each.control);
  });
  return made.box;
}

/**
 * `content` in a scroll of its own, which fills its column's height: its thumb inside its own
 * box, the page's gap kept clear of the band (`kobo.css`).
 */
function scrolling(content: HTMLElement): HTMLElement {
  const padded = stack(0, "start", filling(content));
  padded.classList.add("kobo-scrolled");
  const { box } = scroll(padded, { height: "100%", below: "s24", noGutter: true, fade: "s24", footFade: true });
  box.classList.add("kobo-scrolls");
  return box;
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
      // An example: its symbol on a tile, what to type and what it is; the page's own row, lit in
      // step with the menu beside it.
      const tile = iconTile({ glyph: each.icon, size: "Sm" });
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
    const lit = (at: number) =>
      examples.forEach((each, index) => {
        const on = index === at;
        if (each.hasAttribute("data-lit") === on) return;
        each.toggleAttribute("data-lit", on);
        each.querySelector(".k-icon-tile")?.replaceWith(iconTile({ glyph: EXAMPLES[index]!.icon, size: "Sm", accent: on }));
      });
    const play = async () => {
      if (settings().reduced) {
        lit(1);
        menu.set("wifi");
        return;
      }
      // Drawn for a shot (`?chrome=off`), it stands on its first ask, typed.
      if (!CHROME) {
        lit(0);
        menu.set(EXAMPLES[0].typed);
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
      keyPrompt("Super+Space"),
      gap("s32"),
      wrapped("Type what you want. Enter.", "headline"),
      gap("s8"),
      filling(wrapped("Apps, settings, projects, windows and sums, in one place. It opens over whatever you're doing, on the screen you're on.", "body", "on_surface_variant")),
      gap("s32"),
      filling(stack("s8", "start", ...examples.map(filling))),
    );
    left.classList.add("setup-kotodama-what");
    return splitColumns([2, 3], "s40", [filling(left), filling(fillingHeight(menu.box))], { fillHeight: true });
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

    /**
     * The two, asked the question at the same moment: Kotodama answers with the key, Start in its
     * own time. Held at `state.raceAt`, it is drawn as it stands then; reduced, as it ends.
     */
    const race = () => {
      stop();
      const reduced = settings().reduced;
      const ad = fakeAd(state.tip ?? false, state.reply ?? false);
      const start = otherMenu(QUESTION, startGroups(QUESTION), OTHERS.start.searching, { over: ad, covers: 4 });
      // Kotodama's lane fills its rows with what is on the machine: no web rows under it here.
      const ours = standIn({ ...prefs, web: false }, { live: false, query: QUESTION, hints: [["↑↓", "Select"], ["Enter", "Run"]], took: OTHERS.kotodama.stopped });
      ours.box.inert = true;
      const startClock = document.createElement("span");
      const oursClock = document.createElement("span");
      oursClock.append(clockChip(OTHERS.kotodama.stopped, true));
      const read = (words: string) => startClock.replaceChildren(clockChip(words, false));
      let roast = wrapped(OTHERS.start.still, "small", "on_surface_variant");
      const oursBox = document.createElement("span");
      oursBox.className = "setup-kotodama-ours";
      oursBox.append(ours.box);
      // The race, three lines of two lanes: the names and clocks on one line, the two menus one
      // height, the lines under them starting on one line.
      const head = (name: string, clock: HTMLElement) => row("s8", "centre", text(name, "heading"), fill(), clock);
      const roastHolder = document.createElement("span");
      roastHolder.append(roast);
      start.box.dataset.lane = "start";
      oursBox.dataset.lane = "kotodama";
      const race = stack(
        "s12",
        "start",
        filling(grid(2, "s24", [head(OTHERS.start.name, startClock), head(OTHERS.kotodama.name, oursClock)])),
        fillingHeight(filling(grid(2, "s24", [filling(start.box), filling(oursBox)], { fillHeight: true }))),
        filling(grid(2, "s24", [roastHolder, wrapped(OTHERS.kotodama.roast, "small", "on_surface_variant")])),
      );
      race.classList.add("setup-kotodama-race");
      lanes.replaceChildren(race);
      const [web, search, news, recommended, weather, answerGroup] = start.groups;
      const parts: readonly (readonly [HTMLElement | undefined, number, string?])[] = [
        [web, LANDS.start.web],
        [search, LANDS.start.web],
        [news, LANDS.start.news],
        [recommended, LANDS.start.recommended],
        [ad, LANDS.start.ad, "0 var(--space-s48)"],
        [weather, LANDS.start.weather],
        [answerGroup, LANDS.start.answer],
      ];
      /** Start as it stands `spent` milliseconds after the key: what has landed, and its clock. */
      const at = (spent: number) => {
        for (const [part, lands] of parts) if (part) part.style.opacity = spent >= lands ? "" : "0";
        read(spent >= LANDS.start.done ? OTHERS.start.stopped(seconds(LANDS.start.done)) : seconds(spent));
        if (spent < LANDS.start.done) return;
        start.settle();
        if (roast.textContent === OTHERS.start.still) roast.replaceWith((roast = wrapped(OTHERS.start.roast(QUESTION), "small", "on_surface_variant")));
      };
      // Reduced, or drawn for a shot (`?chrome=off`), the race stands run out, the same every time.
      if (reduced || !CHROME || state.raceAt !== undefined) return at(reduced || !CHROME ? LANDS.start.done : (state.raceAt ?? 0));
      at(0);
      const speed = settings().speed;
      for (const [part, lands, from] of parts) {
        if (!part) continue;
        timers.push(
          window.setTimeout(() => {
            part.style.opacity = "";
            part.animate([{ opacity: 0, translate: from ?? "0 var(--space-s16)" }, { opacity: 1, translate: "0 0" }], { duration: ms("long"), easing: curve("emphasized_decel") });
          }, lands / speed),
        );
      }
      const from = performance.now();
      const tick = () => {
        const spent = (performance.now() - from) * speed;
        read(spent >= LANDS.start.done ? OTHERS.start.stopped(seconds(LANDS.start.done)) : seconds(spent));
        if (spent < LANDS.start.done) frame = requestAnimationFrame(tick);
        else at(LANDS.start.done);
      };
      frame = requestAnimationFrame(tick);
    };
    raceAgain = race;
    race();
    return stack(0, "start", fillingHeight(filling(lanes)));
  };

  // ---------------------------------------------------------------- Yours --

  const yours = (): HTMLElement => {
    // Each setting applies to the menu at once; it is tried in the menu itself, with its keys from
    // here, so this step shows no preview of its own: its foot says so, the keys picked in it.
    const live = document.createElement("span");
    const say = () =>
      live.replaceChildren(
        row("s8", "centre", icon("info", "var(--type-small-icon)", "var(--color-on_surface_variant)"), row("s4", "centre", text("Changes are live as you make them. Press", "small", "on_surface_variant"), caps(KEY_OF[prefs.key]), text("any time to try.", "small", "on_surface_variant"))),
      );
    say();

    // Top, across the body: the keys that open it, three cards side by side.
    const cards = OPENS_WITH.map((each) => {
      const card = choiceCard({ title: each.title, subtitle: each.line, picks: "one", picked: prefs.key === each.id });
      card.querySelector(".k-card-header, .k-card-title-line")?.parentElement?.prepend(caps(each.keys));
      return card;
    });
    const keysGroup = choiceGroup("one", 3, cards);
    keysGroup.classList.add("setup-kotodama-keycards");
    keysGroup.addEventListener("click", () =>
      queueMicrotask(() => {
        const at = cards.findIndex((each) => each.getAttribute("aria-checked") === "true");
        if (at >= 0) prefs.key = OPENS_WITH[at]!.id;
        say();
        recheck();
      }),
    );

    // Left: what it finds, a row a source, each switched on or off.
    const sources = SOURCES.map((each) => ({ ...each, on: () => (each.id === "apps" ? prefs.apps : prefs.finds[each.id]) }));
    const spec = (each: (typeof sources)[number]): ListRowSpec => ({ title: each.title, subtitle: each.line, icon: each.icon, tile: true, accent: each.on(), toggle: each.on() });
    const findsList = listWith(sources.map(spec), {
      onTake: (at) => {
        const each = sources[at];
        if (!each) return;
        const now = !each.on();
        if (each.id === "apps") prefs.apps = now;
        else prefs.finds[each.id] = now;
        findsList.replaceRow(at, spec(each));
      },
    });
    spaceSwitches(findsList);

    // Right: the options, each a row of the same list part as the left's: its title over its
    // note, its control at the row's end.
    const engines = orderList({ items: prefs.engines });
    const motion = tabs({ choices: [{ label: "Full" }, { label: "Fades" }], picked: prefs.slides ? 0 : 1, onPick: (at) => (prefs.slides = at === 0) });
    const mark = tabs({ choices: [{ label: "1.5" }, { label: "1,5" }], picked: prefs.comma ? 1 : 0, onPick: (at) => (prefs.comma = at === 1) });
    const files = button({ label: "Open in Files", kind: "Text", size: "Sm" });
    const web = optionList([{ title: "Offer a web search", subtitle: "Rows at the bottom, in this order", get: () => prefs.web, set: (on) => ((prefs.web = on), engines.setDisabled(!on)) }]);
    web.querySelector(".k-list-rows")?.append(engines.box);
    engines.setDisabled(!prefs.web);
    const typing = optionList([
      { title: "Forgive typos", subtitle: "“fierfox” finds Firefox", get: () => prefs.typos, set: (on) => (prefs.typos = on) },
      { title: "Motion", subtitle: "Fades alone: nothing slides", control: motion },
      { title: "Decimal mark", subtitle: "How sums write a half", control: mark },
    ]);
    const file = optionList([{ title: "Aliases and your own actions", subtitle: "In user.toml", control: files }]);

    const section = (title: string, ...content: HTMLElement[]) => stack("s8", "start", heading(title), ...content.map(filling));
    const left = section("It finds", findsList.box);
    const right = stack("s24", "start", filling(section("When nothing here matches", web)), filling(section("Typing and motion", typing)), filling(section("The config file", file)));
    // The two columns, side by side where both read whole, the list
    // first under it.
    const split = document.createElement("span");
    split.className = "setup-kotodama-yours";
    split.append(left, right);
    new ResizeObserver(() => {
      const scale = Number.parseFloat(getComputedStyle(split).getPropertyValue("--scale")) || settings().scale;
      split.toggleAttribute("data-stacked", split.clientWidth / scale < length("size-column") * 1.25);
    }).observe(split);
    // One rhythm: `space.s24` between sections, `space.s8` between a title and its controls. The
    // whole step scrolls as one where it is taller than the room.
    // The settings scroll; the line stays under them, as much room above it as the template leaves
    // under it, over the band, as Make it yours's newcomer's line does.
    return stack(0, "start", fillingHeight(filling(scrolling(stack("s24", "start", filling(section("Opens with", keysGroup)), filling(split))))), gap("s24"), filling(live));
  };

  // --------------------------------------------------------------- Try it --

  /** The keys on Try it, while it is shown: they press as the keys are pressed. */
  let prompt: HTMLElement | null = null;

  /** The keys pressed, from the keyboard or a click: Try it's KeyPrompt shows the press. */
  const press = () => {
    if (prompt?.isConnected) pressKeys(prompt);
  };

  /**
   * Try it arriving: its caps rise `space.s8` into place and fade in over `long`, a `micro` apart,
   * the `+` between them in its turn, as the step itself slides in; the two lines follow a `micro`
   * after the last. Reduced, they come with the step.
   */
  const arrive = (group: HTMLElement) => {
    if (settings().reduced) return;
    const parts = [...(prompt?.children ?? [])] as HTMLElement[];
    const lines = [...group.querySelectorAll<HTMLElement>(".setup-kotodama-line")];
    parts.forEach((part, at) => part.animate([{ opacity: 0, "--kotodama-rise": 1 }, { opacity: 1, "--kotodama-rise": 0 }], { duration: ms("long"), delay: ms("short") + at * ms("micro"), easing: curve("emphasized_decel"), fill: "backwards" }));
    for (const each of lines) each.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("long"), delay: ms("short") + parts.length * ms("micro"), easing: curve("emphasized_decel"), fill: "backwards" });
  };

  /**
   * The closing page: the keys picked on Yours, large, and two lines under them; nothing to do on
   * it. The group stands two fifths of the way down its room, where the menu itself opens, so it
   * reads as centred rather than sagging below the middle.
   */
  const tryIt = (): HTMLElement => {
    prompt = keyPrompt(KEY_OF[prefs.key]);
    prompt.classList.add("setup-kotodama-keys");
    prompt.addEventListener("click", () => {
      press();
      menu.open();
    });
    const headline = text("It's already yours.", "headline");
    const line = text("Every change was live as you made it. Press it any time, here or anywhere.", "body_large", "on_surface_variant");
    headline.classList.add("setup-kotodama-line");
    line.classList.add("setup-kotodama-line");
    const group = stack(0, "centre", prompt, gap("s40"), headline, gap("s12"), line);
    group.classList.add("setup-kotodama-close");
    const holder = document.createElement("span");
    holder.className = "setup-kotodama-closing";
    holder.append(group);
    // Two fifths of the room the group leaves stand above it, rounded down to a whole device pixel
    // as `menuMeasures` places the menu: the group lands where the menu opens, on the pixel grid.
    // The room's height is the step's and the group's is its content's, so it is worked out here.
    const placing = new ResizeObserver(() => {
      const spare = holder.clientHeight - group.offsetHeight;
      const above = `${Math.max(0, Math.floor((spare * 2) / 5))}px`;
      if (group.style.marginTop !== above) group.style.marginTop = above;
    });
    placing.observe(holder);
    placing.observe(group);
    return holder;
  };

  // ------------------------------------------------------- The menu, open --

  /**
   * The menu open over the whole screen, as on the desktop: at `menuMeasures`'s place for the
   * screen it is on, centred across it two fifths of the way down; its keys open it from any step,
   * Esc or a click beside it shuts it.
   */
  const menu = (() => {
    const layer = document.createElement("span");
    layer.className = "setup-kotodama-over";
    const at = document.createElement("span");
    at.className = "setup-kotodama-at";
    layer.append(at);
    let inside: StandIn | null = null;
    let watched: Element | null = null;
    const place = new ResizeObserver(() => {
      const screen = layer.parentElement;
      if (!screen) return;
      const scale = scaleOf(screen as HTMLElement);
      const width = screen.clientWidth / scale;
      const height = screen.clientHeight / scale;
      const measures = menuMeasures(width, height, true);
      at.style.left = `${Math.round(((width - measures.width) / 2) * scale)}px`;
      at.style.top = `${Math.round(measures.top * scale)}px`;
      at.style.width = `${Math.round(measures.width * scale)}px`;
      at.style.height = `${Math.round(measures.height * scale)}px`;
    });
    const shut = () => {
      state.open = false;
      layer.remove();
      element?.focus({ preventScroll: true });
    };
    layer.addEventListener("click", (event) => {
      if (event.target === layer) shut();
    });
    const open = (animated = true, on?: Element) => {
      const screen = on ?? element?.closest(".setup-room") ?? element;
      if (!screen) return;
      state.open = true;
      if (!layer.isConnected) {
        // A fresh menu each time, as the desktop's is: nothing typed, the first Result current.
        inside = standIn(prefs, { live: true, onEscape: shut });
        at.replaceChildren(inside.box);
        screen.append(layer);
        if (watched !== screen) {
          place.observe(screen);
          watched = screen;
        }
        if (animated && !settings().reduced) at.animate(prefs.slides ? [{ opacity: 0, translate: "0 calc(-1 * var(--space-s8))" }, { opacity: 1, translate: "0 0" }] : [{ opacity: 0 }, { opacity: 1 }], { duration: ms("medium"), easing: curve("emphasized_decel") });
        else if (animated) at.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("short"), easing: curve("standard") });
      }
      const focus = inside;
      requestAnimationFrame(() => focus?.focus());
    };
    return { open };
  })();

  // ----------------------------------------------------------------- Steps --

  const stepView = (at: StepIndex): HTMLElement => {
    stop();
    stop = () => undefined;
    if (at !== 3) prompt = null;
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
    if (to === 3) arrive(next);
    recheck();
  };
  const backTo = (to: number) => show(to as StepIndex);

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
      if (!opens) return;
      event.preventDefault();
      // The menu opens over the screen from any step, as it does on the desktop: the settings
      // made on Yours are in it already.
      press();
      menu.open();
    });
    window.addEventListener("keyup", (event) => {
      if (!here() || (event.key !== "Meta" && event.key !== "OS") || !alone) return;
      alone = false;
      if (prefs.key !== "super") return;
      press();
      menu.open();
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
    if (state.step === 3 && state.from !== undefined) arrive(shown);
    const content = stack("s24", "start", filling(line), fillingHeight(filling(room)));
    element = body(spec.accent, spec.title, WORDS[state.step], content, false);
    element.classList.add("setup-kotodama");
    // On Why, Enter races the three again; the band's Next still goes on.
    element.addEventListener("keydown", (event) => {
      if (state.step !== 1 || event.key !== "Enter" || (event.target as Element).closest("button")) return;
      event.preventDefault();
      raceAgain();
    });
    listen();
    return element;
  };

  const status = (): RailStatus => ({ state: { is: state.passed ? "done" : "todo" } });

  const NEXT: Record<StepIndex, string> = { 0: "Next: why", 1: "Next: yours", 2: "Next: try it", 3: "Next: Git" };
  const NOTE: Record<StepIndex, string | undefined> = {
    0: "Nothing here is required",
    1: "Enter races them again",
    2: "Defaults are fine. Really",
    3: undefined,
  };

  return {
    state,
    body: draw,
    over: (screen) => {
      if (state.open) menu.open(false, screen);
    },
    hints: (): readonly KeyHint[] => {
      if (state.step === 1) return [["Enter", "race again"], ["Esc", "back a step"]];
      if (state.step === 2) return [["Space", "switch it"], ["Alt+↑↓", "move an engine"], ["Tab", "next field"]];
      if (state.step === 3) return state.open ? [["↑ ↓", "pick a Result"], ["Enter", "run it"], ["Esc", "close it"]] : [[KEY_OF[prefs.key], "open it"]];
      return [[KEY_OF[prefs.key], "open it"], ["↑ ↓", "pick a Result"], ["Enter", "run it"]];
    },
    page: (skipped) => ({
      step: { title: spec.title, icon: spec.icon },
      body: draw,
      status,
      next: () => NEXT[state.step],
      skip: () => (state.passed ? undefined : skipped() ? "Later" : "Skip"),
      note: () => NOTE[state.step],
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
