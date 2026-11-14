// A place with nothing in it: `kubaos-ui`'s `empty::empty_state`, `KubaOS.Components`'
// `EmptyState` (`docs/design/components/empty-state.md`). Centred across its room: a 32 symbol in
// `on_surface_variant`, `space.s12` under it a `subheading` title, `space.s4` under that a `body`
// line saying what to do, and `space.s16` under that an optional action. Its words wrap, centred,
// in a column `size.column` wide at most.

import "./feedback.css";
import { icon } from "./icon.ts";
import { stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { wrapped } from "./text.ts";

export interface EmptyStateOptions {
  readonly icon: string;
  readonly title: string;
  readonly body: string;
  /** What to do about it: a button, under the words. */
  readonly action?: Element;
}

export function emptyState(options: EmptyStateOptions): HTMLElement {
  const title = wrapped(options.title, "subheading");
  const body = wrapped(options.body, "body", "on_surface_variant");
  for (const each of [title, body]) each.classList.add("k-empty-words");
  const words = stack("s4", "centre", title, body);
  const said = stack("s12", "centre", icon(options.icon, "var(--icon-s32)", "var(--color-on_surface_variant)"), words);
  const column = stack("s16", "centre", said, ...(options.action ? [options.action] : []));
  column.classList.add("k-empty-column");
  const room = document.createElement("span");
  room.className = "k-empty-state";
  room.append(column);
  return probe(room, { role: "component", name: "EmptyState", variant: options.action ? "action" : "", fixed: { width: false, height: false } });
}
