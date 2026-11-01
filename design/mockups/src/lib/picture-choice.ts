// A picture to pick: `kubaos-ui`'s `picture::picture_choice`, `KubaOS.Components`' `PictureChoice`
// (`docs/design/components/picture-choice.md`). The picture covers the whole tile, a `Media` box at
// the control radius with no frame round it; its name sits under it, or over its foot on a scrim.
//
// Picked, the accent's line burns inside its edge, with the glow outside at night, and its mark
// shows in a `Selected` chip over its top right corner. It is a control in the state model
// (`.k-control` in `lib.css`): lit, the cold tube takes the picked line's place, since the accent
// already means picked; a badge (a `Neutral` chip) sits over its top left corner whatever its state.

import { chip } from "./chip.ts";
import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { media } from "./media.ts";
import { probe } from "./probe.ts";
import { role } from "./radius.ts";
import { clamped } from "./text.ts";

/** A chip over a corner: a symbol and a word. */
export interface Tag {
  readonly icon: string;
  readonly label: string;
}

export interface PictureChoiceOptions {
  /** The picture's address; none holds the placeholder. */
  readonly source?: string;
  /** Held loading or unreadable, whatever the address does. */
  readonly arrival?: "loading" | "broken";
  readonly name: string;
  /** Where the name is: under the picture, or over its foot. */
  readonly place?: "under" | "over";
  readonly picked?: boolean;
  /** What says it is picked, over the top right corner while it is. */
  readonly mark?: Tag;
  /** A fact about the picture whatever its state, over the top left corner. */
  readonly badge?: Tag;
  /** One of a set picked as radios, or any of it as checkboxes. */
  readonly many?: boolean;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** `tag` as a chip `space.s8` in from the corner `corner` names. */
function corner(tag: Tag, kind: "Neutral" | "Selected", side: "start" | "end", disabled: boolean): HTMLElement {
  const held = document.createElement("span");
  held.className = "k-picture-tag";
  held.dataset.side = side;
  held.append(chip({ label: tag.label, icon: tag.icon, kind, disabled }));
  return held;
}

export function pictureChoice(options: PictureChoiceOptions): HTMLElement {
  const place = options.place ?? "under";
  const disabled = options.disabled ?? false;
  const whole = document.createElement("span");
  whole.className = "k-picture-choice";
  whole.dataset.place = place;

  const tile = document.createElement("span");
  tile.className = "k-picture-tile k-control";
  tile.dataset.hot = "";
  tile.setAttribute("role", options.many ? "checkbox" : "radio");
  tile.setAttribute("aria-label", options.name);
  tile.tabIndex = -1;
  if (disabled) tile.setAttribute("aria-disabled", "true");
  pin(tile, options.pinned);
  tile.append(
    media({
      corner: role("control"),
      dimmed: disabled,
      ...(options.source ? { source: options.source } : {}),
      ...(options.arrival ? { arrival: options.arrival } : {}),
    }),
  );
  const tint = document.createElement("span");
  tint.className = "k-picture-tint";
  tile.append(tint);
  if (place === "over") {
    const scrim = document.createElement("span");
    scrim.className = "k-picture-scrim";
    const name = clamped(options.name, "small");
    name.classList.add("k-picture-name-over");
    // Over a photo the name is light whatever the mode: the night's colours, on the dark foot.
    name.dataset.mode = "dark";
    tile.append(scrim, name);
  }
  if (options.badge) tile.append(corner(options.badge, "Neutral", "start", disabled));
  if (options.mark) tile.append(corner(options.mark, "Selected", "end", disabled));
  const picked = focusRing(tile, "control", "accent", false);
  picked.classList.add("k-picked-ring");
  focusRing(tile, "control", "cold", false).dataset.pressable = "";
  whole.append(tile);
  if (place === "under") {
    const name = clamped(options.name, "small");
    name.style.color = "var(--name-colour)";
    whole.append(name);
  }
  whole.setAttribute("aria-checked", String(options.picked ?? false));
  setPickedPicture(whole, options.picked ?? false);
  return probe(whole, { role: "component", name: "PictureChoice", variant: place === "over" ? "name-over" : "name-under", fixed: { width: false, height: false, holds: true } });
}

/** Shows whether `whole`, a picture choice, is picked. */
export function setPickedPicture(whole: HTMLElement, picked: boolean): void {
  whole.setAttribute("aria-checked", String(picked));
  whole.querySelector(".k-picture-tile")?.setAttribute("aria-checked", String(picked));
  whole.style.setProperty("--checked", picked ? "1" : "0");
}
