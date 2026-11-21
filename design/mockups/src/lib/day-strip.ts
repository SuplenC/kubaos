// A row of days to pick one of, each its forecast at a glance: the DayStrip, a DRAFT part
// (`docs/design/components/day-strip.md`). The weather view's week. Every day is a control in the
// state model (`.k-control` in `lib.css`) at the control radius, sharing the row's width equally:
// its name (`label`, `on_surface_variant`) over its symbol (`body`'s icon size, `on_surface`) over
// its high (`body_strong`) over its low (`small`, `on_surface_variant`), centred, `space.s4` apart,
// `space.s8` inside. At rest a day has no surface; hovered and pressed it takes the state layer,
// a faint shape at its radius. The picked day lies on `secondary_container` with its words in
// `on_secondary_container`, as a Selected chip does, its container lifting toward
// `secondary_container_pressed` hovered and pressed. The keyboard's day burns the accent's neon.
//
// The row is one stop for Tab, on the picked day: ← → (h l) pick the day beside, Home and End the
// first and last; a press picks the day pressed.

import "./day-strip.css";
import { type Pinned, pin } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { equalShares } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { text } from "./text.ts";

/** One day of the row. */
export interface StripDay {
  /** Its name: "Today", "Wed". */
  readonly name: string;
  /** Its weather's symbol. */
  readonly symbol: string;
  readonly high: string;
  readonly low: string;
}

export interface DayStripOptions {
  readonly days: readonly StripDay[];
  /** The picked day, the first unless given. */
  readonly picked?: number;
  /** Told of each day picked. */
  readonly onPick?: (index: number) => void;
  /** One day drawn pinned in a state, as the specimens pin it. */
  readonly pinned?: { readonly index: number; readonly state: Pinned };
}

/** One day: its control, its words stacked and centred, and its ring. */
function dayOf(day: StripDay, picked: boolean, pinned: Pinned | undefined): HTMLElement {
  const cell = document.createElement("span");
  cell.className = "k-strip-day k-control";
  cell.setAttribute("role", "radio");
  cell.setAttribute("aria-checked", String(picked));
  cell.toggleAttribute("data-picked", picked);
  pin(cell, pinned);
  const words = document.createElement("span");
  words.className = "k-strip-words";
  // The words' colours are the day's own, so a picked day turns them `on_secondary_container`.
  const name = text(day.name, "label");
  name.style.color = "var(--strip-quiet)";
  const high = text(day.high, "body_strong");
  high.style.color = "var(--strip-ink)";
  const low = text(day.low, "small");
  low.style.color = "var(--strip-quiet)";
  const symbol = icon(day.symbol, "var(--type-body-icon)", "var(--strip-ink)");
  words.append(name, symbol, high, low);
  cell.append(words);
  focusRing(cell, "control", "accent", false).dataset.pressable = "";
  return cell;
}

/** A DayStrip: its days in a row, one picked. */
export function dayStrip(options: DayStripOptions): HTMLElement {
  let picked = Math.max(0, Math.min(options.days.length - 1, options.picked ?? 0));
  const row = document.createElement("span");
  row.className = "k-day-strip";
  row.setAttribute("role", "radiogroup");
  row.tabIndex = 0;
  row.style.setProperty("--days", String(options.days.length));
  const cells = options.days.map((day, index) => dayOf(day, index === picked, options.pinned?.index === index ? options.pinned.state : undefined));
  row.append(...cells);
  equalShares(row, cells.length, "s4");

  function pick(index: number): void {
    if (index < 0 || index >= cells.length || index === picked) return;
    picked = index;
    cells.forEach((cell, at) => {
      cell.toggleAttribute("data-picked", at === index);
      cell.setAttribute("aria-checked", String(at === index));
    });
    options.onPick?.(index);
  }

  cells.forEach((cell, index) => cell.addEventListener("click", () => pick(index)));
  row.addEventListener("keydown", (event) => {
    const k = event.key;
    const to = k === "ArrowLeft" || k === "h" ? picked - 1 : k === "ArrowRight" || k === "l" ? picked + 1 : k === "Home" ? 0 : k === "End" ? cells.length - 1 : null;
    if (to === null) return;
    event.preventDefault();
    pick(to);
  });
  return probe(row, { role: "component", name: "DayStrip", variant: variant(String(options.days.length)), fixed: { width: false, height: true } });
}
