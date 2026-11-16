// How a page's states are named in solo mode's address (`site/solo.ts`), shared with the script
// that captures them (`scripts/shot.ts`): it touches no DOM, so Node runs it as it is.

/** A state as solo mode names it: by its name as the page shows it, and a slug unique on the page. */
export interface StateName {
  readonly name: string;
  readonly slug: string;
  readonly group: string;
}

/** `words` as an address's slug: lower case, accents dropped, every other run a hyphen. */
export function slugOf(words: string): string {
  return words
    .normalize("NFKD")
    .replaceAll(/\p{M}/gu, "")
    .toLowerCase()
    .replaceAll(/[^a-z0-9]+/g, "-")
    .replaceAll(/^-|-$/g, "");
}

/** The state `wanted` names: its slug, its name in any case, its group and name, or its name's slug. */
export function findState<T extends StateName>(states: readonly T[], wanted: string): T | undefined {
  return (
    states.find((state) => state.slug === wanted) ??
    states.find((state) => state.name.toLowerCase() === wanted.toLowerCase() || `${state.group}-${slugOf(state.name)}` === wanted) ??
    states.find((state) => state.slug === slugOf(wanted))
  );
}
