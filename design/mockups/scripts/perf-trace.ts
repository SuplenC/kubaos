// Reads a Chromium trace of one mockup page for `scripts/perf.ts`: the page's main thread as a
// tree of timed events, each event's own time by kind (script, style, layout, paint, other), its
// long tasks, its frames, the layouts its script forced, and the V8 sampler's time per function.
// Every place in the site's code is named by the module it came from, read off the unminified
// bundle's `// src/…` headers, so a culprit is a file a person can open.

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

/** A source line in the trace: a 0-based line of a script at `url`. */
interface CallFrame {
  readonly functionName: string;
  readonly url?: string;
  readonly lineNumber?: number;
}

interface ProfileNode {
  readonly id: number;
  readonly callFrame: CallFrame;
}

/** The trace event fields this file reads; Chromium's trace has many more. */
interface RawEvent {
  readonly name: string;
  readonly ph: string;
  readonly ts: number;
  readonly dur?: number;
  readonly pid: number;
  readonly tid: number;
  readonly id?: string;
  readonly id2?: { readonly local?: string; readonly global?: string };
  readonly args?: {
    readonly name?: string;
    readonly elementCount?: number;
    readonly beginData?: { readonly stackTrace?: readonly CallFrame[] };
    readonly data?: {
      readonly message?: string;
      readonly documentLoaderURL?: string;
      readonly stackTrace?: readonly CallFrame[];
      readonly startTime?: number;
      readonly timeDeltas?: readonly number[];
      readonly cpuProfile?: { readonly nodes?: readonly ProfileNode[]; readonly samples?: readonly number[] };
    };
  };
}

export type Kind = "script" | "style" | "layout" | "paint" | "other";

const STYLE = new Set(["UpdateLayoutTree", "RecalculateStyles", "ParseAuthorStyleSheet"]);
const LAYOUT = new Set(["Layout"]);
const PAINT = new Set(["Paint", "PrePaint", "Layerize", "UpdateLayer", "UpdateLayerTree", "Commit", "PaintImage", "CompositeLayers", "Decode Image", "Decode LazyPixelRef"]);
const SCRIPT = new Set([
  "EvaluateScript",
  "FunctionCall",
  "TimerFire",
  "FireAnimationFrame",
  "FireIdleCallback",
  "EventDispatch",
  "RunMicrotasks",
  "MajorGC",
  "MinorGC",
]);

function kindOf(name: string): Kind {
  if (STYLE.has(name)) return "style";
  if (LAYOUT.has(name)) return "layout";
  if (PAINT.has(name)) return "paint";
  if (SCRIPT.has(name) || name.startsWith("V8.") || name.startsWith("v8.")) return "script";
  return "other";
}

/** One event on the main thread, with the time it spent outside its children. */
export interface Task {
  readonly name: string;
  readonly kind: Kind;
  readonly start: number;
  readonly end: number;
  readonly self: number;
  readonly depth: number;
  readonly parent: Task | null;
  readonly event: RawEvent;
}

export type Times = Record<Kind, number>;

export function noTimes(): Times {
  return { script: 0, style: 0, layout: 0, paint: 0, other: 0 };
}

/** A frame the main thread worked on: its span and its work by kind, in milliseconds. */
export interface Frame {
  readonly start: number;
  readonly end: number;
  readonly busy: number;
  readonly times: Times;
}

/** Where the page's code is, by its bundle's lines: each module's first line, in order. */
class Modules {
  readonly #byScript = new Map<string, { line: number; module: string }[]>();

  constructor(site: string) {
    const assets = path.join(site, "assets");
    for (const file of readdirSync(assets).filter((name) => name.endsWith(".js"))) {
      const heads: { line: number; module: string }[] = [];
      readFileSync(path.join(assets, file), "utf8")
        .split("\n")
        .forEach((text, line) => {
          const head = /^\/\/ ((?:src|\.\.)\/\S+)$/.exec(text);
          if (head) heads.push({ line, module: head[1]!.replace(/^(\.\.\/)+/, "") });
        });
      this.#byScript.set(file, heads);
    }
  }

  /** `frame` as `function · module`, or null for a frame outside the site's code. */
  name(frame: CallFrame): string | null {
    if (!frame.url) return null;
    const heads = this.#byScript.get(path.basename(new URL(frame.url).pathname));
    const line = frame.lineNumber ?? 0;
    let module = path.basename(frame.url);
    for (const head of heads ?? []) {
      if (head.line > line) break;
      module = head.module;
    }
    return `${frame.functionName || "(anonymous)"} · ${module}`;
  }
}

/** Counts names and lists the most frequent, most first, as `name ×n`. */
function mostOften(names: readonly string[], most: number): string[] {
  const counts = new Map<string, number>();
  for (const name of names) counts.set(name, (counts.get(name) ?? 0) + 1);
  return [...counts]
    .sort((a, b) => b[1] - a[1])
    .slice(0, most)
    .map(([name, count]) => (count > 1 ? `${name} ×${count}` : name));
}

const MS = 1000;

/** One page's trace. Times are the trace's microseconds; what it returns is in milliseconds. */
export class Trace {
  readonly tasks: Task[] = [];
  readonly #marks = new Map<string, number>();
  readonly #instants: RawEvent[] = [];
  readonly #frames: { start: number; end: number }[] = [];
  readonly #samples: { at: number; length: number; frame: CallFrame }[] = [];
  readonly #modules: Modules;
  readonly navigationStart: number;

  constructor(json: string, site: string, url: string) {
    this.#modules = new Modules(site);
    // The trace's JSON is Chromium's; only the fields `RawEvent` names are read.
    const parsed = JSON.parse(json) as { traceEvents?: RawEvent[] } | RawEvent[];
    const events = Array.isArray(parsed) ? parsed : (parsed.traceEvents ?? []);
    const ready = events.find((event) => event.name === "TimeStamp" && event.args?.data?.message === "perf:ready");
    if (!ready) throw new Error("the trace holds no `perf:ready` mark");
    const onMain = (event: RawEvent): boolean => event.pid === ready.pid && event.tid === ready.tid;
    const main = events.filter(onMain);
    this.navigationStart = main.find((event) => event.name === "navigationStart" && event.args?.data?.documentLoaderURL === url)?.ts ?? main.reduce((first, event) => Math.min(first, event.ts), ready.ts);
    this.#readTasks(main);
    this.#readFrames(main);
    this.#readProfile(events.filter((event) => event.pid === ready.pid));
    for (const event of main) {
      if (event.name === "TimeStamp" && event.args?.data?.message) this.#marks.set(event.args.data.message, event.ts);
      if ((event.name === "ScheduleStyleRecalculation" || event.name === "InvalidateLayout") && event.args?.data?.stackTrace) this.#instants.push(event);
    }
  }

  #readTasks(main: readonly RawEvent[]): void {
    const complete = main.filter((event) => event.ph === "X" && event.dur !== undefined).sort((a, b) => a.ts - b.ts || (b.dur ?? 0) - (a.dur ?? 0));
    const open: Task[] = [];
    const selfOf = new Map<Task, number>();
    for (const event of complete) {
      const end = event.ts + (event.dur ?? 0);
      while (open.length > 0 && open[open.length - 1]!.end <= event.ts) open.pop();
      const parent = open[open.length - 1] ?? null;
      const task: Task = { name: event.name, kind: kindOf(event.name), start: event.ts, end, self: 0, depth: open.length, parent, event };
      if (parent) selfOf.set(parent, (selfOf.get(parent) ?? 0) - (event.dur ?? 0));
      selfOf.set(task, (selfOf.get(task) ?? 0) + (event.dur ?? 0));
      this.tasks.push(task);
      open.push(task);
    }
    // A task's own time is known once every child is read.
    for (const task of this.tasks) (task as { self: number }).self = Math.max(0, selfOf.get(task) ?? 0);
  }

  /** The frames the main thread rendered, from the long-animation-frame spans Chromium traces. */
  #readFrames(main: readonly RawEvent[]): void {
    const begun = new Map<string, number>();
    for (const event of main) {
      if (event.name !== "AnimationFrame") continue;
      const id = event.id ?? event.id2?.local ?? event.id2?.global ?? "";
      if (event.ph === "b") begun.set(id, event.ts);
      else if (event.ph === "e" && begun.has(id)) {
        this.#frames.push({ start: begun.get(id)!, end: event.ts });
        begun.delete(id);
      }
    }
    this.#frames.sort((a, b) => a.start - b.start);
  }

  /** The V8 sampler's samples on the page's main isolate, each with the time until the next. */
  #readProfile(renderer: readonly RawEvent[]): void {
    const starts = new Map<string, number>();
    for (const event of renderer) if (event.name === "Profile" && event.id && event.args?.data?.startTime !== undefined) starts.set(event.id, event.args.data.startTime);
    const byProfile = new Map<string, { at: number; frame: CallFrame }[]>();
    const nodes = new Map<string, CallFrame>();
    const clock = new Map<string, number>();
    for (const event of renderer) {
      const data = event.args?.data;
      if (event.name !== "ProfileChunk" || !event.id || !data?.cpuProfile) continue;
      for (const node of data.cpuProfile.nodes ?? []) nodes.set(`${event.id}:${node.id}`, node.callFrame);
      let at = clock.get(event.id) ?? starts.get(event.id) ?? 0;
      const samples = byProfile.get(event.id) ?? [];
      (data.cpuProfile.samples ?? []).forEach((node, i) => {
        at += data.timeDeltas?.[i] ?? 0;
        samples.push({ at, frame: nodes.get(`${event.id}:${node}`) ?? { functionName: "(unknown)" } });
      });
      clock.set(event.id, at);
      byProfile.set(event.id, samples);
    }
    // A renderer can run more than one isolate; the page's main one has the most samples.
    const samples = [...byProfile.values()].sort((a, b) => b.length - a.length)[0] ?? [];
    samples.forEach((sample, i) => {
      if (/^\((idle|program|root)\)$/.test(sample.frame.functionName)) return;
      this.#samples.push({ ...sample, length: (samples[i + 1]?.at ?? sample.at) - sample.at });
    });
  }

  /** The time of the mark the page made with `console.timeStamp(name)`. */
  mark(name: string): number {
    const at = this.#marks.get(name);
    if (at === undefined) throw new Error(`the trace holds no \`${name}\` mark`);
    return at;
  }

  #within(from: number, to: number): Task[] {
    return this.tasks.filter((task) => task.start >= from && task.start < to);
  }

  /** The main thread's own time by kind between `from` and `to`, in milliseconds. */
  times(from: number, to: number): Times {
    const times = noTimes();
    for (const task of this.#within(from, to)) times[task.kind] += task.self / MS;
    return times;
  }

  /** How many events named `name` started between `from` and `to`. */
  count(name: string, from: number, to: number): number {
    return this.#within(from, to).filter((task) => task.name === name).length;
  }

  /** The main thread's tasks, outermost, between `from` and `to`. */
  topTasks(from: number, to: number): Task[] {
    return this.#within(from, to).filter((task) => task.depth === 0);
  }

  /** Every frame the main thread worked on between `from` and `to`. */
  frames(from: number, to: number): Frame[] {
    return this.#frames
      .filter((frame) => frame.start >= from && frame.start < to)
      .map((frame) => {
        const times = this.times(frame.start, frame.end);
        return { ...frame, times, busy: Object.values(times).reduce((sum, ms) => sum + ms, 0) };
      });
  }

  /**
   * The style and layout work script asked for while it ran (a read of a box's size after a
   * write), each named by the code that asked, most often first.
   */
  forced(from: number, to: number, most = 3): { count: number; time: number; by: string[] } {
    const forced = this.#within(from, to).filter((task) => {
      if (task.kind !== "layout" && task.kind !== "style") return false;
      for (let up = task.parent; up; up = up.parent) if (up.kind === "script") return true;
      return false;
    });
    const by = new Map<string, { count: number; time: number }>();
    for (const task of forced) {
      const stack = this.#stack(task.event.args?.beginData?.stackTrace);
      const each = by.get(stack) ?? { count: 0, time: 0 };
      by.set(stack, { count: each.count + 1, time: each.time + (task.end - task.start) / MS });
    }
    return {
      count: forced.length,
      time: forced.reduce((sum, task) => sum + (task.end - task.start) / MS, 0),
      by: [...by]
        .sort((a, b) => b[1].time - a[1].time)
        .slice(0, most)
        .map(([stack, { count, time }]) => `${stack} ×${count}, ${time.toFixed(1)} ms`),
    };
  }

  /** The code that made style or layout stale between `from` and `to`, most often first. */
  invalidatedBy(from: number, to: number): string[] {
    return mostOften(
      this.#instants.filter((event) => event.ts >= from && event.ts < to).map((event) => this.#stack(event.args?.data?.stackTrace)),
      3,
    );
  }

  /** The innermost frame of the site's code on `stack`, and its caller, or where it came from. */
  #stack(stack: readonly CallFrame[] | undefined): string {
    const ours = (stack ?? []).map((frame) => this.#modules.name(frame)).filter((name): name is string => name !== null);
    return ours.slice(0, 2).join(" ← ") || "(no script on the stack)";
  }

  /** The functions that ran longest between `from` and `to`, by the sampler, with their time. */
  hottest(from: number, to: number, most = 3): string[] {
    const times = new Map<string, number>();
    for (const sample of this.#samples) {
      if (sample.at < from || sample.at >= to) continue;
      const name = this.#modules.name(sample.frame) ?? sample.frame.functionName;
      times.set(name, (times.get(name) ?? 0) + sample.length);
    }
    return [...times]
      .sort((a, b) => b[1] - a[1])
      .slice(0, most)
      .map(([name, us]) => `${name} ${(us / MS).toFixed(1)} ms`);
  }

  /** What a task did, by kind, most first: `script 80 ms, style 20 ms`. */
  breakdown(task: Task): string {
    const times = this.times(task.start, task.end);
    return (Object.entries(times) as [Kind, number][])
      .filter(([, ms]) => ms >= 1)
      .sort((a, b) => b[1] - a[1])
      .map(([kind, ms]) => `${kind} ${ms.toFixed(0)} ms`)
      .join(", ");
  }
}
