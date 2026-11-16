// Kōbō's room: one screen, its wallpaper under everything, and on it either a scene (the welcome,
// the install running, the done screen) or the questions (StepRailPage), every one of them the
// whole screen: the questions are the street gone out of focus behind glass, with the steps on it.
// The live room is the whole installer: Let's go takes the welcome off the questions, holding the
// last button hands over to the install, and the install hands over to Done.
//
// Every move between them is a recipe (`Track`s), and the live room plays the same recipe the
// pages' players show, so what plays and what a library reads never disagree.

import "./kobo.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import blurred from "../../../../../crates/kubaos-kobo/assets/wallpaper-blur.jpg?url";
import markSvg from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-mark.svg?raw";
import wordmark from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-wordmark.svg?raw";
import sign from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-sign-1024.png?url";
import { backdrop, givesWay } from "../../lib/backdrop.ts";
import { button } from "../../lib/button.ts";
import { hangs } from "../../lib/button-bar.ts";
import { shown, type Words } from "../../lib/content.ts";
import { dialog } from "../../lib/dialog.ts";
import { icon } from "../../lib/icon.ts";
import { type KeyHint, keyHints } from "../../lib/keycap.ts";
import { gap, row, stack } from "../../lib/layout.ts";
import { setCaption, setProgress } from "../../lib/progress.ts";
import { rollingNumber } from "../../lib/rolling-number.ts";
import { spinner } from "../../lib/spinner.ts";
import { battery, koboBrand, type RailPage, stepRailPage } from "../../lib/step-rail.ts";
import { text, wrapped } from "../../lib/text.ts";
import { length, number } from "../../lib/tokens.ts";
import { bezier, css, cueOf, milliseconds, stepsOf, struck } from "../../site/recipe.ts";
import type { Duration, Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { paint, palette } from "../../site/recolour.ts";
import { settings } from "../../site/settings.ts";
import { DONE_HEADLINE, HERO, LAPTOP_DISK, LEDE, NETWORKS, RELEASE, STICK, TIPS, TOTAL, duration } from "./kobo-data.ts";
import {
  accountProblem,
  type Answers,
  detailsBody,
  diskBody,
  diskProblem,
  failedBody,
  filledAnswers,
  freshAnswers,
  installProgress,
  languageBody,
  type LanguageView,
  type Machine,
  networkBody,
  networkProblem,
  reviewBody,
  stepLine,
  timeCaption,
  type ReviewView,
  youBody,
} from "./kobo-parts.ts";

/** The sample laptop: one disk with Windows, the stick, on battery, no cable in. */
export const LAPTOP: Machine = { disks: [LAPTOP_DISK, STICK], online: false, networks: NETWORKS, battery: { percent: 82, charging: false } };

/** A question, or the install's details and failure, on glass. */
export type Panel = "language" | "network" | "you" | "disk" | "review" | "details" | "failed";

/** What the welcome waits on: the hardware being read, a failed read, a stick with no image. */
export type Moment = "rest" | "loading" | "error" | "noimage";

/** Where the room is. */
export type At =
  | { readonly scene: "welcome"; readonly moment?: Moment }
  | { readonly panel: Panel }
  | { readonly scene: "installing"; readonly share: number; readonly tip?: number }
  | { readonly scene: "done"; readonly going?: boolean };

/** The power dialog, open: asked, or the install stopping on the way down. */
export type Power = "ask" | "stopping";

export interface RoomOptions {
  readonly at: At;
  readonly width?: string;
  readonly height?: string;
  readonly machine?: Machine;
  readonly answers?: Answers;
  /** It takes the keyboard and the pointer and moves on through the installer. */
  readonly live?: boolean;
  readonly power?: Power;
  /** The welcome plays its opening as it is drawn. */
  readonly opening?: boolean;
  /** A move's first state: the layer it leaves drawn over the one it arrives on. */
  readonly leaving?: At;
  /** The question the room's page was reached from, drawn leaving it: a step move's first state. */
  readonly stepFrom?: Panel;
  readonly language?: LanguageView;
  readonly review?: ReviewView;
  /** The words on the welcome and the done screen, at the toolbar's length. */
  readonly hero?: string;
  readonly lede?: string;
  readonly headline?: string;
}

/** The room's whole-screen size: the toolbar's window, or a laptop's screen. */
export const SCREEN = { width: "calc(1280 * var(--px))", height: "calc(800 * var(--px))" };

/** A room's layer's size: the room's own, whatever it is set to. */
export const ROOM = { width: "var(--kobo-width)", height: "var(--kobo-height)" };

// ---------------------------------------------------------------- Playing --

/** A duration token in milliseconds at the speed picked. */
function ms(token: Duration): number {
  return milliseconds(token) / settings().speed;
}

/** Which of a move's track sets plays now: reduced, the flicker off, or its own. */
function tracksNow(move: Move): readonly Track[] {
  const { reduced, flicker } = settings();
  if (reduced) return move.reduced === "same" ? move.tracks : move.reduced;
  if (!flicker && move.unflickered) return move.unflickered;
  return move.tracks;
}

/**
 * Plays `tracks` on `root` as the players do: each on its targets, from its cue for its duration,
 * on its curve or the neon's steps, holding its first value until its cue and its last after.
 * Returns when the last one ends.
 *
 * Where several tracks move one property of one element (the street going dark, then lighting up),
 * only the first to start holds its starting value before its cue; a later one would hold its own
 * over the earlier track the whole time it waits. They are started in the order they
 * start, so each later one lies over the one before once it runs.
 */
export function play(root: HTMLElement, tracks: readonly Track[]): Promise<void> {
  const speed = settings().speed;
  let longest = 0;
  const timed = tracks.map((track) => ({ track, delay: cueOf(track) / speed })).sort((a, b) => a.delay - b.delay);
  const held = new Map<Element, Set<string>>();
  for (const { track, delay } of timed) {
    const targets = root.matches(track.target) ? [root] : [...root.querySelectorAll<HTMLElement>(track.target)];
    const from = css(track.from);
    const to = css(track.to);
    const duration = ms(track.duration);
    longest = Math.max(longest, delay + duration);
    const frames: Keyframe[] = struck(track.curve)
      ? stepsOf(track.curve).map((part) => ({ [track.property]: typeof from === "number" && typeof to === "number" ? from + (to - from) * part : part < 0.5 ? from : to }))
      : [{ [track.property]: from }, { [track.property]: to }];
    const easing = struck(track.curve) ? "linear" : `cubic-bezier(${bezier(track.curve).join(", ")})`;
    for (const target of targets) {
      const moved = held.get(target) ?? new Set<string>();
      held.set(target, moved);
      const first = !moved.has(track.property);
      moved.add(track.property);
      // A strike's frames open on its first flicker, not on where it starts from: before its cue it
      // holds its starting value as the element's own, not its first frame.
      if (first && struck(track.curve)) {
        if (track.property === "opacity") target.style.opacity = String(from);
        else target.style.setProperty(track.property, String(from));
      }
      const fill: FillMode = first && !struck(track.curve) ? "both" : "forwards";
      const run = target.animate(frames, { duration, delay, easing, fill });
      // The end state is the element's own once the move is over, so a later move starts from it.
      run.finished.then(() => {
        if (track.property === "opacity") target.style.opacity = String(to);
        else target.style.setProperty(track.property, String(to));
        run.cancel();
      }, () => {});
    }
  }
  return new Promise((done) => setTimeout(done, longest));
}

// ----------------------------------------------------------------- Scenes --

/** A scene's part: it fades and rises into place as the scene's recipe says. */
export function part<E extends HTMLElement>(element: E, name: string): E {
  element.classList.add("kobo-part", `kobo-${name}`);
  return element;
}

/** A column of a scene's parts, `size.column` wide, centred under one another. */
export function column(name: string, ...parts: Element[]): HTMLElement {
  const box = stack(0, "centre", ...parts);
  box.classList.add("kobo-column", `kobo-${name}`);
  return box;
}

/** Kōbō's neon sign, a column wide, its height its picture's at that width on a whole pixel. */
export function neonSign(): HTMLElement {
  const picture = document.createElement("img");
  picture.className = "kobo-sign";
  picture.src = sign;
  picture.alt = "KubaOS";
  const fit = () => {
    // Its height is the picture's own proportion at the column's width, which only the picture knows.
    if (picture.naturalWidth > 0 && picture.offsetWidth > 0) picture.style.height = `${Math.round((picture.offsetWidth * picture.naturalHeight) / picture.naturalWidth)}px`;
  };
  picture.addEventListener("load", fit);
  new ResizeObserver(fit).observe(picture);
  return part(givesWay(picture), "sign-part");
}

/** The line under the welcome's action: what it waits on, what went wrong, or what it promises. */
function welcomeStatus(moment: Moment): HTMLElement {
  const line = (glyph: Element, words: string, color: string) => row("s8", "centre", glyph, text(words, "body", color));
  const shown =
    moment === "loading"
      ? line(spinner("s20"), "Poking at the hardware…", "on_surface_variant")
      : moment === "error"
        ? line(icon("error", "var(--type-body-icon)", "var(--color-error)"), "Couldn't read this machine: lsblk timed out.", "error")
        : moment === "noimage"
          ? line(icon("error", "var(--type-body-icon)", "var(--color-error)"), "No system image on this stick. Rebuild the ISO and flash it again.", "error")
          : line(icon("info", "var(--type-body-icon)", "var(--color-on_surface_variant)"), "Nothing touches a disk until you hold the button on the last screen.", "on_surface_variant");
  shown.classList.add("kobo-status");
  return shown;
}

export function welcomeColumn(moment: Moment = "rest", hero = HERO.short, lede = LEDE.long): HTMLElement {
  const action =
    moment === "error"
      ? button({ label: "Try again", size: "Lg", icon: "restart_alt" })
      : button({ label: "Let's go", size: "Lg", icon: "arrow_forward", trailing: true, ...(moment === "rest" ? {} : { disabled: true }) });
  action.classList.add("kobo-start");
  const hints: readonly KeyHint[] = moment === "noimage" || moment === "loading" ? [] : [["Enter", moment === "error" ? "try again" : "start"]];
  const keyLine = keys(hints);
  return column(
    "welcome",
    part(text("ようこそ", "accent", "primary"), "accent"),
    gap("s8"),
    neonSign(),
    gap("s24"),
    part(wrapped(hero, "hero"), "hero"),
    gap("s16"),
    part(wrapped(lede, "body_large", "on_surface_variant"), "lede"),
    gap("s40"),
    // The action and its key are one thing, close; the promise under them is a statement of its own.
    part(stack(0, "centre", action, gap("s12"), keyLine, gap("s24"), welcomeStatus(moment)), "act"),
  );
}

/** The tip shown while the install runs, the next ones laid in the same place, unseen. */
function tipLine(index: number): HTMLElement {
  const tip = TIPS[index % TIPS.length]!;
  return row("s12", "centre", keyHints([[tip.keys, ""]]), text(tip.does, "body", "on_surface_variant"));
}

/** The install as its scene shows it, `share` of the way through. */
export function installingColumn(share: number, tip = 0): HTMLElement {
  // Two figures' room: at 100 the install hands over to the done screen.
  const percent = rollingNumber({ value: Math.min(99, Math.round(share * 100)), widest: "00", style: "hero_number" });
  const figure = row(0, "end", percent.element, text("%", "hero_number"));
  figure.classList.add("kobo-percent");
  const tips = document.createElement("span");
  tips.className = "kobo-tip";
  const shownTip = tipLine(tip);
  tips.append(shownTip);
  const details = button({ label: "Show the steps and the log", kind: "Text", icon: "terminal" });
  details.classList.add("kobo-details");
  const box = column(
    "installing",
    part(text("インストール中", "accent", "primary"), "accent"),
    gap("s8"),
    part(figure, "figure"),
    gap("s8"),
    part(wrapped("Go stare at a wall, it'll be done before you get bored.", "body_large", "on_surface_variant"), "lede"),
    gap("s32"),
    part(installProgress(share), "progress"),
    gap("s40"),
    part(stack("s8", "centre", text("While you wait", "label", "on_surface_variant"), tips), "tips"),
    gap("s40"),
    part(stack("s16", "centre", details, keyHints([["Tab", "the steps and the log"]])), "act"),
  );
  box.dataset.share = String(share);
  (box as HTMLElement & { roll?: (value: number) => void }).roll = percent.set;
  return box;
}

/**
 * The クバ mark as lit tubes, from the logo's own drawing: each stroke three times, its glow, its
 * tube and its white-hot core, warm in `primary` and cold in `cold`, each stroke drawn along its
 * length as its own `--kobo-stroke` goes from 1 to 0, in the order a hand writes them.
 */
let marks = 0;

export function neonMark(): SVGSVGElement {
  const drawing = new DOMParser().parseFromString(markSvg, "image/svg+xml").documentElement as unknown as SVGSVGElement;
  // Every mark on the page draws its glow with a blur of its own name.
  const glow = `kobo-glow-${marks++}`;
  drawing.querySelector("filter")?.setAttribute("id", glow);
  const paths = [...drawing.querySelectorAll("path")];
  // The drawing is its glow group's strokes, then the tubes', then the cores', in the strokes' order.
  const count = paths.length / 3;
  const layers = ["glow", "tube", "core"] as const;
  const groups = layers.map((layer) => {
    const group = document.createElementNS("http://www.w3.org/2000/svg", "g");
    group.dataset.layer = layer;
    if (layer === "glow") group.setAttribute("filter", `url(#${glow})`);
    return group;
  });
  paths.forEach((path, index) => {
    const stroke = index % count;
    path.removeAttribute("stroke");
    path.setAttribute("pathLength", "1");
    // The last two strokes are the dakuten, the cold ones.
    path.dataset.tone = stroke >= count - 2 ? "cold" : "warm";
    path.dataset.stroke = String(stroke);
    groups[Math.floor(index / count)]!.append(path);
  });
  const frame = drawing.querySelector("g[transform]")!;
  frame.replaceChildren(...groups);
  drawing.removeAttribute("width");
  drawing.removeAttribute("height");
  drawing.classList.add("kobo-mark");
  drawing.setAttribute("aria-label", "クバ");
  return drawing;
}

/** The 完了 hanko, stamped beside the mark as first boot ends. */
export function hanko(): HTMLElement {
  const seal = document.createElement("span");
  seal.className = "kobo-hanko";
  seal.append(text("完", "display"), text("了", "display"));
  return seal;
}

/** Done's lede. Its quoted words are one name, so a space that never breaks holds them on one line. */
export function doneLine(): string {
  return `That's it. Done in ${duration(TOTAL)}. Not bad: Windows would still be on “Getting\u00a0ready”. Pull the stick, reboot, type your password at boot and you're on the desktop as mika.`;
}

/** What Done says once Reboot is taken, in the actions' place. */
function restarting(): HTMLElement {
  const line = row("s8", "centre", spinner("s20"), text("Restarting…", "body", "on_surface_variant"));
  line.classList.add("kobo-status");
  return line;
}

/** What an ending says and offers: Kōbō's done screen's, and first boot's. */
export interface Ending {
  readonly headline: string;
  readonly lede: string;
  /** The actions' line, already laid out: buttons, or what a press set going. */
  readonly actions: HTMLElement;
  readonly hints: readonly KeyHint[];
  /** A line under the keys, where something waits on the next screen. */
  readonly promise?: HTMLElement;
}

/**
 * An ending's column, as both of Kōbō's endings are laid out and moved: クバ alone, the headline
 * word by word, the lede, the actions with their keys.
 */
export function endingColumn(ending: Ending): HTMLElement {
  const words = ending.headline.split(" ").map((word, index) => {
    const shown = part(text(index === 0 ? word : ` ${word}`, "hero"), `word-${index}`);
    shown.classList.add("kobo-word");
    return shown;
  });
  const line = row(0, "end", ...words);
  line.classList.add("kobo-headline");
  ending.actions.classList.add("kobo-status");
  const sign = document.createElement("span");
  sign.append(neonMark());
  const pressed = stack("s12", "centre", ending.actions, keys(ending.hints));
  const act = ending.promise ? stack(0, "centre", pressed, gap("s24"), ending.promise) : pressed;
  return column("done", part(sign, "sign-on"), gap("s24"), line, gap("s16"), part(wrapped(ending.lede, "body_large", "on_surface_variant"), "lede"), gap("s40"), part(act, "act"));
}

export function doneColumn(going = false, headline = DONE_HEADLINE.short): HTMLElement {
  const reboot = button({ label: "Reboot", size: "Lg", icon: "power_settings_new" });
  reboot.classList.add("kobo-reboot");
  const stay = button({ label: "Stay in the live system", kind: "Text", size: "Lg" });
  const actions = going ? restarting() : row("s8", "centre", reboot, stay);
  // Stay's padding hangs past the row's end, so the pair centred is what you see: Reboot's fill to
  // Stay's last letter, not to its unseen padding.
  if (!going) actions.dataset.hangs = "";
  return endingColumn({ headline, lede: doneLine(), actions, hints: going ? [] : [["Enter", "reboot"], ["Tab", "move"]] });
}

/** Key hints that keep a keycap's line of room with none to show, so nothing under them moves. */
export function keys(hints: readonly KeyHint[]): HTMLElement {
  const line = keyHints(hints);
  line.classList.add("kobo-keys");
  return line;
}

/**
 * The 完了 hanko stands on the mark's optical baseline, as a seal is pressed beside a signature. The
 * lowest stroke ends in a round cap, which overshoots the baseline as a round letter does, so the
 * baseline is the stroke's end plus half its cap's radius (a quarter of the tube's width), not the
 * cap's lowest pixel. Where that is in the mark's box is the drawing's own (its strokes and their
 * width), which only the drawing knows, so it is read from it, and holds at every scale.
 */
export function sealed(row: HTMLElement, drawn: SVGSVGElement): void {
  const place = () => {
    const tubes = drawn.querySelector<SVGGElement>('[data-layer="tube"]');
    const view = drawn.viewBox.baseVal;
    if (!tubes || !view || view.height === 0 || drawn.clientHeight === 0) return;
    const ink = tubes.getBBox();
    const overshoot = Number.parseFloat(tubes.querySelector("path")?.getAttribute("stroke-width") ?? "0") / 4;
    const frame = drawn.querySelector("g[transform]")?.getAttribute("transform")?.match(/translate\(([\d.]+) ([\d.]+)\)/);
    const shift = frame ? Number.parseFloat(frame[2]!) : 0;
    const foot = ((shift + ink.y + ink.height + overshoot) / view.height) * drawn.clientHeight;
    row.style.setProperty("--kobo-ink-foot", `${foot}px`);
  };
  new ResizeObserver(place).observe(drawn);
}

/** The foot of a scene: Power and the battery where the rail holds them, the release between. */
function foot(machine: Machine): HTMLElement {
  const power = button({ label: "Power", kind: "Text", size: "Sm", icon: "power_settings_new" });
  power.dataset.power = "";
  const release = text(`KubaOS ${RELEASE.version} ${RELEASE.codename} · ${RELEASE.commit} · ${RELEASE.arch} · UEFI`, "caption", "on_surface_variant");
  const box = document.createElement("span");
  box.className = "kobo-foot";
  box.append(hangs(row(0, "centre", power)), release, machine.battery ? battery(machine.battery.percent, machine.battery.charging) : document.createElement("span"));
  return box;
}

// ---------------------------------------------------------------- The flow --

/** The questions in the order the person meets them; Wi-Fi only with no network. */
function panels(machine: Machine): Panel[] {
  return ["language", ...(machine.online ? [] : (["network"] as const)), "you", "disk", "review", "details"] as Panel[];
}

/** The keys a screen takes that nothing on it shows, listed at the rail's foot. */
const HINTS: Record<Panel, readonly KeyHint[]> = {
  language: [["↑ ↓", "pick"], ["Tab", "next field"]],
  network: [["↑ ↓", "pick"], ["Tab", "next field"]],
  you: [["Tab", "next field"]],
  disk: [["↑ ↓", "pick"], ["Space", "switch"]],
  review: [["Esc", "back"]],
  details: [["Esc", "back to the view"]],
  failed: [["Tab", "move"]],
};

interface Flow {
  readonly answers: Answers;
  readonly machine: Machine;
  readonly options: RoomOptions;
  /** Moves the room to `to`, playing the move between. */
  go(to: At): void;
}

/** The questions at `at`: StepRailPage with Kōbō's lockup, the flow's pages, power and battery. */
function panelTemplate(flow: Flow, at: Panel): HTMLElement {
  const { answers, machine, options } = flow;
  const order = panels(machine);
  const failed = at === "failed";
  const held = (hold: HTMLElement, bar: HTMLElement) => holding(hold, bar, () => flow.go({ scene: "installing", share: 0 }));
  const goTo = (page: string) => (page === "back" ? flow.go({ panel: order[order.indexOf("review") - 1]! }) : flow.go({ panel: page as Panel }));
  const pages: RailPage[] = order.map((page): RailPage => {
    switch (page) {
      case "language":
        return { step: { title: "Language", icon: "language" }, body: () => languageBody(answers, options.language) };
      case "network":
        return { step: { title: "Wi-Fi", icon: "wifi" }, body: () => networkBody(answers, machine), blocker: () => networkProblem(answers, machine), next: () => (answers.network === -1 ? "Skip" : "Continue") };
      case "you":
        return { step: { title: "You", icon: "person" }, body: () => youBody(answers), blocker: () => accountProblem(answers) };
      case "disk":
        return { step: { title: "Disk", icon: "hard_drive" }, body: () => diskBody(answers, machine), blocker: () => diskProblem(answers, machine) };
      case "review":
        return { step: { title: "Last look", icon: "list_alt" }, body: () => reviewBody(answers, machine, goTo, held, options.review), footless: true, fresh: true, lands: ".kobo-hold-button" };
      default:
        return {
          step: { title: "Install", icon: "download" },
          body: () => (failed ? failedBody(answers, machine, { retry: () => flow.go({ scene: "installing", share: 0 }), review: () => flow.go({ panel: "review" }) }) : detailsBody(0.41, () => flow.go({ scene: "installing", share: 0.41 }))),
          footless: true,
          fixed: true,
          fresh: true,
        };
    }
  });
  const index = failed ? order.indexOf("details") : order.indexOf(at);
  return stepRailPage({
    host: "kobo",
    brand: koboBrand(markSvg, wordmark, `Kōbō · ${RELEASE.codename} ${RELEASE.version}`),
    pages,
    current: index,
    reached: index,
    ...(failed ? { failed: true } : {}),
    ...(at === "details" ? { locked: true } : {}),
    ...(options.stepFrom ? { leaving: order.indexOf(options.stepFrom) } : {}),
    hints: (page) => HINTS[failed ? "failed" : (order[page] ?? "language")],
    foot: { power: true, ...(machine.battery ? { battery: machine.battery.percent, charging: machine.battery.charging } : {}) },
    ...(options.live ? { onExit: (forward: boolean) => (forward ? undefined : flow.go({ scene: "welcome" })) } : {}),
  });
}

/**
 * The hold to install: Enter or the pointer held on it fills the banner's foot over `hold`, its
 * words Keep holding…; let go early and it drains over `short`; held to the end, the install starts.
 */
function holding(hold: HTMLElement, bar: HTMLElement, done: () => void): void {
  let started = 0;
  let frame = 0;
  const label = hold.querySelector(".k-button-label");
  const cta = label?.textContent ?? "";
  const tick = () => {
    const share = Math.min(1, (performance.now() - started) / milliseconds("hold"));
    setProgress(bar, share);
    if (share >= 1) {
      stop(false);
      done();
      return;
    }
    frame = requestAnimationFrame(tick);
  };
  const start = () => {
    if (started) return;
    started = performance.now();
    label?.replaceChildren("Keep holding…");
    frame = requestAnimationFrame(tick);
  };
  const stop = (drain = true) => {
    if (!started) return;
    started = 0;
    cancelAnimationFrame(frame);
    label?.replaceChildren(cta);
    if (drain) bar.animate([{ "--progress": bar.style.getPropertyValue("--progress") }, { "--progress": 0 }], { duration: ms("short"), easing: `cubic-bezier(${bezier("standard").join(", ")})` }).finished.then(() => setProgress(bar, 0), () => {});
  };
  hold.addEventListener("pointerdown", start);
  hold.addEventListener("pointerup", () => stop());
  hold.addEventListener("pointerleave", () => stop());
  hold.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      event.stopPropagation();
      if (!event.repeat) start();
    }
  });
  hold.addEventListener("keyup", (event) => {
    if (event.key === "Enter" || event.key === " ") stop();
  });
}

/** The power dialog: what going down now does, by where the install stands. */
function powerDialog(at: At, power: Power, close: () => void): HTMLElement {
  const running = "scene" in at && at.scene === "installing";
  const body = running
    ? "The install stops where it is. Samsung 990 Pro is left half written and won't boot: install KubaOS on it again to use it."
    : "scene" in at && at.scene === "done"
      ? "KubaOS is installed. Pull the USB stick first, or the machine starts this installer again."
      : "Nothing has been written to any disk yet.";
  const made = dialog({
    tone: running ? "Danger" : "Normal",
    icon: "power_settings_new",
    title: running ? "Stop the install?" : "Power off or restart?",
    body,
    ...(power === "stopping" ? { asks: row("s12", "centre", spinner("s20", running ? "error" : "primary"), text("Stopping the install and unmounting the disk…", "body", "on_surface_variant")) } : { dismiss: "Cancel" }),
    actions:
      power === "stopping"
        ? []
        : [
            { label: running ? "Stop and restart" : "Restart", kind: "Tonal", icon: "restart_alt" },
            { label: running ? "Stop and power off" : "Power off", icon: "power_settings_new" },
          ],
    landed: true,
  });
  made.onAnswer = (answer) => {
    if (answer === "Cancel" || answer === "Esc") close();
  };
  const over = document.createElement("span");
  over.className = "kobo-over";
  over.append(made.element);
  requestAnimationFrame(() => made.land());
  return over;
}

/** A scene's backdrop: the wallpaper, the band behind its words, its column. */
function sceneLayer(at: Extract<At, { scene: string }>, options: RoomOptions, width: string, height: string): HTMLElement {
  const content =
    at.scene === "welcome"
      ? welcomeColumn(at.moment ?? "rest", options.hero, options.lede)
      : at.scene === "installing"
        ? installingColumn(at.share, at.tip ?? 0)
        : doneColumn(at.going ?? false, options.headline);
  const dark = at.scene === "installing" ? dusk() * (1 - at.share) : at.scene === "done" && at.going ? 1 : 0;
  const layer = backdrop({ width, height, picture: wallpaper, scrim: "band", content, dark, ...(at.scene === "done" ? { push: 1 } : {}) });
  layer.classList.add("kobo-scene");
  layer.dataset.scene = at.scene;
  const band = document.createElement("span");
  band.className = "kobo-foot-band";
  layer.querySelector(".k-backdrop-layers")?.append(band);
  return layer;
}

/** The questions' layer: the street out of focus behind glass, the steps filling the screen on it. */
function panelLayer(flow: Flow, at: Panel, width: string, height: string): HTMLElement {
  const layer = backdrop({ width, height, picture: blurred, scrim: "glass", content: panelTemplate(flow, at) });
  layer.classList.add("kobo-panel");
  return layer;
}

/** How dark the room is as the install starts: a dialog's scrim, lifting as the system arrives. */
function dusk(): number {
  return number("opacity-scrim");
}

/** The room, at `options.at`: live, it is the whole installer. */
export function room(options: RoomOptions): HTMLElement {
  const width = options.width ?? SCREEN.width;
  const height = options.height ?? SCREEN.height;
  const machine = options.machine ?? LAPTOP;
  const answers = options.answers ?? (("panel" in options.at && options.at.panel !== "language") || ("scene" in options.at && options.at.scene !== "welcome") ? filledAnswers(machine) : freshAnswers(machine));
  const element = document.createElement("span");
  element.className = "kobo-room";
  // A whole screen: a motion player draws it as it is laid out.
  element.dataset.whole = "";
  element.tabIndex = -1;
  element.style.setProperty("--kobo-width", width);
  element.style.setProperty("--kobo-height", height);
  // Kōbō runs on the live ISO, which is always dark.
  element.dataset.mode = "dark";
  paint(element, palette(settings().theme, "dark"));
  // The width the observer hands over, at the scale read once a scale: a room shown again reads
  // nothing back, and the margin changes only when its step does.
  let scaled: { readonly at: number; readonly is: number } | null = null;
  new ResizeObserver((entries) => {
    const width = entries.at(-1)?.borderBoxSize[0]?.inlineSize ?? element.clientWidth;
    if (scaled?.at !== settings().scale) scaled = { at: settings().scale, is: Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || settings().scale };
    const margin = width >= 2 * length("size-column") * scaled.is ? "var(--space-s32)" : "var(--space-s16)";
    if (element.style.getPropertyValue("--kobo-margin") !== margin) element.style.setProperty("--kobo-margin", margin);
  }).observe(element);

  let where: At = options.at;
  let power: Power | undefined = options.power;
  let running: { stop: () => void } | null = null;

  const flow: Flow = { answers, machine, options, go: (to) => go(to) };

  /** Draws the room at `at`, over `leaving` as a move's first state; returns `at`'s layer. */
  const draw = (at: At, leaving?: At): HTMLElement => {
    const arriving = layerOf(at);
    const layers: HTMLElement[] = leaving ? [layerOf(leaving), arriving] : [arriving];
    // A scene leaving lies over the questions it lifts off; a scene arriving lies over those it covers.
    const scene = "scene" in at || (leaving !== undefined && "scene" in leaving);
    const parts: Element[] = [...orderLayers(layers)];
    if (scene) parts.push(foot(machine));
    if (power) parts.push(powerDialog(at, power, () => setPower(undefined)));
    element.replaceChildren(...parts);
    return arriving;
  };

  /**
   * Builds `at`'s layer beside the one shown, as a move's first state: a scene over what it
   * covers, the questions under the scene they are revealed by; the scene's foot with it.
   */
  const arrive = (at: At): HTMLElement => {
    const arriving = layerOf(at);
    const shown = [...element.children].filter((child) => child.classList.contains("k-backdrop") && !child.hasAttribute("data-under"));
    const top = shown.at(-1);
    // HAZARD: `after` returns nothing, so an `after(…) ?? prepend(…)` chain always prepends too,
    // burying an arriving scene under the one it should cover until the move ends.
    if ("scene" in at && top) top.after(arriving);
    else element.prepend(arriving);
    if ("scene" in at && !element.querySelector(".kobo-foot")) arriving.after(foot(machine));
    return arriving;
  };

  /**
   * Reboot taken: the done screen shown stays, its actions giving way to what it is doing, so the
   * press builds no screen; the lights going out is the move that follows.
   */
  const goingDown = (): HTMLElement => {
    const layer = element.querySelector<HTMLElement>('.kobo-scene[data-scene="done"]');
    if (!layer) return draw({ scene: "done", going: true });
    layer.querySelector(".kobo-done .kobo-status")?.replaceWith(restarting());
    layer.querySelector(".kobo-done .kobo-keys")?.replaceChildren();
    return layer;
  };

  /** The questions under any scene, a scene over the questions. */
  const orderLayers = (layers: Element[]) => [...layers].sort((a, b) => Number(a.classList.contains("kobo-scene")) - Number(b.classList.contains("kobo-scene")));

  // Its layers take the room's size, so a room made a whole screen (the site's full screen) takes
  // them with it.
  const layerOf = (at: At): HTMLElement => ("panel" in at ? panelLayer(flow, at.panel, ROOM.width, ROOM.height) : sceneLayer(at, options, ROOM.width, ROOM.height));

  const setPower = (next: Power | undefined) => {
    power = next;
    element.querySelector(".kobo-over")?.remove();
    if (next) element.append(powerDialog(where, next, () => setPower(undefined)));
    else element.focus({ preventScroll: true });
  };

  // The live room builds the next place a press reveals once it is idle, under what shows, where
  // the picture hides it: the questions under the welcome, the install's scene under them. The
  // press then only plays the move, with nothing to build in it.
  const prebuilt = new Map<string, HTMLElement>();
  const keyOf = (at: At): string => ("panel" in at ? "panel" : at.scene);
  const prebuild = (at: At) => {
    requestIdleCallback(() => {
      if (!element.isConnected || prebuilt.has(keyOf(at)) || keyOf(where) === keyOf(at)) return;
      const layer = layerOf(at);
      layer.dataset.under = "";
      layer.inert = true;
      element.append(layer);
      prebuilt.set(keyOf(at), layer);
    });
  };
  const ahead = () => {
    if ("scene" in where && where.scene === "welcome") prebuild({ panel: "language" });
    if ("panel" in where) prebuild({ scene: "installing", share: 0 });
  };

  const go = (to: At): void => {
    const from = where;
    where = to;
    running?.stop();
    running = null;
    if (!options.live) return;
    const move = between(from, to);
    const same = "scene" in from && "scene" in to && from.scene === to.scene;
    const ready = prebuilt.get(keyOf(to));
    const reuse = ready !== undefined && move !== null && !same && !("panel" in to && to.panel !== "language");
    for (const [key, layer] of prebuilt) if (!reuse || layer !== ready) (layer.remove(), prebuilt.delete(key));
    let kept: HTMLElement;
    if (reuse && ready) {
      kept = ready;
      prebuilt.delete(keyOf(to));
      kept.inert = false;
      // A scene arriving over the panel goes over it, its foot with it; a panel revealed under a
      // scene stays under it until the scene is gone.
      if ("scene" in to) {
        delete kept.dataset.under;
        element.append(kept);
        if (!element.querySelector(".kobo-foot")) kept.after(foot(machine));
      }
    } else if (same && "scene" in to && to.scene === "done" && to.going) kept = goingDown();
    else kept = move && !same ? arrive(to) : draw(to);
    const keyboard = () => (element.querySelector<HTMLElement>(".kobo-start, .kobo-reboot, .kobo-details") ?? element.querySelector<HTMLElement>(".k-step-room [tabindex='0'], .k-step-room button:not(:disabled)") ?? element).focus({ preventScroll: true });
    // The move's end is the new place drawn: only the layer it left, and a scene's foot under the
    // panel, are taken away.
    const settle = () => {
      for (const child of [...element.children]) if (child !== kept && child.classList.contains("k-backdrop")) child.remove();
      delete kept.dataset.under;
      if ("panel" in to) element.querySelector(".kobo-foot")?.remove();
      keyboard();
      if ("scene" in to && to.scene === "installing") running = simulate(element, to.share, () => go({ scene: "done" }));
      ahead();
    };
    if (!move) {
      settle();
      return;
    }
    void play(element, tracksNow(move)).then(() => {
      if (where === to) settle();
    });
  };

  draw(where, options.leaving);

  if (options.live) {
    element.addEventListener("click", (event) => {
      const target = event.target as HTMLElement;
      if (target.closest("[data-power]")) return setPower("ask");
      if (target.closest(".kobo-start") && "scene" in where && where.scene === "welcome") go({ panel: "language" });
      else if (target.closest(".kobo-details")) go({ panel: "details" });
      else if (target.closest(".kobo-reboot")) go({ scene: "done", going: true });
    });
    element.addEventListener("keydown", (event) => {
      if (event.defaultPrevented || event.repeat) return;
      if (power && event.key === "Escape") {
        event.preventDefault();
        setPower(undefined);
        return;
      }
      if (!("scene" in where)) return;
      if (event.key === "Enter") {
        event.preventDefault();
        if (where.scene === "welcome" && (where.moment ?? "rest") === "rest") go({ panel: "language" });
        else if (where.scene === "done" && !where.going) go({ scene: "done", going: true });
      }
    });
    if ("scene" in where && where.scene === "installing") running = simulate(element, where.share, () => go({ scene: "done" }));
    if (!options.leaving) ahead();
  }
  if (options.opening) requestAnimationFrame(() => void play(element, tracksNow(OPENING)));
  return element;
}

/**
 * The install running in the live room, sped up so it is seen whole: `scene` ten times over, its
 * figure, bar, step, room's light and tip following.
 */
function simulate(element: HTMLElement, from: number, done: () => void): { stop: () => void } {
  // The sample install's length: a demo's pace, not a design value.
  const span = milliseconds("scene") * 10 * (1 - from);
  const start = performance.now();
  let tip = 0;
  let last = -1;
  let frame = 0;
  const tick = () => {
    const share = Math.min(1, from + ((performance.now() - start) / span) * (1 - from));
    const column = element.querySelector<HTMLElement & { roll?: (value: number) => void }>(".kobo-installing");
    const scene = element.querySelector<HTMLElement>(".kobo-scene");
    if (column && scene) {
      const percent = Math.round(share * 100);
      if (percent !== last) {
        column.roll?.(Math.min(99, percent));
        last = percent;
        const bar = column.querySelector<HTMLElement>(".k-progress");
        if (bar) setProgress(bar, share);
        const dark = scene.querySelector<HTMLElement>(".k-backdrop-dark");
        if (dark) dark.style.opacity = String(dusk() * (1 - share));
        const status = column.querySelector<HTMLElement>(".kobo-install-progress .kobo-status");
        const next = stepLine(share);
        if (status && status.dataset.says !== next.dataset.says) status.replaceWith(next);
        const captioned = column.querySelector<HTMLElement>(".k-progress-captioned");
        const caption = timeCaption(share);
        if (captioned && captioned.dataset.says !== (caption?.words ?? "")) {
          captioned.dataset.says = caption?.words ?? "";
          setCaption(captioned, caption);
        }
      }
      const now = Math.floor(((performance.now() - start) / milliseconds("scene")) / 3);
      if (now !== tip) {
        tip = now;
        const tips = column.querySelector<HTMLElement>(".kobo-tip");
        if (tips) {
          const next = tipLine(tip);
          tips.append(next);
          void play(tips, tracksNow(tipMove(tips.children.length)));
          setTimeout(() => tips.firstElementChild !== next && tips.firstElementChild?.remove(), ms("medium") + ms("micro"));
        }
      }
    }
    if (share >= 1) {
      done();
      return;
    }
    frame = requestAnimationFrame(tick);
  };
  frame = requestAnimationFrame(tick);
  return { stop: () => cancelAnimationFrame(frame) };
}

// ---------------------------------------------------------------- The moves --

/** A part rising `space.s24` into place as it fades in, over `long`, from `cue`. */
function rising(target: string, partName: string, on: string, cue?: Duration | NonEmpty<Duration>): Track<"emphasized_decel">[] {
  const timing = cue === undefined ? {} : { cue };
  return [
    { part: `${partName}'s fade`, on, target, property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel", ...timing },
    { part: `${partName}'s rise`, on, target, property: "--kobo-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel", ...timing },
  ];
}

/** A part fading in over `short`, reduced motion's form of an arrival. */
function fading(target: string, partName: string, on: string): Track<"standard"> {
  return { part: `${partName}'s fade`, on, target, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" };
}

const WELCOME = "the welcome";

/** Kōbō starts: the street lights up and settles back, the sign strikes, the words rise in order. */
export const OPENING: Move = {
  name: "Kōbō starts",
  what: "Black, then the street lights up round its band and settles back; the sign strikes; the words rise in, one after another.",
  tracks: [
    { part: "the street's light", on: WELCOME, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 1, to: 0, duration: "slow", curve: "emphasized_decel" },
    { part: "the street settling", on: WELCOME, target: ".kobo-scene .k-backdrop-picture", property: "scale", from: { scale: "push" }, to: 1, duration: "scene", curve: "emphasized_decel" },
    { part: "the accent", on: WELCOME, target: ".kobo-accent", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    { part: "the sign", on: WELCOME, target: ".kobo-sign", property: "--kobo-sign", from: 0, to: 1, duration: "medium", cue: "short", curve: "neon.strike" },
    ...rising(".kobo-hero", "the hero", WELCOME, ["short", "medium"]),
    ...rising(".kobo-lede", "the lede", WELCOME, ["short", "medium", "micro"]),
    ...rising(".kobo-act", "the action", WELCOME, ["short", "medium", "short"]),
    { part: "the foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "long", cue: ["short", "medium", "short"], curve: "emphasized_decel" },
  ],
  unflickered: [
    { part: "the street's light", on: WELCOME, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 1, to: 0, duration: "slow", curve: "emphasized_decel" },
    { part: "the street settling", on: WELCOME, target: ".kobo-scene .k-backdrop-picture", property: "scale", from: { scale: "push" }, to: 1, duration: "scene", curve: "emphasized_decel" },
    { part: "the accent", on: WELCOME, target: ".kobo-accent", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    { part: "the sign", on: WELCOME, target: ".kobo-sign", property: "--kobo-sign", from: 0, to: 1, duration: "medium", cue: "short", curve: "standard" },
    ...rising(".kobo-hero", "the hero", WELCOME, ["short", "medium"]),
    ...rising(".kobo-lede", "the lede", WELCOME, ["short", "medium", "micro"]),
    ...rising(".kobo-act", "the action", WELCOME, ["short", "medium", "short"]),
    { part: "the foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "long", cue: ["short", "medium", "short"], curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the street's light", on: WELCOME, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    fading(".kobo-welcome", "the words", WELCOME),
    fading(".kobo-foot", "the foot", WELCOME),
  ],
  back: "cuts",
  sample: () => room({ at: { scene: "welcome" } }),
};

/** Let's go: the welcome lifts off, the street goes out of focus, the questions rise in. */
export const ENTERING: Move = {
  name: "Let's go: into the questions",
  what: "The welcome's words rise off; the street goes out of focus behind glass; the questions rise in; the wire strikes on.",
  tracks: [
    { part: "the welcome's words", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the words' rise", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: WELCOME, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", curve: "emphasized_accel" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "their rise", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "the wire", on: "the rail", target: ".k-rail-wire", property: "opacity", from: 0, to: 1, duration: "medium", cue: ["micro", "long"], curve: "neon.strike" },
  ],
  unflickered: [
    { part: "the welcome's words", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the words' rise", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: WELCOME, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", curve: "emphasized_accel" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "their rise", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "the wire", on: "the rail", target: ".k-rail-wire", property: "opacity", from: 0, to: 1, duration: "medium", cue: ["micro", "long"], curve: "standard" },
  ],
  reduced: [
    { part: "the welcome", on: WELCOME, target: ".kobo-scene", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { panel: "language" }, leaving: { scene: "welcome" } }),
};

/** Back from the first question: the welcome comes back over the questions. */
export const LEAVING: Move = {
  name: "Back to the welcome",
  what: "Esc or Back on Language: the questions sink off, the street comes into focus, the welcome's words come back.",
  tracks: [
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "their sink", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: WELCOME, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the welcome's words", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the words' fall", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: -1, to: 0, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the welcome", on: WELCOME, target: ".kobo-scene", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { scene: "welcome" }, leaving: { panel: "language" } }),
};

const INSTALL = "the install";

/** Held to the end: the questions go, the street comes into focus at a scrim's dark, the install rises in. */
export const STARTING: Move = {
  name: "Held: the install starts",
  what: "The questions sink off; the street comes back into focus at a scrim's dark; the figure and its bar rise in.",
  tracks: [
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "their sink", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
    { part: "the scene", on: INSTALL, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the room's dark", on: INSTALL, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 0, to: { opacity: "scrim" }, duration: "slow", curve: "emphasized_decel" },
    ...rising(".kobo-installing > .kobo-accent, .kobo-installing > .kobo-figure", "the figure", INSTALL, "medium"),
    ...rising(".kobo-installing > .kobo-lede, .kobo-installing > .kobo-progress", "the bar", INSTALL, ["medium", "micro"]),
    ...rising(".kobo-installing > .kobo-tips, .kobo-installing > .kobo-act", "the tips", INSTALL, ["medium", "short"]),
    { part: "the foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the scene", on: INSTALL, target: ".kobo-scene", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    { part: "the foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { scene: "installing", share: 0 }, leaving: { panel: "review" } }),
};

/** A step done: the figure rolls, the bar eases on, the room lifts a little out of its dark. */
export const ADVANCING: Move = {
  name: "The light follows the work",
  what: "Sped up: as the bar runs from empty to full, the room's dark lifts with it; the street lights as the system arrives.",
  tracks: [
    { part: "the bar", on: INSTALL, target: ".kobo-installing .k-progress", property: "--progress", from: 0, to: 1, duration: "slow", curve: "linear" },
    { part: "the room's dark", on: INSTALL, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: { opacity: "scrim" }, to: 0, duration: "slow", curve: "linear" },
  ],
  reduced: "same",
  back: "cuts",
  sample: () => room({ at: { scene: "installing", share: 0 } }),
};

/** The tip changing: the old one sinks out, the next rises in where it was. */
function tipMove(count = 2): Move {
  return {
    name: "The next tip",
    what: "Every three `scene`s: the tip fades off, then the next rises in where it was.",
    tracks: [
      { part: "the old tip", on: "the tips", target: `.kobo-tip > :nth-child(${count - 1})`, property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
      ...rising(`.kobo-tip > :nth-child(${count})`, "the next tip", "the tips", "medium"),
    ],
    reduced: [
      { part: "the old tip", on: "the tips", target: `.kobo-tip > :nth-child(${count - 1})`, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
      { ...fading(`.kobo-tip > :nth-child(${count})`, "the next tip", "the tips"), cue: "short" },
    ],
    back: "cuts",
    sample: () => {
      const made = room({ at: { scene: "installing", share: 0.41 } });
      made.querySelector(".kobo-tip")?.append(tipLine(1));
      return made;
    },
  };
}

export const TIP: Move = tipMove();

/** The details: the scene lifts off, and the questions' screen rises with the steps and the log. */
export const DETAILS: Move = {
  name: "Show the steps and the log",
  what: "Tab and Enter: the scene's words rise off, the street goes out of focus; Install rises in with the steps and the log.",
  tracks: [
    { part: "the scene's words", on: INSTALL, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the words' rise", on: INSTALL, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: INSTALL, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", curve: "emphasized_accel" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "their rise", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the scene", on: INSTALL, target: ".kobo-scene", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { panel: "details" }, leaving: { scene: "installing", share: 0.41 } }),
};

/** A failure: the bar turns `error` where it stopped, then the glass rises in on what broke. */
export const FAILING: Move = {
  name: "A step fails",
  what: "The bar drains to `error` where it stopped; the scene lifts off and Install rises in on what broke.",
  tracks: [
    { part: "the scene's words", on: INSTALL, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", cue: "short", curve: "emphasized_accel" },
    { part: "the band", on: INSTALL, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", cue: "short", curve: "emphasized_accel" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", cue: "short", curve: "standard" },
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: ["short", "micro"], curve: "emphasized_decel" },
    { part: "their rise", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: ["short", "micro"], curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the scene", on: INSTALL, target: ".kobo-scene", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the questions", on: "the questions", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { panel: "failed" }, leaving: { scene: "installing", share: 0.38 } }),
};

const DONE = "the done screen";

/** Every word of `headline` rising in, a `micro` after the last. */
function words(headline: string, cue: readonly Duration[]): Track<"emphasized_decel">[] {
  return headline.split(" ").flatMap((_, index) => rising(`.kobo-word-${index}`, `word ${index + 1}`, DONE, [...cue, ...Array<Duration>(index).fill("micro")] as unknown as NonEmpty<Duration>));
}

/** The mark written in, a stroke every `short`, each along its length over `short`. */
export function writing(): Track<"standard">[] {
  return [0, 1, 2, 3, 4, 5].map((stroke) => ({
    part: `stroke ${stroke + 1}`,
    on: DONE,
    target: `.kobo-mark path[data-stroke="${stroke}"]`,
    property: "--kobo-stroke",
    from: 1,
    to: 0,
    duration: "short",
    ...(stroke === 0 ? {} : { cue: Array<Duration>(stroke).fill("short") as unknown as NonEmpty<Duration> }),
    curve: "standard",
  }));
}

/**
 * The arrival: the street goes dark as クバ is written in as unlit glass, then it strikes, and as it
 * comes on the street lights up again round it and comes toward you: the machine is on, and yours.
 */
export const arrival = (strike: "neon.strike" | "standard", headline = DONE_HEADLINE.short): Track[] => [
  { part: "the street going dark", on: DONE, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
  ...writing(),
  { part: "the mark, struck", on: DONE, target: ".kobo-mark", property: "--kobo-burn", from: 0, to: 1, duration: "medium", cue: "scene", curve: strike },
  { part: "the street lighting up", on: DONE, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 1, to: 0, duration: "slow", cue: ["scene", "short"], curve: "emphasized_decel" },
  { part: "the street coming close", on: DONE, target: ".kobo-scene .k-backdrop-picture", property: "scale", from: 1, to: { scale: "push" }, duration: "scene", cue: ["scene", "short"], curve: "emphasized_decel" },
  ...words(headline, ["scene", "medium"]),
  ...rising(".kobo-done > .kobo-lede", "the lede", DONE, ["scene", "medium", "short"]),
  ...rising(".kobo-done > .kobo-act", "the actions", DONE, ["scene", "medium", "medium"]),
];

/** The install done: the street goes dark, the mark writes itself in and strikes, the street lights. */
export const ARRIVING: Move = {
  name: "Done: the arrival",
  what: "The street goes dark as クバ writes itself in, unlit; it strikes, the street lights up and comes close; the headline rises.",
  tracks: arrival("neon.strike") as unknown as NonEmpty<Track>,
  unflickered: arrival("standard") as unknown as NonEmpty<Track<Easing>>,
  reduced: [
    fading(".kobo-done", "the done screen", DONE),
  ],
  back: "cuts",
  sample: () => room({ at: { scene: "done" } }),
};

/** Reboot: the mark switches off, the street goes dark, the words go with it. */
export const GOING: Move = {
  name: "Reboot: the lights go out",
  what: "Restarting…: クバ switches off as the boot splash's sign does; the street and the words go dark over `slow`.",
  tracks: [
    { part: "the mark, off", on: DONE, target: ".kobo-mark", property: "--kobo-burn", from: 1, to: 0, duration: "medium", curve: "neon.strike.reversed" },
    { part: "the room going dark", on: DONE, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 0, to: 1, duration: "slow", cue: "short", curve: "emphasized_accel" },
    { part: "the words", on: DONE, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "slow", cue: "short", curve: "emphasized_accel" },
    { part: "the foot", on: DONE, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "medium", cue: "short", curve: "emphasized_accel" },
  ],
  unflickered: [
    { part: "the mark, off", on: DONE, target: ".kobo-mark", property: "--kobo-burn", from: 1, to: 0, duration: "medium", curve: "standard" },
    { part: "the room going dark", on: DONE, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 0, to: 1, duration: "slow", cue: "short", curve: "emphasized_accel" },
    { part: "the words", on: DONE, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "slow", cue: "short", curve: "emphasized_accel" },
    { part: "the foot", on: DONE, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "medium", cue: "short", curve: "emphasized_accel" },
  ],
  reduced: [
    { part: "the room going dark", on: DONE, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the words", on: DONE, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { scene: "done", going: true } }),
};

/** `tracks` played `by` later: its phase put before each track's own cue. */
export function after(by: Duration, tracks: readonly Track[]): Track[] {
  return tracks.map((track) => ({ ...track, cue: [by, ...(track.cue === undefined ? [] : typeof track.cue === "string" ? [track.cue] : track.cue)] as unknown as NonEmpty<Duration> }));
}

const ENDING = (arrival: readonly Track[]): Track[] => [
  { part: "the install's words", on: INSTALL, target: ".kobo-scene[data-scene='installing'] > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
  { part: "the words' rise", on: INSTALL, target: ".kobo-scene[data-scene='installing'] > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
  { part: "the done screen's band", on: DONE, target: ".kobo-scene[data-scene='done'] > .k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ...after("medium", arrival),
];

/** The install done, from its scene: the install's words rise off, and the done screen arrives. */
export const FINISHING: Move = {
  name: "The install ends",
  what: "At 100%: the figure and its bar rise off over `medium`; then the done screen's arrival plays.",
  tracks: ENDING(arrival("neon.strike")) as unknown as NonEmpty<Track>,
  unflickered: ENDING(arrival("standard")) as unknown as NonEmpty<Track<Easing>>,
  reduced: [
    { part: "the install's words", on: INSTALL, target: ".kobo-scene[data-scene='installing'] > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    // The street stays at its own size, as the install left it: the done screen's picture comes
    // close only as a move, and two pictures at two sizes fading over each other double every sign.
    { part: "the street, as it is", on: DONE, target: ".kobo-scene[data-scene='done'] .k-backdrop-picture", property: "scale", from: 1, to: 1, duration: "instant", curve: "standard" },
    { part: "the done screen", on: DONE, target: ".kobo-scene[data-scene='done']", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { scene: "done" }, leaving: { scene: "installing", share: 1 } }),
};

/** The move between two places of the room, if it has one of its own. */
function between(from: At, to: At): Move | null {
  if ("scene" in from && from.scene === "welcome" && "panel" in to) return ENTERING;
  if ("panel" in from && "scene" in to && to.scene === "welcome") return LEAVING;
  if ("panel" in from && "scene" in to && to.scene === "installing") return STARTING;
  if ("scene" in from && from.scene === "installing" && "panel" in to) return DETAILS;
  if ("scene" in from && from.scene === "installing" && "scene" in to && to.scene === "done") return FINISHING;
  if ("scene" in to && to.scene === "done" && to.going) return GOING;
  return null;
}

/** The words the welcome and the done screen show, as their extremes. */
export const WORDS: { readonly hero: Words; readonly lede: Words; readonly headline: Words } = { hero: HERO, lede: LEDE, headline: DONE_HEADLINE };

export { shown };
