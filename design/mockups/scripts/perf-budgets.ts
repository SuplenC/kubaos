// What `scripts/perf.ts` measures on a mockup page, and the most each measure may be. A page that
// needs more than a default carries an override here, with the reason a reviewer reads.

export type Measure =
  | "load.ready"
  | "load.script"
  | "load.style"
  | "load.layout"
  | "load.paint"
  | "load.longTasks"
  | "load.longTaskTime"
  | "load.elements"
  | "load.animations"
  | "rest.busy"
  | "rest.styles"
  | "rest.layouts"
  | "rest.callbacks"
  | "scroll.median"
  | "scroll.worst"
  | "scroll.styleLayout"
  | "scroll.paint"
  | "scroll.forced"
  | "scroll.forcedTime"
  | "scroll.longTasks"
  | "press.longest"
  | "press.over"
  | "press.busy";

/**
 * Each measure's name in the table and its unit. `meaningful: false` marks one headless software
 * drawing distorts (paint): it is shown, never held to a budget.
 */
export const MEASURES: Record<Measure, { readonly label: string; readonly unit: string; readonly meaningful: boolean }> = {
  "load.ready": { label: "load: navigation to ready", unit: "ms", meaningful: true },
  "load.script": { label: "load: script", unit: "ms", meaningful: true },
  "load.style": { label: "load: style recalculation", unit: "ms", meaningful: true },
  "load.layout": { label: "load: layout", unit: "ms", meaningful: true },
  "load.paint": { label: "load: paint (software)", unit: "ms", meaningful: false },
  "load.longTasks": { label: "load: long tasks", unit: "", meaningful: true },
  "load.longTaskTime": { label: "load: long tasks' time", unit: "ms", meaningful: true },
  "load.elements": { label: "load: elements", unit: "", meaningful: true },
  "load.animations": { label: "rest: animations running", unit: "", meaningful: true },
  "rest.busy": { label: "rest: main thread busy", unit: "ms/s", meaningful: true },
  "rest.styles": { label: "rest: style recalculations", unit: "/s", meaningful: true },
  "rest.layouts": { label: "rest: layouts", unit: "/s", meaningful: true },
  "rest.callbacks": { label: "rest: timers and frame callbacks", unit: "/s", meaningful: true },
  "scroll.median": { label: "scroll: median frame", unit: "ms", meaningful: true },
  "scroll.worst": { label: "scroll: worst frame", unit: "ms", meaningful: true },
  "scroll.styleLayout": { label: "scroll: style + layout a frame", unit: "ms", meaningful: true },
  "scroll.paint": { label: "scroll: paint a frame (software)", unit: "ms", meaningful: false },
  "scroll.forced": { label: "scroll: forced style or layout", unit: "", meaningful: true },
  "scroll.forcedTime": { label: "scroll: forced style and layout a frame", unit: "ms", meaningful: true },
  "scroll.longTasks": { label: "scroll: long tasks", unit: "", meaningful: true },
  "press.longest": { label: "press: longest task", unit: "ms", meaningful: true },
  "press.over": { label: "press: frames over 16.7 ms", unit: "", meaningful: true },
  "press.busy": { label: "press: main thread busy in 1 s", unit: "ms", meaningful: true },
};

/**
 * The most each measure may be on any page. Set from Button (light), Tabs (a molecule) and the
 * Control Center (heavy) as the check measures them here, headless and drawing in software on a
 * machine shared with other work: Button, the simple page, passes each with room for that
 * machine's swings, and a page past them makes the reader wait. The README gives the numbers.
 */
export const BUDGETS: Partial<Record<Measure, number>> = {
  // The site's own chrome is in every load: Button loads in about 1 s, the Control Center in 1.4.
  "load.ready": 2000,
  "load.script": 1000,
  "load.style": 600,
  "load.layout": 200,
  // A page is drawn in one task or a few: three on all three pages.
  "load.longTasks": 5,
  "load.longTaskTime": 1800,
  "load.elements": 12000,
  // At rest nothing runs on any of the three; a live clock ticking once a second still passes.
  "load.animations": 0,
  "rest.busy": 5,
  "rest.styles": 2,
  "rest.layouts": 2,
  "rest.callbacks": 5,
  // A frame is 16.7 ms at 60 Hz; the three pages' median is 2-3 ms, their style and layout under 2.
  "scroll.median": 8,
  "scroll.worst": 80,
  "scroll.styleLayout": 3,
  "scroll.forcedTime": 1,
  "scroll.longTasks": 0,
  // A press answers within one long task's 50 ms, dropping no more than a frame or two.
  "press.longest": 50,
  "press.over": 2,
  "press.busy": 200,
};

/** A page's own budget for a measure, and why it needs more than the default. */
export interface Override {
  readonly budget: number;
  readonly reason: string;
}

/** Overrides by page slug. */
export const PAGE_BUDGETS: Record<string, Partial<Record<Measure, Override>>> = {};

/** The budget `measure` is held to on `slug`, and the reason when it is the page's own. */
export function budgetFor(slug: string, measure: Measure): { budget: number; reason?: string } | null {
  const own = PAGE_BUDGETS[slug]?.[measure];
  if (own) return { budget: own.budget, reason: own.reason };
  const budget = BUDGETS[measure];
  return budget === undefined || !MEASURES[measure].meaningful ? null : { budget };
}
