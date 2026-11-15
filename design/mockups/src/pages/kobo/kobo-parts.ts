// Kōbō's installer screens inside its glass panel, as organisms of library parts: Language, Network,
// You, Disk, Make it yours, Last look, and the install's details and failure. Each screen's body
// reads and writes one `Answers`, and says when what its footer shows has changed (`RECHECK`).
//
// The common path is upfront on every screen, already answered where a good answer can be guessed;
// what only an advanced user changes (more layouts, the username and the machine's name, the
// encryption passphrase, Git) sits behind a Collapsible or a quiet button on the same screen.

import { banner } from "../../lib/banner.ts";
import { button } from "../../lib/button.ts";
import { hangs } from "../../lib/button-bar.ts";
import { card, choiceCard, setPicked } from "../../lib/card.ts";
import { checkbox } from "../../lib/check.ts";
import { chip } from "../../lib/chip.ts";
import { collapsible } from "../../lib/collapsible.ts";
import { descriptionList } from "../../lib/description.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { field, labelled, setInvalid } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { keycap } from "../../lib/keycap.ts";
import { fill, filling, fillingHeight, gap, grid, header, row, stack } from "../../lib/layout.ts";
import { type ListRowSpec, list, scrollingList } from "../../lib/list.ts";
import { log } from "../../lib/log.ts";
import { meter } from "../../lib/meter.ts";
import { pictureChoice } from "../../lib/picture-choice.ts";
import { progressBar } from "../../lib/progress.ts";
import { scroll } from "../../lib/scroll.ts";
import { select } from "../../lib/select.ts";
import { spinner } from "../../lib/spinner.ts";
import { RECHECK } from "../../lib/step-rail.ts";
import { step } from "../../lib/step.ts";
import { switchControl } from "../../lib/switch.ts";
import { text, wrapped } from "../../lib/text.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import {
  bytes,
  type Disk,
  duration,
  FAILURE,
  LANGUAGES,
  LAYOUTS,
  LOG,
  NEEDED,
  type Network,
  RELEASE,
  SAMPLE_TIME,
  SETS,
  signalIcon,
  STEPS,
  stepAt,
  TOTAL,
  ZONES,
  zoneName,
} from "./kobo-data.ts";

/** Every answer the installer asks for, as the person leaves it. */
export interface Answers {
  language: number;
  /** The layouts picked, by index, the default first. */
  layouts: number[];
  zone: number;
  /** The layout and the zone still follow the language: nobody changed them. */
  following: boolean;
  /** The network picked, by index, or -1: the cable, or none. */
  network: number;
  networkPassword: string;
  name: string;
  username: string;
  /** The username was typed, so it no longer follows the name. */
  ownUsername: boolean;
  host: string;
  password: string;
  confirm: string;
  /** The disk picked, by its index among the machine's disks. */
  disk: number;
  mode: "alongside" | "erase";
  encrypt: boolean;
  samePassword: boolean;
  passphrase: string;
  passphraseConfirm: string;
  /** The wallpapers picked, as `set/name`. */
  wallpapers: Set<string>;
  gitName: string;
  gitEmail: string;
  /** The username and machine name drawn open. */
  namesOpen: boolean;
  gitOpen: boolean;
}

/** What the probe found: the disks, the network, the battery. */
export interface Machine {
  readonly disks: readonly Disk[];
  readonly online: boolean;
  /** The networks in range, or still being looked for. */
  readonly networks: readonly Network[] | "scanning";
  /** The battery's percent and whether it charges; none on a desktop. */
  readonly battery?: { readonly percent: number; readonly charging: boolean };
}

/** The key a wallpaper is picked by. */
export function wallpaperKey(set: string, name: string): string {
  return `${set}/${name}`;
}

/** A disk can take KubaOS beside what is on it. */
export function fitsAlongside(disk: Disk): boolean {
  return disk.free >= NEEDED;
}

/** The answers as Kōbō starts them: the image's machine name, encrypted, every wallpaper. */
export function freshAnswers(machine: Machine): Answers {
  const disk = machine.disks.findIndex((each) => !each.stick);
  const target = machine.disks[disk];
  return {
    language: 0,
    layouts: [LANGUAGES[0]!.layout],
    zone: LANGUAGES[0]!.zone,
    following: true,
    network: -1,
    networkPassword: "",
    name: "",
    username: "",
    ownUsername: false,
    host: "tuxedo",
    password: "",
    confirm: "",
    disk,
    mode: target?.other && fitsAlongside(target) ? "alongside" : "erase",
    encrypt: true,
    samePassword: true,
    passphrase: "",
    passphraseConfirm: "",
    wallpapers: new Set(SETS.flatMap((set) => set.wallpapers.map((each) => wallpaperKey(set.name, each.name)))),
    gitName: "",
    gitEmail: "",
    namesOpen: false,
    gitOpen: false,
  };
}

/** The answers a person who has been through every screen leaves. */
export function filledAnswers(machine: Machine, more: Partial<Answers> = {}): Answers {
  return { ...freshAnswers(machine), name: "Mika Nowak", username: "mika", password: "correct horse battery", confirm: "correct horse battery", ...more };
}

/** The username a name suggests: its first word, lowercase, its accents dropped. */
export function usernameOf(name: string): string {
  const first = name.trim().split(/\s+/)[0] ?? "";
  return first
    .normalize("NFD")
    .replaceAll(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .replaceAll(/[^a-z0-9_-]/g, "");
}

const RESERVED = new Set(["root", "daemon", "bin", "sys", "nobody", "admin", "nixbld"]);

export const USERNAME_RULE = "Usernames are lowercase: a-z first, then a-z, 0-9, - or _.";
export const HOSTNAME_RULE = "Lowercase a-z, 0-9 and -, up to 63, with no - at either end.";

export function usernameProblem(username: string): string | undefined {
  if (username === "") return "Pick a username.";
  if (!/^[a-z][a-z0-9_-]*$/.test(username)) return USERNAME_RULE;
  if (RESERVED.has(username)) return "That one's a system account. Pick another.";
  return undefined;
}

export function hostProblem(host: string): string | undefined {
  if (host === "") return "Name this machine.";
  if (!/^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$/.test(host)) return HOSTNAME_RULE;
  return undefined;
}

/** The confirmation so far is the start of the original: not wrong yet, just unfinished. */
function stillTyping(original: string, confirm: string): boolean {
  return confirm !== original && original.startsWith(confirm);
}

function pairProblem(first: string, again: string, words: { readonly none: string; readonly again: string; readonly differ: string }): string | undefined {
  if (first === "") return words.none;
  if (stillTyping(first, again)) return words.again;
  if (first !== again) return words.differ;
  return undefined;
}

export function accountProblem(answers: Answers): string | undefined {
  if (answers.name.trim() === "") return "Type your name.";
  return (
    usernameProblem(answers.username) ??
    hostProblem(answers.host) ??
    pairProblem(answers.password, answers.confirm, { none: "Pick a password.", again: "Type it again to confirm.", differ: "The passwords don't match." })
  );
}

export function diskProblem(answers: Answers, machine: Machine): string | undefined {
  const targets = machine.disks.filter((each) => !each.stick);
  if (targets.length === 0) return "No disks. None at all.";
  const disk = machine.disks[answers.disk];
  if (!disk || disk.stick) return "Pick a disk.";
  if (disk.bytes < NEEDED) return "This disk is too small for KubaOS.";
  if (answers.mode === "alongside" && !fitsAlongside(disk)) return "Not enough free space to live alongside. Erase it, or shrink something first.";
  if (answers.encrypt && !answers.samePassword) {
    return pairProblem(answers.passphrase, answers.passphraseConfirm, {
      none: "Pick an encryption passphrase.",
      again: "Type the passphrase again to confirm.",
      differ: "The passphrases don't match.",
    });
  }
  return undefined;
}

export function networkProblem(answers: Answers, machine: Machine): string | undefined {
  const networks = machine.networks === "scanning" ? [] : machine.networks;
  const picked = networks[answers.network];
  if (!picked?.secured) return undefined;
  const count = [...answers.networkPassword].length;
  if (count === 0) return "Type its password, or go with the cable.";
  if (count < 8) return "Wi-Fi passwords are at least 8 characters.";
  return undefined;
}

export function looksLikeEmail(words: string): boolean {
  const [user, host] = words.split("@");
  return Boolean(user) && host !== undefined && host.includes(".") && !host.startsWith(".") && !host.endsWith(".");
}

export function yoursProblem(answers: Answers): string | undefined {
  if (answers.wallpapers.size === 0) return "Pick at least one wallpaper.";
  const [name, email] = [answers.gitName.trim(), answers.gitEmail.trim()];
  if (name === "" && email !== "") return "Git wants a name to go with that email.";
  if (email !== "" && !looksLikeEmail(email)) return "That isn't an email address.";
  return undefined;
}

/** How many wallpapers there are and how many are picked, as the footer says it. */
export function wallpaperNote(answers: Answers): string | undefined {
  const total = SETS.reduce((sum, set) => sum + set.wallpapers.length, 0);
  const picked = answers.wallpapers.size;
  if (picked === 0) return undefined;
  return picked === total ? `All ${total} picked.` : `${picked} of ${total} picked.`;
}

/** Tells the footer the answers changed. */
function recheck(from: Element): void {
  from.dispatchEvent(new Event(RECHECK, { bubbles: true }));
}

/**
 * `content` in a box padded at its end by the page's padding, which the scroll reaches across: a
 * grid splits the box's own width, so the padding is the box's, not the grid's.
 */
function scrolled(content: HTMLElement): HTMLElement {
  const box = stack(0, "start", filling(content));
  box.classList.add("kobo-scrolled");
  return box;
}

/** A screen's body: its header, then what it asks, `space.s24` apart, the rest scrolling. */
function body(accent: string, title: string, subtitle: string, content: HTMLElement, scrolls = true): HTMLElement {
  const top = header({ accent, title, subtitle });
  if (!scrolls) return stack("s24", "start", filling(top), fillingHeight(filling(content)));
  const { box } = scroll(scrolled(content), { height: "100%", above: "s24", below: "s16", noGutter: true, fade: "s24", footFade: true });
  box.classList.add("kobo-scrolls");
  return stack("s24", "start", filling(top), fillingHeight(filling(box)));
}

/** A field that keeps `set` told of what is typed, its note following it. */
function typed(options: Parameters<typeof field>[0], set: (value: string) => void): HTMLElement {
  return field({ ...options, onInput: (value) => set(value) });
}

/** The Show and Hide toggle for the secret fields on a screen, at a label's right. */
function showToggle(fields: () => readonly HTMLElement[]): HTMLElement {
  const toggle = button({ label: "Show", kind: "Text", size: "Sm", icon: "visibility", alternatives: ["Show", "Hide"] });
  let shown = false;
  toggle.addEventListener("click", () => {
    shown = !shown;
    toggle.querySelector(".k-button-label")?.replaceChildren(shown ? "Hide" : "Show");
    for (const each of fields()) each.querySelector<HTMLElement>(".k-field-value")?.toggleAttribute("data-shown", shown);
  });
  return toggle;
}

// ---------------------------------------------------------------- Language --

/** The date, time and a number as the picked language writes them, at the zone's clock. */
function reads(answers: Answers): HTMLElement {
  const language = LANGUAGES[answers.language]!;
  const zone = ZONES[answers.zone]!;
  const date = new Intl.DateTimeFormat(language.tag, { dateStyle: "full", timeZone: zone.iana }).format(SAMPLE_TIME);
  const time = new Intl.DateTimeFormat(language.tag, { timeStyle: "short", timeZone: zone.iana }).format(SAMPLE_TIME);
  const number = new Intl.NumberFormat(language.tag, { maximumFractionDigits: 2 }).format(1234567.89);
  return descriptionList([
    { label: "Date", value: date },
    { label: "Time", value: time },
    { label: "Numbers", value: number },
  ]);
}

/** The Language screen's options: what it is drawn with. */
export interface LanguageView {
  readonly query?: string;
  /** The layout list drawn open, as a picture of picking one. */
  readonly open?: "layout" | "zone";
  /** The first language's name, as the content's extremes show it. */
  readonly first?: string;
}

export function languageBody(answers: Answers, view: LanguageView = {}): HTMLElement {
  const rows = (): ListRowSpec[] =>
    LANGUAGES.filter((each) => !view.query || each.name.toLowerCase().includes(view.query.toLowerCase())).map((each, index) => ({
      title: index === 0 && view.first ? view.first : each.name,
      detail: each.locale,
      checked: LANGUAGES.indexOf(each) === answers.language,
    }));
  const shown = rows();
  const reading = stack(0, "start", reads(answers));
  const keyboards = stack("s8", "start");
  const zone = select({
    options: ZONES.map(zoneName),
    picked: answers.zone,
    icon: "public",
    ...(view.open === "zone" ? { open: true } : {}),
    onPick: (index) => {
      answers.zone = index;
      answers.following = false;
      reading.replaceChildren(reads(answers));
    },
  });
  const drawKeyboards = () => {
    const parts = answers.layouts.map((layout, place) => {
      const picker = select({
        options: LAYOUTS.map((each) => each.name),
        picked: layout,
        icon: "keyboard",
        ...(view.open === "layout" && place === 0 ? { open: true } : {}),
        onPick: (index) => {
          answers.layouts[place] = index;
          answers.following = false;
        },
      });
      if (place === 0) return filling(picker);
      const drop = button({ label: "Drop", kind: "Text", size: "Sm", icon: "close" });
      drop.addEventListener("click", () => {
        answers.layouts.splice(place, 1);
        drawKeyboards();
      });
      return filling(row("s8", "centre", filling(picker), drop));
    });
    keyboards.replaceChildren(...parts);
  };
  drawKeyboards();
  const add = button({ label: "Add a layout", kind: "Text", size: "Sm", icon: "add", ...(answers.layouts.length >= 4 ? { disabled: true } : {}) });
  add.addEventListener("click", () => {
    if (answers.layouts.length >= 4) return;
    answers.layouts.push(LAYOUTS.findIndex((_, index) => !answers.layouts.includes(index)));
    drawKeyboards();
  });
  const language = shown.length
    ? scrollingList(shown, {
        height: "100%",
        onTake: (index) => {
          const picked = LANGUAGES.indexOf(LANGUAGES.filter((each) => !view.query || each.name.toLowerCase().includes(view.query.toLowerCase()))[index]!);
          answers.language = picked;
          if (answers.following) {
            answers.layouts = [LANGUAGES[picked]!.layout];
            answers.zone = LANGUAGES[picked]!.zone;
          }
          reading.replaceChildren(reads(answers));
        },
      })
    : emptyState({ icon: "search_off", title: "Nothing matches.", body: "Typo? Try its name in English." });
  const left = stack("s8", "start", filling(labelled("Language", field({ placeholder: "Search languages", icon: "search", ...(view.query ? { value: view.query } : {}) }))), fillingHeight(filling(language)));
  const right = stack(
    "s20",
    "start",
    filling(labelled("Keyboard", keyboards, { action: add, note: answers.following ? "Follows the language until you pick another." : "The first is the default; Super+Space switches." })),
    filling(field({ placeholder: "Type here to try it", icon: "keyboard" })),
    filling(labelled("Time zone", zone)),
    filling(card({ header: { title: "How it reads" }, body: [reading] })),
  );
  const columns = grid(2, "s20", [left, right]);
  columns.classList.add("kobo-columns");
  return body("言語と入力", "Language", "What you read and type with. The keyboard and the clock follow it until you change them.", columns);
}

// ----------------------------------------------------------------- Network --

export function networkBody(answers: Answers, machine: Machine): HTMLElement {
  const content = stack("s20", "start");
  const draw = () => {
    const networks = machine.networks;
    const rows: ListRowSpec[] = [{ title: "No Wi-Fi", subtitle: "I've got a cable, or I'll do it later", icon: "wifi_off", checked: answers.network === -1 }];
    if (networks !== "scanning") {
      networks.forEach((each, index) =>
        rows.push({
          title: each.name,
          subtitle: each.secured ? "Secured" : "Open",
          ...(each.secured ? { subtitleIcon: "lock" } : {}),
          icon: signalIcon(each.signal),
          checked: answers.network === index,
        }),
      );
    } else {
      for (let index = 0; index < 3; index++) rows.push({ title: "", loading: true, share: 0.5 + index * 0.1 });
    }
    const wells = list(rows, {
      onTake: (index) => {
        answers.network = index - 1;
        answers.networkPassword = "";
        draw();
        recheck(content);
      },
    });
    const parts: Element[] = [filling(wells)];
    if (networks === "scanning") parts.push(filling(row("s8", "centre", spinner("s16"), text("Looking for networks…", "small", "on_surface_variant"))));
    const picked = networks === "scanning" ? undefined : networks[answers.network];
    if (picked?.secured) {
      const password = typed({ secret: true, icon: "key", value: answers.networkPassword, ...(answers.networkPassword && [...answers.networkPassword].length < 8 ? { state: "Invalid" as const } : {}) }, (value) => {
        answers.networkPassword = value;
        setInvalid(password, value !== "" && [...value].length < 8);
        recheck(content);
      });
      parts.push(filling(labelled(`Password for ${picked.name}`, password, { action: showToggle(() => [password]), note: "Saved for your first boot, nowhere else." })));
    }
    content.replaceChildren(...parts);
  };
  draw();
  return body("接続", "Wi-Fi", "Optional. The install needs no internet; this saves a network for the first boot, which does.", content);
}

// --------------------------------------------------------------------- You --

/** How strong a password looks: 0 empty to 4 strong, by its length and its kinds of character. */
export function strength(password: string): number {
  if (password === "") return 0;
  const kinds = [/[a-z]/, /[A-Z]/, /[0-9]/, /[^a-zA-Z0-9]/].filter((each) => each.test(password)).length;
  const length = [...password].length;
  if (length >= 20 || (length >= 14 && kinds >= 2) || (length >= 12 && kinds >= 3)) return 4;
  if (length >= 10 && kinds >= 2) return 3;
  if (length >= 8 && kinds >= 2) return 2;
  return 1;
}

const STRENGTH = ["", "Weak", "Fair", "Good", "Strong"] as const;
const STRENGTH_TONE = ["outline_variant", "error", "warning", "success", "success"] as const;

/** The strength meter under a password: a quarter a step, in its tone, its word after it. */
function strengthLine(password: string): HTMLElement {
  const level = strength(password);
  const line = document.createElement("span");
  line.className = "kobo-strength";
  line.append(meter([{ share: level / 4, color: STRENGTH_TONE[level]! }, { share: 1 - level / 4, color: "surface_container_highest" }]), text(STRENGTH[level]!, "label", level === 0 ? "on_surface_variant" : STRENGTH_TONE[level]!));
  return line;
}

export function youBody(answers: Answers): HTMLElement {
  const content = stack("s20", "start");
  const login = text("", "mono", "on_surface_variant");
  const showLogin = () => login.replaceChildren(`${answers.username || "you"}@${answers.host || "this-machine"}`);
  showLogin();

  const username = typed({ placeholder: "you", value: answers.username, ...(answers.username && usernameProblem(answers.username) ? { state: "Invalid" as const } : {}) }, (value) => {
    answers.username = value;
    answers.ownUsername = value !== "";
    setInvalid(username, value !== "" && usernameProblem(value) !== undefined);
    showLogin();
    recheck(content);
  });
  const host = typed({ placeholder: "this-machine", value: answers.host, ...(answers.host && hostProblem(answers.host) ? { state: "Invalid" as const } : {}) }, (value) => {
    answers.host = value;
    setInvalid(host, value !== "" && hostProblem(value) !== undefined);
    showLogin();
    recheck(content);
  });
  const name = typed({ placeholder: "Your name", value: answers.name }, (value) => {
    answers.name = value;
    if (!answers.ownUsername) {
      answers.username = usernameOf(value);
      username.querySelector(".k-field-value")?.replaceChildren(answers.username);
    }
    showLogin();
    recheck(content);
  });
  const userProblem = answers.username ? usernameProblem(answers.username) : undefined;
  const hostWrong = answers.host ? hostProblem(answers.host) : undefined;
  const names = grid(2, "s20", [
    labelled("Username", username, userProblem ? { note: userProblem, invalid: true } : { note: "Your login and your home folder." }),
    labelled("Machine name", host, hostWrong ? { note: hostWrong, invalid: true } : { note: "On the network and in your prompt." }),
  ]);
  const more = hangs(collapsible({ title: "Username and machine name", summary: `${answers.username || "you"}@${answers.host || "this-machine"}`, content: names, open: answers.namesOpen || Boolean(userProblem || hostWrong) }));

  const strong = stack(0, "start", strengthLine(answers.password));
  const password = typed({ secret: true, icon: "key", value: answers.password }, (value) => {
    answers.password = value;
    strong.replaceChildren(strengthLine(value));
    recheck(content);
  });
  const mismatch = answers.confirm !== "" && !stillTyping(answers.password, answers.confirm) && answers.password !== answers.confirm;
  const confirm = typed({ secret: true, icon: "key", value: answers.confirm, ...(mismatch ? { state: "Invalid" as const } : {}) }, (value) => {
    answers.confirm = value;
    setInvalid(confirm, value !== "" && !stillTyping(answers.password, value) && answers.password !== value);
    recheck(content);
  });
  const passwords = grid(2, "s20", [
    labelled("Password", stack("s8", "start", filling(password), filling(strong)), { note: "You type it at boot too: it unlocks the disk." }),
    labelled("Type it again", confirm, { action: showToggle(() => [password, confirm]), ...(mismatch ? { note: "The passwords don't match.", invalid: true } : {}) }),
  ]);
  content.append(filling(labelled("Your name", name)), filling(more), filling(passwords));
  return body("アカウント", "You", "Your name and a password you'll still remember at 2am. The rest follows from them.", content);
}

// -------------------------------------------------------------------- Disk --

/** A bar of the disk's parts: kept ones in their colour, the free space outlined, KubaOS lit. */
interface Piece {
  readonly name: string;
  readonly bytes: number;
  readonly kind: "system" | "plumbing" | "data" | "free" | "kubaos";
  /** On the now bar: it doesn't survive the install. */
  readonly gone?: boolean;
  /** On the after bar: it is gone, its room given to KubaOS. */
  readonly hidden?: boolean;
}

/** A disk's thousandths, at least one, so a sliver of a partition still has a place. */
function share(disk: Disk, count: number): number {
  return Math.max(1, Math.round((count / disk.bytes) * 1000));
}

function bar(disk: Disk, pieces: readonly Piece[]): HTMLElement {
  const line = document.createElement("span");
  line.className = "kobo-bar";
  for (const each of pieces) {
    const box = document.createElement("span");
    box.className = "kobo-part-box";
    box.dataset.piece = each.kind === "kubaos" ? "kubaos" : each.name;
    const thousandths = share(disk, each.bytes);
    // A partition under a hundredth of the disk holds no words: it is a sliver of its own width.
    if (thousandths < 10) box.dataset.sliver = "";
    box.style.setProperty("--kobo-share", String(each.hidden ? 0 : thousandths / 1000));
    box.style.setProperty("--kobo-shown", each.hidden ? "0" : "1");
    const filled = document.createElement("span");
    filled.className = "kobo-part-fill";
    filled.dataset.kind = each.kind;
    if (each.gone) filled.dataset.gone = "";
    if (thousandths >= 10) filled.append(text(each.kind === "free" ? `${bytes(each.bytes)} free` : `${each.name} · ${bytes(each.bytes)}`, "label", "currentColor"));
    box.append(filled);
    line.append(box);
  }
  return line;
}

/** What is on `disk` now: its partitions in order, its free space after the system's. */
function nowPieces(disk: Disk): Piece[] {
  const pieces: Piece[] = disk.partitions.map((each) => ({ name: each.name, bytes: each.bytes, kind: each.kind }));
  if (disk.free > 0) pieces.splice(Math.min(3, pieces.length), 0, { name: "free", bytes: disk.free, kind: "free" });
  return pieces;
}

/**
 * What is on it after the install: alongside, KubaOS in the free space; erased, every part's room
 * given to KubaOS. Both list the same parts, so the bar moves from one to the other.
 */
function afterPieces(disk: Disk, mode: Answers["mode"], encrypt: boolean): Piece[] {
  const ours = encrypt ? "KubaOS, encrypted" : "KubaOS";
  const now = nowPieces(disk);
  if (!now.some((each) => each.kind === "free")) now.push({ name: "free", bytes: 0, kind: "free" });
  return now.map((each): Piece => {
    if (each.kind === "free") return { name: ours, bytes: mode === "erase" ? disk.bytes : each.bytes, kind: "kubaos", ...(mode === "alongside" && each.bytes === 0 ? { hidden: true } : {}) };
    return mode === "erase" ? { ...each, hidden: true } : each;
  });
}

/** The map: what's on the disk now over what's on it after, in one grid, labels in a column. */
export function diskMap(disk: Disk, mode: Answers["mode"], encrypt: boolean): HTMLElement {
  const map = document.createElement("span");
  map.className = "kobo-map";
  const now = nowPieces(disk).map((each) => (mode === "erase" && each.kind !== "free" ? { ...each, gone: true } : each));
  map.append(text("Now", "label", "on_surface_variant"), bar(disk, now), text("After", "label", "on_surface_variant"), bar(disk, afterPieces(disk, mode, encrypt)));
  return map;
}

/** The plumbing Windows keeps, said once under the map for whoever wonders about the slivers. */
function plumbing(disk: Disk, mode: Answers["mode"]): string {
  const small = disk.partitions.filter((each) => each.kind === "plumbing").map((each) => `${each.name}, ${bytes(each.bytes)}`);
  if (small.length === 0) return mode === "erase" ? "Nothing on it is kept." : "";
  return mode === "erase" ? `Goes too: ${small.join("; ")}.` : `Kept as they are: ${small.join("; ")}.`;
}

/** A setting in a card: its tile, its name over what it does, its switch at the end. */
function setting(glyph: string, title: string, subtitle: string, on: boolean, flip: (on: boolean) => void): HTMLElement {
  const toggle = switchControl({ on });
  toggle.addEventListener("click", () => flip(toggle.getAttribute("aria-checked") === "true"));
  const line = document.createElement("span");
  line.className = "kobo-setting";
  line.append(iconTile({ glyph, size: "Sm" }), stack("s4", "start", text(title, "body_strong"), wrapped(subtitle, "small", "on_surface_variant")), toggle);
  return line;
}

/** The disk screen's options: drawn with the encryption's own passphrase open. */
export interface DiskView {
  readonly encryptionOpen?: boolean;
}

export function diskBody(answers: Answers, machine: Machine, view: DiskView = {}): HTMLElement {
  const content = stack("s24", "start");
  const draw = () => {
    const targets = machine.disks.filter((each) => !each.stick);
    if (targets.length === 0) {
      content.replaceChildren(
        filling(
          emptyState({
            icon: "hard_drive",
            title: "No disks. None at all.",
            body: "Kōbō found nowhere to install. Check the disk is in and the BIOS shows it, then restart.",
          }),
        ),
      );
      return;
    }
    const rows: ListRowSpec[] = machine.disks.map((each) => ({
      title: each.model,
      subtitle: `${bytes(each.bytes)} · ${each.path}`,
      icon: "hard_drive",
      tall: true,
      ...(each.stick ? { disabled: true, chips: [{ kind: "Neutral" as const, label: "This installer" }] } : {}),
      ...(!each.stick && each.other ? { chips: [{ kind: "Neutral" as const, label: each.other }] } : {}),
      ...(!each.stick && each.bytes < NEEDED ? { chips: [{ kind: "Warning" as const, label: "Too small" }] } : {}),
      checked: machine.disks.indexOf(each) === answers.disk,
    }));
    const disks = list(rows, {
      onTake: (index) => {
        const disk = machine.disks[index];
        if (!disk || disk.stick) return;
        answers.disk = index;
        answers.mode = disk.other && fitsAlongside(disk) ? "alongside" : "erase";
        draw();
        recheck(content);
      },
    });
    const parts: Element[] = [filling(labelled("Where it goes", disks))];
    const disk = machine.disks[answers.disk];
    if (disk && !disk.stick && disk.bytes >= NEEDED) {
      const alongsideOk = fitsAlongside(disk) && disk.partitions.length > 0;
      const other = disk.other ?? "what's on it";
      const alongside = choiceCard({
        glyph: "storage",
        title: disk.other ? `Next to ${disk.other}` : "In the free space",
        subtitle: alongsideOk ? `${other} stays. KubaOS takes the ${bytes(disk.free)} that's free.` : disk.partitions.length === 0 ? "The disk is empty: nothing to keep." : `Only ${bytes(disk.free)} free. Shrink ${other} first.`,
        picked: answers.mode === "alongside",
        ...(alongsideOk ? {} : { disabled: true }),
      });
      const erase = choiceCard({
        glyph: "warning",
        title: "Erase the disk",
        subtitle: disk.partitions.length === 0 ? `KubaOS takes all ${bytes(disk.bytes)}.` : `${disk.other ?? "Everything on it"} and everything else on it goes.`,
        picked: answers.mode === "erase",
      });
      const pick = (mode: Answers["mode"]) => {
        if (answers.mode === mode) return;
        answers.mode = mode;
        setPicked(alongside, mode === "alongside");
        setPicked(erase, mode === "erase");
        morph(map, disk, mode, answers.encrypt);
        under.replaceChildren(plumbing(disk, mode));
        recheck(content);
      };
      if (alongsideOk) alongside.addEventListener("click", () => pick("alongside"));
      erase.addEventListener("click", () => pick("erase"));
      const map = diskMap(disk, answers.mode, answers.encrypt);
      const under = wrapped(plumbing(disk, answers.mode), "small", "on_surface_variant");
      parts.push(filling(stack("s12", "start", filling(grid(2, "s16", [alongside, erase])), filling(map), under)));

      const encryption = stack("s16", "start");
      const drawEncryption = () => {
        const lines: Element[] = [
          filling(
            setting("lock", "Encrypt the disk", answers.encrypt ? "Lose the laptop, keep your secrets. Your password opens it at boot." : "Anyone holding the disk can read it. Your call.", answers.encrypt, (on) => {
              answers.encrypt = on;
              drawEncryption();
              morph(map, disk, answers.mode, on);
              recheck(content);
            }),
          ),
        ];
        if (answers.encrypt) {
          const passphrase = typed({ secret: true, icon: "key", value: answers.passphrase }, (value) => {
            answers.passphrase = value;
            recheck(content);
          });
          const again = typed({ secret: true, icon: "key", value: answers.passphraseConfirm }, (value) => {
            answers.passphraseConfirm = value;
            recheck(content);
          });
          const own = stack(
            "s16",
            "start",
            filling(
              setting("key", "Use my password", "One password for boot and login.", answers.samePassword, (on) => {
                answers.samePassword = on;
                drawEncryption();
                recheck(content);
              }),
            ),
            ...(answers.samePassword ? [] : [filling(grid(2, "s16", [labelled("Passphrase", passphrase), labelled("Type it again", again, { action: showToggle(() => [passphrase, again]) })]))]),
          );
          lines.push(filling(hangs(collapsible({ title: "Its own passphrase", summary: answers.samePassword ? "Your password" : "Its own", content: own, open: view.encryptionOpen ?? !answers.samePassword }))));
        }
        encryption.replaceChildren(...lines);
      };
      drawEncryption();
      parts.push(filling(card({ body: [encryption] })));
    }
    content.replaceChildren(...parts);
  };
  draw();
  return body("ディスク", "Disk", "Where it goes. Nothing is written until you hold the button on the last screen.", content);
}

/**
 * The map moving to `mode`: every part of the after bar eases to its new share and showing over
 * `medium` on `standard_decel`, so KubaOS grows over what erasing takes; the now bar's parts
 * change their look at once. Reduced motion changes it at once.
 */
function morph(map: HTMLElement, disk: Disk, mode: Answers["mode"], encrypt: boolean): void {
  const fresh = diskMap(disk, mode, encrypt);
  const was = [...map.querySelectorAll<HTMLElement>(".kobo-part-box")];
  const next = [...fresh.querySelectorAll<HTMLElement>(".kobo-part-box")];
  const moving = !settings().reduced;
  was.forEach((box, index) => {
    const into = next[index];
    if (!into) return;
    const read = (from: HTMLElement) => ({ "--kobo-share": from.style.getPropertyValue("--kobo-share"), "--kobo-shown": from.style.getPropertyValue("--kobo-shown") });
    const from = read(box);
    const after = read(into);
    box.style.setProperty("--kobo-share", after["--kobo-share"]);
    box.style.setProperty("--kobo-shown", after["--kobo-shown"]);
    if (moving) box.animate([from, after], { duration: milliseconds("medium") / settings().speed, easing: `cubic-bezier(${bezier("standard_decel").join(", ")})` });
    const was = box.firstElementChild;
    const fill = into.firstElementChild as HTMLElement;
    const gone = (each: Element | null) => (each?.hasAttribute("data-gone") ? 1 : 0);
    const before = gone(was);
    was?.replaceWith(fill);
    if (moving && before !== gone(fill)) fill.animate([{ "--kobo-gone": before }, { "--kobo-gone": gone(fill) }], { duration: milliseconds("short") / settings().speed, easing: `cubic-bezier(${bezier("standard").join(", ")})` });
  });
}

// -------------------------------------------------------------- Make it yours --

export function yoursBody(answers: Answers, view: { readonly gitOpen?: boolean; readonly first?: string } = {}): HTMLElement {
  const content = stack("s32", "start");
  const draw = () => {
    const sets = SETS.map((set) => {
      const keys = set.wallpapers.map((each) => wallpaperKey(set.name, each.name));
      const on = keys.filter((key) => answers.wallpapers.has(key)).length;
      const tick = on === 0 ? "Empty" : on === keys.length ? "Full" : "Partial";
      const head = checkbox({ label: set === SETS[0] && view.first ? view.first : set.name, tick });
      head.addEventListener("click", () => {
        for (const key of keys) {
          if (tick === "Full") answers.wallpapers.delete(key);
          else answers.wallpapers.add(key);
        }
        draw();
        recheck(content);
      });
      const count = keys.length === 1 && tick !== "Partial" ? "1 wallpaper" : tick === "Full" ? `all ${keys.length}` : tick === "Empty" ? `${keys.length} wallpapers` : `${on} of ${keys.length}`;
      const pictures = set.wallpapers.map((each) => {
        const key = wallpaperKey(set.name, each.name);
        const picked = answers.wallpapers.has(key);
        const tile = pictureChoice({ source: each.picture, name: each.name, place: "over", picked, mark: { icon: "check", label: "Picked" }, many: true, ...(each.video ? { badge: { icon: "play_arrow", label: "Video" } } : {}) });
        tile.addEventListener("click", () => {
          if (answers.wallpapers.has(key)) answers.wallpapers.delete(key);
          else answers.wallpapers.add(key);
          draw();
          recheck(content);
        });
        return tile;
      });
      return stack("s12", "start", row("s12", "centre", head, chip({ label: count, kind: tick === "Full" ? "Selected" : "Neutral" })), filling(grid(3, "s16", pictures)));
    });
    const name = typed({ placeholder: "Your name", value: answers.gitName }, (value) => {
      answers.gitName = value;
      preview.replaceChildren(gitconfig(answers));
      recheck(content);
    });
    const email = typed({ placeholder: "you@example.com", icon: "mail", value: answers.gitEmail }, (value) => {
      answers.gitEmail = value;
      preview.replaceChildren(gitconfig(answers));
      recheck(content);
    });
    const preview = stack(0, "start", gitconfig(answers));
    const git = hangs(collapsible({
      title: "Git identity",
      summary: answers.gitEmail ? answers.gitEmail : "Later",
      open: view.gitOpen ?? answers.gitOpen,
      content: stack("s16", "start", filling(grid(2, "s20", [labelled("Name", name), labelled("Email", email, answers.gitEmail && !looksLikeEmail(answers.gitEmail) ? { note: "That isn't an email address.", invalid: true } : { note: "No email: you set it up later." })])), filling(card({ body: [preview] }))),
    }));
    content.replaceChildren(...sets.map(filling), filling(git));
  };
  draw();
  return body("壁紙", "Make it yours", "The wallpapers the picker shows; a set's name takes all of it. Change any of it later.", content);
}

/** `~/.gitconfig` as the install writes it. */
function gitconfig(answers: Answers): HTMLElement {
  const entry = (key: string, value: string) => row(0, "centre", text(key, "mono_large", "cold"), text(` = ${value || "…"}`, "mono_large"));
  return stack("s4", "start", text("~/.gitconfig", "mono", "on_surface_variant"), gap("s4"), text("[user]", "mono_large", "primary"), entry("    name", answers.gitName.trim()), entry("    email", answers.gitEmail.trim()));
}

// --------------------------------------------------------------- Last look --

/** The hold to install's options: held this far, as a still sample of the hold. */
export interface ReviewView {
  readonly held?: number;
  readonly lowBattery?: number;
}

/** A card of the review: its symbol and title, a Change that goes back to its screen, the facts. */
function block(glyph: string, title: string, facts: Parameters<typeof descriptionList>[0], change: () => void): HTMLElement {
  const back = button({ label: "Change", kind: "Text", size: "Sm" });
  back.classList.add("kobo-change");
  back.addEventListener("click", change);
  return card({ header: { glyph, title, end: back }, body: [descriptionList(facts)] });
}

/** What the hold does, as the banner's words. */
export function holdWords(answers: Answers, machine: Machine): { readonly danger: boolean; readonly title: string; readonly body: string; readonly cta: string } {
  const disk = machine.disks[answers.disk]!;
  const named = `${disk.model} (${disk.path.replace("/dev/", "")}, ${bytes(disk.bytes)})`;
  if (answers.mode === "erase") {
    const count = disk.partitions.length;
    const parts = count === 0 ? "There's no undo." : `All ${count === 1 ? "1 partition" : `${count} partitions`}${disk.other ? `, ${disk.other} included` : ""}. There's no undo.`;
    return { danger: true, title: `Everything on ${named} gets erased.`, body: parts, cta: "Hold to erase and install" };
  }
  return { danger: false, title: `KubaOS goes into the free space on ${named}.`, body: `${disk.other ?? "Everything else"} stays where it is.`, cta: "Hold to install" };
}

export function reviewBody(answers: Answers, machine: Machine, go: (page: string) => void, held: (button: HTMLElement, foot: HTMLElement) => void, view: ReviewView = {}): HTMLElement {
  const language = LANGUAGES[answers.language]!;
  const zone = ZONES[answers.zone]!;
  const disk = machine.disks[answers.disk];
  const networks = machine.networks === "scanning" ? [] : machine.networks;
  const cards = grid(2, "s16", [
    block("language", "Language", [
      { label: "Language", value: language.name, detail: language.locale },
      { label: "Keyboard", value: answers.layouts.map((each) => LAYOUTS[each]!.name).join(", ") },
      { label: "Time zone", value: zoneName(zone) },
    ], () => go("language")),
    block("person", "You", [
      { label: "Name", value: answers.name || "…" },
      { label: "Login", value: `${answers.username}@${answers.host}` },
      { label: "Git", value: answers.gitEmail ? answers.gitName : "Later", ...(answers.gitEmail ? { detail: answers.gitEmail } : {}) },
    ], () => go("you")),
    block("hard_drive", "Disk", [
      { label: "Disk", value: disk?.model ?? "None picked", ...(disk ? { detail: `${disk.path} · ${bytes(disk.bytes)}` } : {}) },
      { label: "How", value: answers.mode === "erase" ? "Erase everything" : `Next to ${disk?.other ?? "what's there"}, in ${bytes(disk?.free ?? 0)}` },
      { label: "Encryption", value: !answers.encrypt ? "Off. Brave." : answers.samePassword ? "LUKS2, your password" : "LUKS2, its own passphrase" },
    ], () => go("disk")),
    block("deployed_code", "System", [
      { label: "Release", value: `KubaOS ${RELEASE.version} ${RELEASE.codename}`, detail: `${RELEASE.commit} · Linux ${RELEASE.kernel}` },
      { label: "Wallpapers", value: wallpaperNote(answers) ?? "None" },
      { label: "Wi-Fi", value: networks[answers.network]?.name ?? "None" },
    ], () => go("yours")),
  ]);
  const words = holdWords(answers, machine);
  const hold = button({ label: view.held ? "Keep holding…" : words.cta, kind: words.danger ? "Danger" : "Filled", size: "Lg", icon: "download", alternatives: [words.cta, "Keep holding…"] });
  hold.classList.add("kobo-hold-button");
  const back = button({ label: "Back", kind: "Text", icon: "arrow_back" });
  back.addEventListener("click", () => go("back"));
  const foot = progressBar({ fraction: view.held ?? 0, timed: true, ...(words.danger ? { danger: true } : {}) });
  held(hold, foot);
  const actions = hangs(row("s16", "centre", back, fill(), row("s4", "centre", text("or hold", "caption", "on_surface_variant"), keycap("Enter")), hold));
  const confirm = banner({ tone: words.danger ? "Danger" : "Info", icon: words.danger ? "warning" : "storage", title: words.title, body: words.body, actions, foot });
  const top: Element[] = [];
  if (view.lowBattery !== undefined) top.push(filling(banner({ tone: "Warning", icon: "battery_1_bar", title: `${view.lowBattery}% left. Plug the damn thing in.`, body: "" })));
  const { box } = scroll(scrolled(cards), { height: "100%", above: "s20", below: "s20", noGutter: true, fade: "s20", footFade: true });
  box.classList.add("kobo-scrolls");
  return stack(
    "s20",
    "start",
    filling(header({ accent: "確認", title: "Last look", subtitle: "Nothing's been written yet. Read it once, then hold the button." })),
    ...top,
    fillingHeight(filling(box)),
    filling(confirm),
  );
}

// ----------------------------------------------------------- The install --

/** The steps of the install as a well's `Step`s at `share` of the way, in two columns. */
export function stepList(share: number, failedAt?: number): HTMLElement {
  const at = failedAt ?? stepAt(share).index;
  const rows = STEPS.map((each, index) => {
    const state = index < at ? ({ is: "done" } as const) : index === at ? ({ is: "current" } as const) : ({ is: "todo" } as const);
    const before = STEPS.slice(0, index).reduce((sum, s) => sum + s.seconds, 0);
    const timing = index < at ? duration(each.seconds) : index === at ? duration(failedAt === undefined ? share * TOTAL - before : FAILURE.after) : "";
    return step({ title: each.title, ...(timing ? { subtitle: timing } : {}), state, ...(index === at && failedAt === undefined ? { running: true } : {}), ...(index === at && failedAt !== undefined ? { failed: true } : {}) });
  });
  const half = Math.ceil(rows.length / 2);
  return grid(2, "s16", [list(rows.slice(0, half)), list(rows.slice(half))]);
}

/** The log up to the step running, or ending in the failure. */
export function installLog(share: number, failed: boolean, height = "var(--size-log)"): HTMLElement {
  const at = failed ? FAILURE.step : stepAt(share).index;
  const lines = LOG.filter((each) => each.step <= at).map((each) => ({ level: each.level, text: each.text }));
  if (failed) lines.push(...FAILURE.log);
  return log(lines, { height }).box;
}

/** The install's details in the panel: the figure, the bar, the steps, the log, a way back. */
export function detailsBody(share: number, back: () => void): HTMLElement {
  const { index } = stepAt(share);
  const percent = new Intl.NumberFormat("en-GB", { style: "percent" }).format(share);
  const scene = button({ label: "Back to the view", kind: "Text", icon: "arrow_back" });
  scene.addEventListener("click", back);
  const top = row("s16", "end", filling(header({ accent: "インストール中", title: "Installing", subtitle: "Every step, its time, and the log as it's written." })), text(percent, "hero_number"));
  return stack(
    "s20",
    "start",
    filling(top),
    filling(stack("s12", "start", filling(progressBar({ fraction: share, timed: true })), filling(row("s12", "centre", text(STEPS[index]!.title, "body_strong"), text(STEPS[index]!.detail ?? "", "mono", "primary"), fill(), icon("timer", "var(--icon-s16)", "var(--color-on_surface_variant)"), text(duration(share * TOTAL), "mono", "on_surface_variant"))))),
    filling(stepList(share)),
    fillingHeight(filling(installLog(share, false, "100%"))),
    filling(hangs(row("s16", "centre", scene))),
  );
}

/** The install failed: what broke, where the disk stands, the log, and the ways on. */
export function failedBody(answers: Answers, machine: Machine, actions: { readonly retry: () => void; readonly review: () => void }, logOpen = true): HTMLElement {
  const disk = machine.disks[answers.disk];
  const step = STEPS[FAILURE.step]!;
  const where =
    answers.mode === "alongside"
      ? `“${step.title}” stopped after ${duration(FAILURE.after)}. ${disk?.other ?? "What was there"} is untouched; the free space holds a half-copied system.`
      : `“${step.title}” stopped after ${duration(FAILURE.after)}. ${disk?.model ?? "The disk"} won't boot until KubaOS is installed on it again.`;
  const retry = button({ label: "Try again", icon: "restart_alt" });
  retry.addEventListener("click", actions.retry);
  const review = button({ label: "Back to the review", kind: "Text", icon: "arrow_back" });
  review.addEventListener("click", actions.review);
  const save = button({ label: "Save log", kind: "Tonal", icon: "save" });
  const shown = button({ label: logOpen ? "Hide log" : "Show log", kind: "Text", icon: "terminal", alternatives: ["Show log", "Hide log"] });
  const room = stack(0, "start", installLog(0, true, "100%"));
  shown.addEventListener("click", () => {
    const open = room.toggleAttribute("hidden");
    shown.querySelector(".k-button-label")?.replaceChildren(open ? "Show log" : "Hide log");
  });
  if (!logOpen) room.hidden = true;
  return stack(
    "s20",
    "start",
    filling(header({ accent: "失敗", title: "Fuck.", subtitle: where })),
    filling(errorBlock({ message: FAILURE.error, hint: FAILURE.hint })),
    fillingHeight(filling(room)),
    filling(hangs(row("s12", "centre", review, shown, fill(), save, retry))),
  );
}
