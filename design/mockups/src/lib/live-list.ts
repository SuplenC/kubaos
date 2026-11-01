// A list a scan updates while it shows, which never jumps: `KubaOS.Components`' `LiveList` and
// `LiveRow` (`docs/design/components/list.md`), in a scrolling list's well. Its rows are kept by
// their key, so only what changed changes: a row that arrives opens its room over `medium` on
// `standard_decel` and fades in over `slow`; one that leaves fades and gives its room back over
// `medium` on `standard_accel`, taking no press while it goes; one that changes place glides there
// over `medium` on `standard_decel`. With reduced motion a row arrives in its room and fades in over
// `short`, leaves by fading over `short` and giving its room back at once, and changes place at once.
//
// The row under the pointer, or the one the keyboard is on while the list has it, is held still:
// an update that would move it plays with the list scrolling by the same amount as the rows above
// it open or close, so it stays where it is on screen; where the list can't scroll that far (it
// fits its well, or is at its top), or the row itself would change place, the update waits until
// the pointer leaves and the keyboard does, then plays.

import { type ListOptions, type ListRowSpec, listRow, listRows, scaleOf, scrolled, twoLines } from "./list.ts";
import { length } from "./tokens.ts";

export interface LiveRowSpec extends ListRowSpec {
  /** What the row stands for across updates: a network's name, a device's address. */
  readonly key: string;
}

export interface LiveOptions extends ListOptions {
  /** The well's height, a length's CSS. */
  readonly height: string;
}

/** A live list, and what updates it. */
export interface LiveList {
  readonly element: HTMLElement;
  /** Brings the list to `rows`, now or once the row held still is let go. */
  readonly update: (rows: readonly LiveRowSpec[]) => void;
  /** Called with whether an update is waiting for the held row to be let go. */
  onWaiting: (waiting: boolean) => void;
}

/** A row's room: the row and the `space.s4` after it, which it gives back with it as it leaves. */
function liveRow(spec: LiveRowSpec): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-live-row";
  room.dataset.key = spec.key;
  room.style.setProperty("--row-height", twoLines(spec) ? "var(--size-row_two_line)" : "var(--size-row)");
  room.append(listRow(spec));
  return room;
}

/** The height a row's room takes in the list once it has settled, in the page's pixels. */
function settled(spec: LiveRowSpec, scale: number): number {
  return (length(twoLines(spec) ? "size-row_two_line" : "size-row") + length("space-s4")) * scale;
}

export function liveList(initial: readonly LiveRowSpec[], options: LiveOptions): LiveList {
  const column = document.createElement("span");
  column.className = "k-live-rows";
  let specs = [...initial];
  const rooms = new Map<string, HTMLElement>();
  for (const spec of specs) {
    const room = liveRow(spec);
    rooms.set(spec.key, room);
    column.append(room);
  }
  const { box, pane } = scrolled("ScrollingList", column, { height: options.height });
  box.classList.add("k-live");
  const rows = () => [...column.querySelectorAll<HTMLElement>(".k-live-row:not([data-leaving]) > .k-list-row")];
  const keys = listRows(box, pane, rows, options);
  let waiting: readonly LiveRowSpec[] | null = null;

  const live: LiveList = { element: box, update, onWaiting: () => undefined };

  /** The row held still, if any: under the pointer, else the keyboard's while the list has it. */
  function held(): HTMLElement | null {
    if (!keys.pointed() && !box.matches(":focus")) return null;
    return keys.current()?.parentElement ?? null;
  }

  function reduced(): boolean {
    return box.closest("[data-reduced]") !== null;
  }

  function update(next: readonly LiveRowSpec[]): void {
    const anchor = held();
    const scale = scaleOf(box);
    if (anchor) {
      const key = anchor.dataset.key!;
      const before = specs.findIndex((each) => each.key === key);
      const after = next.findIndex((each) => each.key === key);
      const above = (list: readonly LiveRowSpec[], end: number) => list.slice(0, Math.max(0, end)).reduce((sum, each) => sum + settled(each, scale), 0);
      const kept = (list: readonly LiveRowSpec[]) => list.filter((each) => specs.some((old) => old.key === each.key) && next.some((now) => now.key === each.key)).map((each) => each.key);
      const moved = after >= 0 && kept(specs).indexOf(key) !== kept(next).indexOf(key);
      const shift = after < 0 ? 0 : above(next, after) - above(specs, before);
      const total = (list: readonly LiveRowSpec[]) => list.reduce((sum, each) => sum + settled(each, scale), 0);
      const height = pane.scrollHeight + total(next) - total(specs);
      const room = pane.scrollTop + shift;
      if (moved || after < 0 || (shift !== 0 && (room < 0 || room > height - pane.clientHeight))) {
        hold(next);
        return;
      }
      play(next);
      if (shift !== 0) follow(anchor);
      return;
    }
    play(next);
  }

  function hold(next: readonly LiveRowSpec[]): void {
    waiting = next;
    live.onWaiting(true);
  }

  /** Plays what waited, once nothing is held. */
  function release(): void {
    if (!waiting || held()) return;
    const next = waiting;
    waiting = null;
    live.onWaiting(false);
    play(next);
  }

  /** Scrolls the list with `anchor` while the rows above it open and close, so it stays still. */
  function follow(anchor: HTMLElement): void {
    const from = anchor.offsetTop - pane.scrollTop;
    const until = performance.now() + 1000 * 2;
    const frame = () => {
      pane.scrollTop = anchor.offsetTop - from;
      if (performance.now() < until && column.querySelector("[data-moving]")) requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  }

  /** Brings the rows to `next`: the fewest removes, inserts and moves, each with its motion. */
  function play(next: readonly LiveRowSpec[]): void {
    const wanted = new Set(next.map((each) => each.key));
    const keyed = keys.current()?.parentElement?.dataset.key;
    const tops = new Map<string, number>();
    for (const [key, room] of rooms) tops.set(key, room.offsetTop);
    for (const [key, room] of rooms) {
      if (wanted.has(key)) continue;
      rooms.delete(key);
      leave(room);
    }
    let after: Element | null = null;
    for (const spec of next) {
      let room = rooms.get(spec.key);
      if (!room) {
        room = liveRow(spec);
        rooms.set(spec.key, room);
        arrive(room);
      } else if (JSON.stringify(specs.find((each) => each.key === spec.key)) !== JSON.stringify(spec)) {
        const old = room.firstElementChild as HTMLElement;
        const fresh = listRow(spec);
        room.replaceChildren(fresh);
        keys.swap(old, fresh);
      }
      // Each row goes after the one before it in `next`, a leaving row left where it was.
      const place: ChildNode | null = after ? after.nextSibling : column.firstChild;
      let skip: ChildNode | null = place;
      while (skip instanceof HTMLElement && skip.hasAttribute("data-leaving") && skip !== room) skip = skip.nextSibling;
      if (skip !== room) column.insertBefore(room, skip);
      after = room;
    }
    for (const [key, room] of rooms) {
      const was = tops.get(key);
      if (was === undefined || room.hasAttribute("data-arriving")) continue;
      glide(room, was - room.offsetTop);
    }
    if (keyed !== undefined && !wanted.has(keyed)) {
      const old = specs.findIndex((each) => each.key === keyed);
      const heir = next.find((each, index) => index >= old && specs.some((kept) => kept.key === each.key)) ?? next[next.length - 1];
      const room = heir ? rooms.get(heir.key) : undefined;
      if (room?.firstElementChild instanceof HTMLElement && keys.current()) keys.swap(keys.current()!, room.firstElementChild);
    }
    specs = [...next];
    keys.refresh();
  }

  function arrive(room: HTMLElement): void {
    room.dataset.arriving = "";
    room.dataset.opening = "";
    room.dataset.moving = "";
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        delete room.dataset.arriving;
        settle(room);
      }),
    );
  }

  function leave(room: HTMLElement): void {
    room.dataset.leaving = "";
    room.dataset.moving = "";
    room.firstElementChild?.removeAttribute("data-current");
    // With reduced motion only its fade is a move; its room goes once it has faded.
    const last = reduced() ? "opacity" : "--grow";
    room.addEventListener("transitionend", (event) => {
      if (event.target === room && event.propertyName === last) room.remove();
    });
  }

  /** A kept row moved `by` pixels: it starts where it was and glides to its place. */
  function glide(room: HTMLElement, by: number): void {
    if (by === 0 || reduced()) return;
    room.dataset.still = "";
    room.style.setProperty("--from-y", `${by}px`);
    room.style.setProperty("--glide", "1");
    room.dataset.moving = "";
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        delete room.dataset.still;
        room.style.setProperty("--glide", "0");
        settle(room);
      }),
    );
  }

  function settle(room: HTMLElement): void {
    const end = (event: TransitionEvent) => {
      if (event.target !== room) return;
      room.removeEventListener("transitionend", end);
      delete room.dataset.moving;
      delete room.dataset.opening;
    };
    room.addEventListener("transitionend", end);
  }

  box.addEventListener("pointerleave", () => requestAnimationFrame(release));
  box.addEventListener("blur", () => requestAnimationFrame(release));
  return live;
}
