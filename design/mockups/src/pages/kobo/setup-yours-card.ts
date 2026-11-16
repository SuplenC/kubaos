// First boot's Make it yours card, as an organism of library parts, in three steps of its own on a
// Stepper under its title. What for: the three profiles as lit lg choice cards, any of them or
// none. Your apps: every app of the catalogue by category, a category index beside them that
// follows the scroll, the apps the picks brought ticked and lit in a wave as the step arrives, each
// tick saying where it came from. Go: how many apps and which, by category, then Install; then the
// install running in the background, finished, or an app that failed.
//
// It is drawn once and kept: a pick, a tick and the install moving on change it in place, and a
// step turns inside the card (its header and band stay) rather than the rail's page.

import { choiceCard, choiceGroup } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { collapsible } from "../../lib/collapsible.ts";
import { descriptionList } from "../../lib/description.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { hangs } from "../../lib/button-bar.ts";
import { icon } from "../../lib/icon.ts";
import type { KeyHint } from "../../lib/keycap.ts";
import { fill, filling, fillingHeight, flow, gap, row, splitColumns, stack } from "../../lib/layout.ts";
import { type ListItem, type ListRowSpec, listLabel, listWith, type Rows } from "../../lib/list.ts";
import { log, type LogLine } from "../../lib/log.ts";
import { progressBar } from "../../lib/progress.ts";
import { scroll } from "../../lib/scroll.ts";
import { RECHECK, type RailPage, type RailStatus } from "../../lib/step-rail.ts";
import { setStep, stepper } from "../../lib/stepper.ts";
import { text, wrapped } from "../../lib/text.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import { body } from "./kobo-parts.ts";
import { CARDS } from "./setup-cards.ts";
import { type App, APPS, app, type Category, CATEGORIES, PROFILES, type ProfileId, VSCODE } from "./setup-data.ts";

/** The card's steps, by their index. */
export const STEPS = ["What for", "Your apps", "Go"] as const;
export type StepIndex = 0 | 1 | 2;

/** Where the card is: choosing (any step), its apps installing, or done with them. */
export type Phase = "choosing" | "installing" | "done";

/** What the person has said on the card, and how far its install is. */
export interface Yours {
  readonly picked: Set<ProfileId>;
  /** Apps ticked or unticked by hand, over what the picks say. */
  readonly hand: Map<string, boolean>;
  step: StepIndex;
  phase: Phase;
  readonly installed: Set<string>;
  /** The app that didn't install, with the cause. */
  failed?: { readonly app: string; readonly error: string; readonly hint: string };
  /** A step turn's first state: the step left, drawn leaving over this one. */
  readonly from?: StepIndex;
}

/** Whether an app goes on: as the hand says, else as a pick proposing it says. */
export function on(state: Yours, each: App): boolean {
  return state.hand.get(each.id) ?? each.profiles.some((p) => state.picked.has(p));
}

/** Every app that goes on, in the catalogue's order. */
export function kept(state: Yours): readonly App[] {
  return APPS.filter((each) => on(state, each));
}

/** The picks that brought an app, when the hand hasn't changed it. */
function broughtBy(state: Yours, each: App): readonly ProfileId[] {
  return state.hand.has(each.id) ? [] : each.profiles.filter((p) => state.picked.has(p));
}

/** A count of apps, its word chosen by the count. */
export function apps(count: number): string {
  return count === 1 ? "1 app" : `${count} apps`;
}

/**
 * Where a view of the card starts. A view `name<N` is `name` reached from step N, drawn as the
 * step turn's first state.
 */
export function yoursAt(view: string): Yours {
  const [name, from] = view.split("<") as [string, string | undefined];
  const base = (picked: ProfileId[], step: StepIndex, more: Partial<Yours> = {}): Yours => ({
    picked: new Set(picked),
    hand: new Map(),
    step,
    phase: "choosing",
    installed: new Set(),
    ...(from === undefined ? {} : { from: Number(from) as StepIndex }),
    ...more,
  });
  const all: ProfileId[] = ["developer", "designer", "casual"];
  const handPicked = () => new Map([["vscode", true], ["discord", true]]);
  const installedUpTo = (state: Yours, count: number) => {
    for (const each of kept(state).slice(0, count)) state.installed.add(each.id);
    return state;
  };
  switch (name) {
    case "developer":
      return base(["developer"], 0);
    case "all":
      return base(all, 0);
    case "apps":
      return base(["designer"], 1, { hand: handPicked() });
    case "apps-none":
      return base([], 1);
    case "vscode":
      return base(["developer"], 1, { hand: new Map([["vscode", true]]) });
    case "go":
      return base(["designer"], 2, { hand: handPicked() });
    case "go-none":
      return base([], 2);
    case "installing":
      return installedUpTo(base(all, 2, { phase: "installing" }), 7);
    case "done": {
      const state = base(all, 2, { phase: "done" });
      return installedUpTo(state, kept(state).length);
    }
    case "failed": {
      const state = base(all, 2, { phase: "done", failed: { app: "steam", error: "Steam didn't install: Valve's server timed out", hint: "The rest are in. Try it again, or carry on without it." } });
      for (const each of kept(state)) if (each.id !== "steam") state.installed.add(each.id);
      return state;
    }
    default:
      return base([], 0);
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
 * A step turning inside the card: the step left slides `space.s24` off toward where it is going and
 * fades over `short`; the next slides in from the other side and fades in over `long`, a `short`
 * after. Shorter than a page's `space.s32`, with the header and the band still, so it reads as the
 * card moving on, not the rail. With reduced motion the step left goes at once and the next fades in.
 */
function turn(leaving: HTMLElement | null, arriving: HTMLElement, forward: boolean): void {
  const { reduced } = settings();
  const toward = forward ? -1 : 1;
  if (leaving) {
    leaving.dataset.leaving = "";
    const gone = leaving.animate(reduced ? [{ opacity: 0 }, { opacity: 0 }] : [{ opacity: 1, "--yours-slide": 0 }, { opacity: 0, "--yours-slide": toward }], { duration: reduced ? 0 : ms("short"), easing: curve("emphasized_accel"), fill: "forwards" });
    void gone.finished.then(() => leaving.remove(), () => leaving.remove());
  }
  if (reduced) arriving.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("short"), easing: curve("standard"), fill: "backwards" });
  else arriving.animate([{ opacity: 0, "--yours-slide": -toward }, { opacity: 1, "--yours-slide": 0 }], { duration: ms("long"), delay: ms("short"), easing: curve("emphasized_decel"), fill: "backwards" });
}

/**
 * The apps the picks brought lighting up as Your apps arrives: each comes up from `opacity.subdued`
 * over `long`, a `micro` after the one before, the seventh on with the rest. Reduced, they are lit.
 */
function wave(rows: readonly HTMLElement[]): void {
  rows.forEach((each, index) => {
    each.dataset.wave = String(Math.min(index, 6));
  });
  if (settings().reduced) return;
  requestAnimationFrame(() => rows.forEach((each, index) => {
    each.animate([{ opacity: "var(--opacity-subdued)" }, { opacity: 1 }], { duration: ms("long"), delay: Math.min(index, 6) * ms("micro"), easing: curve("emphasized_decel"), fill: "backwards" });
  }));
}

/** What a row says changing under the keys (a tick, VS Code's line): it fades in over `short`. */
function changed(element: Element | null): void {
  if (!element || settings().reduced) return;
  element.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("short"), easing: curve("standard") });
}

// ----------------------------------------------------------------- Rows --

/** An app's row: its symbol on a tile, lit when it goes on; its line; where its tick came from. */
function appSpec(state: Yours, each: App): ListRowSpec {
  const going = on(state, each);
  const by = broughtBy(state, each);
  const profile = (id: ProfileId) => PROFILES.find((x) => x.id === id)!;
  const chips = state.hand.get(each.id) === true ? [{ kind: "Cold" as const, label: "Ticked by you", icon: "person", iconOnly: true }] : by.map((id) => ({ kind: "Neutral" as const, label: profile(id).title, icon: profile(id).icon, iconOnly: true }));
  // Ticked anyway, VS Code's line gives way to what Kōbō thinks of that.
  const line = each.id === "vscode" && going ? VSCODE.ticked : each.line;
  return { title: each.name, subtitle: line, icon: each.icon, tile: true, accent: going, checked: going, chips };
}

/** A category's label among the apps: its name and how many of its apps go on. */
function categoryLabel(state: Yours, category: Category): HTMLElement {
  const of = APPS.filter((each) => each.category === category.id);
  const label = listLabel(`${category.title} · ${of.filter((each) => on(state, each)).length} of ${of.length}`, true);
  label.dataset.category = category.id;
  return label;
}

/** A category's row in the index: its symbol, lit while any of its apps goes on, and its count. */
function indexSpec(state: Yours, category: Category): ListRowSpec {
  const of = APPS.filter((each) => each.category === category.id);
  const count = of.filter((each) => on(state, each)).length;
  return { title: category.title, icon: category.icon, tile: true, accent: count > 0, detail: `${count}/${of.length}` };
}

/**
 * What a step scrolls, held the page's padding in from the scroll's end, which reaches across that
 * padding so its thumb floats in it, as every Kōbō page's body does (`kobo.css`).
 */
function padded(content: HTMLElement): HTMLElement {
  const box = stack(0, "start", filling(content));
  box.classList.add("kobo-scrolled");
  return box;
}

// ----------------------------------------------------------------- The card --

/** The live card: its body, kept and changed in place, and what its rail page reads. */
export interface YoursCard {
  readonly state: Yours;
  readonly body: () => HTMLElement;
  readonly page: (act: () => boolean, skipped: () => boolean) => RailPage;
  /** Starts the install, as the `Filled` button does at Go; returns whether it did anything. */
  readonly install: () => boolean;
  /** The keys the rail lists at its foot for the step shown. */
  readonly hints: () => readonly KeyHint[];
  /** Called when the install has finished, with whether an app failed. */
  onFinished: (failed: boolean) => void;
}

/** The log of the install, as far as it has gone. */
function installLog(state: Yours): readonly LogLine[] {
  const lines: LogLine[] = [{ level: "info", text: `$ kubaos apps install ${kept(state).map((each) => each.id).join(" ")}` }];
  for (const each of kept(state)) {
    if (state.installed.has(each.id)) lines.push({ level: "info", text: `ok  ${each.id}` });
    else if (state.failed?.app === each.id) lines.push({ level: "error", text: `${each.id}: ${state.failed.error}` });
  }
  return lines;
}

const WORDS: Record<StepIndex, string> = {
  0: "What do you use it for? Pick any, all or none: I'll pick the apps, and you see them all next.",
  1: "Every app there is. The lit ones go on: your picks chose them; tick or untick any.",
  2: "That's what goes on. It installs in the background while you do the rest.",
};

export function yoursCard(view: string, options: { readonly hood?: boolean } = {}): YoursCard {
  const state = yoursAt(view);
  const spec = CARDS.yours;
  let element: HTMLElement | null = null;
  let line: HTMLElement | null = null;
  let room: HTMLElement | null = null;
  let shown: HTMLElement | null = null;
  let timer = 0;
  const recheck = () => element?.dispatchEvent(new Event(RECHECK, { bubbles: true }));

  // ------------------------------------------------------------- What for --

  const whatFor = (): HTMLElement => {
    const cards = PROFILES.map((profile) => {
      const brings = APPS.filter((each) => each.profiles.includes(profile.id));
      // The first apps only it brings, then a count of the rest: what the pick means, at a glance.
      const own = brings.filter((each) => each.profiles.length < PROFILES.length).slice(0, 4);
      const foot = [...own.map((each) => ({ glyph: each.icon, label: each.name })), { glyph: "add", label: `and ${brings.length - own.length} more` }];
      return choiceCard({ glyph: profile.icon, title: profile.title, subtitle: profile.line, size: "Lg", picks: "many", picked: state.picked.has(profile.id), foot });
    });
    const group = choiceGroup("many", 3, cards);
    group.classList.add("setup-yours-profiles");
    const sync = () =>
      queueMicrotask(() => {
        let moved = false;
        PROFILES.forEach((profile, index) => {
          const now = cards[index]!.getAttribute("aria-checked") === "true";
          if (now === state.picked.has(profile.id)) return;
          moved = true;
          if (now) state.picked.add(profile.id);
          else state.picked.delete(profile.id);
        });
        if (moved) recheck();
      });
    group.addEventListener("click", sync);
    group.addEventListener("keydown", sync);
    const newcomer = row("s8", "centre", icon("info", "var(--type-small-icon)", "var(--color-on_surface_variant)"), wrapped("Not sure? Everyday is a good start. Nothing here is final: the next step shows every app.", "small", "on_surface_variant"));
    return stack("s16", "start", fillingHeight(filling(group)), filling(newcomer));
  };

  // ------------------------------------------------------------ Your apps --

  const yourApps = (arriving: boolean): HTMLElement => {
    const items: ListItem[] = [];
    const byIndex = new Map<number, App>();
    const labels = new Map<string, HTMLElement>();
    for (const category of CATEGORIES) {
      const label = categoryLabel(state, category);
      labels.set(category.id, label);
      items.push(label);
      for (const each of APPS.filter((one) => one.category === category.id)) {
        byIndex.set(items.length, each);
        items.push(appSpec(state, each));
      }
    }
    let index: { box: HTMLElement; rows: Rows; replaceRow: (at: number, spec: ListRowSpec) => HTMLElement } | null = null;
    const list = listWith(items, {
      flow: true,
      onTake: (at) => {
        const each = byIndex.get(at);
        if (!each || state.phase === "installing") return;
        const now = !on(state, each);
        const byPicks = each.profiles.some((p) => state.picked.has(p));
        if (now === byPicks) state.hand.delete(each.id);
        else state.hand.set(each.id, now);
        if (state.phase === "done") state.phase = "choosing";
        const fresh = list.replaceRow(at, appSpec(state, each));
        changed(fresh.querySelector(".k-list-check"));
        if (each.id === "vscode") changed(fresh.querySelector(".k-list-title")?.nextElementSibling ?? null);
        const category = CATEGORIES.find((c) => c.id === each.category)!;
        // The label stays where it is, its words drawn again: the list keeps its place among the rows.
        labels.get(category.id)?.replaceChildren(...categoryLabel(state, category).childNodes);
        index?.replaceRow(CATEGORIES.indexOf(category), indexSpec(state, category));
        recheck();
      },
    });
    list.box.classList.add("setup-yours-list");
    index = listWith(
      CATEGORIES.map((category) => indexSpec(state, category)),
      { current: 0, onTake: (at) => jumpTo(at, true) },
    );
    index.box.classList.add("setup-yours-index");
    const { box: scroller, view } = scroll(padded(splitColumns([2, 5], "s24", [index.box, list.box])), { height: "100%", above: "s24", below: "s16", noGutter: true, fade: "s24", footFade: true });
    scroller.classList.add("kobo-scrolls", "setup-yours-scroll");
    const behaviour = (): ScrollBehavior => (settings().reduced ? "auto" : "smooth");
    /** To category `at`: the apps scroll to its label; from the keys, the keys go to its first app. */
    const jumpTo = (at: number, keys: boolean) => {
      const category = CATEGORIES[at];
      const label = category ? labels.get(category.id) : undefined;
      if (!label) return;
      view.scrollTo({ top: label.offsetTop, behavior: behaviour() });
      if (!keys) return;
      const first = label.nextElementSibling as HTMLElement | null;
      if (first) {
        list.rows.put(first);
        list.box.focus({ preventScroll: true });
      }
    };
    // ↑ ↓ in the index move the apps with it; Enter and a click go there with the keys.
    index.box.addEventListener("keydown", (event) => {
      if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
      queueMicrotask(() => {
        const rows = [...(index?.box.querySelectorAll(".k-list-row") ?? [])];
        const at = rows.indexOf(index?.rows.current() as Element);
        if (at >= 0) jumpTo(at, false);
      });
    });
    // The index follows the scroll: the category whose label has reached the top is the one lit.
    let lit = 0;
    view.addEventListener(
      "scroll",
      () => {
        const top = view.scrollTop + 1;
        let at = 0;
        CATEGORIES.forEach((category, i) => {
          const label = labels.get(category.id);
          if (label && label.offsetTop <= top) at = i;
        });
        if (at === lit || !index) return;
        lit = at;
        const target = index.box.querySelectorAll<HTMLElement>(".k-list-row")[at];
        if (target) index.rows.put(target);
      },
      { passive: true },
    );
    if (arriving) wave([...list.box.querySelectorAll<HTMLElement>(".k-list-row[data-picked]")].filter((each) => !each.querySelector(".k-chip[data-kind='Cold']")));
    return scroller;
  };

  // -------------------------------------------------------------------- Go --

  const progress = (): HTMLElement | null => {
    if (state.phase !== "installing") return null;
    const total = kept(state).length;
    const bar = progressBar({ fraction: total === 0 ? 0 : state.installed.size / total });
    return stack("s4", "start", filling(bar), filling(row("s8", "centre", text("Installing", "small", "on_surface_variant"), fill(), text(`${state.installed.size} of ${total}`, "small", "on_surface_variant"))));
  };

  /** An app's chip at Go: its symbol while it waits, then where its install is. */
  const appChip = (each: App) => {
    const yours = state.hand.get(each.id) === true;
    if (state.installed.has(each.id)) return chip({ label: each.name, kind: "Accent", icon: "check" });
    if (state.failed?.app === each.id) return chip({ label: each.name, kind: "Warning", icon: "error" });
    if (state.phase === "installing" && kept(state).find((one) => !state.installed.has(one.id))?.id === each.id) return chip({ label: each.name, kind: "Neutral", icon: "progress_activity" });
    return chip({ label: each.name, kind: yours ? "Cold" : "Neutral", icon: each.icon });
  };

  const hood = (): HTMLElement => {
    const facts = [
      { label: "Picked", value: state.picked.size === 0 ? "Nothing" : PROFILES.filter((each) => state.picked.has(each.id)).map((each) => each.title).join(", ") },
      { label: "Installs", value: kept(state).length === 0 ? "Nothing" : kept(state).map((each) => each.id).join(", ") },
    ];
    const lines = state.phase === "choosing" ? null : installLog(state);
    const content = stack("s16", "start", filling(descriptionList(facts)), filling(stack("s8", "start", text("The same in a terminal", "label", "on_surface_variant"), wrapped(spec.command, "mono"))), ...(lines ? [filling(log(lines).box)] : []));
    const summary = lines ? `${lines.length} lines of log` : kept(state).length === 0 ? "Installs nothing" : `Installs ${apps(kept(state).length)}`;
    return hangs(collapsible({ title: "Under the hood", summary, content, open: (options.hood ?? false) || state.failed !== undefined }));
  };

  const go = (): HTMLElement => {
    const all = kept(state);
    const count = all.length;
    const vscode = all.some((each) => each.id === "vscode");
    const figure = state.phase === "done" ? `${apps(state.installed.size)} in` : count === 0 ? "Nothing" : apps(count);
    const lede = state.failed
      ? `${app(state.failed.app).name} didn't make it. The rest are on the machine.`
      : state.phase === "done"
        ? "Find them with Super, then their name."
        : state.phase === "installing"
          ? "going on in the background. Carry on with Git and the rest while they land."
          : count === 0
            ? "No apps: you get KubaOS as it comes, a terminal, a browser and the basics. Back to pick some, or carry on."
            : "go on this machine, in the background: carry on with Git and the rest while they land.";
    const big = text(figure, "display", "primary_lit");
    big.classList.add("setup-yours-figure");
    const left = stack("s8", "start", big, filling(wrapped(lede, "body_large", "on_surface_variant")), ...(vscode && state.phase === "choosing" ? [filling(wrapped(`VS Code included. ${VSCODE.ticked}`, "small", "primary"))] : []));
    const bar = progress();
    if (bar) left.append(gap("s8"), filling(bar));
    const groups = CATEGORIES.filter((category) => all.some((each) => each.category === category.id)).map((category) =>
      stack("s8", "start", row("s8", "centre", icon(category.icon, "var(--type-label-icon)", "var(--color-on_surface_variant)"), text(category.title, "label", "on_surface_variant")), filling(flow("s8", ...all.filter((each) => each.category === category.id).map(appChip)))),
    );
    const failure = state.failed ? [filling(errorBlock({ message: `Fuck. ${state.failed.error}.`, hint: state.failed.hint }))] : [];
    const columns = count === 0 ? filling(left) : filling(splitColumns([1, 2], "s40", [left, stack("s16", "start", ...groups.map((each) => filling(each)))]));
    const later = wrapped(spec.skip, "small", "on_surface_variant");
    later.classList.add("setup-later");
    const content = stack("s24", "start", ...failure, columns, ...(state.phase === "choosing" ? [filling(later)] : []), filling(hood()));
    const { box } = scroll(padded(content), { height: "100%", above: "s24", below: "s16", noGutter: true, fade: "s24", footFade: true });
    box.classList.add("kobo-scrolls");
    return box;
  };

  // ----------------------------------------------------------------- Steps --

  const stepView = (at: StepIndex, arriving: boolean): HTMLElement => {
    const view = at === 0 ? whatFor() : at === 1 ? yourApps(arriving) : go();
    const holder = document.createElement("span");
    holder.className = "setup-yours-step";
    holder.dataset.step = String(at);
    holder.append(view);
    return holder;
  };

  const headerFor = (at: StepIndex): HTMLElement => body(spec.accent, spec.title, WORDS[at], document.createElement("span"), false).querySelector<HTMLElement>(".k-header")!;

  /** Shows step `to`, turning to it from the one shown. */
  const show = (to: StepIndex) => {
    if (!room) return;
    const forward = to > state.step;
    const from = state.step;
    state.step = to;
    const next = stepView(to, to === 1 && from === 0);
    const left = shown;
    room.append(next);
    shown = next;
    if (line) setStep(line, to, state.phase === "choosing" ? backTo : undefined);
    element?.querySelector(".k-header")?.replaceWith(headerFor(to));
    turn(left, next, forward);
    // The keys stay where they were, on Next: Enter, Enter, Enter goes through to Install.
    recheck();
  };
  const backTo = (to: number) => show(to as StepIndex);

  /** The step shown drawn again where it is, as the install moves on: no turn. */
  const redraw = () => {
    if (!room || !shown) return;
    const again = stepView(state.step, false);
    shown.replaceWith(again);
    shown = again;
    if (line) setStep(line, state.step, state.phase === "choosing" ? backTo : undefined);
    recheck();
  };

  const draw = (): HTMLElement => {
    if (element) return element;
    line = stepper({ steps: [...STEPS], current: state.step, ...(state.phase === "choosing" ? { onBack: backTo } : {}) });
    room = document.createElement("span");
    room.className = "setup-yours-room";
    if (state.from !== undefined) {
      const left = stepView(state.from, false);
      left.dataset.leaving = "";
      room.append(left);
    }
    shown = stepView(state.step, state.from === 0 && state.step === 1);
    room.append(shown);
    const content = stack("s24", "start", filling(line), fillingHeight(filling(room)));
    element = body(spec.accent, spec.title, WORDS[state.step], content, false);
    element.classList.add("setup-yours");
    return element;
  };

  const tick = () => {
    const next = kept(state).find((each) => !state.installed.has(each.id) && each.id !== state.failed?.app);
    if (!next) {
      state.phase = "done";
      redraw();
      handle.onFinished(state.failed !== undefined);
      return;
    }
    state.installed.add(next.id);
    redraw();
    timer = window.setTimeout(tick, ms("medium"));
  };

  const install = (): boolean => {
    if (state.phase === "installing") return false;
    const count = kept(state).filter((each) => !state.installed.has(each.id)).length;
    if (count === 0) return false;
    delete state.failed;
    state.phase = "installing";
    redraw();
    window.clearTimeout(timer);
    timer = window.setTimeout(tick, ms("medium"));
    return true;
  };

  const status = (): RailStatus => {
    if (state.phase === "installing") return { state: { is: "todo" }, running: true };
    if (state.failed) return { state: { is: "todo" }, failed: true };
    if (state.phase === "done") return { state: { is: "done" } };
    return { state: { is: "todo" } };
  };

  /** The `Filled` button's words and symbol at the step shown. */
  const next = (): { readonly label: string; readonly icon?: string } => {
    const count = kept(state).length;
    if (state.failed) return { label: `Try ${app(state.failed.app).name} again`, icon: "restart_alt" };
    if (state.phase !== "choosing") return { label: "Continue" };
    if (state.step === 0) return { label: count > 0 ? "Next: your apps" : "Next: pick apps myself" };
    if (state.step === 1) return { label: "Next: check and go" };
    return count > 0 ? { label: `Install ${apps(count)}`, icon: "download" } : { label: "Continue" };
  };

  const note = (): string | undefined => {
    const all = kept(state);
    const yours = all.filter((each) => state.hand.get(each.id) === true).length;
    if (state.phase === "installing") return "Installing in the background: carry on.";
    if (state.phase !== "choosing") return undefined;
    if (state.step === 0) return all.length ? `${apps(all.length)} picked for you` : undefined;
    if (state.step === 1) return yours ? `${apps(all.length)}: ${all.length - yours} from your picks, ${yours} yours` : `${apps(all.length)} from your picks`;
    return undefined;
  };

  const handle: YoursCard = {
    state,
    body: draw,
    install,
    onFinished: () => undefined,
    hints: (): readonly KeyHint[] => {
      if (state.phase !== "choosing" || state.step === 2) return [["Tab", "next field"]];
      if (state.step === 0) return [["← →", "pick"], ["Space", "light it"], ["Tab", "next field"]];
      return [["↑ ↓ ← →", "move"], ["Space", "tick"], ["Tab", "index, apps"]];
    },
    page: (act, skipped) => ({
      step: { title: spec.title, icon: spec.icon },
      body: draw,
      status,
      next: () => next().label,
      nextIcon: () => next().icon,
      skip: () => (state.phase === "choosing" ? (skipped() ? "Later" : "Skip") : undefined),
      note,
      // Back goes back a step inside the card while it is being chosen; from What for, and once the
      // install has started, it leaves the card.
      back: () => {
        if (state.phase !== "choosing" || state.step === 0) return false;
        show((state.step - 1) as StepIndex);
        return true;
      },
      act: () => {
        if (state.phase === "choosing" && state.step < 2) {
          show((state.step + 1) as StepIndex);
          return true;
        }
        if (state.phase === "choosing" && kept(state).length === 0) {
          // Nothing to install, said and seen: the card is done as it is.
          state.phase = "done";
          recheck();
          return false;
        }
        return act();
      },
    }),
  };
  return handle;
}

/** The apps a view installs, for the words around it: the arrival's and the ending's. */
export function installing(view: string): number {
  return kept(yoursAt(view)).length;
}
