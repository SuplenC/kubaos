// What broke and what to do next: `kubaos-ui`'s `error::error_block`, `KubaOS.Components`'
// `ErrorBlock` (`docs/design/components/error-block.md`). A status message (`lib/status.ts`) on
// `error_container` at `opacity.error_fill`, its edge lit in `error`: the `error` symbol at
// `subheading`'s 24, `space.s16` from the message in `subheading` (`body_strong` past
// `LONG_MESSAGE` characters, since a whole command line in 16 px semibold is a wall) over the hint
// in `body`, `space.s4` under it. The words wrap.

import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { statusBox } from "./status.ts";
import { wrapped } from "./text.ts";

/** Past this many characters a message drops from `subheading` to `body_strong`. */
export const LONG_MESSAGE = 90;

export interface ErrorBlockOptions {
  /** What broke. */
  readonly message: string;
  /** The next thing to try. */
  readonly hint: string;
}

/** Whether `message` is long enough to drop to `body_strong`. */
export function long(message: string): boolean {
  return [...message].length > LONG_MESSAGE;
}

export function errorBlock(options: ErrorBlockOptions): HTMLElement {
  const words = stack("s4", "start", wrapped(options.message, long(options.message) ? "body_strong" : "subheading"), wrapped(options.hint, "body", "on_surface_variant"));
  const said = row("s16", "centre", icon("error", "var(--type-subheading-icon)", "var(--color-error)"), filling(words));
  const box = statusBox("error", "rgb(from var(--color-error_container) r g b / var(--opacity-error_fill))", filling(said));
  box.classList.add("k-error-block");
  return probe(box, { role: "component", name: "ErrorBlock", variant: long(options.message) ? "long" : "", fixed: { width: false, height: false } });
}
