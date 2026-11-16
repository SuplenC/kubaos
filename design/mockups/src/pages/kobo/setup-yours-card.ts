// First boot's Make it yours card, as an organism of library parts: what the person uses the machine
// for (any of three profiles, none included) and the apps those bring, as a live list grouped by the
// profile that brought each. It is drawn once and kept: picking a profile,
// dropping or adding an app and the install moving on change it in place, so the list's rows
// arrive, leave and glide as LiveList moves them.

import { choiceCard, choiceGroup, setPicked } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { collapsible } from "../../lib/collapsible.ts";
import { descriptionList } from "../../lib/description.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { button } from "../../lib/button.ts";
import { hangs } from "../../lib/button-bar.ts";
import { keyHints } from "../../lib/keycap.ts";
import { fill, filling, row, stack } from "../../lib/layout.ts";
import { type LiveItem, type LiveList, liveList } from "../../lib/live-list.ts";
import { log, type LogLine } from "../../lib/log.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { RECHECK, type RailPage, type RailStatus } from "../../lib/step-rail.ts";
import { text, wrapped } from "../../lib/text.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import { body } from "./kobo-parts.ts";
import { CARDS } from "./setup-cards.ts";
import { type App, APPS, app, PROFILES, type ProfileId, proposed } from "./setup-data.ts";

/** Where the card is: choosing, its apps installing, or done with them. */
export type Phase = "choosing" | "installing" | "done";

/** What the person has said on the card, and how far its install is. */
export interface Yours {
  readonly picked: Set<ProfileId>;
  /** Proposed apps unticked: kept in the list, so they can be ticked back. */
  readonly dropped: Set<string>;
  /** Apps ticked from More apps, in the order they were. */
  readonly added: Set<string>;
  /** More apps shown at the list's end. */
  more: boolean;
  phase: Phase;
  readonly installed: Set<string>;
  /** The app that didn't install, with the cause. */
  failed?: { readonly app: string; readonly error: string; readonly hint: string };
}

/** Where a view of the card starts. */
export function yoursAt(view: string): Yours {
  const all = new Set<ProfileId>(["developer", "designer", "casual"]);
  const base = (picked: ProfileId[], more: Partial<Yours> = {}): Yours => ({
    picked: new Set(picked),
    dropped: new Set(),
    added: new Set(),
    more: false,
    phase: "choosing",
    installed: new Set(),
    ...more,
  });
  const everything = () => [...all];
  switch (view) {
    case "developer":
      return base(["developer"]);
    case "two":
      return base(["developer", "designer"]);
    case "all":
      return base(everything());
    case "dropped":
      return base(["developer", "casual"], { dropped: new Set(["docker", "steam"]), added: new Set(["kdenlive"]), more: true });
    case "installing": {
      const state = base(everything(), { phase: "installing" });
      for (const each of kept(state).slice(0, 7)) state.installed.add(each.id);
      return state;
    }
    case "done": {
      const state = base(everything(), { phase: "done" });
      for (const each of kept(state)) state.installed.add(each.id);
      return state;
    }
    case "failed": {
      const state = base(everything(), { phase: "done", failed: { app: "steam", error: "Steam didn't install: Valve's server timed out", hint: "The rest are in. Try it again, or untick it and carry on." } });
      for (const each of kept(state)) if (each.id !== "steam") state.installed.add(each.id);
      return state;
    }
    default:
      return base([]);
  }
}

/** The apps the list shows ticked or not: the proposal, then those added from More apps. */
function listed(state: Yours): readonly { readonly app: App; readonly group: string }[] {
  const shown: { app: App; group: string }[] = proposed(state.picked).map((each) => ({ app: each.app, group: each.by }));
  for (const id of state.added) if (!shown.some((each) => each.app.id === id)) shown.push({ app: app(id), group: "added" });
  return shown;
}

/** What gets installed: the listed apps still ticked. */
export function kept(state: Yours): readonly App[] {
  return listed(state)
    .filter((each) => !state.dropped.has(each.app.id))
    .map((each) => each.app);
}

/** The apps More apps offers: every app no picked profile proposes and nobody has added. */
function others(state: Yours): readonly App[] {
  const shown = new Set(listed(state).map((each) => each.app.id));
  return APPS.filter((each) => !shown.has(each.id));
}

/** A count of apps, its word chosen by the count. */
export function apps(count: number): string {
  return count === 1 ? "1 app" : `${count} apps`;
}

const GROUP_LABEL: Record<string, string> = { developer: "Developer", designer: "Designer", casual: "Everyday", added: "Added by you", more: "More apps" };

/** The list's rows by the phase: ticks while choosing, each app's place in the install after. */
function rows(state: Yours): LiveItem[] {
  const items: LiveItem[] = [];
  let group = "";
  const choosing = state.phase === "choosing";
  for (const each of listed(state)) {
    // Once the install has started, what was unticked isn't part of it.
    if (!choosing && state.dropped.has(each.app.id)) continue;
    if (each.group !== group) {
      group = each.group;
      items.push({ key: `group-${group}`, label: GROUP_LABEL[group]! });
    }
    items.push(appRow(state, each.app));
  }
  if (choosing && state.more) {
    const rest = others(state);
    if (rest.length > 0) items.push({ key: "group-more", label: GROUP_LABEL.more! });
    for (const each of rest) items.push({ key: each.id, title: each.name, subtitle: each.line, icon: each.icon, tile: true, checked: false });
  }
  return items;
}

/** One app's row: ticked while it is to be installed, then its chip as the install reaches it. */
function appRow(state: Yours, each: App): LiveItem {
  const base = { key: each.id, title: each.name, subtitle: each.line, icon: each.icon, tile: true };
  if (state.installed.has(each.id)) return { ...base, chips: [{ kind: "Accent", label: "Installed" }] };
  if (state.failed?.app === each.id) return { ...base, chips: [{ kind: "Warning", label: "Failed" }] };
  if (state.phase === "installing") {
    const next = kept(state).find((one) => !state.installed.has(one.id));
    return next?.id === each.id ? { ...base, chips: [{ kind: "Neutral", label: "Installing" }] } : base;
  }
  return { ...base, checked: !state.dropped.has(each.id) };
}

/** The card's state in a chip and a line, as every card's status says it. */
function statusWords(state: Yours): { readonly chip: HTMLElement; readonly words: string } {
  const count = kept(state).length;
  if (state.phase === "installing") return { chip: chip({ label: "Running", kind: "Neutral", icon: "progress_activity" }), words: `Installing: ${state.installed.size} of ${count} in. Carry on, it works in the background.` };
  if (state.failed) return { chip: chip({ label: "To do", kind: "Neutral" }), words: `${state.installed.size} of ${count} in; one didn't make it.` };
  if (state.phase === "done") return { chip: chip({ label: "Done", kind: "Accent", icon: "check" }), words: `${apps(count)} in. Find them with Super, then their name.` };
  if (state.picked.size === 0 && state.added.size === 0) return { chip: chip({ label: "To do", kind: "Neutral" }), words: "Nothing picked yet. Skip it and you get the basics." };
  return { chip: chip({ label: "To do", kind: "Neutral" }), words: `${apps(count)} to install. Untick any you don't want.` };
}

/** What the card offers as its `Filled` button now. */
function action(state: Yours): { readonly label: string; readonly icon?: string } | undefined {
  if (state.phase === "installing") return undefined;
  if (state.failed) return { label: `Try ${app(state.failed.app).name} again`, icon: "restart_alt" };
  const count = kept(state).filter((each) => !state.installed.has(each.id)).length;
  if (count === 0) return undefined;
  return { label: `Install ${apps(count)}`, icon: "download" };
}

/** The live card: its body, kept and changed in place, and what its rail page reads. */
export interface YoursCard {
  readonly state: Yours;
  readonly body: () => HTMLElement;
  readonly page: (act: () => boolean, skipped: () => boolean) => RailPage;
  /** Starts the install, as the `Filled` button does; returns whether it did anything. */
  readonly install: () => boolean;
  /** Called when the install has finished, with whether an app failed. */
  onFinished: (failed: boolean) => void;
}

/**
 * A part arriving: it rises `space.s24` into place as it fades in over `long` on
 * `emphasized_decel`; with reduced motion it only fades, over `short`.
 */
function rise(block: HTMLElement): void {
  const { reduced, speed } = settings();
  block.classList.add("kobo-part");
  block.animate([{ opacity: 0, "--kobo-rise": reduced ? 0 : 1 }, { opacity: 1, "--kobo-rise": 0 }], {
    duration: milliseconds(reduced ? "short" : "long") / speed,
    easing: `cubic-bezier(${bezier(reduced ? "standard" : "emphasized_decel").join(", ")})`,
  });
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

export function yoursCard(view: string, options: { readonly hood?: boolean } = {}): YoursCard {
  const state = yoursAt(view);
  const spec = CARDS.yours;
  let element: HTMLElement | null = null;
  let live: LiveList | null = null;
  let timer = 0;
  const parts = { status: null as HTMLElement | null, profiles: [] as HTMLElement[], progress: null as HTMLElement | null, failure: null as HTMLElement | null, hood: null as HTMLElement | null, more: null as HTMLElement | null, list: null as HTMLElement | null };

  const recheck = () => element?.dispatchEvent(new Event(RECHECK, { bubbles: true }));
  let drawn = false;
  // The list shows once something is picked, rising in as it arrives.
  let hiding = 0;
  const showList = (any: boolean) => {
    const list = parts.list;
    if (!list) return;
    window.clearTimeout(hiding);
    if (any) {
      const arriving = drawn && list.hidden === true;
      list.hidden = false;
      if (arriving) rise(list);
      return;
    }
    // Nothing picked: the rows leave as the list moves them, then the list goes with nothing in it.
    if (!drawn) list.hidden = true;
    else hiding = window.setTimeout(() => (list.hidden = rows(state).length === 0), milliseconds("medium") / settings().speed);
  };

  /** Every part that follows the state, brought to it; the list moves its own rows. */
  const refresh = () => {
    if (!element) return;
    const words = statusWords(state);
    parts.status?.replaceChildren(row("s12", "start", words.chip, filling(wrapped(words.words, "body", "on_surface_variant"))));
    for (const [index, profile] of PROFILES.entries()) {
      const cell = parts.profiles[index]!;
      setPicked(cell, state.picked.has(profile.id));
      if (state.phase === "installing") cell.setAttribute("aria-disabled", "true");
      else cell.removeAttribute("aria-disabled");
    }
    const items = rows(state);
    live?.update(items);
    showList(items.length > 0);
    if (parts.progress) {
      const total = kept(state).length;
      parts.progress.hidden = state.phase !== "installing";
      const bar = parts.progress.querySelector<HTMLElement>(".k-progress");
      if (bar) setProgress(bar, total === 0 ? 0 : state.installed.size / total);
      // A text is as wide as its words were when drawn: a new count is a new text.
      parts.progress.querySelector(".setup-yours-count")?.replaceChildren(text(`${state.installed.size} of ${total}`, "small", "on_surface_variant"));
    }
    if (parts.more) parts.more.style.visibility = state.phase === "choosing" ? "" : "hidden";
    drawn = true;
    if (parts.failure) {
      parts.failure.replaceChildren(...(state.failed ? [errorBlock({ message: `Fuck. ${state.failed.error}.`, hint: state.failed.hint })] : []));
      parts.failure.hidden = !state.failed;
    }
    parts.hood?.replaceWith((parts.hood = hood()));
    recheck();
  };

  const hood = (): HTMLElement => {
    const facts = [
      { label: "Picked", value: state.picked.size === 0 ? "Nothing" : PROFILES.filter((each) => state.picked.has(each.id)).map((each) => each.title).join(", ") },
      { label: "Installs", value: kept(state).length === 0 ? "Nothing" : kept(state).map((each) => each.id).join(", ") },
    ];
    const lines = state.phase === "choosing" ? null : installLog(state);
    const content = stack("s16", "start", filling(descriptionList(facts)), filling(stack("s8", "start", text("The same in a terminal", "label", "on_surface_variant"), wrapped(spec.command, "mono"))), ...(lines ? [filling(log(lines).box)] : []));
    const summary = lines ? `${lines.length} lines of log` : kept(state).length === 0 ? "Installs nothing yet" : `Installs ${apps(kept(state).length)}`;
    return hangs(collapsible({ title: "Under the hood", summary, content, open: (options.hood ?? false) || state.failed !== undefined }));
  };

  const pickProfiles = (): HTMLElement => {
    parts.profiles = PROFILES.map((profile) => choiceCard({ glyph: profile.icon, title: profile.title, subtitle: profile.line, size: "Lg", picks: "many", picked: state.picked.has(profile.id), disabled: state.phase === "installing" }));
    const group = choiceGroup("many", 3, parts.profiles);
    group.classList.add("setup-yours-profiles");
    const sync = () =>
      queueMicrotask(() => {
        if (state.phase === "installing") return;
        let changed = false;
        for (const [index, profile] of PROFILES.entries()) {
          const on = parts.profiles[index]!.getAttribute("aria-checked") === "true";
          if (on === state.picked.has(profile.id)) continue;
          changed = true;
          if (on) state.picked.add(profile.id);
          else state.picked.delete(profile.id);
        }
        if (!changed) return;
        // Picking more after an install asks for the new apps only.
        if (state.phase === "done" && !state.failed) state.phase = "choosing";
        refresh();
      });
    group.addEventListener("click", sync);
    group.addEventListener("keydown", sync);
    return group;
  };

  const appsColumn = (): HTMLElement => {
    const more = button({ label: "More apps", kind: "Text", size: "Sm", disclosure: state.more });
    more.addEventListener("click", () => {
      state.more = !state.more;
      more.style.setProperty("--disclosed", state.more ? "1" : "0");
      refresh();
      // The apps arrive at the list's end: the page scrolls to them, so what the press did shows.
      const label = element?.querySelector<HTMLElement>('.setup-yours-apps [data-key="group-more"]');
      if (state.more && label) label.scrollIntoView({ block: "start", behavior: settings().reduced ? "auto" : "smooth" });
    });
    parts.more = hangs(more);
    const head = row("s8", "centre", text("Apps", "subheading"), fill(), parts.more);
    head.classList.add("setup-yours-head");
    // The list is as tall as its apps: the page is the one thing that scrolls.
    live = liveList(rows(state), {
      onTake: (index) => {
        const taken = rows(state).filter((each) => !("label" in each))[index];
        if (!taken || state.phase === "installing") return;
        const id = taken.key;
        const proposedNow = listed(state).some((each) => each.app.id === id && each.group !== "added");
        if (proposedNow) {
          if (state.dropped.has(id)) state.dropped.delete(id);
          else state.dropped.add(id);
        } else if (state.added.has(id)) state.added.delete(id);
        else state.added.add(id);
        if (state.phase === "done") state.phase = "choosing";
        refresh();
      },
    });
    const count = document.createElement("span");
    count.className = "setup-yours-count";
    parts.progress = stack("s4", "start", filling(progressBar({ fraction: 0 })), filling(row("s8", "centre", text("Installing", "small", "on_surface_variant"), fill(), count)));
    parts.progress.classList.add("setup-yours-progress");
    const column = stack("s12", "start", filling(head), filling(parts.progress), filling(live.element));
    column.classList.add("setup-yours-apps");
    return column;
  };

  const draw = (): HTMLElement => {
    if (element) return element;
    parts.status = document.createElement("span");
    parts.status.className = "setup-status";
    parts.failure = document.createElement("span");
    parts.failure.className = "setup-yours-failure";
    parts.list = appsColumn();
    parts.list.classList.add("setup-yours-list", "kobo-part");
    parts.hood = hood();
    const later = row("s12", "centre", wrapped(spec.skip, "small", "on_surface_variant"), keyHints([["Super+Space", `then “${spec.later}”`]]));
    later.classList.add("setup-later");
    const content = stack("s20", "start", ...[parts.status, pickProfiles(), parts.failure, parts.list, later, parts.hood].map((each) => filling(each)));
    element = body(spec.accent, spec.title, spec.why, content);
    element.classList.add("setup-yours");
    refresh();
    return element;
  };

  const tick = () => {
    const next = kept(state).find((each) => !state.installed.has(each.id) && each.id !== state.failed?.app);
    if (!next) {
      state.phase = "done";
      refresh();
      handle.onFinished(state.failed !== undefined);
      return;
    }
    state.installed.add(next.id);
    refresh();
    timer = window.setTimeout(tick, milliseconds("medium") / settings().speed);
  };

  const install = (): boolean => {
    if (state.phase === "installing") return false;
    const count = kept(state).filter((each) => !state.installed.has(each.id)).length;
    if (count === 0) return false;
    delete state.failed;
    state.phase = "installing";
    refresh();
    window.clearTimeout(timer);
    timer = window.setTimeout(tick, milliseconds("medium") / settings().speed);
    return true;
  };

  const status = (): RailStatus => {
    if (state.phase === "installing") return { state: { is: "todo" }, running: true };
    if (state.failed) return { state: { is: "todo" }, failed: true };
    if (state.phase === "done") return { state: { is: "done" } };
    return { state: { is: "todo" } };
  };

  const handle: YoursCard = {
    state,
    body: draw,
    install,
    onFinished: () => undefined,
    page: (act, skipped) => ({
      step: { title: spec.title, icon: spec.icon },
      body: draw,
      status,
      next: () => action(state)?.label ?? "Continue",
      nextIcon: () => action(state)?.icon,
      skip: () => (action(state) ? (skipped() ? "Later" : "Skip") : undefined),
      note: () => (state.phase === "installing" ? "Installing in the background: carry on." : undefined),
      act,
    }),
  };
  return handle;
}

/** The apps a view installs, for the words around it: the arrival's and the ending's. */
export function installing(view: string): number {
  return kept(yoursAt(view)).length;
}

/** The empty card's profile cards alone, as a still: what a newcomer meets first. */
export function profilesStill(picked: readonly ProfileId[]): HTMLElement {
  return choiceGroup("many", 3, PROFILES.map((profile) => choiceCard({ glyph: profile.icon, title: profile.title, subtitle: profile.line, picks: "many", picked: picked.includes(profile.id) })));
}

