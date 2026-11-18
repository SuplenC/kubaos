// Phone's own parts and values: what the section keeps per window (its settings, and what KDE
// Connect reports in the sample: the phones paired, a pairing on its way), the pairing as Kōbō's
// "Your phone" card draws it, a paired phone's row end, and the confirm before forgetting one.
//
// Its organisms, kept in the Settings app because only it draws them:
// - `pairingPart`: the pairing, the prompt and notices of Kōbō's first-boot phone card
//   (`pages/kobo/setup-cards.ts`, `setup-parts.ts`), with its one action under them;
// - `rowAction`: a quiet button at a row's end, in the control column;
// - `forgetQuestion`: the Danger question the library's `dialogOver` asks before forgetting one.

import "./phone.css";
import "../../../kobo/setup.css";
import { banner } from "../../../../lib/banner.ts";
import { button } from "../../../../lib/button.ts";
import { hangs } from "../../../../lib/button-bar.ts";
import { card } from "../../../../lib/card.ts";
import type { DialogOptions } from "../../../../lib/dialog.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import { errorBlock } from "../../../../lib/error-block.ts";
import { row, stack } from "../../../../lib/layout.ts";
import { wrapped } from "../../../../lib/text.ts";
import { type Ask, CARDS, type Prompt, USER } from "../../../kobo/setup-cards.ts";
import { promptCard } from "../../../kobo/setup-parts.ts";
import type { Values } from "../settings-values.ts";

/** A phone KDE Connect knows as paired (`kubaos_phone::Device`). */
export interface Phone {
  readonly id: string;
  readonly name: string;
  /** Connected now, so it can be rung or sent to. */
  readonly reachable: boolean;
  /** Percent, while it is in reach and shares it. */
  readonly battery?: number;
}

/**
 * Where pairing stands while no request has been answered: waiting for a phone, still waiting after
 * 30 s (the address shows), a phone asking, or KDE Connect not running.
 */
export type Pairing = "waiting" | "address" | "request" | "stopped";

/** The section's values: its settings, then what the sample's KDE Connect reports. */
export interface PhoneValues {
  link: boolean;
  name: string;
  sendCopies: boolean;
  passwords: boolean;
  inbox: string;
  notifications: boolean;
  touchpad: boolean;
  media: boolean;
  commands: boolean;
  phones: readonly Phone[];
  pairing: Pairing;
  /** A firewall is on: Settings says what KDE Connect needs let through. */
  firewall: boolean;
}

/**
 * KubaOS's defaults: the link off, KDE Connect's own sharing, no plugin beyond the link's own; and
 * a laptop that has never paired a phone.
 */
export const DEFAULTS: PhoneValues = {
  link: false,
  name: "",
  sendCopies: true,
  passwords: true,
  inbox: "~/Downloads",
  notifications: false,
  touchpad: false,
  media: false,
  commands: false,
  phones: [],
  pairing: "waiting",
  firewall: false,
};

/** The phone that asks in Kōbō's card, as it is once paired. */
export const ASKING: Phone = { id: "pixel", name: "Pixel 9", reachable: true, battery: 82 };

const CARD = CARDS.phone;

/** The laptop's address on its network, as Kōbō's card shows it. */
export const ADDRESS = CARD.facts("waiting").find((fact) => fact.label === "Address")?.value ?? "";

/** What KDE Connect lets through in a name: phones cut it at 32 and drop these marks. */
export function nameProblem(value: string): string | null {
  if ([...value].length > 32) return "At most 32 letters: phones cut the rest.";
  return /["',;:.!?()[\]<>]/.test(value) ? "No quotes, brackets or ,;:.!? in it." : null;
}

/** The name phones see the laptop by: the one set here while KDE Connect takes it, else its host name. */
export function shownName(v: PhoneValues): string {
  return v.name && nameProblem(v.name) === null ? v.name : USER.host;
}

/** The paired phone a send or a ring goes to: the first in reach, else the first. */
export function preferred(v: PhoneValues): Phone | undefined {
  return v.phones.find((each) => each.reachable) ?? v.phones[0];
}

/** How many phones are paired, as a row's detail says it. */
export function counted(v: PhoneValues): string {
  const count = v.phones.length;
  if (count === 0) return "None";
  return count === 1 ? "1 phone" : `${count} phones`;
}

/** What a paired phone's row says under its name. */
export function phoneLine(phone: Phone): string {
  if (!phone.reachable) return "Away: open KDE Connect on it, on this Wi-Fi.";
  return phone.battery === undefined ? "In reach." : `In reach, battery ${phone.battery} %.`;
}

/** Draws the section's region again: what KDE Connect reports changed under it. */
export function redraw(values: Values): void {
  values.window?.querySelector<HTMLElement>(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
}

/** The card's words name the laptop by its host name; here, by the name it shows up as. */
function named(words: string, v: PhoneValues): string {
  return words.replaceAll(USER.host, shownName(v));
}

/** The card's prompt at `view`, its words naming the laptop as it shows up. */
function promptAt(view: string, v: PhoneValues, text?: string): HTMLElement {
  const prompt: Prompt = CARD.views[view]!.prompt!;
  return promptCard({ ...prompt, text: named(text ?? prompt.text, v) });
}

/** The card's notice at `view`, drawn as Kōbō draws a notice: a card with its symbol and lines. */
function noticeAt(view: string, v: PhoneValues): HTMLElement {
  const ask = CARD.asks(view).find((each): each is Extract<Ask, { is: "notice" }> => each.is === "notice")!;
  return card({ header: { glyph: ask.icon, title: ask.title }, body: ask.lines.map((line) => wrapped(named(line, v), "body", "on_surface_variant")) });
}

/** The one thing to press under a state, as Kōbō's band has it, at the settings' small size. */
function action(label: string, icon: string, press: () => void): HTMLElement {
  const made = button({ label, icon, kind: "Filled", size: "Sm" });
  made.addEventListener("click", press);
  return hangs(row("s12", "centre", made));
}

/**
 * The pairing, before Your phone's rows. Off, what to get on the phone first; on with
 * no phone, the wait, humming, and after 30 s the address and why a network hides it; a phone
 * asking, the key to compare and Accept; KDE Connect down, why and Start it again. Once a phone is
 * paired and nothing asks, it takes no room: the phone's own row says how it is.
 */
export function pairingPart(values: Values, v: PhoneValues): HTMLElement {
  const parts: HTMLElement[] = [];
  if (!v.link) parts.push(noticeAt("off", v));
  else if (v.pairing === "stopped") {
    const failed = CARD.views.failed!.failed!;
    parts.push(
      errorBlock({ message: `Fuck. ${failed.error}.`, hint: failed.hint }),
      action(CARD.action("failed")!.label, "restart_alt", () => {
        v.pairing = "waiting";
        redraw(values);
      }),
    );
  } else if (v.pairing === "request") {
    parts.push(
      promptAt("request", v, CARD.views.request!.summary),
      action(CARD.action("request")!.label, "check", () => {
        v.phones = [...v.phones.filter((each) => each.id !== ASKING.id), ASKING];
        v.pairing = "waiting";
        redraw(values);
      }),
    );
  } else if (v.phones.length === 0) {
    parts.push(promptAt("waiting", v));
    if (v.pairing === "address") parts.push(noticeAt("address", v));
    if (v.pairing === "address" && v.firewall) {
      parts.push(banner({ tone: "Warning", icon: "warning", title: "A firewall is on.", body: "KDE Connect needs TCP and UDP 1714 to 1764 and mDNS let through on this network." }));
    }
  }
  const made = stack("s16", "start", ...parts);
  made.classList.add("ph-pairing");
  made.toggleAttribute("data-none", parts.length === 0);
  return made;
}

/** Paired phones with none to list: where pairing one starts, or the link off. */
export function nonePaired(v: PhoneValues): HTMLElement {
  const made = !v.link
    ? emptyState({ icon: "smartphone", title: "The phone link is off", body: "Turn it on under Link to see the phones paired with it." })
    : v.phones.length === 0
      ? emptyState({ icon: "smartphone", title: "No phone paired", body: named(`Under Your phone: open KDE Connect on the phone and tap ${USER.host}.`, v) })
      : stack("s16", "start");
  made.classList.add("ph-pairing");
  made.toggleAttribute("data-none", v.link && v.phones.length > 0);
  return made;
}

/** A quiet button at a row's end, in the control column, its words its own. */
export function rowAction(label: string, icon: string | undefined, press: () => void, disabled = false): HTMLElement {
  const made = button({ label, ...(icon ? { icon } : {}), kind: "Text", size: "Sm", disabled });
  made.addEventListener("click", (event) => {
    // The press is the button's: the row under it neither takes it nor moves the keyboard.
    event.stopPropagation();
    if (!disabled) press();
  });
  // It hangs, as a bar's Text button does: its label ends on the column's edge, with the switches'.
  const end = hangs(document.createElement("span"));
  end.className = "ph-row-action";
  end.append(made);
  return end;
}

/** The question before a phone is forgotten. */
export function forgetQuestion(phone: Phone): DialogOptions {
  return {
    tone: "Danger",
    title: `Forget ${phone.name}?`,
    body: `This laptop stops trusting it: no clipboard, files or ringing until ${phone.name} pairs again.`,
    dismiss: "Cancel",
    actions: [{ label: "Forget", kind: "Danger", icon: "warning" }],
  };
}

