// The system monitor's panel as `config/quickshell/modules/sysmon/` draws it, from the library's
// parts, with the shots' stand-in readings (`test/shots/fakes`): the header with its mood line and
// btop, a card per reading, the busiest processes with a process's details, and the info button in
// the bottom right corner. The system monitor's pages build their samples from these. This file is
// no page.

import "./sysmon.css";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { icon } from "../../lib/icon.ts";
import { fill, filling, grid, row, splitColumns, stack } from "../../lib/layout.ts";
import { type Level, meter, minute, sparkline, tint } from "../../lib/meter.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { skeleton } from "../../lib/skeleton.ts";
import { slider } from "../../lib/slider.ts";
import { type Column, table } from "../../lib/table.ts";
import { tabs } from "../../lib/tabs.ts";
import { clamped, text } from "../../lib/text.ts";
import { floatingTooltip, tooltip } from "../../lib/tooltip.ts";
import { withHint } from "../../lib/hint.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { bellPill, rimPill, statusPill, sysmonPill, wallpaper } from "./bar-parts.ts";
import { CORNER, POWER_ICONS, POWER_STOPS, TALL } from "./control-center-parts.ts";

export { CORNER, TALL };

/** The line under the title for the worst reading's step, as `panel.js` says it. */
export function mood(level: Level): string {
  return level >= 2 ? "This is fine" : level === 1 ? "Breaking a sweat" : "Cool as a cucumber";
}

/** A process as a detailed sample lists it. */
export interface Process {
  readonly name: string;
  readonly cpu: string;
  readonly memory: string;
  readonly command: string;
  readonly facts: string;
}

export const PROCESSES: readonly Process[] = [
  { name: "firefox", cpu: "38.4%", memory: "2.1 GB", command: "/usr/lib/firefox/firefox -contentproc -childID 12 -isForBrowser -prefsLen 31000 -prefMapSize 244787", facts: "pid 4211 · kuba · Sleeping · 87 threads · since Tue 05:46" },
  { name: "Hyprland", cpu: "12.1%", memory: "384 MB", command: "Hyprland", facts: "pid 1022 · kuba · Running · 14 threads · since Tue 05:41" },
  { name: "rust-analyzer", cpu: "9.7%", memory: "1.8 GB", command: "/home/kuba/.nix-profile/bin/rust-analyzer", facts: "pid 6630 · kuba · Sleeping · 22 threads · since Tue 09:12" },
  { name: "quickshell", cpu: "4.2%", memory: "256 MB", command: "qs -c kubaos", facts: "pid 1130 · kuba · Sleeping · 31 threads · since Tue 05:41" },
  { name: "ghostty", cpu: "1.3%", memory: "150 MB", command: "ghostty", facts: "pid 1388 · kuba · Sleeping · 9 threads · since Tue 05:42" },
  { name: "pipewire", cpu: "0.8%", memory: "42 MB", command: "/usr/bin/pipewire", facts: "pid 988 · kuba · Sleeping · 4 threads · since Tue 05:41" },
  { name: "kubaos-sysmon", cpu: "0.4%", memory: "18 MB", command: "kubaos-sysmon", facts: "pid 1101 · kuba · Sleeping · 3 threads · since Tue 05:41" },
];

/** By memory: the same processes, heaviest first. */
const BY_MEMORY: readonly Process[] = [...PROCESSES].sort((a, b) => bytes(b.memory) - bytes(a.memory));

function bytes(said: string): number {
  const [value, unit] = said.split(" ");
  return Number(value) * (unit === "GB" ? 1024 : 1);
}

export const COLUMNS: readonly Column[] = [
  { title: "Name", weight: 2 },
  { title: "CPU", weight: 1, numeric: true },
  { title: "Memory", weight: 1, numeric: true },
];

/** The CPU's sixteen cores, busy by their share, as the shots' stand-in. */
const CORES = [0.42, 0.18, 0.66, 0.08, 0.31, 0.44, 0.12, 0.58, 0.27, 0.09, 0.36, 0.71, 0.22, 0.4, 0.15, 0.52];

export interface SysmonOptions {
  /** The engine's samples: detailed, or the light ones it opens on, their lines placeholders. */
  readonly sample?: "detailed" | "light";
  /** The CPU's temperature step and the backup disk's: the shots' "fine", default and "critical". */
  readonly heat?: Level;
  readonly disk?: Level;
  /** A machine with a GPU and a battery, or neither. */
  readonly bare?: boolean;
  /** Btrfs quotas off: the snapshots' size unknown, its info button there. */
  readonly quotasOff?: boolean;
  /** Scrolled this share of the way down. */
  readonly down?: number;
  /** The process shown, by its row, and the stop it asks about. */
  readonly picked?: number;
  readonly confirming?: "term" | "kill";
  readonly order?: "cpu" | "memory";
  /** A card shown, as under the pointer. */
  readonly tip?: "btop" | "about" | "quotas";
  /** kubaos-sysmon not running: the line that says so, above the cards. */
  readonly unavailable?: boolean;
  /** The picked process's command, for its longest. */
  readonly command?: string;
  /** The engine replied with no processes. */
  readonly noProcesses?: boolean;
  /** Answers the pointer: the order tabs, a row's details, End and Kill. */
  readonly live?: boolean;
}

/** The worst step the readings shown are at, which the mood line says. */
export function worst(options: SysmonOptions): Level {
  return Math.max(options.heat ?? 0, options.disk ?? 1) as Level;
}

/**
 * A line that ticks, in `style`: its words over a placeholder `share` of the room wide, one or the
 * other shown. The two hold one box, so the words arriving move nothing.
 */
function waiting(words: string, style: "mono" | "mono_large", color: string, waits: boolean, share: number, end = false): HTMLElement {
  const box = document.createElement("span");
  box.className = "sm-wait";
  box.toggleAttribute("data-waiting", waits);
  box.toggleAttribute("data-end", end);
  box.style.setProperty("--share", String(share));
  const bone = skeleton({ bone: "line", style });
  bone.classList.add("sm-bone");
  const shown = style === "mono" ? clamped(words, style, color) : text(words, style, color);
  shown.classList.add("sm-words");
  box.append(bone, shown);
  return box;
}

/** A reading's card: its symbol and title, the headline reading at the end, what it holds below. */
function section(glyph: string, title: string, value: HTMLElement | null, ...lines: HTMLElement[]): HTMLElement {
  const head = row("s8", "centre", icon(glyph, "var(--type-body_strong-icon)", "var(--color-primary)"), text(title, "body_strong"), fill(), ...(value ? [value] : []));
  const body = stack("s4", "start", ...lines.map(filling));
  return card({ body: [stack("s8", "start", filling(head), ...(lines.length ? [filling(body)] : []))] });
}

/** A thing's name with its reading at the end of the row, the reading in its step. */
function named(name: string, reading: string, fine: string, level: Level = 0): HTMLElement {
  return row("s12", "centre", clamped(name, "label"), fill(), text(reading, "mono", tint(level, fine)));
}

/** One bar a core, filled from the bottom, `space.s4` apart. */
function cores(shares: readonly number[], empty: boolean): HTMLElement {
  const bars = shares.map((share) => {
    const bar = document.createElement("span");
    bar.className = "sm-core";
    const busy = document.createElement("span");
    busy.className = "sm-core-busy";
    busy.style.setProperty("--busy", String(empty ? 0 : share));
    bar.append(busy);
    return bar;
  });
  return grid(shares.length, "s4", bars);
}

/** The cards: CPU, memory, GPU, network, disks, power and fans. */
export function readings(options: SysmonOptions = {}): HTMLElement {
  const light = options.sample === "light";
  const heat = options.heat ?? 0;
  const disk = options.disk ?? 1;
  const cpu = section(
    "memory",
    "CPU",
    text(heat === 2 ? "94%" : "37%", "mono_large", tint(heat, "on_surface")),
    sparkline({ samples: minute(600_000, (s) => (heat === 2 ? 88 : 36) + 8 * Math.sin(s / 6)), color: tint(heat, "primary") }).box,
    cores(heat === 2 ? CORES.map((each) => Math.min(1, each + 0.5)) : CORES, light),
    waiting(`16 cores · 3.4 GHz · ${heat === 2 ? 97 : heat === 1 ? 61 : 48} °C`, "mono", tint(heat, "on_surface_variant"), light, 0.7),
  );
  const memory = section(
    "memory_alt",
    "Memory",
    text("71%", "mono_large"),
    meter([
      { share: 0.55, color: "primary" },
      { share: 0.15, color: "tertiary" },
    ]),
    clamped("17 GB used · 4.6 GB cache of 31 GB", "mono", "on_surface_variant"),
    clamped("Swap 1.2 GB of 8.0 GB", "mono", "on_surface_variant"),
  );
  const gpu = section("developer_board", "GPU", text("23%", "mono_large"), named("AMD Radeon 780M", "", "on_surface"), meter([{ share: 0.23, color: "primary" }]), clamped("VRAM 870 MB of 2.0 GB · 54 °C · 1800 MHz", "mono", "on_surface_variant"));
  const network = section("swap_vert", "Network", text("↓ 1.2 MB/s  ↑ 82 KB/s", "mono_large"), waiting("wlp1s0 · 192.168.1.42", "mono", "on_surface_variant", light, 0.45));
  const snapshots = row("s4", "centre", waiting(options.quotasOff ? "14 snapshots · size unknown" : "14 snapshots · 6.0 GB", "mono", "on_surface_variant", light, 0.6));
  snapshots.classList.add("sm-snapshots");
  snapshots.toggleAttribute("data-explained", options.quotasOff ?? false);
  if (options.quotasOff && !light) {
    const why = iconButton({ icon: "info", label: "Why the size is unknown", kind: "Text", size: "Sm", hint: false, ...(options.tip === "quotas" ? { pinned: "lit" as const } : {}) });
    snapshots.append(floatingTooltip(why, tooltip("Snapshot size unknown", "Btrfs counts the space snapshots hold only with its quotas on, and they are off. Turning them on (sudo btrfs quota enable /) makes deleting snapshots slower."), { side: "above", held: "end", shown: options.tip === "quotas" }));
  }
  const disks = section(
    "hard_drive",
    "Disks",
    waiting("R 3.2 MB/s  W 793 KB/s", "mono_large", "on_surface", light, 0.4, true),
    stack("s4", "start", filling(named("/", "200 GB free of 932 GB", "on_surface_variant")), filling(meter([{ share: 0.79, color: "primary" }]))),
    stack("s4", "start", filling(named("Backup", disk === 2 ? "4 GB free of 477 GB" : "48 GB free of 477 GB", "on_surface_variant", disk)), filling(meter([{ share: disk === 2 ? 0.99 : disk === 1 ? 0.9 : 0.6, color: tint(disk, "primary") }]))),
    snapshots,
  );
  const power = options.bare
    ? section("bolt", "Power", null, clamped("No battery: on mains power", "mono", "on_surface_variant"), slider({ icon: POWER_ICONS[2]!, icons: POWER_ICONS, size: "Compact", stops: POWER_STOPS.length, index: 2, names: POWER_STOPS }))
    : section(
        "bolt",
        "Power",
        text("64%", "mono_large"),
        meter([{ share: 0.64, color: "primary" }]),
        clamped("Discharging · 11.4 W · 4 h 17 min to empty", "mono", "on_surface_variant"),
        clamped("Health 93% of its design capacity", "mono", "on_surface_variant"),
        slider({ icon: POWER_ICONS[2]!, icons: POWER_ICONS, size: "Compact", stops: POWER_STOPS.length, index: 2, names: POWER_STOPS }),
      );
  const fans = section("mode_fan", "Fans", null, waiting("cpu_fan  2380 rpm", "mono", "on_surface_variant", light, 0.7));
  const cards = [cpu, memory, ...(options.bare ? [] : [gpu]), network, disks, power, ...(options.bare ? [] : [fans])];
  return stack("s8", "start", ...cards.map(filling));
}

/** A row's shape while the processes are counted: a bar where each column's words will be. */
function placeholderRow(index: number): HTMLElement {
  const shares = [0.7, 0.45, 0.6, 0.35, 0.55];
  const bones = COLUMNS.map((column, at) => {
    const share = column.numeric ? shares[(index + at + 2) % shares.length]! + 0.3 : shares[(index + at) % shares.length]!;
    return waiting("", column.numeric ? "mono_large" : "mono", "on_surface", true, share, column.numeric);
  });
  const shape = splitColumns(
    COLUMNS.map((column) => column.weight),
    "s12",
    bones,
  );
  shape.classList.add("sm-placeholder");
  return shape;
}

/** The process shown, under its row: what it runs, whose it is, End and Kill. */
function details(process: Process, confirming: "term" | "kill" | undefined, live: boolean, redraw: (next: "term" | "kill" | undefined) => void): HTMLElement {
  const hinted = (made: HTMLButtonElement, words: string) => (confirming ? made : withHint(made, words));
  const end = hinted(button({ label: "End", kind: confirming === "term" ? "Filled" : "Tonal", size: "Sm", disabled: confirming === "kill" }), "Ask it to quit; it may save its work first");
  const kill = hinted(button({ label: "Kill", kind: confirming === "kill" ? "Danger" : "Text", size: "Sm", disabled: confirming === "term" }), "Stop it at once; unsaved work is lost");
  end.tabIndex = -1;
  kill.tabIndex = -1;
  const actions = row("s8", "centre", end, kill);
  if (confirming) {
    const cancel = button({ label: "Cancel", kind: "Text", size: "Sm" });
    cancel.tabIndex = -1;
    actions.append(filling(clamped(`${confirming === "kill" ? "Kill" : "End"} ${process.name}?`, "body_strong", confirming === "kill" ? "error" : "on_surface")), cancel);
    if (live) cancel.addEventListener("click", () => redraw(undefined));
  }
  if (live) {
    end.addEventListener("click", () => redraw(confirming === "term" ? undefined : "term"));
    kill.addEventListener("click", () => redraw(confirming === "kill" ? undefined : "kill"));
  }
  const body = stack("s4", "start", filling(clamped(process.command, "mono", "on_surface_variant", 2)), filling(clamped(process.facts, "small", "on_surface_variant")), filling(actions));
  body.classList.add("sm-details-body");
  const box = document.createElement("span");
  box.className = "sm-details";
  box.append(body);
  new ResizeObserver(() => box.style.setProperty("--natural", `${body.offsetHeight}px`)).observe(body);
  return box;
}

/** The busiest processes: their heading with the order, the column heads, the rows, a row's details. */
export function processes(options: SysmonOptions = {}): HTMLElement {
  let order = options.order ?? "cpu";
  let picked = options.picked ?? -1;
  let confirming = options.confirming;
  const live = options.live ?? false;
  const holder = document.createElement("span");
  holder.className = "sm-rows";

  function draw(): void {
    const list = order === "cpu" ? PROCESSES : BY_MEMORY;
    const shown = list.slice(0, 5);
    if (options.sample === "light") {
      const heads = table(COLUMNS, []);
      holder.replaceChildren(filling(heads), ...shown.map((_, index) => filling(placeholderRow(index))));
      return;
    }
    if (options.noProcesses) {
      holder.replaceChildren(filling(table(COLUMNS, [])), clamped("No processes to show", "small", "on_surface_variant"));
      return;
    }
    const rows = table(
      COLUMNS,
      shown.map((each, index) => ({ cells: [each.name, each.cpu, each.memory], picked: index === picked })),
      {
        onPick: (index) => {
          if (!live) return;
          picked = picked === index ? -1 : index;
          confirming = undefined;
          draw();
        },
      },
    );
    const found = shown[picked];
    const process = found && options.command ? { ...found, command: options.command } : found;
    if (process) {
      const under = rows.querySelectorAll(".k-table-row")[picked];
      under?.after(
        details(process, confirming, live, (next) => {
          confirming = next;
          draw();
        }),
      );
    }
    holder.replaceChildren(filling(rows));
  }

  const order0 = tabs({
    choices: [{ label: "CPU" }, { label: "Memory" }],
    picked: order === "cpu" ? 0 : 1,
    onPick: (index) => {
      order = index === 0 ? "cpu" : "memory";
      picked = -1;
      draw();
    },
  });
  order0.querySelectorAll<HTMLElement>("[tabindex]").forEach((each) => (each.tabIndex = -1));
  const head = row("s8", "centre", icon("list_alt", "var(--type-body_strong-icon)", "var(--color-primary)"), text("Processes", "body_strong"), fill(), order0);
  draw();
  return stack("s8", "start", filling(head), filling(holder));
}

/** The whole panel: the header, the cards and the processes, the info button in the corner. */
export function sysmon(options: SysmonOptions = {}): HTMLElement {
  const btop = iconButton({ icon: "terminal", label: "Open btop", kind: "Text", size: "Sm", hint: false, ...(options.tip === "btop" ? { pinned: "lit" as const } : {}) });
  const btopTip = floatingTooltip(btop, tooltip("Open btop", "Every process and sensor, in a terminal."), { side: "below", held: "end", shown: options.tip === "btop" });
  const problems = options.unavailable ? [clamped("kubaos-sysmon isn't running: the monitor needs it", "small", "error")] : [];
  const body = stack("s16", "start", ...[...problems, readings(options), processes(options)].map(filling));
  body.classList.add("sm-body");
  const level = worst(options);
  const page = panelPage({ title: "System", subtitle: mood(level), actions: [btopTip], body, ...(options.down !== undefined ? { down: options.down } : {}) });
  page.element.classList.add("sm-panel");
  const line = page.element.querySelectorAll<HTMLElement>(".k-panel-header .k-text")[1];
  if (line) line.style.color = `var(--color-${tint(level, "on_surface_variant")})`;
  const about = iconButton({ icon: "info", label: "Uptime and kernel", kind: "Tonal", size: "Sm", hint: false, ...(options.tip === "about" ? { pinned: "lit" as const } : {}) });
  about.tabIndex = -1;
  const load = clamped("Load 1.02  0.88  0.75", "mono", "on_surface_variant");
  const corner = floatingTooltip(about, tooltip("Up 3 d 4 h", "Linux 6.17.0-5-generic", load), { side: "above", held: "end", shown: options.tip === "about" });
  corner.classList.add("sm-about");
  page.element.append(corner);
  return page.element;
}

/** The panel open on the right edge of a screen corner, under the bar's panel pills. */
export function sysmonOnScreen(options: SysmonOptions = {}, height = TALL, reveal?: number, rings = false): HTMLElement {
  const room = screen({ width: CORNER, height, wallpaper, bar: [rings ? rimPill() : sysmonPill(), bellPill(), statusPill()] });
  room.element.dataset.open = "";
  hang(room, sheet({ content: sysmon(options), edge: "right", ...(reveal !== undefined ? { reveal } : {}) }));
  return room.element;
}
