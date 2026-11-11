// The button that opens a surface's shortcuts (`docs/design/components/shortcuts.md`): a Text
// IconButton with `info`, its name and key on a tooltip, the `Shortcuts` popover `space.s8` above
// it from its start. It opens on a press or `?`, and closes on a press again, Esc or a press
// outside; Esc closes it before anything under it takes the key.

import { iconButton } from "./icon-button.ts";
import { type ShortcutGroup, shortcuts } from "./shortcuts.ts";
import { floatingTooltip, keyTooltip } from "./tooltip.ts";

export interface ShortcutsButton {
  /** The anchor holding the button, its tooltip and the popover: where the button goes. */
  readonly element: HTMLElement;
  readonly button: HTMLButtonElement;
  readonly popover: HTMLElement;
  readonly open: () => boolean;
  readonly set: (open: boolean) => void;
}

export function shortcutsButton(groups: readonly ShortcutGroup[], options: { readonly shown?: boolean } = {}): ShortcutsButton {
  const button = iconButton({ icon: "info", label: "Shortcuts", kind: "Text", size: "Md", hint: false });
  const anchor = floatingTooltip(button, keyTooltip("Shortcuts", "?"), { side: "above" });
  anchor.classList.add("k-shortcuts-anchor");
  const card = shortcuts(groups);
  anchor.append(card);
  let open = false;
  const set = (on: boolean) => {
    open = on;
    anchor.toggleAttribute("data-open", on);
    button.toggleAttribute("data-pinned", false);
  };
  button.addEventListener("click", (event) => {
    event.stopPropagation();
    set(!open);
  });
  card.addEventListener("click", (event) => event.stopPropagation());
  document.addEventListener("pointerdown", (event) => {
    if (open && !anchor.contains(event.target as Node)) set(false);
  }, true);
  set(options.shown ?? false);
  return { element: anchor, button, popover: card, open: () => open, set };
}

/**
 * Takes `?` to open the shortcuts and Esc to close them on `root`, before anything else on it takes
 * the key: listens in the capture phase.
 */
export function takeShortcutKeys(root: HTMLElement, made: ShortcutsButton): void {
  root.addEventListener(
    "keydown",
    (event) => {
      if (event.key === "?") {
        event.preventDefault();
        event.stopPropagation();
        made.set(!made.open());
      } else if (event.key === "Escape" && made.open()) {
        event.preventDefault();
        event.stopPropagation();
        made.set(false);
      }
    },
    true,
  );
}
