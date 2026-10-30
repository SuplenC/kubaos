// Every mockup page, by level. A new page is a file under its level's folder and its line here.

import type { AnyPage, Level } from "../site/page.ts";
import button from "./atoms/button.ts";
import colour from "./foundations/colour.ts";

export const PAGES: readonly AnyPage[] = [colour, button];

export function pagesOf(level: Level): AnyPage[] {
  return PAGES.filter((page) => page.level === level);
}
