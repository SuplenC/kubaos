// The Updates page's organisms (About › Updates), kept in the Settings app because nothing else draws
// them: the update's card in its five states, what an update holds, the steps it runs, its history,
// and the questions a disruptive change asks over the window (the library's `dialogOver`). Each is
// assembled from library parts; the steps are a well of `Step`s as Kōbō's install lists its own, and
// a failure is an `ErrorBlock` with its `Log`, as Kōbō shows one.

import "./updates.css";
import { button } from "../../../../lib/button.ts";
import { buttonBar, hangs } from "../../../../lib/button-bar.ts";
import { card } from "../../../../lib/card.ts";
import { descriptionList, type Fact } from "../../../../lib/description.ts";
import type { DialogOptions } from "../../../../lib/dialog.ts";
import { errorBlock } from "../../../../lib/error-block.ts";
import { filling, grid, row, stack } from "../../../../lib/layout.ts";
import { list } from "../../../../lib/list.ts";
import { log, type LogLine, logRoom } from "../../../../lib/log.ts";
import { progressBar, setProgress } from "../../../../lib/progress.ts";
import { step, type StepOptions } from "../../../../lib/step.ts";
import { clamped } from "../../../../lib/text.ts";
import { settingsWell } from "../../../../lib/sidebar-page.ts";
import { settings } from "../../../../site/settings.ts";
import { RELEASE } from "../../../kobo/kobo-data.ts";

/** Where an update stands, as the last check and the last run left it. */
export type UpdateState = "current" | "checking" | "waiting" | "updating" | "failed";

/** The values the parts read and change: the section's own (`apps.ts`). */
export interface UpdateValues {
  channel: number;
  firmware: boolean;
  bitwarden: boolean;
  nix: boolean;
  update: UpdateState;
  /** The step running, or the one that failed, by its index in `STEPS`. */
  at: number;
  /** The update's output shown under its card. */
  logOpen: boolean;
  /** The release installed, by its version. */
  version: string;
}

/** The release the daily check found: the next of the installed series. */
export const NEXT = "0.4.1";

/** The channels, as `kubaos channel` names them. */
export const CHANNELS = ["Stable", "Edge"] as const;

/** One step of `kubaos update`, as `docs/USING.md` lists them, and how long it took on the sample laptop. */
interface UpdateStep {
  readonly title: string;
  /** What it does, plainly; the second picks the release by channel. */
  readonly line: string | readonly [string, string];
  readonly seconds: number;
  /** The switch in What updates that leaves it out. */
  readonly off?: (values: UpdateValues) => boolean;
  /** It runs only when the release asks for it, and this one doesn't. */
  readonly needless?: true;
}

export const STEPS: readonly UpdateStep[] = [
  { title: "Checks it can", line: "Another update running, under 3 GiB free or no GitHub stops it here.", seconds: 2 },
  { title: "Picks the release", line: ["The newest signed release of Shinjuku.", "main's newest commit, checked against KubaOS's keys."], seconds: 3 },
  { title: "Shows what it brings", line: "What you read on this page. Update is your yes.", seconds: 0 },
  { title: "Takes a snapshot", line: "Of the system, and saves your Nix and Suite locks, so it can be undone.", seconds: 4 },
  { title: "Upgrades Ubuntu's packages", line: "apt's upgrade, then KubaOS's own files move to the release.", seconds: 48 },
  { title: "Moves Hyprland", line: "Only when the release pins a new version; held otherwise.", seconds: 0, needless: true },
  { title: "Brings the package list in line", line: "Adds what the release lists; removes only what KubaOS put there.", seconds: 6 },
  { title: "Runs the system's migrations", line: "One-off fixes the release needs, each once, as root.", seconds: 3 },
  { title: "Updates your Nix packages", line: "Your flake moves, your home switches, your Suites take their patches.", seconds: 41 },
  { title: "Moves Bitwarden and Nix", line: "Within their series; a new series waits for a release.", seconds: 9, off: (values) => !values.bitwarden && !values.nix },
  { title: "Runs your migrations", line: "One-off fixes to your own files, each once, as you.", seconds: 1 },
  { title: "Updates firmware", line: "What needs no reboot installs; what does waits for your yes.", seconds: 7, off: (values) => !values.firmware },
];

/** The step a failed sample stops at: the system's migrations. */
export const FAILS_AT = 7;

/** What the program said in each step, as the run's log keeps it. */
const SAID: readonly (readonly LogLine[])[] = [
  [{ level: "step", text: "==> Checking" }, { level: "info", text: "41 GiB free on /, 212 GiB on /nix; origin reachable" }],
  [{ level: "step", text: "==> Fetching" }, { level: "info", text: "v0.4.1 at 9f8e7d6, verified against allowed_signers" }],
  [{ level: "step", text: "==> Reviewing" }, { level: "info", text: "Confirmed in Settings" }],
  [{ level: "step", text: "==> Snapshot" }, { level: "info", text: "snapper: #214 before kubaos update" }],
  [{ level: "step", text: "==> System packages" }, { level: "debug", text: "apt-get upgrade -y" }, { level: "info", text: "Unpacking firefox (146.0.1) over (146.0)" }],
  [{ level: "step", text: "==> Hyprland" }, { level: "info", text: "Pin unchanged: skipped" }],
  [{ level: "step", text: "==> Manifest" }, { level: "info", text: "files: installed loupe, showtime" }],
  [{ level: "step", text: "==> System migrations" }, { level: "info", text: "Running 1792765570-backlight-rule.sh" }],
  [{ level: "step", text: "==> Home" }, { level: "debug", text: "nix flake update --flake ~/.config/kubaos/home" }, { level: "info", text: "Building the home: 14 of 31" }],
  [{ level: "step", text: "==> Bitwarden" }, { level: "info", text: "2026.12.0, the newest of its series" }],
  [{ level: "step", text: "==> User migrations" }, { level: "info", text: "None new" }],
  [{ level: "step", text: "==> Firmware" }, { level: "info", text: "Dock 1.0 → 1.1: applied" }],
];

/** How the sample's failure ends its log. */
const FAILURE: readonly LogLine[] = [
  { level: "debug", text: "install -m 644 90-backlight.rules /etc/udev/rules.d/" },
  { level: "error", text: "install: cannot create regular file '/etc/udev/rules.d/90-backlight.rules': Read-only file system" },
  { level: "error", text: "system_migrations: 1792765570-backlight-rule.sh exited 1" },
  { level: "info", text: "Log: ~/.local/state/kubaos/update/2026-11-17-update.log" },
];

/** The changes the next release brings, as its preview lists them: kind, subject, scope and pull request. */
const CHANGES: readonly Fact[] = [
  { label: "New", value: "Each screen keeps its workspaces as a group", detail: "hypr · #412" },
  { label: "New", value: "The lock's clock flies to the bar as you unlock", detail: "lock · #418" },
  { label: "Fixed", value: "The bar's update dot clears after a rollback", detail: "bar · #421" },
  { label: "Fixed", value: "The phone reconnects after the laptop wakes", detail: "phone · #424" },
  { label: "Faster", value: "The Command menu opens a frame sooner", detail: "kotodama · #427" },
];

/** The release as a person reads it: Shinjuku 0.4.0. */
export function releaseName(version: string): string {
  return `${RELEASE.codename} ${version}`;
}

/** The mockup plays an update twenty times faster than the sample laptop ran it: content, not motion. */
const FAST_FORWARD = 20;

/** The log up to step `at`, ending in the failure when it failed there. */
function linesTo(at: number, failed: boolean): LogLine[] {
  const lines = SAID.slice(0, at + 1).flat();
  return failed ? [...lines, ...FAILURE] : lines;
}

/** Step `index`'s line, its channel's when it picks the release. */
function lineOf(each: UpdateStep, values: UpdateValues): string {
  return typeof each.line === "string" ? each.line : each.line[values.channel]!;
}

/** Each step as `values.update` leaves it: done, running, failed, to do, or off by a switch. */
function stepFor(each: UpdateStep, index: number, values: UpdateValues): StepOptions {
  const base = { title: each.title, subtitle: lineOf(each, values) };
  const off = each.off?.(values) ?? false;
  const ran = values.update === "current" || values.update === "checking" || index < values.at;
  if (values.update === "updating" && index === values.at) return { ...base, state: { is: "current" }, running: true };
  if (values.update === "failed" && index === values.at) return { ...base, state: { is: "current" }, failed: true };
  if (off) return { ...base, state: { is: "todo" }, words: { label: "Off" } };
  if (each.needless) return { ...base, state: { is: "todo" }, words: { label: ran ? "Skipped" : "Not needed" } };
  return { ...base, state: { is: ran && values.update !== "waiting" ? "done" : "todo" } };
}

/**
 * The steps an update runs, each where `values` leaves it, as Kōbō lists its install's:
 * `Step`s in two wells side by side, so all twelve show at once.
 */
export function stepsWell(values: UpdateValues): HTMLElement {
  const rows = STEPS.map((each, index) => step(stepFor(each, index, values)));
  const half = Math.ceil(rows.length / 2);
  const made = grid(2, "s16", [list(rows.slice(0, half)), list(rows.slice(half))]);
  made.classList.add("up-steps");
  return made;
}

/** The changes the next release brings, as a card of facts, its pull requests under each. */
export function changesCard(): HTMLElement {
  return card({
    header: { glyph: "upgrade", title: `What's new in ${NEXT}`, subtitle: "5 changes, and 7 internal ones" },
    body: [descriptionList(CHANGES)],
  });
}

/** What the next release changes, by the part of the machine it touches, each with its download. */
const HOLDS: readonly Fact[] = [
  { label: "System", value: "12 Ubuntu packages, 2 of them security fixes", detail: "84 MB" },
  { label: "Apps", value: "Bitwarden 2026.12.0, and the dock's firmware 1.1", detail: "212 MB" },
  { label: "The shell", value: `KubaOS ${NEXT}: 2 new, 2 fixed, 1 faster`, detail: "6 MB" },
];

/** What an update holds, before it runs: the system, the apps and the shell, each with its size. */
export function holdsCard(): HTMLElement {
  return card({
    header: { glyph: "package", title: `What ${releaseName(NEXT)} holds`, subtitle: "302 MB to download; nothing installs until you say" },
    body: [descriptionList(HOLDS)],
  });
}

/** One update this machine ran: when, to which release, what it changed, and how it ended. */
export interface PastUpdate {
  readonly date: string;
  readonly version: string;
  readonly changed: string;
  readonly ok: boolean;
}

/** The updates this machine ran, the newest first. */
export const HISTORY: readonly PastUpdate[] = [
  { date: "16 November", version: "0.4.0", changed: "8 changes, 31 packages", ok: true },
  { date: "9 November", version: "0.3.2", changed: "Stopped at the system's migrations; rolled back", ok: false },
  { date: "9 November", version: "0.3.2", changed: "3 fixes, 12 packages", ok: true },
  { date: "28 October", version: "0.3.1", changed: "2 fixes, 6 packages", ok: true },
  { date: "14 October", version: "0.3.0", changed: "11 changes, 48 packages", ok: true },
];

/** The history as a well of rows: each update's release, its date and what it changed, done or failed by its symbol. */
export function historyWell(history: readonly PastUpdate[] = HISTORY): HTMLElement {
  return settingsWell(
    history.map((each) => ({
      title: releaseName(each.version),
      subtitle: `${each.date}: ${each.changed}`,
      icon: each.ok ? "check_circle" : "error",
      tile: true,
      detail: each.ok ? "Done" : "Failed",
    })),
  );
}

/** What the card's buttons do: the update run, checked for, retried or rolled back. */
export interface UpdateActions {
  readonly check: () => void;
  readonly update: () => void;
  readonly rollBack: () => void;
}

/** The card's button bar: small, as every control on a Settings page is. */
function bar(label: string, icon: string, act: () => void, kind?: "Tonal"): HTMLElement {
  const made = buttonBar({ actions: [{ label, icon, ...(kind ? { kind } : {}) }], size: "Sm" });
  made.actions[0]!.addEventListener("click", act);
  return made.element;
}

/** Up to date: the release, its channel and the last update, and a way to look again. */
function current(values: UpdateValues, act: UpdateActions): HTMLElement {
  return card({
    header: { glyph: "check_circle", title: "Up to date", subtitle: "Checked an hour ago" },
    body: [
      descriptionList([
        { label: "Release", value: releaseName(values.version), detail: RELEASE.commit },
        { label: "Channel", value: CHANNELS[values.channel]! },
        { label: "Last update", value: "2 days ago, every step done" },
      ]),
    ],
    actions: bar("Check now", "manage_search", act.check, "Tonal"),
  });
}

/** Checking: the release installed, while the newest is asked for; nothing to press meanwhile. */
function checking(values: UpdateValues): HTMLElement {
  return card({
    header: { glyph: "progress_activity", title: "Checking for updates", subtitle: "Asking for the newest release" },
    body: [
      descriptionList([
        { label: "Release", value: releaseName(values.version), detail: RELEASE.commit },
        { label: "Channel", value: CHANNELS[values.channel]! },
      ]),
    ],
    actions: buttonBar({ actions: [{ label: "Check now", icon: "manage_search", kind: "Tonal", disabled: true }], size: "Sm" }).element,
  });
}

/** Waiting: the next release, what it brings in a line each, and Update. */
function waiting(values: UpdateValues, act: UpdateActions): HTMLElement {
  return card({
    header: { glyph: "system_update_alt", title: `${releaseName(NEXT)} is ready`, subtitle: "Found an hour ago" },
    body: [
      descriptionList([
        { label: "Changes", value: "2 new, 2 fixed, 1 faster" },
        { label: "Packages", value: "12 from Ubuntu, 2 of them security fixes" },
        { label: "Channel", value: CHANNELS[values.channel]! },
      ]),
    ],
    actions: bar("Update", "system_update_alt", act.update),
  });
}

/** The output's toggle and its room under the card, open as `values` says; the toggle opens and closes it. */
function output(values: UpdateValues, failed: boolean, ...actions: HTMLElement[]): { foot: HTMLElement; room: HTMLElement; log: ReturnType<typeof log> } {
  const shown = log(linesTo(values.at, failed));
  const room = logRoom(shown.box, values.logOpen, "s16");
  const labels = failed ? (["Show log", "Hide log"] as const) : (["Show output", "Hide output"] as const);
  const toggle = button({ label: labels[values.logOpen ? 1 : 0], kind: "Text", size: "Sm", icon: "terminal", alternatives: labels });
  toggle.addEventListener("click", () => {
    values.logOpen = !values.logOpen;
    room.style.setProperty("--log-open", values.logOpen ? "1" : "0");
    for (const label of toggle.querySelectorAll<HTMLElement>(".k-text")) label.textContent = labels[values.logOpen ? 1 : 0];
  });
  // A Text button at the row's start hangs its padding outside it, so its label sits on the edge.
  const foot = hangs(row("s8", "centre", toggle, filling(document.createElement("span")), ...actions));
  foot.classList.add("up-foot");
  return { foot, room, log: shown };
}

/** Updating: the release it moves to, the step running, how far it is, and its output a press away. */
function updating(values: UpdateValues): HTMLElement {
  const now = STEPS[values.at]!;
  const bar = progressBar({ fraction: values.at / STEPS.length });
  const said = clamped(SAID[values.at]!.at(-1)!.text, "mono", "on_surface_variant");
  said.classList.add("up-said");
  const { foot, room } = output(values, false);
  const made = card({
    header: { glyph: "progress_activity", title: `Updating to ${releaseName(NEXT)}`, subtitle: `Step ${values.at + 1} of ${STEPS.length}: ${now.title}` },
    body: [bar, said, foot],
  });
  return stack(0, "start", filling(made), filling(room));
}

/** Failed: what stopped it in one sentence, the step and the next thing to try, its log, and the way back. */
function failed(values: UpdateValues, act: UpdateActions): HTMLElement {
  const block = errorBlock({
    message: "A system migration failed, so the update stopped.",
    hint: `Step ${values.at + 1} of ${STEPS.length}; nothing after it ran. Roll back to ${releaseName(values.version)}, or read the log and try again.`,
  });
  const back = button({ label: "Roll back", kind: "Tonal", size: "Sm", icon: "history" });
  back.addEventListener("click", act.rollBack);
  const again = button({ label: "Try again", size: "Sm", icon: "restart_alt" });
  again.addEventListener("click", act.update);
  const { foot, room } = output(values, true, back, again);
  return stack(0, "start", filling(stack("s16", "start", filling(block), filling(foot))), filling(room));
}

/** The update's card, before the Updates sub-section's rows, in the state `values` holds. */
export function updateCard(values: UpdateValues, act: UpdateActions): HTMLElement {
  const made = { current, checking: (shown: UpdateValues) => checking(shown), waiting, failed, updating: (shown: UpdateValues) => updating(shown) }[values.update](values, act);
  made.classList.add("up-card");
  made.dataset.update = values.update;
  return made;
}

/**
 * Plays the update from its first step to its end in `frame`'s window, a step at a time: the card's
 * bar, its line and its log, and the steps if their page is open, follow in place; once done, `done`.
 */
export function play(values: UpdateValues, frame: () => HTMLElement | null, done: () => void): void {
  const next = () => {
    const window = frame();
    if (!window?.isConnected || values.update !== "updating") return;
    if (values.at >= STEPS.length - 1) {
      values.update = "current";
      values.version = NEXT;
      values.logOpen = false;
      done();
      return;
    }
    values.at++;
    const shown = window.querySelector<HTMLElement>(".up-card[data-update='updating']");
    if (shown) {
      setProgress(shown.querySelector<HTMLElement>(".k-progress")!, values.at / STEPS.length);
      const subtitle = shown.querySelector<HTMLElement>(".k-card-subtitle");
      if (subtitle) subtitle.textContent = `Step ${values.at + 1} of ${STEPS.length}: ${STEPS[values.at]!.title}`;
      const said = shown.querySelector<HTMLElement>(".up-said");
      if (said) said.textContent = SAID[values.at]!.at(-1)!.text;
      const lines = shown.querySelector<HTMLElement>(".k-log-lines");
      lines?.closest(".k-log-room")?.replaceWith(logRoom(log(linesTo(values.at, false)).box, values.logOpen, "s16"));
    }
    window.querySelector(".up-steps")?.replaceWith(stepsWell(values));
    later();
  };
  const later = () => setTimeout(next, Math.max(STEPS[values.at]!.seconds, 1) * (1000 / FAST_FORWARD) / settings().speed);
  later();
}

/** The question before following `channel`: what it means, and that nothing moves until the next update. */
export function channelQuestion(channel: number, version: string): DialogOptions {
  return channel === 1
    ? {
        icon: "git_branch",
        title: "Follow edge?",
        body: "Each update then takes main as it lands, checked against KubaOS's keys: new things first, and the bugs with them. Nothing changes until the next update.",
        dismiss: "Cancel",
        actions: [{ label: "Follow edge", icon: "git_branch" }],
      }
    : {
        icon: "key",
        title: "Back to stable?",
        body: `Each update then takes signed releases only. Until one is newer than ${releaseName(version)}, updates bring packages alone.`,
        dismiss: "Cancel",
        actions: [{ label: "Follow stable", icon: "key" }],
      };
}

/** The question before a rollback: what goes back, and what stays. */
export function rollBackQuestion(version: string): DialogOptions {
  return {
    icon: "history",
    title: `Roll back to ${releaseName(version)}?`,
    body: "KubaOS's release, your flake.lock and your Suites' tools go back to where this update found them. What the migrations changed stays.",
    dismiss: "Cancel",
    actions: [{ label: "Roll back", icon: "history" }],
  };
}
