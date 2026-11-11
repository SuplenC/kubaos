// Rows in a well: `kubaos-ui`'s `list::list`, `list::list_row`, `list::scrolling_list` and
// `list::empty_list`, `KubaOS.Components`' `List`, `ListRow`, `ListRows`, `ScrollingList` and
// `EmptyList` (`docs/design/components/list.md`). Its look is `lists.css`.
//
// The well is a content box, at the content radius, `space.s8` inside, its rows `space.s4` apart,
// so a row's corners are concentric inside it at the row radius; a well whose rows scroll pads them
// `space.s12`, its sides and its scroll padding alike. A row is `space.s16` in, so its
// words start `space.s24` from the well's edge.
//
// A row is `size.row` tall with one line and `size.row_two_line` with two: a leading symbol or
// tile, the title over its subtitle, then at its end the facts and controls it carries, `space.s12`
// apart. The row the keyboard is on is lit, on `secondary_container`, and the pointer moving onto a
// row puts the keyboard on it, so one row is lit at a time; a danger row lights in `error`. A
// picked row shows its tick, its title in `primary_lit`. A row that can't be taken is dimmed and is
// passed over.
//
// A scrolling list's rows scroll right up to the well's edge, cut by its rounded shape, the
// `space.s12` above the first row and below the last its scroll padding, and its thumb floats in
// the well's padding at its end.
//
// Between the rows a well may hold a section's label, a note, and a disclosure line over a drawer
// of rows; a row that can be forgotten keeps a quiet × in its tick's room; a list that reorders
// lifts the dragged row and draws a drop line where it lands; a list that adds marks its current
// row with a +.

import "./lists.css";
import { button } from "./button.ts";
import { type ChipKind, chip } from "./chip.ts";
import { iconButton } from "./icon-button.ts";
import { icon, missingIcon } from "./icon.ts";
import { iconTile } from "./icon-tile.ts";
import { keycap, keys } from "./keycap.ts";
import { filling, row, stack } from "./layout.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { skeleton } from "./skeleton.ts";
import { switchControl } from "./switch.ts";
import { focusRing } from "./focus.ts";
import { clamped, marked, type Run } from "./text.ts";
import { length } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** A chip near a row's end: a fact that matters at a glance. */
export interface RowChip {
  readonly kind: ChipKind;
  readonly label: string;
}

/**
 * How taking a row ends, one of three: it opens a page of its own (a chevron at its end), it is
 * picked (a tick, whose room is kept when unticked), or something simply happens (neither). A
 * row never has both a chevron and a tick: each says what pressing it does.
 */
export type RowTaking =
  | { readonly opens: true; readonly checked?: never }
  | { readonly opens?: false; readonly checked?: boolean };

export type ListRowSpec = RowTaking & {
  readonly title: string;
  /** The characters of the title a query matched, lit in `primary`: the Command menu's. */
  readonly matched?: readonly Run[];
  readonly subtitle?: string;
  /** A symbol before the subtitle, `small`'s: the lock of a secured network. */
  readonly subtitleIcon?: string;
  readonly icon?: string;
  /** The symbol on an icon tile: sm beside one line, md beside two. */
  readonly tile?: boolean;
  /** An app's own icon on the tile, `icon` its placeholder while it is missing. */
  readonly picture?: string;
  /** A photo covering the tile: a contact, an album. */
  readonly photo?: string;
  /** The tile's symbol in the accent: the row of the list that comes first, the default. */
  readonly accent?: boolean;
  /** Always two lines tall, a title alone included, so every row's words start in one column. */
  readonly tall?: boolean;
  /** `space.s16` after the tile, not `space.s12`, clear of an app's own icon. */
  readonly clear?: boolean;
  /** The subtitle right under the title, with no gap. */
  readonly tight?: boolean;
  readonly chips?: readonly RowChip[];
  /** A short fact at the end, in `mono`: a device path, a size. */
  readonly detail?: string;
  /** The keys that take it from anywhere, as `Ctrl+Enter`: one keycap each. */
  readonly keys?: string;
  /** A switch at its end, on or off: a setting the row turns. */
  readonly toggle?: boolean;
  /** Its action can't be taken back: Shut down. */
  readonly danger?: boolean;
  readonly disabled?: boolean;
  /** Still being found: a tiled two-line row's place, its parts as skeletons. */
  readonly loading?: boolean;
  /** How much of its words' room a loading row's title takes; its subtitle that much of it again. */
  readonly share?: number;
  /**
   * Saved, so it can be forgotten: a quiet × over its tick's room, shown while the row is lit or the
   * × is, and Delete on the row does the same. A network known, a device paired.
   */
  readonly forget?: () => void;
  /** Lifted by a drag, drawn at `opacity.subdued` so the row it passes over shows through. */
  readonly lifted?: boolean;
  /** Taking it adds it: a + in its tick's room while it is the current row. */
  readonly adds?: boolean;
};

/** A state a row is pinned in, as the specimens pin them: the keyboard on it, or pressed. */
export type RowPinned = "lit" | "pressed";

/** Whether `spec` shows two lines: a subtitle, a tall row, or a loading one. */
export function twoLines(spec: ListRowSpec): boolean {
  return spec.subtitle !== undefined || spec.tall === true || spec.loading === true;
}

/** The leading symbol or tile, or nothing. */
function lead(spec: ListRowSpec): Element | null {
  if (!spec.icon) return null;
  const two = twoLines(spec);
  if (spec.tile) {
    const tile = iconTile({
      glyph: spec.icon,
      size: two ? "Md" : "Sm",
      ...(spec.picture ? { picture: spec.picture } : {}),
      ...(spec.photo ? { photo: spec.photo } : {}),
      ...(spec.accent ? { accent: true } : {}),
    });
    tile.classList.add("k-list-lead");
    return tile;
  }
  const size = two ? "var(--icon-two_line)" : "var(--type-body_strong-icon)";
  const glyph = spec.icon === "missing" ? missingIcon(spec.icon, size) : icon(spec.icon, size, "var(--ink)");
  glyph.classList.add("k-list-lead");
  return glyph;
}

/** The title over its subtitle. */
function words(spec: ListRowSpec): HTMLElement {
  const title = spec.matched ? marked(spec.title, "body_strong", "on_surface", spec.matched) : clamped(spec.title, "body_strong");
  if (spec.matched) title.dataset.clamped = "line";
  title.classList.add("k-list-title");
  title.style.color = "var(--title)";
  const lines: Element[] = [title];
  if (spec.subtitle !== undefined) {
    const subtitle = clamped(spec.subtitle, "small");
    subtitle.style.color = "var(--muted)";
    if (spec.subtitleIcon) {
      const glyph = icon(spec.subtitleIcon, "var(--type-small-icon)", "var(--muted)");
      lines.push(filling(row("s4", "centre", glyph, filling(subtitle))));
    } else lines.push(subtitle);
  }
  return filling(stack(spec.tight ? 0 : "s4", "start", ...lines));
}

/** The parts at a row's end, in order: chips, the detail, keys, a switch, then a chevron or a tick. */
function ends(spec: ListRowSpec): Element[] {
  const parts: Element[] = [];
  for (const each of spec.chips ?? []) parts.push(chip({ label: each.label, kind: each.kind }));
  if (spec.detail !== undefined) {
    const detail = clamped(spec.detail, "mono");
    detail.classList.add("k-list-detail");
    detail.style.color = "var(--muted)";
    parts.push(detail);
  }
  if (spec.keys) {
    const caps = row("s4", "centre", ...keys(spec.keys).map(keycap));
    caps.classList.add("k-list-keys");
    parts.push(caps);
  }
  if (spec.toggle !== undefined) {
    // The row is the control: its switch shows the setting and takes no stop or press of its own.
    const toggle = switchControl({ on: spec.toggle, ...(spec.disabled ? { disabled: true } : {}) });
    toggle.tabIndex = -1;
    toggle.classList.add("k-list-switch");
    parts.push(toggle);
  }
  if (spec.opens) parts.push(icon("chevron_right", "var(--type-body_strong-icon)", "var(--muted)"));
  else if (spec.checked !== undefined || spec.forget || spec.adds) {
    const room = document.createElement("span");
    room.className = "k-list-check";
    if (spec.checked) room.append(icon("check", "var(--type-body_strong-icon)", "var(--tick)"));
    else if (spec.adds) {
      const mark = icon("add", "var(--type-body_strong-icon)", "var(--title)");
      mark.classList.add("k-list-add");
      room.append(mark);
    } else if (spec.forget) room.append(forgetButton(spec.forget));
    parts.push(room);
  }
  return parts;
}

/**
 * The × that forgets a saved row: a Text sm icon button centred on the tick's room, quiet until the
 * row or the × itself is lit, fading over `micro`. Its press is its own, not the row's.
 */
function forgetButton(forget: () => void): HTMLElement {
  const made = iconButton({ icon: "close", label: "Forget", kind: "Text", size: "Sm", hint: false });
  made.classList.add("k-list-forget");
  made.tabIndex = -1;
  made.addEventListener("click", (event) => {
    event.stopPropagation();
    forget();
  });
  return made;
}

/** A loading row's skeletons: the tile, then its title and subtitle as shares of their room. */
function bones(spec: ListRowSpec): HTMLElement {
  const share = spec.share ?? 0.6;
  const tile = skeleton({ bone: "block", size: "control_md", radius: "control" });
  tile.classList.add("k-list-bone-tile");
  const title = skeleton({ bone: "line", style: "body_strong" });
  title.style.setProperty("--share", String(share));
  const subtitle = skeleton({ bone: "line", style: "small" });
  subtitle.style.setProperty("--share", String(share * share));
  for (const each of [title, subtitle]) each.classList.add("k-list-bone-line");
  return filling(row("s12", "centre", tile, filling(stack("s4", "start", title, subtitle))));
}

/** One row, the keyboard on it when `current`, or pinned in a state whatever the pointer does. */
export function listRow(spec: ListRowSpec, current = false, pinned?: RowPinned): HTMLElement {
  const item = document.createElement("span");
  item.className = "k-list-row";
  item.setAttribute("role", "option");
  item.dataset.lines = twoLines(spec) ? "two" : "one";
  item.toggleAttribute("data-danger", spec.danger ?? false);
  item.toggleAttribute("data-picked", spec.checked === true);
  item.toggleAttribute("data-clear", (spec.clear ?? false) && spec.tile === true);
  item.toggleAttribute("data-lifted", spec.lifted ?? false);
  item.toggleAttribute("data-forget", spec.forget !== undefined);
  const inert = spec.disabled === true || spec.loading === true;
  item.toggleAttribute("data-current", current && !inert);
  if (pinned && !inert) item.dataset.pinned = pinned;
  if (spec.disabled) item.setAttribute("aria-disabled", "true");
  if (spec.loading) {
    item.dataset.loading = "";
    item.setAttribute("aria-busy", "true");
  }
  if (spec.checked !== undefined) item.setAttribute("aria-selected", String(spec.checked));
  if (spec.toggle !== undefined) item.setAttribute("aria-checked", String(spec.toggle));
  if (spec.loading) item.append(bones(spec));
  else {
    const first = lead(spec);
    item.append(filling(row("s12", "centre", ...(first ? [first] : []), words(spec), ...ends(spec))));
  }
  if (spec.toggle !== undefined && !inert) {
    item.addEventListener("click", () => {
      const on = item.getAttribute("aria-checked") !== "true";
      item.setAttribute("aria-checked", String(on));
      item.querySelector<HTMLElement>(".k-switch")?.click();
    });
  }
  const shape = spec.loading ? "two-line" : twoLines(spec) ? "two-line" : "one-line";
  const symbol = spec.icon ? (spec.tile ? "tile" : "icon") : undefined;
  return probe(item, { role: "component", name: "ListRow", variant: variant(shape, symbol, spec.danger && "danger", spec.loading && "loading"), fixed: FIXED_HEIGHT });
}

/** Whether a row can take the keyboard: one that isn't disabled, loading or a note. */
export function takes(element: Element | undefined): element is HTMLElement {
  if (element instanceof HTMLElement && element.classList.contains("k-list-disclosure")) return true;
  return element instanceof HTMLElement && element.classList.contains("k-list-row") && element.getAttribute("aria-disabled") !== "true" && !element.hasAttribute("data-loading");
}

export interface ListOptions {
  /** The row the keyboard is on, by its index; none until the list takes the keyboard. */
  readonly current?: number;
  /** The current row pinned in a state, whatever the pointer does. */
  readonly pinned?: RowPinned;
  /** The well's edge pinned lit, as the list with the keyboard. */
  readonly focused?: boolean;
  /** The selection is the list's, drawn behind its rows and sliding between them: the Command menu. */
  readonly slides?: boolean;
  /** Plain j and k move too; off where a field forwards its letters to the list. */
  readonly vimKeys?: boolean;
  /** Home and End are a text field's, which forwards its other keys to the list. */
  readonly fieldKeys?: boolean;
  /** What taking a row does, by its index among the rows. */
  readonly onTake?: (index: number) => void;
}

/** A list's well: the rows' room, with its hairline and focus tube drawn over what it holds. */
export function well(name: string, ...content: Element[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-list k-sunken";
  box.append(...content);
  const edge = document.createElement("span");
  edge.className = "k-list-edge";
  box.append(edge);
  focusRing(box, "content", "accent", false);
  return probe(box, { role: "component", name, variant: "", fixed: { width: false, height: false } });
}

/** The rows `space.s4` apart, as the well holds them. */
function rowStack(rows: readonly HTMLElement[]): HTMLElement {
  const column = stack("s4", "start", ...rows);
  column.classList.add("k-list-rows");
  return column;
}

/** Where a list's keyboard and pointer are, for an owner whose rows change under them. */
export interface Rows {
  /** The row the keyboard is on, if any. */
  readonly current: () => HTMLElement | null;
  /** Whether the pointer is over the list. */
  readonly pointed: () => boolean;
  /** Shows the current row again, after the rows changed. */
  readonly refresh: () => void;
  /** `fresh` takes `old`'s place, and the keyboard with it if it was there. */
  readonly swap: (old: HTMLElement, fresh: HTMLElement) => void;
}

/**
 * The keys and the pointer on the rows `rowsOf` gives, in `box`, which takes the keyboard
 * (`ListRows`): ↑ ↓ (j k, Ctrl+J/K/N/P) move over the rows that can be taken, round from either end;
 * Home and End go to the first and the last; Page Up and Down by as many rows as show whole, held at
 * the ends; Enter takes one, once per press. The pointer moving onto a row puts the keyboard on it.
 * `pane` is what scrolls, if anything: the row reached is kept in view, the well's padding clear of
 * its edge. The keyboard stays on its row, not its place, as the rows change.
 */
export function listRows(box: HTMLElement, pane: HTMLElement | null, rowsOf: () => readonly HTMLElement[], options: ListOptions): Rows {
  box.tabIndex = 0;
  box.setAttribute("role", "listbox");
  let at: HTMLElement | null = rowsOf()[options.current ?? -1] ?? null;
  let pointer = false;
  const highlight = options.slides ? slider(box, rowsOf()) : null;
  const index = () => (at ? rowsOf().indexOf(at) : -1);

  /** Shows the current row lit while the list has the keyboard or the pointer. */
  function show(): void {
    const shown = pointer || box.matches(":focus") || options.current !== undefined;
    for (const each of rowsOf()) each.toggleAttribute("data-current", shown && each === at);
    highlight?.(shown ? index() : -1);
  }

  function move(to: number): void {
    const item = rowsOf()[to];
    if (!item) return;
    at = item;
    show();
    if (!pane) return;
    const inset = length("space-s12") * scaleOf(box);
    const top = offsetIn(item, pane) - inset;
    const bottom = offsetIn(item, pane) + item.offsetHeight + inset;
    if (top < pane.scrollTop) pane.scrollTop = top;
    else if (bottom > pane.scrollTop + pane.clientHeight) pane.scrollTop = bottom - pane.clientHeight;
  }

  function step(by: 1 | -1): void {
    const rows = rowsOf();
    for (let i = 1; i <= rows.length; i++) {
      const to = (((index() + by * i) % rows.length) + rows.length) % rows.length;
      if (takes(rows[to])) return move(to);
    }
  }

  /** To row `to` held to the ends, or the nearest after it that can be taken, else before. */
  function jump(to: number): void {
    const rows = rowsOf();
    const from = Math.max(0, Math.min(rows.length - 1, to));
    for (let i = from; i < rows.length; i++) if (takes(rows[i])) return move(i);
    for (let i = from - 1; i >= 0; i--) if (takes(rows[i])) return move(i);
  }

  /** As many rows as show whole in the room the rows scroll in, at least one. */
  function page(): number {
    const rows = rowsOf();
    const item = rows[Math.max(0, index())];
    if (!item || !pane) return rows.length;
    const gap = length("space-s4") * scaleOf(box);
    const room = pane.clientHeight - 2 * length("space-s12") * scaleOf(box);
    return Math.max(1, Math.floor((room + gap) / (item.offsetHeight + gap)));
  }

  function take(): void {
    if (!takes(at ?? undefined) || !at) return;
    const item = at;
    item.dataset.keyed = "";
    setTimeout(() => delete item.dataset.keyed, 100);
    item.click();
  }

  /** The row of the list under `event`, if it can be taken. */
  function rowAt(event: Event): HTMLElement | null {
    const item = (event.target as Element).closest<HTMLElement>(".k-list-row, .k-list-disclosure");
    return item && takes(item) && rowsOf().includes(item) ? item : null;
  }

  box.addEventListener("pointermove", (event) => {
    pointer = true;
    const item = rowAt(event);
    if (item && item !== at) move(rowsOf().indexOf(item));
  });
  box.addEventListener("click", (event) => {
    const item = rowAt(event);
    if (item) options.onTake?.(rowsOf().indexOf(item));
  });
  box.addEventListener("pointerleave", () => {
    pointer = false;
    show();
  });
  box.addEventListener("keydown", (event) => {
    if (event.altKey) return;
    const ctrl = event.ctrlKey;
    const key = event.key.toLowerCase();
    const plain = (options.vimKeys ?? true) && !ctrl;
    if (event.key === "ArrowDown" || (plain && key === "j") || (ctrl && (key === "j" || key === "n"))) step(1);
    else if (event.key === "ArrowUp" || (plain && key === "k") || (ctrl && (key === "k" || key === "p"))) step(-1);
    else if (!options.fieldKeys && event.key === "Home") jump(0);
    else if (!options.fieldKeys && event.key === "End") jump(rowsOf().length - 1);
    else if (event.key === "PageDown") jump(index() + page());
    else if (event.key === "PageUp") jump(Math.max(0, index()) - page());
    else if (event.key === "Enter") {
      if (!event.repeat) take();
    } else return;
    event.preventDefault();
  });
  box.addEventListener("focus", () => {
    if (!takes(at ?? undefined) || !rowsOf().includes(at!)) {
      const picked = rowsOf().findIndex((each) => each.hasAttribute("data-picked"));
      jump(picked >= 0 ? picked : 0);
    } else move(index());
  });
  box.addEventListener("blur", show);
  if (options.focused) box.dataset.pinned = "lit";
  requestAnimationFrame(() => (at ? move(index()) : show()));
  const swap = (old: HTMLElement, fresh: HTMLElement) => {
    if (at === old) at = fresh;
    show();
  };
  return { current: () => at, pointed: () => pointer, refresh: show, swap };
}

/**
 * The selection drawn behind `rows` as one shape that slides between them over `short` on
 * `standard`: the Command menu's, which moves with the keys while the rows stay put. Returns what
 * moves it onto a row's index, or hides it at -1.
 */
function slider(box: HTMLElement, rows: readonly HTMLElement[]): (index: number) => void {
  box.dataset.slides = "";
  const shape = document.createElement("span");
  shape.className = "k-list-highlight";
  rows[0]?.parentElement?.prepend(shape);
  let shown = false;
  return (index) => {
    const item = rows[index];
    shape.toggleAttribute("data-shown", item !== undefined);
    if (!item) {
      shown = false;
      return;
    }
    // Arriving, it is on its row at once; only a move between rows slides.
    shape.toggleAttribute("data-still", !shown);
    // The row's own box, so the shape keeps the inset the rows have from the well's edges.
    shape.style.setProperty("--top", `${item.offsetTop}px`);
    shape.style.setProperty("--left", `${item.offsetLeft}px`);
    shape.style.setProperty("--width", `${item.offsetWidth}px`);
    shape.style.setProperty("--height", `${item.offsetHeight}px`);
    shape.toggleAttribute("data-danger", item.hasAttribute("data-danger"));
    shown = true;
  };
}

/**
 * `item`'s top in `pane`'s content, in the page's pixels: a live row's room is transformed, which
 * makes it the row's offset parent, so the offsets are added up to the pane.
 */
function offsetIn(item: HTMLElement, pane: HTMLElement): number {
  let top = 0;
  for (let at: HTMLElement | null = item; at && at !== pane; at = at.offsetParent as HTMLElement | null) top += at.offsetTop;
  return top;
}

/** The page's scale at `element`: its CSS pixels, which are device pixels, per logical pixel. */
export function scaleOf(element: Element): number {
  return Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || settings().scale;
}

/** `rows` in a well, as tall as they are: `List` holding `ListRows`. */
/**
 * What a well holds: a row, or one of the parts between rows (`listLabel`, `listNote`,
 * `listDisclosure`, `dropLine`), which the keys pass over but for a disclosure line.
 */
export type ListItem = ListRowSpec | HTMLElement;

/** `items` drawn: each spec a row, the keyboard on the one at `current`; a part as it is. */
function itemsOf(items: readonly ListItem[], options: ListOptions): HTMLElement[] {
  return items.map((item, index) => {
    if (item instanceof HTMLElement) return item;
    return listRow(item, index === options.current, index === options.current ? options.pinned : undefined);
  });
}

export function list(specs: readonly ListItem[], options: ListOptions = {}): HTMLElement {
  const rows = itemsOf(specs, options);
  const box = well("List", rowStack(rows));
  listRows(box, null, () => rows, options);
  return box;
}

export interface ScrollingOptions extends ListOptions {
  /** How many rows tall the well is, by its rows' height token: `row` or `row_two_line`. */
  readonly height: string;
  /** Scrolled this share of the way down, as a sample of it scrolled. */
  readonly down?: number;
  /** The thumb pinned as under the pointer. */
  readonly thumbPinned?: boolean;
}

/**
 * `rows` scrolling in a well `options.height` tall (a length's CSS): `ScrollingList` holding
 * `ListRows`. The rows run up to the well's edge, cut by its own rounded shape; the thumb floats in
 * the well's padding at its end while they are taller than it.
 */
export function scrollingList(specs: readonly ListItem[], options: ScrollingOptions): HTMLElement {
  const rows = itemsOf(specs, options);
  const { pane, thumb, box } = scrolled("ScrollingList", rowStack(rows), options);
  listRows(box, pane, () => rows, options);
  if (options.down !== undefined) {
    const down = options.down;
    requestAnimationFrame(() => {
      pane.scrollTop = Math.round((pane.scrollHeight - pane.clientHeight) * down);
      thumb();
    });
  }
  return box;
}

/**
 * `content` scrolling in a well, its wheel, thumb and height wired: what a scrolling list and a
 * live one share. Returns the well, what scrolls, and what places the thumb.
 */
export function scrolled(name: string, content: HTMLElement, options: { readonly height: string; readonly thumbPinned?: boolean }): { box: HTMLElement; pane: HTMLElement; thumb: () => void } {
  const pane = document.createElement("span");
  pane.className = "k-list-pane";
  pane.append(content);
  const rail = document.createElement("span");
  rail.className = "k-list-thumb";
  rail.toggleAttribute("data-pinned", options.thumbPinned ?? false);
  const box = well(name, pane, rail);
  box.dataset.scrolls = "";
  box.style.setProperty("--height", options.height);

  /** Places the thumb along the well's padding-inset rail, in whole device pixels. */
  function place(): void {
    const overflow = pane.scrollHeight - pane.clientHeight;
    rail.toggleAttribute("data-shown", overflow > 1);
    if (overflow <= 1) return;
    const inset = length("space-s8") * scaleOf(box);
    const room = pane.clientHeight - 2 * inset;
    const tall = Math.max(Math.round((room * pane.clientHeight) / pane.scrollHeight), Math.round(length("space-s24") * scaleOf(box)));
    const top = Math.round(inset + ((room - tall) * pane.scrollTop) / overflow);
    rail.style.setProperty("--top", `${top}px`);
    rail.style.setProperty("--tall", `${tall}px`);
  }

  pane.addEventListener("scroll", place, { passive: true });
  new ResizeObserver(place).observe(content);
  wheel(pane, box);
  drag(rail, pane, box);
  requestAnimationFrame(place);
  return { box, pane, thumb: place };
}

/** A wheel's notch, as the toolkits count it, and the least a single event turns for a notch. */
const NOTCH = 100;
const CLICK = 40;

/**
 * The wheel over `pane`: a mouse's notch scrolls a row and its gap (`size.row` and `space.s4`) at
 * once, a touchpad's glide follows the fingers pixel for pixel. The page's box is zoomed to the
 * display's ratio, which shrinks the deltas the browser reports by that ratio, so they are scaled
 * back to device pixels first.
 */
function wheel(pane: HTMLElement, box: HTMLElement): void {
  pane.addEventListener(
    "wheel",
    (event) => {
      if (event.deltaY === 0 || event.ctrlKey) return;
      const delta = event.deltaMode === WheelEvent.DOM_DELTA_PIXEL ? event.deltaY * devicePixelRatio : Math.sign(event.deltaY) * NOTCH;
      const before = pane.scrollTop;
      if (Math.abs(delta) >= CLICK) {
        const notches = Math.sign(delta) * Math.max(1, Math.round(Math.abs(delta) / NOTCH));
        pane.scrollTop += notches * (length("size-row") + length("space-s4")) * scaleOf(box);
      } else pane.scrollTop += delta;
      // At an end, the page scrolls on instead.
      if (pane.scrollTop !== before) event.preventDefault();
    },
    { passive: false },
  );
}

/** The thumb dragged: the rows follow it, the pointer's travel scaled from the viewport's pixels. */
function drag(thumb: HTMLElement, pane: HTMLElement, box: HTMLElement): void {
  thumb.addEventListener("pointerdown", (event) => {
    event.preventDefault();
    thumb.setPointerCapture(event.pointerId);
    thumb.dataset.dragged = "";
    const ratio = box.offsetHeight / Math.max(1, box.getBoundingClientRect().height);
    const from = event.clientY;
    const start = pane.scrollTop;
    const room = pane.clientHeight - 2 * length("space-s8") * scaleOf(box) - thumb.offsetHeight;
    const travel = pane.scrollHeight - pane.clientHeight;
    const follow = (moved: PointerEvent) => {
      pane.scrollTop = start + (((moved.clientY - from) * ratio) / Math.max(1, room)) * travel;
    };
    const end = () => {
      delete thumb.dataset.dragged;
      thumb.removeEventListener("pointermove", follow);
    };
    thumb.addEventListener("pointermove", follow);
    thumb.addEventListener("pointerup", end, { once: true });
    thumb.addEventListener("pointercancel", end, { once: true });
  });
}

/** The well with nothing in it: `message` centred on one line. */
export function emptyList(message: string): HTMLElement {
  const words = clamped(message, "body", "on_surface_variant");
  words.classList.add("k-list-empty-words");
  const room = document.createElement("span");
  room.className = "k-list-empty";
  room.append(words);
  return well("EmptyList", room);
}

/**
 * A section's label among a well's rows: `label` in `on_surface_variant`, `space.s12` in as a row's
 * words are; after rows, `space.s12` clear of them besides the `space.s4` they carry. The keys pass
 * over it.
 */
export function listLabel(words: string, afterRows: boolean): HTMLElement {
  const label = document.createElement("span");
  label.className = "k-list-label";
  label.toggleAttribute("data-after", afterRows);
  label.append(clamped(words, "label", "on_surface_variant"));
  return probe(label, { role: "component", name: "ListLabel", variant: afterRows ? "after-rows" : "", fixed: { width: false, height: true } });
}

/**
 * A note among a well's rows, where rows would be: one line of `body` a one-line row tall, its words
 * `space.s12` in; `on_surface_variant`, or `error` when it says something failed. The keys pass
 * over it.
 */
export function listNote(words: string, failed = false): HTMLElement {
  const line = document.createElement("span");
  line.className = "k-list-note";
  line.append(clamped(words, "body", failed ? "error" : "on_surface_variant"));
  return probe(line, { role: "component", name: "ListNote", variant: failed ? "failed" : "", fixed: { width: false, height: true } });
}

export interface DisclosureOptions {
  /** The drawer under it open: the button says Hide, its chevron turned. */
  readonly open: boolean;
  /** What a press, or Enter with the keys on the line, does: opens or closes the drawer. */
  readonly onToggle?: (open: boolean) => void;
}

/**
 * A disclosure line over a drawer of rows: a note's words, then a Text sm button at the line's end
 * saying Show or Hide, holding the wider label's width so it never moves, its chevron turning with
 * the drawer. The keys reach it as a row's, lighting its button.
 */
export function listDisclosure(words: string, options: DisclosureOptions): HTMLElement {
  let open = options.open;
  const line = document.createElement("span");
  line.className = "k-list-disclosure";
  line.setAttribute("role", "option");
  const toggle = button({ label: open ? "Hide" : "Show", kind: "Text", size: "Sm", alternatives: ["Show", "Hide"], disclosure: open });
  toggle.tabIndex = -1;
  line.append(clamped(words, "body", "on_surface_variant"), toggle);
  line.addEventListener("click", () => {
    open = !open;
    toggle.style.setProperty("--disclosed", open ? "1" : "0");
    for (const label of toggle.querySelectorAll<HTMLElement>(".k-button-label")) label.textContent = open ? "Hide" : "Show";
    options.onToggle?.(open);
  });
  return probe(line, { role: "component", name: "ListDisclosure", variant: open ? "open" : "", fixed: { width: false, height: true } });
}

/**
 * Where a dragged row lands: a pill `stroke.thin` thick in `primary`, in the gap before the row it
 * goes before, inset `space.s12` as the rows' words are.
 */
export function dropLine(): HTMLElement {
  const line = document.createElement("span");
  line.className = "k-list-drop";
  return probe(line, { role: "component", name: "ListDropLine", variant: "", fixed: { width: false, height: true } });
}
