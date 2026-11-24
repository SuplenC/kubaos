// A page's main column with a picture beside it, kept in view while the column scrolls (Settings ›
// Camera's controls and its live preview): a draft layout with no twin yet.
//
// The picture is 40% of the room, held between `size.beside_picture_least` and
// `size.beside_picture_most`, and takes whatever the column leaves past `size.beside_controls_most`;
// the column takes the rest, `space.s24` after it. Where that would leave the column narrower than
// `size.beside_controls_least`, the picture stands above it instead, `size.beside_picture_least`
// wide at its start, `space.s32` over it, and scrolls away with the page.

import "./beside.css";
import { probe, workedOut } from "./probe.ts";
import { length } from "./tokens.ts";
import { scaleOf } from "./list.ts";

/** The picture's width for a room `room` wide, in the room's own pixels, as `beside.css` works it out. */
function pictureFor(room: number, scale: number): number {
  const at = (name: string) => length(name) * scale;
  const share = Math.min(at("size-beside_picture_most"), Math.max(at("size-beside_picture_least"), room * 0.4));
  return Math.max(share, room - at("size-beside_controls_most") - at("space-s24"));
}

/** `main` with `picture` beside it, or above it in a room too narrow for both. */
export function beside(main: HTMLElement, picture: HTMLElement): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-beside";
  main.classList.add("k-beside-main");
  const aside = document.createElement("span");
  aside.className = "k-beside-aside";
  aside.append(picture);
  box.append(main, aside);
  workedOut(aside, "size-beside_picture_least", "size-beside_picture_most", "size-beside_controls_most", "space-s24");
  const place = () => {
    const scale = scaleOf(box);
    const room = box.clientWidth;
    const above = room - length("space-s24") * scale - pictureFor(room, scale) < length("size-beside_controls_least") * scale;
    if (box.hasAttribute("data-above") === above) return;
    box.toggleAttribute("data-above", above);
    probe(box, { role: "layout", axis: above ? "y" : "x", gap: length(above ? "space-s32" : "space-s24"), align: "start" });
  };
  probe(box, { role: "layout", axis: "x", gap: length("space-s24"), align: "start" });
  new ResizeObserver(place).observe(box);
  return box;
}
