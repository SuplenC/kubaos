// Desktop's organisms, kept in the Settings app because only it draws them: the system monitor's readings, the pill the bar draws from them over the
// list that picks and orders them; and the workspace groups' page with only the laptop panel.

import { banner } from "../../../../lib/banner.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import { row, stack, filling } from "../../../../lib/layout.ts";
import { well } from "../../../../lib/list.ts";
import { orderList } from "../../../../lib/order-list.ts";
import { number } from "../../../../lib/tokens.ts";
import { settings } from "../../../../site/settings.ts";
import { type Reading as Shown, type SysmonStyle, sysmonPill } from "../../../shell/bar-parts.ts";

/** A metric of `sysmon.toml`'s `metrics`, as `kubaos-sysmon` names it. */
export type Metric = "cpu" | "ram" | "temp" | "gpu" | "net" | "power";

/** A reading the bar's system monitor can show: its metric, its plain name, and whether it shows. */
export interface Reading {
  readonly id: Metric;
  readonly name: string;
  on: boolean;
}

/** What the pill shows for each, as `Sysmon.qml`'s `metric` draws it: the shots' stand-in values. */
const DRAWN: Record<Metric, Shown> = {
  cpu: { icon: "memory", text: "37%", fill: 0.37 },
  ram: { icon: "memory_alt", text: "71%", fill: 0.71 },
  temp: { icon: "thermostat", text: "61°", fill: 0.61 },
  gpu: { icon: "developer_board", text: "12%", fill: 0.12 },
  net: { icon: "swap_vert", text: "1.2 MB/s", fill: 0, unscaled: true },
  power: { icon: "bolt", text: "9.4 W", fill: 0.31 },
};

const STYLES: readonly SysmonStyle[] = ["numbers", "bars", "sparklines"];

export interface ReadingsOptions {
  readonly readings: readonly Reading[];
  /** The style's index: numbers, bars, graphs. */
  readonly style: number;
  /** The last reading's switch was turned off, and turned back on: the pill needs one. */
  readonly refused: boolean;
  /** A row drawn lifted, as mid-drag: a sample's. */
  readonly lifted: number | null;
  /** Told after every change, with the readings as they stand and whether one was refused. */
  readonly changed: (readings: Reading[], refused: boolean) => void;
}

/** The reading whose row had the keyboard when the list was drawn again, to put it back there. */
let keyboardOn: Metric | null = null;

/** The pill as the bar draws it, with `readings` on, in `style`. */
function pillOf(readings: readonly Reading[], style: number): HTMLElement {
  return row("s8", "start", sysmonPill({ readings: readings.filter((each) => each.on).map((each) => DRAWN[each.id]), style: STYLES[style] ?? "numbers" }));
}

/**
 * The readings' page's head. The pill as the bar draws it, over an OrderList
 * in a well: each reading switched on or off and dragged into place, the pill following at once.
 * Turning off the last one is refused, as `kubaos-sysmon` refuses an empty list, and says why.
 */
export function readingsPart(options: ReadingsOptions): HTMLElement {
  const items = options.readings.map((each) => ({ ...each }));
  const preview = pillOf(items, options.style);
  const list = orderList({
    items,
    ...(options.lifted !== null ? { lifted: options.lifted } : {}),
    changed: () => {
      const before = options.readings;
      let refused = false;
      if (!items.some((each) => each.on)) {
        const last = items.find((each) => before.find((was) => was.id === each.id)?.on);
        if (last) last.on = true;
        refused = true;
      }
      preview.replaceChildren(...pillOf(items, options.style).childNodes);
      keyboardOn = items[[...list.box.children].indexOf(document.activeElement?.closest(".k-order-row") ?? list.box)]?.id ?? null;
      options.changed(items.map((each) => ({ ...each })), refused);
      // The region is drawn again once the rows have glided into place, so the resets follow.
      const settle = number("duration-medium") / settings().speed;
      setTimeout(() => list.box.closest(".st-region")?.dispatchEvent(new CustomEvent("st-redraw")), settle);
    },
  });
  if (keyboardOn) {
    const at = items.findIndex((each) => each.id === keyboardOn);
    keyboardOn = null;
    requestAnimationFrame(() => list.box.querySelectorAll<HTMLElement>(".k-order-row")[at]?.focus({ preventScroll: true }));
  }
  const parts: HTMLElement[] = [filling(preview), filling(well("List", list.box))];
  if (options.refused) parts.push(filling(banner({ tone: "Info", icon: "info", title: "The bar needs one reading at least.", body: "To take the system monitor out of the bar, turn it off under Bar." })));
  return stack("s16", "start", ...parts);
}

/** The workspace groups' page while only the laptop panel has ever been plugged in. */
export function groupsEmpty(): HTMLElement {
  return emptyState({ icon: "desktop_windows", title: "Only the laptop panel so far", body: "Plug in a screen: it gets a group of its own, which you can pin here." });
}
