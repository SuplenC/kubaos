// Every mockup page, by level. A new page is a file under its level's folder and its line here.

import type { Level, Page } from "../site/page.ts";
import button from "./atoms/button.ts";

export const PAGES: readonly Page[] = [button];

export function pagesOf(level: Level): Page[] {
  return PAGES.filter((page) => page.level === level);
}
