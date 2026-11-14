// A job followed to its end: `KubaOS.Components`' `ProgressPanel`
// (`docs/design/components/progress-panel.md`), the Command menu's progress View. A column as wide
// as `size.column` at most, centred in its room, its parts `space.s16` apart and centred: the job's
// state as the room's one big symbol (48), its title in `headline`, a progress bar while it knows
// how far it is, and a `body` line with its state and what it last said.
//
// While the job can't tell how far it is, its symbol is a spinner; once it can, an hourglass over
// the bar. It succeeds with a check and the bar full, and fails with the error symbol and the line
// in `error`, and no bar.

import "./feedback.css";
import { icon } from "./icon.ts";
import { stack } from "./layout.ts";
import { progressBar } from "./progress.ts";
import { probe } from "./probe.ts";
import { spinner } from "./spinner.ts";
import { wrapped } from "./text.ts";

export type JobState = "running" | "succeeded" | "failed";

/** A job as `KotodamaSearch.jobs` lists it. */
export interface Job {
  readonly title: string;
  readonly state: JobState;
  /** How far it is, 0 to 1, or below 0 while it can't tell. */
  readonly progress: number;
  /** What it last said, or nothing. */
  readonly message?: string;
}

/** Whether `job` knows how far it is: it does once it has succeeded. */
export function known(job: Job): boolean {
  return job.state === "succeeded" || job.progress >= 0;
}

/** The job's state as its line says it: Done, Failed, its percent, or Running. */
function stateWords(job: Job): string {
  if (job.state === "succeeded") return "Done";
  if (job.state === "failed") return "Failed";
  return job.progress >= 0 ? `${Math.round(job.progress * 100).toLocaleString()}%` : "Running";
}

/** The line under the bar: the state, then what the job last said. */
export function jobLine(job: Job): string {
  const state = stateWords(job);
  return job.message ? `${state} · ${job.message}` : state;
}

/** The symbol over the title: a spinner while the job can't tell, else what its state says. */
function lead(job: Job): Element {
  if (job.state === "failed") return icon("error", "var(--icon-s48)", "var(--color-error)");
  if (!known(job)) return spinner("s48");
  return icon(job.state === "succeeded" ? "check_circle" : "hourglass_top", "var(--icon-s48)", "var(--color-primary)");
}

export function progressPanel(job: Job): HTMLElement {
  const title = wrapped(job.title, "headline");
  title.classList.add("k-progress-panel-words");
  const line = wrapped(jobLine(job), "body", job.state === "failed" ? "error" : "on_surface_variant");
  line.classList.add("k-progress-panel-words");
  const parts: Element[] = [lead(job), title];
  if (known(job) && job.state !== "failed") {
    const bar = progressBar({ fraction: job.state === "succeeded" ? 1 : job.progress });
    parts.push(bar);
  }
  parts.push(line);
  const column = stack("s16", "centre", ...parts);
  column.classList.add("k-progress-panel-column");
  const room = document.createElement("span");
  room.className = "k-progress-panel";
  room.append(column);
  return probe(room, { role: "component", name: "ProgressPanel", variant: job.state, fixed: { width: false, height: false } });
}
