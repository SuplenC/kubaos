// What the pages of Kōbō's first boot share: the window drawn at a card's view, the live window,
// the measures every card page has, and its keys.

import { type Measure, type Move, type NonEmpty, screen } from "../../site/page.ts";
import { type CardId, CARD_IDS, type Views } from "./setup-cards.ts";
import { finishing, onward, setupRoom, type SetupOptions } from "./setup-room.ts";

/** Where each card ends when it is gone through in order: TUXEDO's key queued for its reboot. */
const WALKED: Views = { tuxedo: "queued", fingerprint: "done", yubikey: "done", bitwarden: "done", git: "done", phone: "done" };

/** The way to `card` taken in order: the cards before it gone through, the ones after it untouched. */
function walk(card: CardId): Partial<Views> {
  return Object.fromEntries(CARD_IDS.slice(0, CARD_IDS.indexOf(card)).map((id) => [id, WALKED[id]]));
}

/** The window at `card`'s `view`, reached by going through the cards before it, `more` on top. */
export function atCard(card: CardId, view: string, more: Partial<SetupOptions> = {}): () => HTMLElement {
  return screen(() => setupRoom({ place: { at: "card", card }, ...more, views: { ...walk(card), ...more.views, [card]: view } }));
}

/** The live stage: the window on the desktop at `card`, reached in order, going on through first boot. */
export function liveCard(card: CardId): () => HTMLElement {
  return screen(() => setupRoom({ place: { at: "card", card }, live: true, desk: true, views: walk(card) }));
}

/** Continue or Skip onto `card`: its page slides in as the wire's light grows to its step. */
export function arrivesAt(card: CardId, view: string): Move {
  return onward(atCard(card, view));
}

/** `card`'s work finishing at `view`, beside a card done already, so the wire has a run to light. */
export function finishesAt(card: CardId, view: string): Move {
  const index = CARD_IDS.indexOf(card);
  const neighbour = CARD_IDS[index === 0 ? 1 : index - 1]!;
  return finishing(index, atCard(card, view, { views: { [neighbour]: "done" } }));
}

/** The keys every card takes; a card adds its own. */
export const CARD_KEYS: NonEmpty<{ readonly keys: NonEmpty<string>; readonly does: string }> = [
  { keys: ["Enter"], does: "the card's work, or Continue once it's done" },
  { keys: ["Esc"], does: "back a card; on the first, the arrival" },
  { keys: ["↑", "↓"], does: "in the rail, picks a card; Space opens it" },
  { keys: ["Tab", "Shift+Tab"], does: "rail, the card's fields, Under the hood, the band" },
];

/** Every card page's measures, its own added after. */
export const CARD_MEASURES: readonly Measure[] = [
  { part: "Rail's width", token: "size-rail" },
  { part: "Page's sides", token: "space-s32" },
  { part: "Header → status", token: "space-s24" },
  { part: "Between the card's parts", token: "space-s20" },
  { part: "Chip → summary", token: "space-s12" },
  { part: "Body → footer band", token: "space-s16" },
  { part: "Band's buttons", token: "size-control_md" },
];
