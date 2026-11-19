// About's organisms, kept in the Settings app because nothing else draws them: the machine's name
// card, the update's card at a glance, the hardware's tiles, the facts in groups, the rename dialog,
// the banner a rename leaves until the restart, and Copy all with its toast. Each is assembled from
// library parts: a Card, an IconTile, a Meter, a DescriptionList, a Dialog with a Field, a Banner
// and a Toast.
//
// One draft part, proposed for the library: `hiddenFact`, a fact whose value stays hidden until it
// is clicked (the network's addresses, the serial number).

import "./about.css";
import markUrl from "../../../../../../../crates/kubaos-kobo/assets/logo/kubaos-mark.svg?url";
import { banner, tipLine } from "../../../../lib/banner.ts";
import { button } from "../../../../lib/button.ts";
import { buttonBar, hangs } from "../../../../lib/button-bar.ts";
import { card, cardHeader } from "../../../../lib/card.ts";
import { actionCell, descriptionList, type Fact } from "../../../../lib/description.ts";
import { type DialogOptions, dialogOver } from "../../../../lib/dialog.ts";
import { errorBlock } from "../../../../lib/error-block.ts";
import { field, labelled, note, setInvalid } from "../../../../lib/field.ts";
import { iconButton } from "../../../../lib/icon-button.ts";
import { iconTile } from "../../../../lib/icon-tile.ts";
import { filling, grid, row, stack } from "../../../../lib/layout.ts";
import { scaleOf } from "../../../../lib/list.ts";
import { length } from "../../../../lib/tokens.ts";
import { level, meter, tint } from "../../../../lib/meter.ts";
import { progressBar } from "../../../../lib/progress.ts";
import { clamped, text, wrapped } from "../../../../lib/text.ts";
import { icon } from "../../../../lib/icon.ts";
import { withHint } from "../../../../lib/hint.ts";
import { toast, type ToastOptions, toastRoom } from "../../../../lib/toast.ts";
import { RELEASE } from "../../../kobo/kobo-data.ts";
import { NEXT, releaseName, STEPS, type UpdateValues } from "./updates-parts.ts";

/** A disk: its name, its size and how much of it is used, in GB. */
export interface Disk {
  readonly name: string;
  readonly size: number;
  readonly used: number;
}

/** A screen: its name, its resolution and its refresh rate. */
export interface Screen {
  readonly name: string;
  readonly resolution: string;
  readonly hertz: number;
}

/** What the machine is, as `hostnamectl`, `dmidecode`, `lsblk`, `upower` and Hyprland report it. */
export interface Machine {
  readonly name: string;
  readonly maker: string;
  readonly model: string;
  readonly firmware: string;
  readonly processor: { readonly name: string; readonly cores: number; readonly threads: number };
  readonly memory: { readonly total: number; readonly used: number };
  readonly graphics: { readonly name: string; readonly kind: string };
  readonly disks: readonly Disk[];
  readonly battery?: { readonly health: number; readonly cycles: number };
  readonly screens: readonly Screen[];
  readonly addresses: readonly string[];
  readonly mac: string;
  readonly serial: string;
}

/** The sample machines: the laptop, a desktop of four disks, three screens and no battery, and the longest words. */
export const MACHINES: readonly Machine[] = [
  {
    name: "kuba-pc",
    maker: "Lenovo",
    model: "ThinkPad X1 Carbon Gen 12",
    firmware: "N3YET62W (1.27)",
    processor: { name: "Core Ultra 7 155H", cores: 16, threads: 22 },
    memory: { total: 32, used: 11 },
    graphics: { name: "Intel Arc", kind: "integrated" },
    disks: [{ name: "Samsung PM9C1a", size: 1000, used: 588 }],
    battery: { health: 91, cycles: 214 },
    screens: [{ name: "Built-in", resolution: "2880 × 1800", hertz: 120 }],
    addresses: ["192.168.1.24", "fd00::5a1e:24"],
    mac: "a4:c3:f0:5e:12:9b",
    serial: "PF4XK2M1",
  },
  {
    name: "studio",
    maker: "ASUS",
    model: "ProArt B650-Creator",
    firmware: "3024 (2026-09-12)",
    processor: { name: "Ryzen 9 7950X", cores: 16, threads: 32 },
    memory: { total: 64, used: 41 },
    graphics: { name: "Radeon RX 7800 XT", kind: "16 GB" },
    disks: [
      { name: "Samsung 990 Pro", size: 2000, used: 1310 },
      { name: "WD Black SN850X", size: 4000, used: 3820 },
      { name: "Crucial MX500", size: 1000, used: 212 },
      { name: "Seagate IronWolf", size: 8000, used: 5140 },
    ],
    screens: [
      { name: "Dell U2723QE", resolution: "3840 × 2160", hertz: 60 },
      { name: "LG 27GP950", resolution: "3840 × 2160", hertz: 144 },
      { name: "BenQ PD2705Q", resolution: "2560 × 1440", hertz: 60 },
    ],
    addresses: ["10.0.0.12", "fd00::12"],
    mac: "04:42:1a:9c:e0:3f",
    serial: "MB2309K4A11",
  },
  {
    name: "kuba-thinkpad-x1-carbon-gen-12-workstation-at-the-office",
    maker: "Lenovo",
    model: "ThinkPad X1 Carbon Gen 12 Aura Edition (21KC00BWPB) with Intel vPro",
    firmware: "N3YET62W (1.27)",
    processor: { name: "Core Ultra 7 165U vPro Enterprise", cores: 12, threads: 14 },
    memory: { total: 64, used: 58 },
    graphics: { name: "Intel Graphics", kind: "integrated" },
    disks: [{ name: "Samsung PM9C1a", size: 2000, used: 1960 }],
    battery: { health: 78, cycles: 1214 },
    screens: [{ name: "Built-in", resolution: "2880 × 1800", hertz: 120 }],
    addresses: ["192.168.178.142", "2a02:810d:4b40:7e00:5a1e:24ff:fe3c:9b12"],
    mac: "a4:c3:f0:5e:12:9b",
    serial: "PF4XK2M1",
  },
];

/** A size in GB as a person reads it: TB from a thousand. */
export function size(gb: number): string {
  return gb >= 1000 ? `${(gb / 1000).toLocaleString(undefined, { maximumFractionDigits: 1 })} TB` : `${gb} GB`;
}

/** `n` of a thing, its word in the singular or the plural, as a whole phrase: "1 core", "16 cores". */
function counted(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

/** A processor's cores and threads, as its tile and its fact say them. */
export function coresAndThreads(processor: Machine["processor"]): string {
  return `${counted(processor.cores, "core", "cores")}, ${counted(processor.threads, "thread", "threads")}`;
}

/** How full `used` of `total` is, 0 to 1. */
const share = (used: number, total: number) => Math.min(1, Math.max(0, used / total));

/** A meter's colour: the accent until it nears full, then warning, then error, as the system monitor's. */
const filled = (fraction: number) => tint(level(fraction * 100, 95, 5), "primary");

/** The machine's name, large, with the system's mark, what it runs and what it is, and Rename. */
export function nameCard(name: string, machine: Machine, version: string, rename: () => void): HTMLElement {
  const mark = iconTile({ name: "KubaOS", size: "Lg", picture: markUrl });
  const shown = wrapped(name, "headline");
  shown.classList.add("ab-name");
  const line = wrapped(`KubaOS ${releaseName(version)} · ${machine.maker} ${machine.model}`, "small", "on_surface_variant");
  const words = filling(stack("s4", "start", filling(shown), filling(line)));
  const act = button({ label: "Rename", kind: "Tonal", size: "Sm", icon: "draw" });
  act.addEventListener("click", rename);
  const made = card({ body: [row("s16", "centre", mark, words, act)] });
  made.classList.add("ab-name-card");
  return made;
}

/** What the card at a glance does: check, update, or open the Updates page. */
export interface GlanceActions {
  readonly check: () => void;
  readonly update: () => void;
  readonly open: () => void;
}

/**
 * The update's card on About, one shape in every state: its first line the state with its symbol
 * and the release's line, Settings (the Updates page) and the quick action at that line's end; under
 * it the progress or the reason where there is one, then `tip`, its sub-section's part tip
 * (`tips.ts`), as a Settings row carries its tip.
 */
export function glanceCard(values: UpdateValues, act: GlanceActions, tip?: string): HTMLElement {
  const last = "Last updated 16 November";
  const shape = {
    current: { glyph: "check_circle", title: "Up to date", subtitle: `Checked today at 09:12 · ${last}`, quick: { label: "Check now", kind: "Tonal" as const, icon: "manage_search" } },
    checking: { glyph: "progress_activity", title: "Checking", subtitle: `Asking for the newest release · ${last}`, quick: { label: "Check now", kind: "Tonal" as const, icon: "manage_search", disabled: true } },
    waiting: { glyph: "system_update_alt", title: "An update is ready", subtitle: `${releaseName(NEXT)}, found today at 09:12 · 302 MB`, quick: { label: "Update", kind: "Filled" as const, icon: "system_update_alt" } },
    updating: { glyph: "progress_activity", title: `Updating to ${releaseName(NEXT)}`, subtitle: `Step ${values.at + 1} of ${STEPS.length}: ${STEPS[values.at]!.title}`, quick: null },
    failed: { glyph: "error", title: "Couldn't update", subtitle: `Tried today at 09:40 · ${last}`, quick: { label: "Try again", kind: "Filled" as const, icon: "restart_alt" } },
  }[values.update];
  // Not "Updates": beside Update, the two read alike. It opens the update settings.
  const page = button({ label: "Settings", kind: "Text", size: "Sm", icon: "settings" });
  page.addEventListener("click", act.open);
  const buttons: HTMLElement[] = [page];
  if (shape.quick) {
    const quick = button({ ...shape.quick, size: "Sm" });
    quick.addEventListener("click", values.update === "current" ? act.check : act.update);
    buttons.push(quick);
  }
  const line = row("s16", "centre", filling(cardHeader({ glyph: shape.glyph, title: shape.title, subtitle: shape.subtitle })), row("s8", "centre", ...buttons));
  const body: HTMLElement[] = [line];
  if (values.update === "updating") body.push(progressBar({ fraction: values.at / STEPS.length }));
  if (values.update === "failed") body.push(errorBlock({ message: "A system migration failed, so the update stopped.", hint: `Nothing after step ${values.at + 1} ran; ${releaseName(values.version)} is still here.` }));
  // The line, what it brings, and the tip `space.s12` apart, as a Settings row keeps its tip close.
  const made = card({ body: [stack("s12", "start", ...[...body, ...(tip ? [tipLine(tip)] : [])].map((each) => filling(each)))] });
  // The Updates page's playback finds the card by these and moves its bar and its line in place.
  made.classList.add("up-card");
  made.dataset.update = values.update;
  return made;
}

/**
 * One tile at a glance: the part's symbol and its figure on one line, cut with an ellipsis, a line
 * under it, and how full it is where that applies. The symbol says which part it is, so the line
 * doesn't; the part's name is the tile's accessible name and, with the whole figure, its hint.
 */
function glanceTile(kind: string, glyph: string, title: string, subtitle: string, fullness?: number): HTMLElement {
  const head = row("s8", "centre", icon(glyph, "var(--type-subheading-icon)", "var(--color-on_surface)"), filling(clamped(title, "subheading")));
  const parts: HTMLElement[] = [filling(stack("s4", "start", filling(head), filling(clamped(subtitle, "small", "on_surface_variant"))))];
  if (fullness !== undefined) parts.push(meter([{ share: fullness, color: filled(fullness) }]));
  const made = card({ body: parts });
  made.classList.add("ab-glance");
  made.setAttribute("role", "group");
  made.setAttribute("aria-label", kind);
  // The figure and the line may be cut at a narrow window: the hint says both whole.
  return withHint(made, `${kind}: ${title}, ${subtitle}`);
}

/**
 * The hardware at a glance: processor, memory, disk and graphics, memory and disk with their meters.
 * Four to a row while the room is wider than a text column (`size.column`), two to a row below
 * it, so a narrow window cuts no figure to a letter.
 */
export function glance(machine: Machine): HTMLElement {
  const total = machine.disks.reduce((sum, each) => sum + each.size, 0);
  const used = machine.disks.reduce((sum, each) => sum + each.used, 0);
  const disks = machine.disks.length === 1 ? `${size(total - used)} free` : `${machine.disks.length} disks, ${size(total - used)} free`;
  const tiles = [
      glanceTile("Processor", "memory", machine.processor.name, coresAndThreads(machine.processor)),
      glanceTile("Memory", "memory_alt", `${machine.memory.total} GB`, `${machine.memory.used} GB in use`, share(machine.memory.used, machine.memory.total)),
      glanceTile(machine.disks.length === 1 ? "Disk" : "Disks", "hard_drive", size(total), disks, share(used, total)),
      glanceTile("Graphics", "developer_board", machine.graphics.name, machine.graphics.kind.charAt(0).toUpperCase() + machine.graphics.kind.slice(1)),
  ].map(filling);
  const room = stack(0, "start");
  let columns = 0;
  const lay = (wanted: number) => {
    if (wanted === columns) return;
    columns = wanted;
    room.replaceChildren(filling(grid(wanted, "s16", tiles, { uniformRows: true })));
  };
  lay(4);
  new ResizeObserver(() => lay(room.clientWidth > length("size-column") * scaleOf(room) ? 4 : 2)).observe(room);
  return filling(room);
}

/** Which hidden facts have been shown, and how one is marked shown, by its label. */
export interface Shown {
  readonly has: (label: string) => boolean;
  readonly add: (label: string) => void;
}

/** A fact's copy action: an IconButton whose hint names what it copies, pinned lit for a sample. */
function copyButton(fact: Fact, copy: (fact: Fact) => void, pinned: boolean): HTMLElement {
  const made = iconButton({ icon: "content_paste", label: `Copy ${fact.label.toLowerCase()}`, kind: "Text", size: "Sm", ...(pinned ? { pinned: "lit" as const } : {}) });
  made.addEventListener("click", () => copy(fact));
  return made;
}

/**
 * A group of facts as a card, each line with its copy action (the Description List's line action);
 * `hidden` names the labels whose values wait for a click, and whose action comes only once shown.
 */
export function factsCard(facts: readonly Fact[], copy: (fact: Fact) => void, hidden: readonly string[] = [], shown: Shown = { has: () => false, add: () => undefined }, pinned?: string): HTMLElement {
  const waiting = (each: Fact) => hidden.includes(each.label) && !shown.has(each.label);
  const list = descriptionList(facts.map((each) => (waiting(each) ? each : { ...each, action: copyButton(each, copy, each.label === pinned) })));
  const values = [...list.querySelectorAll<HTMLElement>(".k-fact-value")];
  facts.forEach((each, index) => {
    const slot = values[index];
    if (!slot || !hidden.includes(each.label)) return;
    const label = slot.previousElementSibling;
    // Hidden, the label and its Show button share one centre line; shown, the line is a fact's again.
    const hiding = (on: boolean) => {
      label?.classList.toggle("ab-hidden-label", on);
      slot.classList.toggle("ab-hidden-value", on);
    };
    hiding(waiting(each));
    slot.replaceChildren(
      hiddenFact(each.value, !waiting(each), () => shown.add(each.label), () => {
        hiding(false);
        const cell = actionCell(slot);
        if (cell && !cell.firstElementChild) cell.append(copyButton(each, copy, each.label === pinned));
      }),
    );
  });
  return card({ body: [list] });
}

/**
 * Draft part, proposed for the library: a fact hidden until it is clicked. Hidden, it is a Text sm
 * button, "Show", with the eye, its padding hung so its symbol starts where every value does;
 * pressed, the value takes its place in `mono`, and stays for the window. `revealed` hears when a
 * value is shown by a press, so its line can take its action then.
 */
export function hiddenFact(value: string, open: boolean, opened: () => void, revealed: () => void = () => undefined): HTMLElement {
  const room = document.createElement("span");
  room.className = "ab-hidden";
  const reveal = () => room.replaceChildren(text(value, "mono"));
  if (open) {
    reveal();
    return room;
  }
  const show = button({ label: "Show", kind: "Text", size: "Sm", icon: "visibility" });
  show.addEventListener("click", () => {
    opened();
    reveal();
    delete room.dataset.hidden;
    revealed();
  });
  room.append(hangs(row(0, "centre", show)));
  room.dataset.hidden = "";
  return room;
}

/** The facts of the hardware, a line each, every disk and screen its own. */
export function hardwareFacts(machine: Machine): Fact[] {
  return [
    { label: "Maker and model", value: `${machine.maker} ${machine.model}` },
    { label: "Processor", value: `${machine.processor.name}, ${coresAndThreads(machine.processor)}` },
    { label: "Firmware (BIOS)", value: machine.firmware },
    ...machine.disks.map((each, index) => ({ label: machine.disks.length === 1 ? "Disk" : `Disk ${index + 1}`, value: `${each.name}, ${size(each.size)}`, detail: `${Math.round(share(each.used, each.size) * 100)}% full, ${size(each.size - each.used)} free` })),
    ...(machine.battery ? [{ label: "Battery", value: `${machine.battery.health}% of its first charge`, detail: `${machine.battery.cycles} cycles` }] : []),
    ...machine.screens.map((each, index) => ({ label: machine.screens.length === 1 ? "Screen" : `Screen ${index + 1}`, value: each.name, detail: `${each.resolution} at ${each.hertz} Hz` })),
  ];
}

/** The facts of the system it runs. */
export function systemFacts(version: string): Fact[] {
  return [
    { label: "KubaOS", value: releaseName(version), detail: `on Ubuntu 26.04 · ${RELEASE.commit}` },
    { label: "Kernel", value: RELEASE.kernel },
    { label: "Desktop", value: "Hyprland 0.52.1" },
    { label: "Architecture", value: RELEASE.arch },
    { label: "Session", value: "Wayland" },
    { label: "Installed", value: "12 November 2026" },
    { label: "Up for", value: "3 days, 4 hours" },
    { label: "Secure boot", value: "On" },
    { label: "Disk encryption", value: "On, LUKS2" },
  ];
}

/** The labels of the network identity's facts, each hidden until clicked. */
export const HIDDEN = ["IP addresses", "Network card", "Serial number"] as const;

/** The facts that identify the machine on a network or to its maker. */
export function identityFacts(machine: Machine): Fact[] {
  return [
    { label: "IP addresses", value: machine.addresses.join(", ") },
    { label: "Network card", value: machine.mac },
    { label: "Serial number", value: machine.serial },
  ];
}

/** Every fact on the page as plain text, a group under its heading; hidden ones not yet shown left out. */
export function factsText(groups: readonly (readonly [string, readonly Fact[]])[], name: string, shown: Set<string>): { text: string; count: number; left: number } {
  let count = 0;
  let left = 0;
  const lines = [`Name: ${name}`];
  for (const [title, facts] of groups) {
    lines.push("", title);
    for (const each of facts) {
      if ((HIDDEN as readonly string[]).includes(each.label) && !shown.has(each.label)) {
        left++;
        continue;
      }
      count++;
      lines.push(`${each.label}: ${each.value}${each.detail ? ` (${each.detail})` : ""}`);
    }
  }
  return { text: lines.join("\n"), count, left };
}

/** What Copy all's toast says. */
export function copiedWords(count: number, left: number): ToastOptions {
  return { icon: "content_paste", words: left === 0 ? `Copied ${count} facts` : `Copied ${count} facts; ${left} hidden left out` };
}

/** One fact's toast once its copy button copied it. */
export function copiedFact(fact: Fact): ToastOptions {
  return { icon: "content_paste", words: `Copied ${fact.label.toLowerCase()}` };
}

/** A fact as one line of text: "Label: value". */
export function factLine(fact: Fact): string {
  return `${fact.label}: ${fact.value}`;
}

/** Copy all as an Info banner: what it is for and what it gives, the button beside them. */
export function copyAll(copy: () => void): HTMLElement {
  const act = button({ label: "Copy all", kind: "Tonal", size: "Sm", icon: "content_paste" });
  act.addEventListener("click", copy);
  return banner({ tone: "Info", icon: "info", title: "Reporting a problem?", body: "Copy every fact on this page as text. Hidden ones go in once shown.", aside: act });
}

/** The toast room of `frame`, the window, made the first time it is asked for. */
const rooms = new WeakMap<HTMLElement, ReturnType<typeof toastRoom>>();
export function sayIn(frame: HTMLElement, options: ToastOptions): void {
  let made = rooms.get(frame);
  if (!made) {
    made = toastRoom();
    rooms.set(frame, made);
    frame.append(made.room);
  }
  if (!made.room.isConnected) frame.append(made.room);
  made.say(options);
}

/** A toast drawn still at the foot of `frame`: a sample's. */
export function toastStill(frame: HTMLElement, options: ToastOptions): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-toast-room";
  const shown = toast(options);
  room.append(shown);
  frame.append(room);
  requestAnimationFrame(() => (shown.style.left = `${Math.floor((room.clientWidth - shown.offsetWidth) / 2)}px`));
  return frame;
}

/** The installer's rule for a machine's name (`kubaos_core::engine::validate::valid_hostname`). */
export function validName(name: string): boolean {
  return name.length >= 1 && name.length <= 63 && !name.startsWith("-") && !name.endsWith("-") && /^[a-z0-9-]+$/.test(name);
}

/** What Kōbō says of a name it won't take (`kubaos_kobo::model`), or null for a good one. */
export function nameProblem(name: string): string | null {
  if (name === "") return "Name this machine.";
  return validName(name) ? null : "Lowercase a-z, 0-9 and -, up to 63, with no - at either end.";
}

/** The rename dialog's options around `asks`, Rename disabled until `ok`. */
function renameOptions(asks: HTMLElement, ok: boolean): DialogOptions {
  return {
    icon: "draw",
    title: "Rename this machine",
    body: "Its name on the network, in the terminal's prompt, and to your phone.",
    asks,
    dismiss: "Cancel",
    actions: [{ label: "Rename", icon: "check", disabled: !ok }],
  };
}

/**
 * The rename dialog over `frame`, the field holding `from` (or `typed`, a sample's): the installer's
 * message under the field while the name is not valid, and Rename off while it is, or unchanged.
 * Renamed, `renamed` hears the new name. Drawn `still`, it is a sample's picture.
 */
export function renameDialog(frame: HTMLElement, from: string, renamed: (name: string) => void, typed = from, still = false): HTMLElement {
  let value = typed;
  let act: HTMLButtonElement | null = null;
  const line = document.createElement("span");
  line.className = "ab-rule";
  /** The name as typed: the field and its message follow it, and Rename turns on only for a new, valid one. */
  const tell = (now: string) => {
    value = now;
    const wrong = nameProblem(now);
    setInvalid(input, wrong !== null);
    line.replaceChildren(...(wrong ? [note(wrong, true)] : []));
    if (act) act.disabled = wrong !== null || now === from;
  };
  const input = field({ value: typed, onInput: tell });
  const asks = labelled("Name", input);
  asks.append(line);
  const over = dialogOver(frame, renameOptions(asks, nameProblem(typed) === null && typed !== from), (answer) => {
    if (answer === "Rename" && nameProblem(value) === null && value !== from) renamed(value);
  }, still);
  act = [...over.dialog.element.querySelectorAll<HTMLButtonElement>(".k-dialog-buttons button")].at(-1) ?? null;
  tell(typed);
  return over.room;
}

/** The banner a rename leaves on About until the restart. */
export function restartBanner(was: string, restart: () => void): HTMLElement {
  const bar = buttonBar({ actions: [{ label: "Restart", icon: "restart_alt" }], size: "Sm" });
  bar.actions[0]!.addEventListener("click", restart);
  return banner({ tone: "Info", icon: "restart_alt", title: "Restart to finish", body: `Apps that are open still know this machine as ${was}.`, actions: bar.element });
}
