// The OrderList: things in an order that matters, each offered or not (`docs/design/components/
// order-list.md`). A row each, `size.control_md` tall: a grab handle, the thing's name and a Switch
// at its end, a hairline between rows. A row is dragged by its handle, the others gliding out of its
// way over `medium` on `standard_decel`; Alt with ↑ or ↓ moves the focused row a place. Disabled, the
// whole list recedes to `opacity.disabled_content` and takes no pointer, its rows kept where they are
// so nothing jumps. Its look is `lists.css`.
//
// It is the web search's engines on first boot's Kotodama card: the order is the menu's order of
// its web rows, and a switched-off engine isn't offered.

import "./lists.css";
import { icon } from "./icon.ts";
import { filling, row } from "./layout.ts";
import { probe } from "./probe.ts";
import { switchControl } from "./switch.ts";
import { text } from "./text.ts";
import { cubic } from "./tokens.ts";
import { milliseconds } from "../site/recipe.ts";
import { settings } from "../site/settings.ts";

/** One thing in the list: its name, and whether it is offered. */
export interface OrderItem {
  readonly name: string;
  on: boolean;
}

export interface OrderListOptions {
  /** The things, in their order; the list moves them in this array as it reorders them. */
  readonly items: OrderItem[];
  readonly disabled?: boolean;
  /** A row drawn lifted, as mid-drag: a sample of the drag. */
  readonly lifted?: number;
  /** Told after every change: an order moved or a switch turned. */
  readonly changed?: () => void;
}

export interface OrderList {
  readonly box: HTMLElement;
  readonly setDisabled: (off: boolean) => void;
}

/** A duration token in milliseconds under the speed setting. */
function ms(name: Parameters<typeof milliseconds>[0]): number {
  return milliseconds(name) / settings().speed;
}

function curve(name: string): string {
  return `cubic-bezier(${cubic(`easing-${name}`).join(", ")})`;
}

export function orderList(options: OrderListOptions): OrderList {
  const items = options.items;
  const box = document.createElement("span");
  box.className = "k-order-list";
  box.setAttribute("role", "list");
  const rows = new Map<OrderItem, HTMLElement>();
  const order = () => items.map((each) => rows.get(each)!);
  /** Puts the rows in the items' order, each gliding from where it was (reduced: at once). */
  const place = (focus?: HTMLElement) => {
    const before = new Map(order().map((each) => [each, each.getBoundingClientRect().top]));
    box.replaceChildren(...order());
    if (!settings().reduced)
      for (const [each, top] of before) {
        const box = each.getBoundingClientRect();
        const by = (top - box.top) / box.height;
        if (by !== 0) each.animate([{ "--order-glide": by }, { "--order-glide": 0 }], { duration: ms("medium"), easing: curve("standard_decel") });
      }
    focus?.focus({ preventScroll: true });
  };
  const move = (item: OrderItem, by: number) => {
    const from = items.indexOf(item);
    const to = Math.max(0, Math.min(items.length - 1, from + by));
    if (from === to) return;
    items.splice(from, 1);
    items.splice(to, 0, item);
    place(rows.get(item));
    options.changed?.();
  };
  items.forEach((item, index) => {
    const handle = icon("swap_vert", "var(--type-body-icon)", "var(--color-on_surface_variant)");
    handle.classList.add("k-order-grab");
    const control = switchControl({ on: item.on });
    control.addEventListener("click", () =>
      queueMicrotask(() => {
        item.on = control.getAttribute("aria-checked") === "true";
        options.changed?.();
      }),
    );
    const line = row("s12", "centre", handle, filling(text(item.name, "body")), control);
    line.classList.add("k-order-row");
    line.setAttribute("role", "listitem");
    line.tabIndex = 0;
    if (options.lifted === index) line.dataset.lifted = "";
    line.addEventListener("keydown", (event) => {
      if (!event.altKey || (event.key !== "ArrowUp" && event.key !== "ArrowDown")) return;
      event.preventDefault();
      move(item, event.key === "ArrowUp" ? -1 : 1);
    });
    handle.addEventListener("pointerdown", (event) => {
      event.preventDefault();
      line.dataset.lifted = "";
      const pitch = line.getBoundingClientRect().height;
      let start = event.clientY;
      const follow = (moved: PointerEvent) => {
        const by = Math.round((moved.clientY - start) / pitch);
        if (by === 0) return;
        move(item, by);
        start += by * pitch;
      };
      const drop = () => {
        delete line.dataset.lifted;
        window.removeEventListener("pointermove", follow);
        window.removeEventListener("pointerup", drop);
      };
      window.addEventListener("pointermove", follow);
      window.addEventListener("pointerup", drop);
    });
    rows.set(item, line);
  });
  box.append(...order());
  const setDisabled = (off: boolean) => box.toggleAttribute("data-disabled", off);
  setDisabled(options.disabled ?? false);
  probe(box, { role: "component", name: "OrderList", variant: "", fixed: { width: false, height: false } });
  return { box, setDisabled };
}
