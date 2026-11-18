// Power: the battery, sleep and lock, and performance, from `power.toml` and `idle.toml`.

import { tabs } from "../../../../lib/tabs.ts";
import type { SettingsSection } from "../settings-parts.ts";
import { D, type Values } from "../settings-values.ts";
import { choice, fact, opens, pick, stops, toggle, unseen } from "./kit.ts";

const SOURCES = ["On battery", "Plugged in"] as const;
const STOPS = ["Super Saver", "Power Saver", "Balanced", "Performance", "Max Performance"] as const;
const STOP_ICONS = ["bedtime", "eco", "balance", "bolt", "rocket_launch"] as const;
const LOCK_TIMES = ["30 s", "1 min", "2 min", "5 min", "10 min", "15 min", "30 min", "1 h", "Never"] as const;
const SUSPEND_TIMES = ["1 min", "5 min", "15 min", "30 min", "1 h", "2 h", "Never"] as const;
const LID = ["Suspend", "Lock", "Nothing"] as const;
const ASPM = ["Firmware's default", "Never doze", "Doze", "Doze deepest"] as const;
const PANEL = ["60 Hz", "Adaptive", "Native"] as const;
const LOW_AT = Array.from({ length: 10 }, (_, i) => `${(i + 1) * 5} %`);
const CRITICAL_AT = Array.from({ length: 18 }, (_, i) => `${i + 3} %`);

/** A time a setting accepts: `90s`, `2m`, `1h30m`, minutes alone, or a battery level `25%`. */
function keepProblem(value: string): string | null {
  const level = /^(\d{1,2})\s*%$/.exec(value.trim());
  if (level) return Number(level[1]) >= 1 && Number(level[1]) <= 99 ? null : "A level from 1 % to 99 %.";
  if (/^(\d+h)?(\d+m)?(\d+s)?$/.test(value.trim()) && value.trim() !== "") return null;
  if (/^\d+$/.test(value.trim())) return null;
  return "Try 90s, 2m, 1h30m, or a level as 25 %.";
}

function mhzProblem(value: string): string | null {
  if (!/^\d+$/.test(value.trim())) return "Whole MHz, as 1600; 0 for no cap.";
  const mhz = Number(value);
  return mhz === 0 || (mhz >= 400 && mhz <= 6000) ? null : "From 400 to 6000 MHz, or 0 for no cap.";
}

/** Power, whole: the section the other ten follow. */
export function power(values: Values): SettingsSection {
  const at = (pair: [number, number]) => () => pair[values.source]!;
  const put = (pair: [number, number]) => (index: number) => (pair[values.source] = index);
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "bolt",
    title: "Power",
    accent: "電源",
    lede: "Watt you need, when you need it.",
    subs: [
      {
        title: "Battery",
        body: "How it charges, and what happens as it runs out.",
        wells: [
          [
            choice("charge-limit", "Charge limit", "Stops charging here. Kinder to a battery that lives at a desk.", ["80 %", "90 %", "Full"], () => values.chargeLimit, (index) => (values.chargeLimit = index), { initial: () => D.chargeLimit }),
            pick("low-stop", "When the battery runs low", "Drops to a saver stop to last longer; plugging in ends it.", ["Power Saver", "Super Saver", "Do nothing"], () => values.lowStop, (index) => (values.lowStop = index), {
              drives: true,
              initial: () => D.lowStop,
              warns: () => (values.lowStop === 2 ? { title: "Nothing slows down as it runs out.", body: "At 2 % the laptop powers off, whatever is open." } : null),
            }),
            stops("low-at", "Low means", "The level that counts as running low.", "battery_1_bar", LOW_AT, () => values.lowAt, (index) => (values.lowAt = index), { shows: () => values.lowStop !== 2, initial: () => D.lowAt }),
          ],
          [
            toggle("critical-warning", "Warn before it dies", "A toast and a sound, once, before it powers off.", () => values.criticalWarning, (on) => (values.criticalWarning = on), { drives: true, initial: () => D.criticalWarning }),
            stops("critical-at", "Warn at", "The level the warning comes at.", "battery_0_bar", CRITICAL_AT, () => values.criticalAt, (index) => (values.criticalAt = index), { shows: () => values.criticalWarning, initial: () => D.criticalAt }),
            fact("power-off", "Power off", "", "At 2 %", { disabled: "Ubuntu's UPower does this, and can't hibernate under Secure Boot." }),
            toggle("battery-notes", "Bad night reports", "Says so when a sleep drained more than it should.", () => values.batteryNotes, (on) => (values.batteryNotes = on), { initial: () => D.batteryNotes }),
          ],
        ],
      },
      {
        title: "Sleep & lock",
        body: "What happens when you walk away, per power source.",
        wells: [
          [
            choice("source", "Setting for", "Battery and plugged in each keep their own times.", SOURCES, () => values.source, (index) => (values.source = index), { drives: true }),
            pick("lock", "Lock after", "Locks the screen once you have been away this long.", LOCK_TIMES, at(values.lock), put(values.lock), { initial: () => D.lock[values.source]!, unseen: unseen(values.lock, D.lock, () => values.source) }),
            pick("screens-off", "Screens off after", "Turns every screen off once you have been away this long.", LOCK_TIMES, at(values.screensOff), put(values.screensOff), { initial: () => D.screensOff[values.source]!, unseen: unseen(values.screensOff, D.screensOff, () => values.source) }),
            pick("suspend", "Sleep after", "Puts the laptop to sleep once you have been away this long.", SUSPEND_TIMES, at(values.suspend), put(values.suspend), { initial: () => D.suspend[values.source]!, unseen: unseen(values.suspend, D.suspend, () => values.source) }),
            pick("lid", "When the lid closes", "Docked, closing it never does anything.", LID, at(values.lid), put(values.lid), { initial: () => D.lid[values.source]!, unseen: unseen(values.lid, D.lid, () => values.source) }),
          ],
          [
            toggle("keep-awake", "Keep awake", "Holds off the lock, screens off and sleep. Ends by itself.", () => values.keepAwake, (on) => (values.keepAwake = on), { drives: true, initial: () => D.keepAwake }),
            {
              id: "keep-for",
              title: "For",
              line: "A time, or until the battery is down to a level.",
              control: { kind: "field", value: () => values.keepFor, placeholder: "1h, 90m or 25 %", check: keepProblem, set: (value) => (values.keepFor = value) },
              shows: () => values.keepAwake,
              initial: () => D.keepFor,
            },
          ],
        ],
      },
      {
        title: "Performance",
        body: "Quiet and cool, or fast and loud.",
        wells: [
          [
            {
              id: "stop-battery",
              title: "Power stop on battery",
              line: "Each stop sets the processor, the screen and the fans at once.",
              control: { kind: "slider", icon: "eco", names: STOPS, icons: STOP_ICONS, index: () => values.stopOn[0], set: (index) => (values.stopOn[0] = index) },
              initial: () => D.stopOn[0],
            },
            {
              id: "stop-plugged",
              title: "Power stop plugged in",
              line: "Plugging in goes back to this one.",
              control: { kind: "slider", icon: "balance", names: STOPS, icons: STOP_ICONS, index: () => values.stopOn[1], set: (index) => (values.stopOn[1] = index) },
              initial: () => D.stopOn[1],
            },
            opens("stops", "Tune each stop", "The screen's rate, the drives' dozing and a clock cap, per stop.", "stops", () => "5 stops"),
          ],
          [
            choice("scheduler", "CPU scheduler", "Decides which program runs next. LAVD puts what you're waiting on first.", ["LAVD", "Linux's own"], () => values.scheduler, (index) => (values.scheduler = index), { initial: () => D.scheduler }),
            toggle("yields", "Builds step aside", "Compilers and Nix builds run at low priority, so typing stays smooth.", () => values.yields, (on) => (values.yields = on), { initial: () => D.yields }),
            toggle("panel-saving", "Panel power saving", "The saver stops dim the screen to save power. Off for colour work.", () => values.panelSaving, (on) => (values.panelSaving = on), { initial: () => D.panelSaving }),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "stops",
        title: "Tune each stop",
        body: "What a stop does beyond its name. Most never need it.",
        back: "Performance",
        before: () =>
          tabs({
            choices: STOPS.map((label) => ({ label })),
            picked: values.tuned,
            onPick: (index) => {
              values.tuned = index;
              values.window?.querySelector<HTMLElement>(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
            },
          }),
        wells: [
          [
            choice("panel", "Screen refresh", "How often the laptop's screen redraws. Lower saves power.", PANEL, () => values.panel[values.tuned]!, (index) => (values.panel[values.tuned] = index), {
              initial: () => D.panel[values.tuned]!,
              unseen: unseen(values.panel, D.panel, () => values.tuned),
            }),
            pick("aspm", "Drives dozing (ASPM)", "How deeply the SSD and Wi-Fi card nap between uses.", ASPM, () => values.aspm[values.tuned]!, (index) => (values.aspm[values.tuned] = index), {
              drives: true,
              initial: () => D.aspm[values.tuned]!,
              unseen: unseen(values.aspm, D.aspm, () => values.tuned),
              warns: () => (values.aspm[values.tuned] === 3 ? { title: "Some drives and Wi-Fi cards drop off at this depth.", body: "If yours reset, pick Doze instead; the kernel's log says so." } : null),
            }),
            {
              id: "cpu-max",
              title: "Clock cap",
              line: "The fastest the processor may run, in MHz. 0 lets it run free.",
              control: { kind: "field", value: () => values.cpuMax[values.tuned]!, placeholder: "0", check: mhzProblem, set: (value) => (values.cpuMax[values.tuned] = value) },
              initial: () => D.cpuMax[values.tuned]!,
              unseen: unseen(values.cpuMax, D.cpuMax, () => values.tuned),
            },
          ],
        ],
      },
    ],
  };
}
