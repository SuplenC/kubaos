// First boot's Make it yours card, as an organism of library parts: what the person uses the machine
// for (any of three profiles, none included), the apps those bring as a live list grouped by the
// profile that brought each, and beside it the settings that come with them: Git's name and email
// with Developer, the Bitwarden login with Bitwarden. It is drawn once and kept: picking a profile,
// dropping or adding an app and the install moving on change it in place, so the list's rows
// arrive, leave and glide as LiveList moves them.

import { choiceCard, choiceGroup, setPicked } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { collapsible } from "../../lib/collapsible.ts";
import { descriptionList } from "../../lib/description.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { field, labelled } from "../../lib/field.ts";
import { button } from "../../lib/button.ts";
import { hangs } from "../../lib/button-bar.ts";
import { keyHints } from "../../lib/keycap.ts";
import { fill, filling, grid, row, splitColumns, stack } from "../../lib/layout.ts";
import { type LiveItem, type LiveList, liveList } from "../../lib/live-list.ts";
import { log, type LogLine } from "../../lib/log.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { select } from "../../lib/select.ts";
import { RECHECK, type RailPage, type RailStatus } from "../../lib/step-rail.ts";
import { text, wrapped } from "../../lib/text.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import { body, looksLikeEmail, showToggle } from "./kobo-parts.ts";
import { CARDS, USER } from "./setup-cards.ts";
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
  gitName: string;
  gitEmail: string;
  readonly bitwarden: { server: number; email: string; password: string; code: string };
  phase: Phase;
  readonly installed: Set<string>;
  /** The app that didn't install, with the cause. */
  failed?: { readonly app: string; readonly error: string; readonly hint: string };
}

const FILLED_BITWARDEN = { server: 1, email: USER.email, password: "correct horse battery staple", code: "482 913" };
const EMPTY_BITWARDEN = { server: 0, email: "", password: "", code: "" };

/** Where a view of the card starts. */
export function yoursAt(view: string): Yours {
  const all = new Set<ProfileId>(["developer", "designer", "casual"]);
  const base = (picked: ProfileId[], more: Partial<Yours> = {}): Yours => ({
    picked: new Set(picked),
    dropped: new Set(),
    added: new Set(),
    more: false,
    gitName: USER.name,
    gitEmail: "",
    bitwarden: { ...EMPTY_BITWARDEN },
    phase: "choosing",
    installed: new Set(),
    ...more,
  });
  const everything = () => [...all];
  switch (view) {
    case "developer":
      return base(["developer"]);
    case "two":
      return base(["developer", "designer"], { gitEmail: USER.email });
    case "filled":
      return base(["developer"], { gitEmail: USER.email, bitwarden: { ...FILLED_BITWARDEN } });
    case "invalid":
      return base(["developer"], { gitEmail: "mika@nowak" });
    case "all":
      return base(everything(), { gitEmail: USER.email, bitwarden: { ...FILLED_BITWARDEN } });
    case "dropped":
      return base(["developer", "casual"], { dropped: new Set(["docker", "steam"]), added: new Set(["kdenlive"]), more: true, gitEmail: USER.email });
    case "installing": {
      const state = base(everything(), { gitEmail: USER.email, bitwarden: { ...FILLED_BITWARDEN }, phase: "installing" });
      for (const each of kept(state).slice(0, 7)) state.installed.add(each.id);
      return state;
    }
    case "done": {
      const state = base(everything(), { gitEmail: USER.email, bitwarden: { ...FILLED_BITWARDEN }, phase: "done" });
      for (const each of kept(state)) state.installed.add(each.id);
      return state;
    }
    case "failed": {
      const state = base(everything(), { gitEmail: USER.email, bitwarden: { ...FILLED_BITWARDEN }, phase: "done", failed: { app: "steam", error: "Steam didn't install: Valve's server timed out", hint: "The rest are in. Try it again, or untick it and carry on." } });
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

/** The Git email typed and not an email: what keeps the install waiting. */
function gitProblem(state: Yours): string | undefined {
  if (!state.picked.has("developer") || state.gitEmail === "" || looksLikeEmail(state.gitEmail)) return undefined;
  return "The Git email doesn't look like one.";
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

/** A setting's heading: its title, and what it is for under it. */
function heading(title: string, line: string): HTMLElement {
  return stack("s4", "start", text(title, "subheading"), filling(wrapped(line, "small", "on_surface_variant")));
}

/** A field that tells `set` what is typed. */
function typed(options: Parameters<typeof field>[0], set: (value: string) => void): HTMLElement {
  return field({ ...options, onInput: set });
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
 * A setting arriving: it rises `space.s24` into place as it fades in over `long` on
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
  const parts = { status: null as HTMLElement | null, profiles: [] as HTMLElement[], progress: null as HTMLElement | null, settings: null as HTMLElement | null, failure: null as HTMLElement | null, hood: null as HTMLElement | null, more: null as HTMLElement | null, well: null as HTMLElement | null };

  const recheck = () => element?.dispatchEvent(new Event(RECHECK, { bubbles: true }));
  // Which settings show, so only one that arrives moves.
  const shown = { git: false, bitwarden: false };
  let drawn = false;

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
    parts.well?.toggleAttribute("data-empty", items.length === 0);
    if (parts.progress) {
      const total = kept(state).length;
      parts.progress.hidden = state.phase !== "installing";
      const bar = parts.progress.querySelector<HTMLElement>(".k-progress");
      if (bar) setProgress(bar, total === 0 ? 0 : state.installed.size / total);
      parts.progress.querySelector(".setup-yours-count")?.replaceChildren(`${state.installed.size} of ${total}`);
    }
    if (parts.more) parts.more.style.visibility = state.phase === "choosing" ? "" : "hidden";
    const wanted = { git: state.picked.has("developer"), bitwarden: kept(state).some((each) => each.id === "bitwarden") };
    if (parts.settings && (!drawn || wanted.git !== shown.git || wanted.bitwarden !== shown.bitwarden)) {
      const blocks = settingsColumn();
      parts.settings.replaceChildren(...blocks);
      // A setting that comes with what was just picked rises in where it lands; one that goes
      // simply goes, and what is under it closes up.
      if (drawn)
        for (const block of blocks) {
          const fresh = (block.classList.contains("setup-yours-git") && !shown.git) || (block.classList.contains("setup-yours-bitwarden") && !shown.bitwarden);
          if (fresh) rise(block);
        }
      shown.git = wanted.git;
      shown.bitwarden = wanted.bitwarden;
    }
    drawn = true;
    if (parts.failure) {
      parts.failure.replaceChildren(...(state.failed ? [errorBlock({ message: `Fuck. ${state.failed.error}.`, hint: state.failed.hint })] : []));
      parts.failure.hidden = !state.failed;
    }
    parts.hood?.replaceWith((parts.hood = hood()));
    recheck();
  };

  /** Settings arrive with what needs them: Git with Developer, the login with Bitwarden ticked. */
  const settingsColumn = (): HTMLElement[] => {
    const blocks: HTMLElement[] = [];
    if (state.picked.has("developer")) {
      const invalid = gitProblem(state) !== undefined;
      const name = labelled("Your name", typed({ value: state.gitName, icon: "person" }, (value) => (state.gitName = value)));
      const email = labelled(
        "Email",
        typed({ value: state.gitEmail, placeholder: "you@example.com", icon: "mail", ...(invalid ? { state: "Invalid" as const } : {}) }, (value) => {
          state.gitEmail = value;
          recheck();
        }),
        { note: invalid ? "That doesn't look like an email." : "Public on every commit. Empty: set it later.", ...(invalid ? { invalid: true } : {}) },
      );
      const block = stack("s16", "start", filling(heading("Git", "The name and email on every commit you make.")), filling(name), filling(email));
      block.classList.add("setup-yours-git");
      blocks.push(filling(block));
    }
    if (kept(state).some((each) => each.id === "bitwarden")) {
      const server = labelled("Server", select({ options: ["bitwarden.com", "bitwarden.eu", "Self-hosted"], picked: state.bitwarden.server }));
      const email = labelled("Email", typed({ value: state.bitwarden.email, placeholder: "you@example.com", icon: "mail" }, (value) => (state.bitwarden.email = value)));
      const password = labelled("Master password", typed({ value: state.bitwarden.password, secret: true, icon: "key" }, (value) => (state.bitwarden.password = value)), { action: showToggle(() => [password]), note: "In memory, never on disk." });
      const code = labelled("Two-step code", typed({ value: state.bitwarden.code, placeholder: "123 456" }, (value) => (state.bitwarden.code = value)));
      const block = stack("s16", "start", filling(heading("Bitwarden", "Log in once: passwords, SSH keys and commit signing come from the vault. Empty: log in later, in the app.")), filling(email), filling(password), filling(grid(2, "s16", [server, code])));
      block.classList.add("setup-yours-bitwarden");
      blocks.push(filling(block));
    }
    if (blocks.length === 0) {
      const none = wrapped("Git and Bitwarden come with what you pick.", "small", "on_surface_variant");
      none.classList.add("setup-yours-none");
      blocks.push(filling(none));
    }
    return blocks;
  };

  const hood = (): HTMLElement => {
    const facts = [
      { label: "Picked", value: state.picked.size === 0 ? "Nothing" : PROFILES.filter((each) => state.picked.has(each.id)).map((each) => each.title).join(", ") },
      { label: "Installs", value: kept(state).length === 0 ? "Nothing" : kept(state).map((each) => each.id).join(", ") },
      { label: "Git writes", value: "~/.config/git-identity/identity.gitconfig" },
    ];
    const lines = state.phase === "choosing" ? null : installLog(state);
    const content = stack("s16", "start", filling(descriptionList(facts)), filling(stack("s8", "start", text("The same in a terminal", "label", "on_surface_variant"), wrapped(spec.command, "mono"))), ...(lines ? [filling(log(lines).box)] : []));
    const summary = lines ? `${lines.length} lines of log` : `Installs: ${apps(kept(state).length)}`;
    return hangs(collapsible({ title: "Under the hood", summary, content, open: (options.hood ?? false) || state.failed !== undefined }));
  };

  const pickProfiles = (): HTMLElement => {
    parts.profiles = PROFILES.map((profile) => choiceCard({ glyph: profile.icon, title: profile.title, subtitle: profile.line, picks: "many", picked: state.picked.has(profile.id), disabled: state.phase === "installing" }));
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
      // The apps arrive at the list's end: it scrolls to them, so what the press did shows.
      const pane = element?.querySelector<HTMLElement>(".setup-yours-apps .k-list-pane");
      const label = pane?.querySelector<HTMLElement>('[data-key="group-more"]');
      if (state.more && pane && label) pane.scrollTo({ top: label.offsetTop, behavior: settings().reduced ? "auto" : "smooth" });
    });
    parts.more = hangs(more);
    const head = row("s8", "centre", text("Apps", "subheading"), fill(), parts.more);
    head.classList.add("setup-yours-head");
    live = liveList(rows(state), {
      height: "100%",
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
    const empty = document.createElement("span");
    empty.className = "setup-yours-empty";
    empty.append(wrapped("Pick what you use it for, and its apps show up here.", "body", "on_surface_variant"));
    const well = document.createElement("span");
    well.className = "setup-yours-well";
    well.append(live.element, empty);
    parts.well = well;
    parts.progress = stack("s4", "start", filling(progressBar({ fraction: 0 })), filling(row("s8", "centre", text("Installing", "small", "on_surface_variant"), fill(), text("", "small", "on_surface_variant"))));
    parts.progress.querySelector(".k-row > :last-child")?.classList.add("setup-yours-count");
    parts.progress.classList.add("setup-yours-progress");
    const column = stack("s12", "start", filling(head), filling(parts.progress), filling(well));
    column.classList.add("setup-yours-apps");
    return column;
  };

  const draw = (): HTMLElement => {
    if (element) return element;
    parts.status = document.createElement("span");
    parts.status.className = "setup-status";
    parts.failure = document.createElement("span");
    parts.failure.className = "setup-yours-failure";
    parts.settings = stack("s32", "start");
    parts.settings.classList.add("setup-yours-settings");
    const columns = splitColumns([1, 1], "s24", [filling(appsColumn()), filling(parts.settings)]);
    parts.hood = hood();
    const later = row("s12", "centre", wrapped(spec.skip, "small", "on_surface_variant"), keyHints([["Super+Space", `then “${spec.later}”`]]));
    later.classList.add("setup-later");
    const content = stack("s20", "start", ...[parts.status, pickProfiles(), parts.failure, columns, later, parts.hood].map((each) => filling(each)));
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
    if (count === 0 || gitProblem(state)) return false;
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
      blocker: () => (state.phase === "choosing" ? gitProblem(state) : undefined),
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

