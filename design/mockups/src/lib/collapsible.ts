// A titled part that opens to show more: `kubaos-ui`'s `collapsible::collapsible`,
// `KubaOS.Components`' `Collapsible` (`docs/design/components/collapsible.md`). The header is a
// control as tall as a row (`size.row`) at the control radius, `space.s12` inside: its title in
// `body_strong` and a chevron at its end that turns from down to up as it opens. Open, what it
// holds sits `space.s8` under the header, `space.s12` in at the sides, in line with the title.
//
// The header is a control in the state model (`.k-control` in `lib.css`) with nothing filled under
// its line, so lit it draws the tube and pressed the tube runs white-hot. The room under it eases
// from nothing to what it holds over `medium` on `standard_decel` and back over `short` on
// `standard_accel`, the chevron turning with it; with reduce motion it opens at once and what it
// holds fades in.

import { type Pinned, pin } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { filling, row } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { clamped } from "./text.ts";

export interface CollapsibleOptions {
  readonly title: string;
  /** What it shows open. */
  readonly content: Element;
  readonly open?: boolean;
  /** No toggle: the header dims and takes no press and no stop for Tab. */
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

let made = 0;

export function collapsible(options: CollapsibleOptions): HTMLElement {
  const id = `k-collapsible-${made++}`;
  let open = options.open ?? false;
  const disabled = options.disabled ?? false;

  const part = document.createElement("span");
  part.className = "k-collapsible";

  const header = document.createElement("button");
  header.className = "k-collapsible-header k-control";
  header.type = "button";
  header.dataset.hot = "";
  header.disabled = disabled;
  header.setAttribute("aria-controls", id);
  pin(header, options.pinned);
  const title = clamped(options.title, "body_strong");
  title.style.color = "var(--title)";
  const chevron = icon("expand_more", "var(--type-body_strong-icon)", "var(--mark)");
  chevron.classList.add("k-collapsible-chevron");
  const line = row("s12", "centre", filling(title), chevron);
  header.append(line);
  focusRing(header, "control", "accent", false).dataset.pressable = "";

  const body = document.createElement("span");
  body.className = "k-collapsible-body";
  body.id = id;
  body.setAttribute("role", "region");
  const content = document.createElement("span");
  content.className = "k-collapsible-content";
  content.append(options.content);
  body.append(content);
  part.append(header, body);

  // The room eases between nothing and what it holds, which only layout knows: its height in the
  // part's own CSS pixels, kept as it changes.
  new ResizeObserver(() => part.style.setProperty("--content-height", `${content.offsetHeight}px`)).observe(content);

  /** Shows `open`: the header's state, the room's, and whether what it holds takes the keyboard. */
  function show(): void {
    part.toggleAttribute("data-open", open);
    header.setAttribute("aria-expanded", String(open));
    body.inert = !open;
    probe(part, { role: "component", name: "Collapsible", variant: variant(open && "open"), fixed: { width: false, height: false, holds: true } });
  }

  /** Lets what it holds reach past the room once it is fully open, so a lit control's glow isn't cut. */
  function settle(): void {
    part.toggleAttribute("data-settled", open);
  }

  part.addEventListener("transitionend", (event) => {
    if (event.target === part && event.propertyName === "--open") settle();
  });
  header.addEventListener("click", () => {
    open = !open;
    part.removeAttribute("data-settled");
    show();
    // With nothing to play (reduced motion) the room is where it ends at once.
    if (getComputedStyle(part).transitionDuration.split(",").every((each) => Number.parseFloat(each) === 0)) settle();
  });
  // Enter and Space press it once per press, never on key repeat.
  header.addEventListener("keydown", (event) => {
    if ((event.key === "Enter" || event.key === " ") && event.repeat) event.preventDefault();
  });

  show();
  settle();
  return part;
}
