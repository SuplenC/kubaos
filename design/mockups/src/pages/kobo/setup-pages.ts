// What the pages of Kōbō's first boot share: the window drawn at a card's view, the live window,
// the measures every card page has, and its keys.

import { type Measure, type Move, type NonEmpty, screen } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { type CardId, CARD_IDS, type Views } from "./setup-cards.ts";
import { finishing, onward, setupRoom, type SetupOptions } from "./setup-room.ts";

/** Where each card ends when it is gone through in order: the apps in, the drivers' key queued. */
const WALKED: Views = { yours: "done", drivers: "queued", fingerprint: "done", yubikey: "done", phone: "done" };

/** The way to `card` taken in order: the cards before it gone through, the ones after it untouched. */
function walk(card: CardId): Partial<Views> {
  return Object.fromEntries(CARD_IDS.slice(0, CARD_IDS.indexOf(card)).map((id) => [id, WALKED[id]]));
}

/** The window at `card`'s `view`, reached by going through the cards before it, `more` on top. */
export function atCard(card: CardId, view: string, more: Partial<SetupOptions> = {}): () => HTMLElement {
  return screen(() => setupRoom({ place: { at: "card", card }, ...more, views: { ...walk(card), ...more.views, [card]: view } }));
}

/** The live stage: first boot at `card`, reached in order, at the toolbar's window size, going on. */
export function liveCard(card: CardId): () => HTMLElement {
  return screen(() => setupRoom({ place: { at: "card", card }, live: true, views: walk(card), width: windowSize().width, height: windowSize().height }));
}

/** The window as the Command menu opens it on `card` alone, at `view`, on the desktop. */
export function aloneAt(card: CardId, view: string, live = false): () => HTMLElement {
  return screen(() => setupRoom({ place: { at: "card", card }, alone: true, live, views: { [card]: view } }));
}

/** Continue or Skip onto `card`: the card before it slides off as its page slides in. */
export function arrivesAt(card: CardId, view: string): Move {
  const before = CARD_IDS[CARD_IDS.indexOf(card) - 1] ?? CARD_IDS[1]!;
  return onward(atCard(card, view, { turnedFrom: before }));
}

/** `card` at `view` on a Full HD screen: the card's own layout with more room. */
export function fullHd(card: CardId, view: string): () => HTMLElement {
  return atCard(card, view, { width: "calc(1920 * var(--px))", height: "calc(1080 * var(--px))" });
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
