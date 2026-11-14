// A quick toggle: `kubaos-ui`'s `tile::tile`, `KubaOS.Components`' `Tile` (`docs/design/components/
// tile.md`). A symbol, a title and a status line, as tall as a two-line row (`size.row_two_line`) at
// the control radius, `space.s12` inside, filling its room: the control center's Wi-Fi, Bluetooth
// and the rest, two to a row. A press, Enter or Space toggles it.
//
// It is a control in the state model (`.k-control` in `lib.css`), lit as a switch is: off, on
// `surface_container_high`, its line the tube; on, filled with the accent, at night the unlit fill
// (`primary_fill`) with its content `on_primary_fill`, its line the core inside the fill's edge.
//
// A tile with more behind it ends in a detail arrow: a small control that stays quiet, a circle
// `size.control_sm` across `space.s12` from the tile's end, a 16 chevron in the tile's content
// colour, no fill. It lights and presses on its own, and the tile gives way while the pointer is on
// it. → opens it from the keyboard; it is no stop for Tab.

import { type Pinned, pin } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { clamped } from "./text.ts";

export interface TileOptions {
  readonly glyph: string;
  readonly title: string;
  /** The status line under the title: the network, "Off", "2 connected"; none centres the title. */
  readonly subtitle?: string;
  readonly on?: boolean;
  /** It has more behind it: the detail arrow at its end, and what it opens. */
  readonly detail?: () => void;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
  /** The detail arrow pinned in a state, whatever the pointer does. */
  readonly detailPinned?: Pinned;
  /** What toggling does besides turning it: the status line it shows now, if that changes. */
  readonly onToggle?: (on: boolean) => string | void;
}

/** The detail arrow: a quiet circle with a chevron, lit and pressed on its own. */
function detailArrow(open: () => void, pinned: Pinned | undefined): HTMLElement {
  const arrow = document.createElement("span");
  arrow.className = "k-tile-detail k-control";
  arrow.dataset.hot = "";
  arrow.setAttribute("role", "button");
  pin(arrow, pinned);
  const slot = document.createElement("span");
  slot.className = "k-icon-slot";
  slot.style.setProperty("--height", "var(--size-control_sm)");
  slot.append(icon("chevron_right", "var(--type-button_small-icon)", "var(--glyph)"));
  arrow.append(slot);
  focusRing(arrow, "full", "accent", false).dataset.pressable = "";
  arrow.addEventListener("click", (event) => {
    event.stopPropagation();
    open();
  });
  return probe(arrow, { role: "component", name: "IconButton", variant: "Text Sm", fixed: { width: true, height: true } });
}

export function tile(options: TileOptions): HTMLElement {
  let on = options.on ?? false;
  const disabled = options.disabled ?? false;
  const element = document.createElement("span");
  element.className = "k-tile k-control";
  element.setAttribute("role", "switch");
  element.tabIndex = disabled ? -1 : 0;
  if (disabled) element.setAttribute("aria-disabled", "true");
  pin(element, options.pinned);

  const status = clamped(options.subtitle ?? "", "small");
  status.style.color = "var(--muted)";
  const title = clamped(options.title, "body_strong");
  title.style.color = "var(--title)";
  const words = stack("s4", "start", title, ...(options.subtitle === undefined ? [] : [status]));
  const glyph = icon(options.glyph, "var(--type-body_strong-icon)", "var(--glyph)");
  element.append(row("s12", "centre", glyph, filling(words)));
  const ring = focusRing(element, "control", "accent", on);
  ring.dataset.pressable = "";
  if (options.detail) {
    element.dataset.detail = "";
    element.append(detailArrow(options.detail, options.detailPinned));
  }

  /** Shows `on`: its fill, content and line. */
  function turn(): void {
    element.setAttribute("aria-checked", String(on));
    element.style.setProperty("--checked", on ? "1" : "0");
    ring.toggleAttribute("data-filled", on);
  }

  function toggle(): void {
    if (disabled) return;
    on = !on;
    turn();
    const said = options.onToggle?.(on);
    if (typeof said === "string") status.textContent = said;
  }

  element.addEventListener("click", toggle);
  element.addEventListener("keydown", (event) => {
    if (disabled) return;
    if (event.key === "Enter" || event.key === " ") {
      if (!event.repeat) toggle();
    } else if (event.key === "ArrowRight" && options.detail) options.detail();
    else return;
    event.preventDefault();
  });
  turn();
  return probe(element, { role: "component", name: "Tile", variant: variant(options.detail && "detail"), fixed: FIXED_HEIGHT });
}
