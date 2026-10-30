// On or off: `kubaos-ui`'s `switch::switch`, `KubaOS.Components`' `Switch`. A track
// `size.switch_width` by `size.switch_height`, a circle at each end, with a knob that grows from
// `size.switch_knob_off` to `size.switch_knob_on` as it moves across (`docs/design/components/
// switch.md`). It takes effect at once; a click, Space or Enter toggles it, and the knob moves over
// `short`.
//
// Its states are the neon's (`.k-control` in `lib.css`). Off, nothing is filled with the accent:
// its line is the tube over the track's outline. On, the track is filled with the accent, so at
// night it rests on the unlit fill, and its line is the core inside the fill's edge.

import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { probe } from "./probe.ts";

export interface SwitchOptions {
  readonly on?: boolean;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** Turns `control` on or off: its value, and its line, which is the core inside the fill on. */
function turn(control: HTMLButtonElement, ring: HTMLElement, on: boolean): void {
  control.setAttribute("aria-checked", String(on));
  control.style.setProperty("--checked", on ? "1" : "0");
  ring.toggleAttribute("data-filled", on);
}

export function switchControl(options: SwitchOptions = {}): HTMLButtonElement {
  const control = document.createElement("button");
  control.className = "k-switch k-control";
  control.type = "button";
  control.setAttribute("role", "switch");
  control.dataset.hot = "";
  control.disabled = options.disabled ?? false;
  pin(control, options.pinned);
  const knob = document.createElement("span");
  knob.className = "k-switch-knob";
  control.append(knob);
  const ring = focusRing(control, "full", "accent", false);
  ring.dataset.pressable = "";
  turn(control, ring, options.on ?? false);
  control.addEventListener("click", () => turn(control, ring, control.getAttribute("aria-checked") !== "true"));
  return probe(control, { role: "component", name: "Switch", variant: "", fixed: { width: true, height: true } });
}
