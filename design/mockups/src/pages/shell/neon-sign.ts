// The boot splash's neon sign, from the splash's own pictures (`system/files/plymouth/kubaos/`):
// the dead glass (`sign-base`) and a layer of tubes per group over it, at the size the splash
// draws them, in the exact centre of its screen. Every screen from the boot menu to the session,
// and the way down at power off, draws it here, so it never moves between them. Shared by the
// Boot page and the Power menu's room.

import "./neon-sign.css";
import signBa from "../../../../../system/files/plymouth/kubaos/sign-ba@2x.png?url";
import signBase from "../../../../../system/files/plymouth/kubaos/sign-base@2x.png?url";
import signDakuten from "../../../../../system/files/plymouth/kubaos/sign-dakuten@2x.png?url";
import signKu from "../../../../../system/files/plymouth/kubaos/sign-ku@2x.png?url";
import signKuba from "../../../../../system/files/plymouth/kubaos/sign-kuba@2x.png?url";
import signOs from "../../../../../system/files/plymouth/kubaos/sign-os@2x.png?url";
import { durationMs, number } from "../../lib/tokens.ts";

/** The tubes' groups in the order they switch on (`kubaos.script`'s GROUP); they go out the other way. */
export const GROUPS = ["ku", "ba", "dakuten", "kuba", "os"] as const;
export type Group = (typeof GROUPS)[number];

const PICTURES: Record<Group, string> = { ku: signKu, ba: signBa, dakuten: signDakuten, kuba: signKuba, os: signOs };

/** The selector of `group`'s tubes. */
export const groupOf = (group: Group) => `.neon-sign-${group}`;

/** The dead glass under the tubes. */
export const GLASS = ".neon-sign-base";

/** The whole sign, glass and tubes. */
export const SIGN = ".neon-sign";

function layer(src: string, cls: string): HTMLImageElement {
  const img = document.createElement("img");
  img.src = src;
  img.alt = "";
  img.draggable = false;
  img.className = cls;
  return img;
}

/**
 * A room as large as its screen with the sign in its exact centre, and `under` in a column
 * centred under it, starting at the picture's foot: the glow is the picture's own clear space.
 * `lit` is how many groups burn, in switch-on order. The splash draws in device pixels from the
 * @2× pictures' 1× size, halved and rounded down as `sign-base.png` is; `--splash-scale` on an
 * ancestor turns those into the sample's logical pixels.
 */
export function neonRoom(lit: number = GROUPS.length, ...under: Element[]): HTMLElement {
  const room = document.createElement("span");
  room.className = "neon-room";
  const sign = document.createElement("span");
  sign.className = "neon-sign";
  const glass = layer(signBase, "neon-sign-base");
  const size = () => {
    if (glass.naturalWidth === 0) return;
    room.style.setProperty("--sign-device-w", String(Math.floor(glass.naturalWidth / 2)));
    room.style.setProperty("--sign-device-h", String(Math.floor(glass.naturalHeight / 2)));
  };
  glass.addEventListener("load", size);
  size();
  sign.append(glass, ...GROUPS.map((group, i) => {
    const tubes = layer(PICTURES[group], `neon-sign-g neon-sign-${group}`);
    tubes.style.opacity = i < lit ? "1" : "0";
    return tubes;
  }));
  const below = document.createElement("span");
  below.className = "neon-under";
  below.append(...under);
  room.append(sign, below);
  return room;
}

/**
 * The lit sign's hum, as the splash plays it once every group is on: the whole sign steps to a new
 * level between `opacity.neon_hum_floor` and full every `micro`, and every loop of three `scene`s
 * and two `short`s バ blinks a `scene` and a `slow` in, down to `opacity.neon_blink_low`, back, down
 * to `opacity.neon_blink_half` and back, each step a `neon_blink`.
 */
const humFloor = () => number("opacity-neon_hum_floor");
const blinkLevels = () => [number("opacity-neon_blink_low"), 1, number("opacity-neon_blink_half"), 1];

const loopMs = () => 3 * durationMs("scene") + 2 * durationMs("short");
const blinkAtMs = () => durationMs("scene") + durationMs("slow");
const blinkStepMs = () => durationMs("neon_blink");

/**
 * Starts the hum on every sign in `root`, looping until its animations are cancelled: one
 * animation for the sign's level and one for バ's blink, each a loop of held steps, so it asks for
 * no timers. Only once the sign is fully lit, never during its switch-on; reduce motion plays none.
 */
export function hum(root: ParentNode, speed = 1): void {
  const loop = loopMs() / speed;
  const step = durationMs("micro") / speed;
  const steps = Math.round(loop / step);
  const floor = humFloor();
  const levels: Keyframe[] = Array.from({ length: steps }, (_, i) => ({ offset: i / steps, easing: "step-end", opacity: floor + (1 - floor) * Math.random() }));
  const at = blinkAtMs() / speed;
  const each = blinkStepMs() / speed;
  const blink: Keyframe[] = [
    { offset: 0, easing: "step-end", opacity: 1 },
    ...blinkLevels().map((level, i) => ({ offset: (at + i * each) / loop, easing: "step-end", opacity: level })),
    { offset: 1, opacity: 1 },
  ];
  for (const sign of root.querySelectorAll<HTMLElement>(SIGN)) {
    sign.animate([...levels, { offset: 1, opacity: levels[0]!.opacity! }], { duration: loop, iterations: Infinity, composite: "replace" });
    sign.querySelector<HTMLElement>(groupOf("ba"))?.animate(blink, { duration: loop, iterations: Infinity });
  }
}

/** Lights the first `lit` groups of every sign in `root`, the rest dark. */
export function light(root: ParentNode, lit: number): void {
  GROUPS.forEach((group, i) => {
    for (const tubes of root.querySelectorAll<HTMLElement>(groupOf(group))) tubes.style.opacity = i < lit ? "1" : "0";
  });
}
