// Kōbō's room: one screen, its wallpaper under everything, and on it either a full-screen scene
// (the welcome, the install running, the done screen) or the glass panel the questions are asked
// in (StepRailPage). The live room is the whole installer: Let's go lifts the welcome off the
// panel, holding the last button hands over to the install, and the install hands over to Done.
//
// Every move between them is a recipe (`Track`s), and the live room plays the same recipe the
// pages' players show, so what plays and what a library reads never disagree.

import "./kobo.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import blurred from "../../../../../crates/kubaos-kobo/assets/wallpaper-blur.jpg?url";
import markSvg from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-mark.svg?raw";
import wordmark from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-wordmark.svg?raw";
import sign from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-sign-1024.png?url";
import { backdrop, givesWay, glassPanel } from "../../lib/backdrop.ts";
import { button } from "../../lib/button.ts";
import { hangs } from "../../lib/button-bar.ts";
import { shown, type Words } from "../../lib/content.ts";
import { dialog } from "../../lib/dialog.ts";
import { icon } from "../../lib/icon.ts";
import { type KeyHint, keyHints } from "../../lib/keycap.ts";
import { fill, filling, gap, row, stack } from "../../lib/layout.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { rollingNumber } from "../../lib/rolling-number.ts";
import { spinner } from "../../lib/spinner.ts";
import { battery, koboBrand, type RailPage, stepRailPage } from "../../lib/step-rail.ts";
import { text, wrapped } from "../../lib/text.ts";
import { length, number } from "../../lib/tokens.ts";
import { bezier, css, cueOf, milliseconds, stepsOf, struck } from "../../site/recipe.ts";
import type { Duration, Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { paint, palette } from "../../site/recolour.ts";
import { settings } from "../../site/settings.ts";
import { DONE_HEADLINE, HERO, LAPTOP_DISK, LEDE, NETWORKS, RELEASE, STEPS, STICK, stepAt, TIPS, TOTAL, duration } from "./kobo-data.ts";
import {
  accountProblem,
  type Answers,
  detailsBody,
  diskBody,
  type DiskView,
  diskProblem,
  failedBody,
  filledAnswers,
  freshAnswers,
  languageBody,
  type LanguageView,
  type Machine,
  networkBody,
  networkProblem,
  reviewBody,
  type ReviewView,
  wallpaperNote,
  youBody,
  yoursBody,
  yoursProblem,
} from "./kobo-parts.ts";

/** The sample laptop: one disk with Windows, the stick, on battery, no cable in. */
export const LAPTOP: Machine = { disks: [LAPTOP_DISK, STICK], online: false, networks: NETWORKS, battery: { percent: 82, charging: false } };

/** A screen of the panel. */
export type Panel = "language" | "network" | "you" | "disk" | "yours" | "review" | "details" | "failed";

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
  readonly language?: LanguageView;
  readonly disk?: DiskView;
  readonly review?: ReviewView;
  readonly gitOpen?: boolean;
  /** The first wallpaper set's name, as the content's extremes show it. */
  readonly setName?: string;
  /** The words on the welcome and the done screen, at the toolbar's length. */
  readonly hero?: string;
  readonly lede?: string;
  readonly headline?: string;
}

/** The room's whole-screen size: the toolbar's window, or a laptop's screen. */
export const SCREEN = { width: "calc(1280 * var(--px))", height: "calc(800 * var(--px))" };

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
 */
export function play(root: HTMLElement, tracks: readonly Track[]): Promise<void> {
  const speed = settings().speed;
  let longest = 0;
  for (const track of tracks) {
    const targets = root.matches(track.target) ? [root] : [...root.querySelectorAll<HTMLElement>(track.target)];
    const from = css(track.from);
    const to = css(track.to);
    const duration = ms(track.duration);
    const delay = cueOf(track) / speed;
    longest = Math.max(longest, delay + duration);
    const frames: Keyframe[] = struck(track.curve)
      ? stepsOf(track.curve).map((part) => ({ [track.property]: typeof from === "number" && typeof to === "number" ? from + (to - from) * part : part < 0.5 ? from : to }))
      : [{ [track.property]: from }, { [track.property]: to }];
    const easing = struck(track.curve) ? `steps(${frames.length}, jump-none)` : `cubic-bezier(${bezier(track.curve).join(", ")})`;
    for (const target of targets) {
      const run = target.animate(struck(track.curve) ? frames : frames, { duration, delay, easing: struck(track.curve) ? "linear" : easing, fill: "both" });
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
function part<E extends HTMLElement>(element: E, name: string): E {
  element.classList.add("kobo-part", `kobo-${name}`);
  return element;
}

/** A column of a scene's parts, `size.column` wide, centred under one another. */
function column(name: string, ...parts: Element[]): HTMLElement {
  const box = stack(0, "centre", ...parts);
  box.classList.add("kobo-column", `kobo-${name}`);
  return box;
}

/** Kōbō's neon sign, a column wide, its height its picture's at that width on a whole pixel. */
function neonSign(): HTMLElement {
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
  const { index } = stepAt(share);
  // Two figures' room: at 100 the install hands over to the done screen.
  const percent = rollingNumber({ value: Math.min(99, Math.round(share * 100)), widest: "00", style: "hero_number" });
  const figure = row(0, "end", percent.element, text("%", "hero_number"));
  figure.classList.add("kobo-percent");
  const left = Math.max(0, TOTAL * (1 - share));
  const timing = new Intl.NumberFormat("en-GB", { style: "unit", unit: "minute", unitDisplay: "long", maximumFractionDigits: 0 }).format(Math.max(1, Math.ceil(left / 60)));
  const now = row(
    "s12",
    "centre",
    text(STEPS[index]!.title, "body_strong"),
    text(STEPS[index]!.detail ?? "", "mono", "primary"),
    fill(),
    icon("timer", "var(--icon-s16)", "var(--color-on_surface_variant)"),
    text(`About ${timing} left`, "small", "on_surface_variant"),
  );
  now.classList.add("kobo-status");
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
    part(stack("s12", "start", filling(progressBar({ fraction: share, timed: true })), filling(now)), "progress"),
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

/** The 完了 hanko, stamped beside the mark. */
function hanko(): HTMLElement {
  const seal = document.createElement("span");
  seal.className = "kobo-hanko";
  seal.append(text("完", "display"), text("了", "display"));
  return seal;
}

export function doneLine(): string {
  return `That's it. Done in ${duration(TOTAL)}. Not bad: Windows would still be on “Getting ready”. Pull the stick, reboot, type your password at boot and you're on the desktop as mika.`;
}

/** What Done says once Reboot is taken, in the actions' place. */
function restarting(): HTMLElement {
  const line = row("s8", "centre", spinner("s20"), text("Restarting…", "body", "on_surface_variant"));
  line.classList.add("kobo-status");
  return line;
}

export function doneColumn(going = false, headline = DONE_HEADLINE.short): HTMLElement {
  const words = headline.split(" ").map((word, index) => {
    const shown = part(text(index === 0 ? word : ` ${word}`, "hero"), `word-${index}`);
    shown.classList.add("kobo-word");
    return shown;
  });
  const line = row(0, "end", ...words);
  line.classList.add("kobo-headline");
  const reboot = button({ label: "Reboot", size: "Lg", icon: "power_settings_new" });
  reboot.classList.add("kobo-reboot");
  const stay = button({ label: "Stay in the live system", kind: "Text", size: "Lg" });
  const actions = going ? restarting() : row("s8", "centre", reboot, stay);
  actions.classList.add("kobo-status");
  const drawn = neonMark();
  const mark = row("s8", "start", drawn, part(hanko(), "stamp"));
  mark.classList.add("kobo-sealed");
  sealed(mark, drawn);
  return column(
    "done",
    mark,
    gap("s24"),
    line,
    gap("s16"),
    part(wrapped(doneLine(), "body_large", "on_surface_variant"), "lede"),
    gap("s40"),
    part(stack("s12", "centre", actions, keys(going ? [] : [["Enter", "reboot"], ["Tab", "move"]])), "act"),
  );
}

/** Key hints that keep a keycap's line of room with none to show, so nothing under them moves. */
function keys(hints: readonly KeyHint[]): HTMLElement {
  const line = keyHints(hints);
  line.classList.add("kobo-keys");
  return line;
}

/**
 * The 完了 hanko stands on the mark's baseline: its foot on the foot of the mark's lowest stroke,
 * as a seal is pressed beside a signature. Where that foot is, in the mark's box, is the drawing's
 * own (its strokes and their width), which only the drawing knows, so it is read from it.
 */
function sealed(row: HTMLElement, drawn: SVGSVGElement): void {
  const place = () => {
    const tubes = drawn.querySelector<SVGGElement>('[data-layer="tube"]');
    const view = drawn.viewBox.baseVal;
    if (!tubes || !view || view.height === 0 || drawn.clientHeight === 0) return;
    const ink = tubes.getBBox();
    const half = Number.parseFloat(tubes.querySelector("path")?.getAttribute("stroke-width") ?? "0") / 2;
    const frame = drawn.querySelector("g[transform]")?.getAttribute("transform")?.match(/translate\(([\d.]+) ([\d.]+)\)/);
    const shift = frame ? Number.parseFloat(frame[2]!) : 0;
    const foot = ((shift + ink.y + ink.height + half) / view.height) * drawn.clientHeight;
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

/** The panel's pages in the order the person meets them; Wi-Fi only with no network. */
function panels(machine: Machine): Panel[] {
  return ["language", ...(machine.online ? [] : (["network"] as const)), "you", "disk", "yours", "review", "details"] as Panel[];
}

/** The keys a screen takes that nothing on it shows, listed at the rail's foot. */
const HINTS: Record<Panel, readonly KeyHint[]> = {
  language: [["↑ ↓", "pick"], ["Tab", "next field"]],
  network: [["↑ ↓", "pick"], ["Tab", "next field"]],
  you: [["Tab", "next field"]],
  disk: [["↑ ↓", "pick"], ["Space", "switch"]],
  yours: [["Space", "pick or drop"], ["Tab", "next set"]],
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

/** The panel at `at`: StepRailPage with Kōbō's lockup, the flow's pages, power and battery. */
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
        return { step: { title: "Disk", icon: "hard_drive" }, body: () => diskBody(answers, machine, options.disk), blocker: () => diskProblem(answers, machine) };
      case "yours":
        return { step: { title: "Make it yours", icon: "wallpaper" }, body: () => yoursBody(answers, { ...(options.gitOpen ? { gitOpen: true } : {}), ...(options.setName ? { first: options.setName } : {}) }), blocker: () => yoursProblem(answers), note: () => wallpaperNote(answers) };
      case "review":
        return { step: { title: "Last look", icon: "list_alt" }, body: () => reviewBody(answers, machine, goTo, held, options.review), footless: true, fresh: true };
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

function panelLayer(flow: Flow, at: Panel, width: string, height: string): HTMLElement {
  const panel = glassPanel(blurred, panelTemplate(flow, at));
  const layer = backdrop({ width, height, picture: wallpaper, scrim: "dim", content: panel });
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
  new ResizeObserver(() => {
    const wide = element.clientWidth >= 2 * length("size-column") * (Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || settings().scale);
    element.style.setProperty("--kobo-margin", wide ? "var(--space-s32)" : "var(--space-s16)");
  }).observe(element);

  let where: At = options.at;
  let power: Power | undefined = options.power;
  let running: { stop: () => void } | null = null;

  const flow: Flow = { answers, machine, options, go: (to) => go(to) };

  /** Draws the room at `at`, over `leaving` as a move's first state; returns `at`'s layer. */
  const draw = (at: At, leaving?: At): HTMLElement => {
    const arriving = layerOf(at);
    const layers: HTMLElement[] = leaving ? [layerOf(leaving), arriving] : [arriving];
    // A scene leaving lies over the panel it lifts off; a scene arriving lies over the panel it covers.
    const scene = "scene" in at || (leaving !== undefined && "scene" in leaving);
    const parts: Element[] = [...orderLayers(layers)];
    if (scene) parts.push(foot(machine));
    if (power) parts.push(powerDialog(at, power, () => setPower(undefined)));
    element.replaceChildren(...parts);
    return arriving;
  };

  /**
   * Builds `at`'s layer beside the one shown, as a move's first state: a scene over what it
   * covers, a panel under the scene it is revealed by; the scene's foot with it.
   */
  const arrive = (at: At): HTMLElement => {
    const arriving = layerOf(at);
    const shown = [...element.children].filter((child) => child.classList.contains("k-backdrop") && !child.hasAttribute("data-under"));
    if ("scene" in at) (shown.at(-1) ?? element.firstChild)?.after(arriving) ?? element.prepend(arriving);
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

  /** The panel under any scene, a scene over the panel. */
  const orderLayers = (layers: Element[]) => [...layers].sort((a, b) => Number(a.classList.contains("kobo-scene")) - Number(b.classList.contains("kobo-scene")));

  const layerOf = (at: At): HTMLElement => ("panel" in at ? panelLayer(flow, at.panel, width, height) : sceneLayer(at, options, width, height));

  const setPower = (next: Power | undefined) => {
    power = next;
    element.querySelector(".kobo-over")?.remove();
    if (next) element.append(powerDialog(where, next, () => setPower(undefined)));
    else element.focus({ preventScroll: true });
  };

  // The live room builds the next place a press reveals once it is idle, under what shows, where
  // the picture hides it: the panel under the welcome, the install's scene under the panel. The
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
        const { index } = stepAt(share);
        const status = column.querySelector(".kobo-status");
        const words = status?.querySelectorAll(".k-text");
        if (words?.[0]) words[0].textContent = STEPS[index]!.title;
        if (words?.[1]) words[1].textContent = STEPS[index]!.detail ?? "";
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

/** Let's go: the welcome lifts off the panel; the panel rises in; the rail's wire strikes on. */
export const ENTERING: Move = {
  name: "Let's go: into the panel",
  what: "The welcome's words rise off and its band clears; the glass panel rises in; the wire strikes on.",
  tracks: [
    { part: "the welcome's words", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the words' rise", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: WELCOME, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", curve: "emphasized_accel" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "the panel's rise", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "the wire", on: "the rail", target: ".k-rail-wire", property: "opacity", from: 0, to: 1, duration: "medium", cue: ["micro", "long"], curve: "neon.strike" },
  ],
  unflickered: [
    { part: "the welcome's words", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the words' rise", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: WELCOME, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", curve: "emphasized_accel" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "the panel's rise", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "the wire", on: "the rail", target: ".k-rail-wire", property: "opacity", from: 0, to: 1, duration: "medium", cue: ["micro", "long"], curve: "standard" },
  ],
  reduced: [
    { part: "the welcome", on: WELCOME, target: ".kobo-scene", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { panel: "language" }, leaving: { scene: "welcome" } }),
};

/** Back from the first question: the welcome comes back over the panel. */
export const LEAVING: Move = {
  name: "Back to the welcome",
  what: "Esc or Back on Language: the panel sinks off, the band and the welcome's words come back.",
  tracks: [
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the panel's sink", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: WELCOME, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the welcome's words", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the words' fall", on: WELCOME, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: -1, to: 0, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the welcome", on: WELCOME, target: ".kobo-scene", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    { part: "the scene's foot", on: WELCOME, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { scene: "welcome" }, leaving: { panel: "language" } }),
};

const INSTALL = "the install";

/** Held to the end: the panel goes, the room dims to a scrim's dark, the install's scene rises in. */
export const STARTING: Move = {
  name: "Held: the install starts",
  what: "The panel sinks off; the room goes down to a scrim's dark; the figure and its bar rise in.",
  tracks: [
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the panel's sink", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
    { part: "the scene", on: INSTALL, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the room's dark", on: INSTALL, target: ".kobo-scene .k-backdrop-dark", property: "opacity", from: 0, to: { opacity: "scrim" }, duration: "slow", curve: "emphasized_decel" },
    ...rising(".kobo-installing > .kobo-accent, .kobo-installing > .kobo-figure", "the figure", INSTALL, "medium"),
    ...rising(".kobo-installing > .kobo-lede, .kobo-installing > .kobo-progress", "the bar", INSTALL, ["medium", "micro"]),
    ...rising(".kobo-installing > .kobo-tips, .kobo-installing > .kobo-act", "the tips", INSTALL, ["medium", "short"]),
    { part: "the foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
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

/** The details: the scene lifts off the panel, which rises with the steps and the log. */
export const DETAILS: Move = {
  name: "Show the steps and the log",
  what: "Tab and Enter: the scene's words rise off and its band clears; the panel rises in on Install.",
  tracks: [
    { part: "the scene's words", on: INSTALL, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the words' rise", on: INSTALL, target: ".kobo-scene > .k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "medium", curve: "emphasized_accel" },
    { part: "the band", on: INSTALL, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", curve: "emphasized_accel" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: "the panel's rise", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the scene", on: INSTALL, target: ".kobo-scene", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { panel: "details" }, leaving: { scene: "installing", share: 0.41 } }),
};

/** A failure: the bar turns `error` where it stopped, then the panel rises in on what broke. */
export const FAILING: Move = {
  name: "A step fails",
  what: "The bar drains to `error` where it stopped; the scene lifts off and the panel rises in on what broke.",
  tracks: [
    { part: "the scene's words", on: INSTALL, target: ".kobo-scene > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "medium", cue: "short", curve: "emphasized_accel" },
    { part: "the band", on: INSTALL, target: ".kobo-scene > .k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "long", cue: "short", curve: "emphasized_accel" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "micro", cue: "short", curve: "standard" },
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "long", cue: ["short", "micro"], curve: "emphasized_decel" },
    { part: "the panel's rise", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "long", cue: ["short", "micro"], curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the scene", on: INSTALL, target: ".kobo-scene", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the scene's foot", on: INSTALL, target: ".kobo-foot", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the panel", on: "the panel", target: ".kobo-panel > .k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => room({ at: { panel: "failed" }, leaving: { scene: "installing", share: 0.38 } }),
};

const DONE = "the done screen";

/** Every word of the headline rising in, a `micro` after the last. */
function words(cue: readonly Duration[]): Track<"emphasized_decel">[] {
  return DONE_HEADLINE.short.split(" ").flatMap((_, index) => rising(`.kobo-word-${index}`, `word ${index + 1}`, DONE, [...cue, ...Array<Duration>(index).fill("micro")] as unknown as NonEmpty<Duration>));
}

/** The mark written in, a stroke every `short`, each along its length over `short`. */
function writing(): Track<"standard">[] {
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

const DONE_TRACKS = (strike: "neon.strike" | "standard"): Track[] => [
  { part: "the street coming close", on: DONE, target: ".kobo-scene .k-backdrop-picture", property: "scale", from: 1, to: { scale: "push" }, duration: "scene", curve: "emphasized_decel" },
  ...writing(),
  { part: "the mark, struck", on: DONE, target: ".kobo-mark", property: "--kobo-burn", from: 0, to: 1, duration: "medium", cue: "scene", curve: strike },
  { part: "the hanko", on: DONE, target: ".kobo-hanko", property: "--kobo-stamp", from: 0, to: 1, duration: "short", cue: ["scene", "medium"], curve: "pop" },
  ...words(["scene", "medium"]),
  ...rising(".kobo-done > .kobo-lede", "the lede", DONE, ["scene", "medium", "short"]),
  ...rising(".kobo-done > .kobo-act", "the actions", DONE, ["scene", "medium", "medium"]),
];

/** The install done: the street comes close, the mark writes itself in and strikes, the hanko lands. */
export const ARRIVING: Move = {
  name: "Done: the arrival",
  what: "The street comes toward you; クバ writes itself in and strikes; 完了 is stamped; the headline rises word by word.",
  tracks: DONE_TRACKS("neon.strike") as unknown as NonEmpty<Track>,
  unflickered: DONE_TRACKS("standard") as unknown as NonEmpty<Track<Easing>>,
  reduced: [
    { part: "the street coming close", on: DONE, target: ".kobo-scene .k-backdrop-picture", property: "scale", from: 1, to: { scale: "push" }, duration: "short", curve: "standard" },
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
function after(by: Duration, tracks: readonly Track[]): Track[] {
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
  tracks: ENDING(DONE_TRACKS("neon.strike")) as unknown as NonEmpty<Track>,
  unflickered: ENDING(DONE_TRACKS("standard")) as unknown as NonEmpty<Track<Easing>>,
  reduced: [
    { part: "the install's words", on: INSTALL, target: ".kobo-scene[data-scene='installing'] > .k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
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

/** The moves a panel page shares, as their words. */
export const WORDS: { readonly hero: Words; readonly lede: Words; readonly headline: Words } = { hero: HERO, lede: LEDE, headline: DONE_HEADLINE };

export { shown };
