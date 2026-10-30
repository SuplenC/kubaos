// The keyboard panel: every key and what it does, where focus goes, and where it lands.

import type { Keyboard } from "../page.ts";
import { el, label, prose } from "../dom.ts";
import { subheading } from "./section.ts";

export function keyboard(spec: Keyboard): HTMLElement {
  const keys = el("dl", "page-list");
  for (const row of spec.keys) {
    keys.append(
      el("dt", "page-keys", ...row.keys.map((key) => el("kbd", "page-key", label(key, "keycap", "on_surface")))),
      el("dd", "", prose(row.does)),
    );
  }
  return el("div", "page-part", keys, subheading("Where focus goes"), prose(spec.order), subheading("Where focus lands"), prose(spec.lands));
}
