// The Settings app's own parts, as SidebarPage draws them: a setting as data (its name, a plain line
// on what it does, its control, its default, whether it has a backend yet), and the region that draws
// a section's sub-sections from them. The template's rows are `ListRow`s, which end in a switch, a
// fact or a chevron; a setting that needs a segmented choice, a slider, a select or a field gets it at
// the row's end, in one column `size.menu` wide, so every control of a page lines up.
//
// Every setting can go back to its default with a hold. Beside its control while it differs from it,
// a held `IconButton` (Text, sm, `restart_alt`) whose ring fills round it; its place is kept while it
// is gone, so the control never moves. At the end of its sub-section's heading while anything in the
// sub-section differs, and of the page's title while anything in the section does, a held `Button`
// (Text, sm, `restart_alt` and "Reset") whose body fills from its start. Each fades out once there
// is nothing left to put back.
//
// Its organisms, kept in the app because only Settings ends a row in a column of controls with a
// reset before each:
// - `settingRow`: a `ListRow` ending in a control (Tabs, Slider, Select or Field) in a fixed column,
//   with its reset's place before the control;
// - the dependent row, which appears under the row it depends on (`shows`);
// - the risky value's `Banner` under its well (`warns`);
// - the sub-page a row opens, with its way back (`SubPage`).

import "./settings.css";
import { banner, tip } from "../../../lib/banner.ts";
import { button } from "../../../lib/button.ts";
import { field, setInvalid } from "../../../lib/field.ts";
import { withHint } from "../../../lib/hint.ts";
import { iconButton } from "../../../lib/icon-button.ts";
import { filling, row as layoutRow, section, stack } from "../../../lib/layout.ts";
import { probe } from "../../../lib/probe.ts";
import { type ListRowSpec, ROW_CONTROL } from "../../../lib/list.ts";
import { type RegionHost, settingsWell, type Section, type SubSection } from "../../../lib/sidebar-page.ts";
import { select } from "../../../lib/select.ts";
import { setSlider, slider } from "../../../lib/slider.ts";
import { tabs } from "../../../lib/tabs.ts";
import { durationMs, length } from "../../../lib/tokens.ts";
import { settings } from "../../../site/settings.ts";

/** What a setting is set with. */
export type Control =
  | { readonly kind: "switch"; readonly on: () => boolean; readonly set: (on: boolean) => void }
  | { readonly kind: "choice"; readonly choices: readonly string[]; readonly picked: () => number; readonly set: (index: number) => void }
  | { readonly kind: "slider"; readonly icon: string; readonly names: readonly string[]; readonly icons?: readonly string[]; readonly index: () => number; readonly set: (index: number) => void }
  | { readonly kind: "select"; readonly options: readonly string[]; readonly picked: () => number; readonly set: (index: number) => void }
  | {
      readonly kind: "field";
      readonly value: () => string;
      readonly placeholder: string;
      /** What is wrong with `value`, said plainly, or null. */
      readonly check: (value: string) => string | null;
      readonly set: (value: string) => void;
    }
  | { readonly kind: "opens"; readonly to: string; readonly detail?: () => string }
  /** A library part that sets it on its own: a colour picker's swatches, its hue track. */
  | { readonly kind: "part"; readonly make: () => HTMLElement }
  | { readonly kind: "fact"; readonly detail: () => string };

/** A risky value's warning, shown under the well while the value holds. */
export interface Warning {
  readonly title: string;
  readonly body: string;
}

/** A setting's value as its control holds it. */
type Value = number | boolean | string;

/** One setting: a row of its sub-section. */
export interface Setting {
  readonly id: string;
  readonly title: string;
  /** One plain line on what it does, for someone who doesn't know the term. */
  readonly line: string;
  readonly control: Control;
  /** Its default, for the value shown: a reset puts it back. None: a view, not a setting. */
  readonly initial?: () => Value;
  /**
   * The values it holds that its row doesn't show (the other power source's, the other stops'):
   * whether they are all at their defaults, and putting them back. The row's reset leaves them; its
   * sub-section's and its section's take them too.
   */
  readonly unseen?: { readonly at: () => boolean; readonly reset: () => void };
  /** No backend writes it today: the page lists it as not built yet. */
  readonly unbuilt?: true;
  /** Why it can't change here; the row is dimmed and says so in place of its line. */
  readonly disabled?: string;
  /** A dependent row: shown, under the row it depends on, only while this holds. */
  readonly shows?: () => boolean;
  /** Its value changes what else shows: the region is drawn again when it changes. */
  readonly drives?: true;
  /** A risky value's warning, under the setting's well. */
  readonly warns?: () => Warning | null;
  /** A faster or hidden way to do what the setting is about, so it is no secret: one sentence, keys as `[Key]`, under the row's line. */
  readonly tip?: string;
}

/** A sub-section: a heading, a line, and its wells of settings. */
export interface SettingsSub {
  readonly title: string;
  readonly body?: string;
  readonly wells: readonly (readonly Setting[])[];
  /**
   * A part more than rows, before its wells: the screens' arrangement, the networks in range; none
   * while it has nothing to show. It is handed its sub-section, so it can draw the sub-section's
   * `partTip` inside itself.
   */
  readonly before?: (sub: SettingsSub) => HTMLElement | null;
  /** A tip about the whole sub-section, a sentence as a setting's is: under its heading, over its wells. */
  readonly tip?: string;
  /**
   * A tip about the part before its wells, drawn inside that part as a row's tip is drawn under its
   * row (About's update card): Home's search finds the sub-section by it as by a heading's tip.
   */
  readonly partTip?: string;
}

/** A page a row opens, inside its section: its heading, its line and its wells. */
export interface SubPage {
  readonly id: string;
  readonly title: string;
  readonly body: string;
  /** Where Back goes: the sub-section the row that opened it is in. */
  readonly back: string;
  readonly wells: readonly (readonly Setting[])[];
  /** What comes before its wells: a choice that picks what they set. */
  readonly before?: () => HTMLElement;
  /** What comes after its wells: a record the settings don't change, such as a history. */
  readonly after?: () => HTMLElement;
}

/** A section of Settings: its sidebar row, its header and its sub-sections. */
export interface SettingsSection {
  readonly icon: string;
  readonly title: string;
  readonly accent: string;
  /** The pun under the title. */
  readonly lede: string;
  readonly subs: readonly SettingsSub[];
  readonly pages?: readonly SubPage[];
  /** The sub-page drawn open as the window opens: a sample's. */
  readonly opened?: string;
}

/** Whether `setting` shows now: a dependent row only while what it depends on holds. */
function showing(setting: Setting): boolean {
  return setting.shows ? setting.shows() : true;
}

/** The settings of a well that show now. */
function shownIn(well: readonly Setting[]): Setting[] {
  return well.filter(showing);
}

/** The value Home's search shows at a result's end, as words. */
function valueOf(control: Control): string | undefined {
  switch (control.kind) {
    case "choice":
      return control.choices[control.picked()];
    case "select":
      return control.options[control.picked()];
    case "slider":
      return control.names[control.index()];
    case "field":
      return control.value() || undefined;
    case "opens":
      return control.detail?.();
    case "fact":
      return control.detail();
    case "switch":
    case "part":
      return undefined;
  }
}

/** The value `control` holds now, as its default is written. */
function current(control: Control): Value | undefined {
  switch (control.kind) {
    case "switch":
      return control.on();
    case "choice":
    case "select":
      return control.picked();
    case "slider":
      return control.index();
    case "field":
      return control.value();
    default:
      return undefined;
  }
}

/** Puts `control` to `value`, as a person picking it would. */
function put(control: Control, value: Value): void {
  if (control.kind === "switch") control.set(value as boolean);
  else if (control.kind === "choice" || control.kind === "select" || control.kind === "slider") control.set(value as number);
  else if (control.kind === "field") control.set(value as string);
}

/**
 * How a section's settings go back to their defaults: a setting's own value, all it holds, and those
 * of the sub-page a row opens.
 */
export class Defaults {
  private readonly section: SettingsSection;

  constructor(section: SettingsSection) {
    this.section = section;
  }

  /** The sub-page a row opens, if it is one. */
  private pageOf(setting: Setting): SubPage | undefined {
    const control = setting.control;
    return control.kind === "opens" ? this.section.pages?.find((page) => page.id === control.to) : undefined;
  }

  /** Whether the row can be reset at all: it has a default, or opens a page of settings that do. */
  resettable(setting: Setting): boolean {
    if (setting.disabled) return false;
    const page = this.pageOf(setting);
    if (page) return page.wells.flat().some((each) => this.resettable(each));
    return setting.initial !== undefined;
  }

  /** Whether the value its row shows is its default; a row opening a page, whether all of the page's are. */
  shownAt(setting: Setting): boolean {
    const page = this.pageOf(setting);
    if (page) return this.allAt(page.wells.flat());
    return setting.initial === undefined || current(setting.control) === setting.initial();
  }

  /** Whether everything `settings` hold, shown or not, is at its default. */
  allAt(settings: readonly Setting[]): boolean {
    return settings.every((each) => each.disabled !== undefined || (this.shownAt(each) && (each.unseen?.at() ?? true)));
  }

  /** Puts back the value `setting`'s row shows; a row opening a page, every value of the page. */
  resetShown(setting: Setting): void {
    const page = this.pageOf(setting);
    if (page) this.resetAll(page.wells.flat());
    else if (setting.initial && !setting.disabled) put(setting.control, setting.initial());
  }

  /** Puts back everything `settings` hold, shown or not. */
  resetAll(settings: readonly Setting[]): void {
    for (const each of settings) {
      if (each.disabled) continue;
      this.resetShown(each);
      each.unseen?.reset();
    }
  }

  /** Every setting of sub-section `sub`, dependents included. */
  ofSub(sub: SettingsSub): Setting[] {
    return sub.wells.flat();
  }

  /** Every setting of the section, its sub-pages' through the rows that open them. */
  ofSection(): Setting[] {
    return this.section.subs.flatMap((sub) => this.ofSub(sub));
  }

  /** What a row's reset goes back to, said in its hint. */
  words(setting: Setting): string {
    const control = setting.control;
    const initial = setting.initial?.();
    switch (control.kind) {
      case "switch":
        return `Hold to reset to ${initial ? "on" : "off"}`;
      case "choice":
        return `Hold to reset to ${control.choices[initial as number]}`;
      case "select":
        return `Hold to reset to ${control.options[initial as number]}`;
      case "slider":
        return `Hold to reset to ${control.names[initial as number]}`;
      case "field":
        return initial === "" ? "Hold to clear it" : `Hold to reset to ${initial as string}`;
      default:
        return `Hold to reset ${setting.title}`;
    }
  }
}

/** The row the template draws for `setting`: its words, and its switch, fact or chevron. */
export function specOf(setting: Setting): ListRowSpec {
  const base = { title: setting.title, subtitle: setting.disabled ?? setting.line, ...(setting.disabled ? { disabled: true } : {}), ...(setting.tip ? { tip: setting.tip } : {}) };
  const control = setting.control;
  const value = valueOf(control);
  if (control.kind === "switch") return { ...base, toggle: control.on() };
  if (control.kind === "opens") return { ...base, opens: true as const, ...(value !== undefined ? { detail: value } : {}) };
  return { ...base, ...(value !== undefined ? { detail: value } : {}) };
}

/** The sub-sections as the template reads them, for the sidebar and Home's search. */
export function subsOf(section: SettingsSection): SubSection[] {
  return section.subs.map((sub) => ({ title: sub.title, ...(sub.body ? { body: sub.body } : {}), ...((sub.tip ?? sub.partTip) ? { tip: sub.tip ?? sub.partTip } : {}), wells: sub.wells.map((well) => shownIn(well).map(specOf)) }));
}

/** Where `id` is among `section`'s rows as they show now: its sub-section, and its index among the page's rows. */
export function placeOf(section: SettingsSection, id: string): { sub: number; row: number } {
  let row = 0;
  for (const [sub, each] of section.subs.entries()) {
    for (const well of each.wells) {
      for (const setting of shownIn(well)) {
        if (setting.id === id) return { sub, row };
        row++;
      }
    }
  }
  throw new Error(`no setting ${id}`);
}

/** Every setting of `section`, its sub-pages' included: for the page's list of what isn't built. */
export function settingsOf(section: SettingsSection): Setting[] {
  const all = section.subs.flatMap((sub) => sub.wells.flat());
  for (const sub of section.pages ?? []) all.push(...sub.wells.flat());
  return all;
}

/** A sample's pins: the resets drawn lit, or held this far. */
export interface ResetPins {
  /** The reset of the row of setting `id`, or of the page's title (`"section"`), or of sub-section `"sub:N"`. */
  readonly at: string;
  readonly lit?: boolean;
  readonly held?: number;
}

/** What the region and the page's title share: the title's reset, and how the region puts things back. */
interface Shared {
  head: HTMLElement | null;
  resetSection: () => void;
}

/**
 * The hold-to-reset button, in a room of its own that keeps its place while it is gone: it fades
 * out over `short` on `standard` when there is nothing left to reset, and back in when there is.
 */
function resetButton(label: string, act: () => void, pins?: ResetPins, atEnd = false): HTMLElement {
  const pinned = {
    ...(pins?.held !== undefined ? { held: pins.held } : {}),
    ...(pins?.lit || pins?.held !== undefined ? { pinned: "lit" as const } : {}),
  };
  // A heading's or the title's stands at the end of its line and says what it does, its body
  // filling as it is held; a row's sits beside its control, a quiet round one whose ring fills.
  const made = atEnd
    ? withHint(button({ label: "Reset", icon: "restart_alt", kind: "Text", size: "Sm", hold: act, ...pinned }), label, { held: "end" })
    : iconButton({ icon: "restart_alt", label, kind: "Text", size: "Sm", hold: act, ...pinned });
  const room = document.createElement("span");
  room.className = "st-reset";
  room.append(made);
  // Its press is its own: the row under it neither takes it nor flips its switch.
  room.addEventListener("click", (event) => event.stopPropagation());
  return room;
}

/** Shows or hides a reset, its room kept; a hidden one takes no stop. */
function showReset(room: HTMLElement | null, shown: boolean): void {
  if (!room) return;
  room.toggleAttribute("data-gone", !shown);
  const made = room.querySelector<HTMLButtonElement>("button");
  if (made) made.tabIndex = shown ? 0 : -1;
}

/** `section` as SidebarPage's section: its rows from its settings, drawn by `settingsRegion`. */
export function asSection(section: SettingsSection, pins?: ResetPins): Section {
  const defaults = new Defaults(section);
  const shared: Shared = { head: null, resetSection: () => {} };
  const region = settingsRegion(section, defaults, shared, pins);
  return {
    icon: section.icon,
    title: section.title,
    accent: section.accent,
    lede: section.lede,
    // A getter: Home's search reads the rows as they show now, dependents and all.
    get subs() {
      return subsOf(section);
    },
    trailing: () => {
      shared.head = resetButton(`Hold to reset ${section.title}`, () => shared.resetSection(), pins?.at === "section" ? pins : undefined, true);
      showReset(shared.head, pins?.at === "section" || !defaults.allAt(defaults.ofSection()));
      return shared.head;
    },
    region,
  };
}

/**
 * The control a setting's row ends in, in its column. `changed` is told after the
 * person changes it.
 */
function controlOf(setting: Setting, row: HTMLElement, changed: () => void): HTMLElement | null {
  const control = setting.control;
  const disabled = setting.disabled !== undefined;
  switch (control.kind) {
    case "choice":
      return tabs({
        choices: control.choices.map((label) => ({ label })),
        picked: control.picked(),
        disabled,
        onPick: (index) => {
          if (index === control.picked()) return;
          control.set(index);
          changed();
        },
      });
    case "slider":
      return slider({
        icon: control.icon,
        size: "Compact",
        stops: control.names.length,
        index: control.index(),
        names: control.names,
        ...(control.icons ? { icons: control.icons } : {}),
        widest: control.names.reduce((a, b) => (b.length > a.length ? b : a), ""),
        disabled,
        changed: (index) => {
          if (index === control.index()) return;
          control.set(index);
          changed();
        },
      });
    case "select": {
      const made = select({
        size: ROW_CONTROL,
        options: control.options,
        picked: control.picked(),
        disabled,
        onPick: (index) => {
          if (index !== control.picked()) {
            control.set(index);
            changed();
          }
          row.closest<HTMLElement>(".k-list")?.focus({ preventScroll: true });
        },
      });
      // The open select keeps its keys from the page's rows under it.
      made.addEventListener("keydown", (event) => {
        if (event.key !== "Tab") event.stopPropagation();
      });
      return made;
    }
    case "field": {
      const line = row.querySelector<HTMLElement>(".k-list-title + *");
      const plain = setting.line;
      const tell = (value: string) => {
        const problem = value === "" ? null : control.check(value);
        setInvalid(made, problem !== null);
        row.toggleAttribute("data-invalid", problem !== null);
        if (line) {
          line.textContent = problem ?? plain;
          line.style.color = problem === null ? "var(--muted)" : "var(--color-error)";
        }
        return problem;
      };
      const made = field({
        size: ROW_CONTROL,
        placeholder: control.placeholder,
        ...(control.value() ? { value: control.value() } : {}),
        onInput: (value) => {
          if (tell(value) === null) {
            control.set(value);
            changed();
          }
        },
      });
      made.addEventListener("keydown", (event) => {
        if (event.key === "Escape" || event.key === "Enter") {
          event.preventDefault();
          row.closest<HTMLElement>(".k-list")?.focus({ preventScroll: true });
        }
        // Typing stays in the field: j, k and the arrows are the text's, not the rows'.
        if (event.key !== "Tab") event.stopPropagation();
      });
      requestAnimationFrame(() => tell(control.value()));
      return made;
    }
    case "part":
      return control.make();
    default:
      return null;
  }
}

/** Where a row's reset goes: before its control, its fact or its switch. */
function controlEnd(row: HTMLElement): Element | null {
  return row.querySelector(".st-control, .k-list-detail, .k-list-switch, .k-list-row > * > .k-icon:last-child, .st-tail > .k-icon:last-child");
}

/** `row`, a `ListRow`, ending in `setting`'s control in the control column, its reset before it. */
function settingRow(row: HTMLElement, setting: Setting, changed: () => void, reset: HTMLElement | null): void {
  row.dataset.setting = setting.id;
  if (setting.unbuilt) row.dataset.unbuilt = "";
  if (setting.shows) row.dataset.depends = "";
  const control = controlOf(setting, row, changed);
  if (control) {
    row.querySelector(".k-list-detail")?.remove();
    const room = document.createElement("span");
    room.className = "st-control";
    room.append(control);
    row.firstElementChild?.append(room);
    // A press in a select or a field keeps the keyboard there: the row's own press would take it
    // back to the well. The row still becomes the current one.
    if (setting.control.kind === "select" || setting.control.kind === "field") {
      room.addEventListener("click", (event) => {
        event.stopPropagation();
        row.dispatchEvent(new PointerEvent("pointermove"));
      });
    }
  }
  if (reset) {
    const end = controlEnd(row);
    if (end) end.before(reset);
    else row.firstElementChild?.append(reset);
  }
  // Everything after the words in one row of its own, so a stacked row is the words over it.
  const band = row.firstElementChild;
  if (band && !band.querySelector(":scope > .st-tail") && band.children.length > 1) {
    const tail = layoutRow("s12", "centre", ...[...band.children].slice(1));
    tail.classList.add("st-tail");
    band.append(tail);
  }
}

/** `row`'s first line as laid out: side by side, or stacked, the words over their tail. */
function laidOut(row: HTMLElement, stacked: boolean): void {
  row.toggleAttribute("data-stacked", stacked);
  const band = row.firstElementChild;
  if (band instanceof HTMLElement) probe(band, stacked ? { role: "layout", axis: "y", gap: length("space-s8"), align: "end" } : { role: "layout", axis: "x", gap: length("space-s12"), align: "centre" });
  // Stacked, the row is a stacked row of its own, as tall as its two lines.
  const said = JSON.parse(row.dataset.probe ?? "{}") as { variant?: string; fixed?: object };
  const words = (said.variant ?? "").split(" ").filter((word) => word !== "" && word !== "stacked");
  const tall = stacked || words.includes("tip");
  row.dataset.probe = JSON.stringify({ ...said, variant: [...words, ...(stacked ? ["stacked"] : [])].join(" "), fixed: { width: false, height: !tall } });
}

/** Shows `setting`'s value on `row` again, with its control's own motion where it has one. */
function sync(row: HTMLElement, setting: Setting, changed: () => void): void {
  const control = setting.control;
  // A press made here is the reset's, not the person's: the row neither takes it nor moves the keyboard.
  const quietly = (target: HTMLElement) => {
    target.addEventListener("click", (event) => event.stopPropagation(), { once: true });
    target.click();
  };
  if (control.kind === "switch") {
    const toggle = row.querySelector<HTMLElement>(".k-switch");
    if (toggle && (toggle.getAttribute("aria-checked") === "true") !== control.on()) quietly(toggle);
    row.setAttribute("aria-checked", String(control.on()));
  } else if (control.kind === "choice") {
    const tab = row.querySelectorAll<HTMLElement>(".st-control [role='tab']")[control.picked()];
    if (tab && tab.getAttribute("aria-selected") !== "true") quietly(tab);
  } else if (control.kind === "slider") {
    const made = row.querySelector(".st-control")?.firstElementChild;
    if (made) setSlider(made, control.index());
  } else if (control.kind === "select" || control.kind === "field") {
    const room = row.querySelector<HTMLElement>(".st-control");
    const made = controlOf(setting, row, changed);
    if (room && made) room.replaceChildren(made);
  }
}

/**
 * The region a section of Settings draws in place of the template's wells: the same sub-sections,
 * their rows ending in their controls and their resets, a dependent row under the row it depends on,
 * a risky value's warning under its well; or, once a row opened one, its sub-page with Back.
 */
export function settingsRegion(spec: SettingsSection, defaults: Defaults, shared: Shared, pins?: ResetPins): (host: RegionHost) => HTMLElement {
  return (host) => {
    const root = stack("s32", "start");
    root.classList.add("st-region");
    let open: SubPage | null = (spec.pages ?? []).find((each) => each.id === spec.opened) ?? null;
    const bySetting = new WeakMap<HTMLElement, Setting>();
    const all = new Map<string, Setting>();
    /** Each reset drawn, and whether it shows now. */
    let resets: { room: HTMLElement; shown: () => boolean }[] = [];

    /** Shows each reset that has something to put back, and hides the rest, the page's title's too. */
    function refreshResets(): void {
      for (const each of resets) showReset(each.room, each.shown());
      if (pins?.at !== "section") showReset(shared.head, !defaults.allAt(defaults.ofSection()));
    }

    /** A setting changed: the region is drawn again if that changes what shows, else its resets follow. */
    const changedFor = (setting: Setting) => () => {
      if (setting.drives || setting.warns) draw();
      else refreshResets();
    };

    /** The values of `settings` put back: each row shows its own with its control's motion, then the region follows. */
    function restore(which: readonly Setting[], reset: () => void): void {
      const from = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      reset();
      for (const row of root.querySelectorAll<HTMLElement>(".k-list-row[data-setting]")) {
        const setting = bySetting.get(row);
        if (setting) sync(row, setting, changedFor(setting));
      }
      refreshResets();
      // A reset held from the keyboard fades out with nothing left to put back: the keyboard goes
      // on to the well it put back, never to the page behind.
      if (from?.closest("[data-gone]")) {
        const well = from.closest(".k-list-row")?.closest<HTMLElement>(".k-list") ?? from.closest(".k-section")?.querySelector<HTMLElement>(".k-list") ?? root.querySelector<HTMLElement>(".k-list");
        well?.focus({ preventScroll: true });
      }
      // What shows may change: drawn again once the controls have moved.
      if (which.some((each) => each.drives || each.warns || each.shows)) setTimeout(draw, durationMs("medium") / settings().speed);
    }

    /** A reset, kept with what it asks of it. */
    const kept = (room: HTMLElement, shown: () => boolean): HTMLElement => {
      resets.push({ room, shown });
      return room;
    };

    /** The pins for the reset at `at`, if a sample pinned it. */
    const pinned = (at: string) => (pins?.at === at ? pins : undefined);

    /** Draws a well of `settings`, each row ending in its control and its reset. */
    const wellOf = (list: readonly Setting[]): HTMLElement => {
      const shown = shownIn(list);
      const made = settingsWell(shown.map(specOf));
      const rows = [...made.querySelectorAll<HTMLElement>(".k-list-row")];
      rows.forEach((row, index) => {
        const setting = shown[index]!;
        all.set(setting.id, setting);
        bySetting.set(row, setting);
        const reset = defaults.resettable(setting)
          ? kept(
              resetButton(defaults.words(setting), () => restore([setting], () => defaults.resetShown(setting)), pinned(setting.id)),
              () => pins?.at === setting.id || !defaults.shownAt(setting),
            )
          : null;
        settingRow(row, setting, changedFor(setting), reset);
      });
      return made;
    };

    /** A well's settings, then the warning of a risky value in it. */
    const wellsOf = (wells: readonly (readonly Setting[])[]): HTMLElement[] => {
      const parts: HTMLElement[] = [];
      for (const list of wells) {
        parts.push(wellOf(list));
        for (const setting of shownIn(list)) {
          const warning = setting.warns?.();
          if (warning) parts.push(banner({ tone: "Warning", icon: "warning", title: warning.title, body: warning.body }));
        }
      }
      return parts;
    };

    /** A heading with its reset at the end of its line, putting back everything `settings` hold. */
    const headed = (title: string, which: readonly Setting[], at: string, said?: string) => ({
      words: title,
      ...(said ? { tip: tip(said) } : {}),
      trailing: kept(
        resetButton(`Hold to reset ${title}`, () => restore(which, () => defaults.resetAll(which)), pinned(at), true),
        () => pins?.at === at || !defaults.allAt(which),
      ),
    });

    /** A sub-section: its heading and its reset, then its wells. */
    const subOf = (sub: SettingsSub, index: number): HTMLElement =>
      section(headed(sub.title, defaults.ofSub(sub), `sub:${index}`, sub.tip), sub.body, ...[sub.before?.(sub) ?? null].filter((part): part is HTMLElement => part !== null), ...wellsOf(sub.wells));

    /** A sub-page: Back to where it was opened, then its heading and its reset over its wells. */
    const pageOf = (sub: SubPage): HTMLElement[] => {
      const back = button({ label: sub.back, icon: "arrow_back", kind: "Text", size: "Sm" });
      back.classList.add("st-back");
      back.addEventListener("click", () => close());
      const parts: HTMLElement[] = [];
      if (sub.before) parts.push(sub.before());
      parts.push(...wellsOf(sub.wells));
      if (sub.after) parts.push(sub.after());
      return [back, filling(section(headed(sub.title, sub.wells.flat(), "page"), sub.body, ...parts))];
    };

    function draw(): void {
      const had = root.contains(document.activeElement);
      const lit = root.querySelector<HTMLElement>(".k-list-row[data-current]")?.dataset.setting;
      resets = [];
      root.toggleAttribute("data-subpage", open !== null);
      root.replaceChildren(...(open ? pageOf(open) : spec.subs.map((sub, index) => filling(subOf(sub, index)))));
      refreshResets();
      if (!root.isConnected) return;
      fit();
      host.refresh();
      if (!had) return;
      // The keyboard stays on the setting it was on, in its well drawn again.
      const again = lit ? root.querySelector<HTMLElement>(`.k-list-row[data-setting="${lit}"]`) : null;
      if (again) {
        again.dispatchEvent(new PointerEvent("pointermove"));
        again.closest<HTMLElement>(".k-list")?.focus({ preventScroll: true });
      }
    }

    shared.resetSection = () => restore(defaults.ofSection(), () => defaults.resetAll(defaults.ofSection()));

    /** The row that opened the sub-page, which Back puts the keyboard on again. */
    let opener: string | null = null;

    /** Into a sub-page: the page turns deeper, the sub-page coming in from the end at its top. */
    function openPage(id: string, from?: string): void {
      const page = (spec.pages ?? []).find((each) => each.id === id) ?? null;
      if (!page) return;
      opener = from ?? null;
      host.turn(
        true,
        () => {
          open = page;
          draw();
          host.enter(0);
        },
        undefined,
        Math.max(0, spec.subs.findIndex((sub) => sub.title === page.back)),
      );
    }

    /** Back: the page turns back out, the section scrolled where it was, the keyboard on the row that opened it. */
    function close(): void {
      const from = opener;
      host.turn(false, () => {
        open = null;
        draw();
        const row = from ? root.querySelector<HTMLElement>(`.k-list-row[data-setting="${from}"]`) : null;
        if (row) {
          row.dispatchEvent(new PointerEvent("pointermove"));
          row.closest<HTMLElement>(".k-list")?.focus({ preventScroll: true });
        }
      });
    }

    // A reset held past its acting fades away under the pointer, so the press's release lands on
    // its row: a press that started on a reset is the reset's, and no row takes its click.
    let pressedOnReset = false;
    root.addEventListener("pointerdown", (event) => (pressedOnReset = (event.target as HTMLElement).closest(".st-reset") !== null), true);
    root.addEventListener(
      "click",
      (event) => {
        // The reset's own work presses controls too (`sync`): only the person's release is held back.
        if (!pressedOnReset || !event.isTrusted) return;
        pressedOnReset = false;
        if ((event.target as HTMLElement).closest(".st-reset")) return;
        event.stopPropagation();
        event.preventDefault();
      },
      true,
    );

    /** What a press on a row does past the template's: a switch flips, a row opens its page. */
    root.addEventListener("click", (event) => {
      const target = event.target as HTMLElement;
      const row = target.closest<HTMLElement>(".k-list-row");
      if (!row || target.closest(".st-control")) return;
      const setting = bySetting.get(row);
      if (!setting || setting.disabled) return;
      const control = setting.control;
      if (control.kind === "switch") {
        // The row's own press has flipped its switch already. The switch says where it stands: the
        // row's own `aria-checked` flips twice, as the switch's press comes back up through it.
        control.set(row.querySelector(".k-switch")?.getAttribute("aria-checked") === "true");
        if (setting.drives || setting.warns) requestAnimationFrame(draw);
        else refreshResets();
      } else if (control.kind === "opens") openPage(control.to, setting.id);
    });

    /** The keys a setting's control and its reset take on its row, before the page's rows see them. */
    root.addEventListener(
      "keydown",
      (event) => {
        const target = event.target as HTMLElement;
        // On a row's reset, Tab goes on from its row and Shift+Tab back to it.
        if (event.key === "Tab" && target.closest(".k-list-row .st-reset")) {
          const well = target.closest<HTMLElement>(".k-list");
          well?.focus({ preventScroll: true });
          if (event.shiftKey) {
            event.preventDefault();
            event.stopPropagation();
          }
          return;
        }
        if (!target.classList.contains("k-list")) return;
        if (open && event.key === "Escape") {
          event.preventDefault();
          event.stopPropagation();
          close();
          return;
        }
        const row = target.querySelector<HTMLElement>(".k-list-row[data-current]");
        const setting = row ? bySetting.get(row) : undefined;
        if (!row || !setting || setting.disabled) return;
        // Tab from a row with a reset showing goes to the reset first: it comes before the control.
        if (event.key === "Tab" && !event.shiftKey) {
          const reset = row.querySelector<HTMLButtonElement>(".st-reset:not([data-gone]) button");
          if (!reset) return;
          reset.focus({ preventScroll: true });
          event.preventDefault();
          event.stopPropagation();
          return;
        }
        const control = setting.control;
        const room = row.querySelector<HTMLElement>(".st-control");
        const side = event.key === "ArrowLeft" ? -1 : event.key === "ArrowRight" ? 1 : 0;
        const take = event.key === "Enter" || event.key === " ";
        if (control.kind === "choice" && (side !== 0 || take)) {
          const count = control.choices.length;
          const to = side !== 0 ? Math.max(0, Math.min(count - 1, control.picked() + side)) : (control.picked() + 1) % count;
          room?.querySelectorAll<HTMLElement>("[role='tab']")[to]?.click();
        } else if (control.kind === "slider" && side !== 0) {
          const to = Math.max(0, Math.min(control.names.length - 1, control.index() + side));
          if (to !== control.index()) {
            const made = room?.firstElementChild;
            if (made) setSlider(made, to);
            control.set(to);
            changedFor(setting)();
          }
        } else if (control.kind === "select" && take) {
          const surface = room?.querySelector<HTMLElement>("[role='combobox']");
          surface?.querySelector<HTMLElement>(".k-select-head")?.click();
          surface?.focus({ preventScroll: true });
        } else if (control.kind === "field" && take) {
          room?.querySelector<HTMLElement>(".k-field")?.focus({ preventScroll: true });
        } else return;
        event.preventDefault();
        event.stopPropagation();
      },
      true,
    );

    // The find box draws a row again, plain: its control and its reset go back on.
    new MutationObserver(() => {
      for (const row of root.querySelectorAll<HTMLElement>(".k-list-row:not([data-setting])")) {
        const title = row.querySelector(".k-list-title")?.textContent ?? "";
        const setting = [...all.values()].find((each) => each.title === title);
        if (!setting) continue;
        bySetting.set(row, setting);
        const reset = defaults.resettable(setting)
          ? kept(
              resetButton(defaults.words(setting), () => restore([setting], () => defaults.resetShown(setting))),
              () => !defaults.shownAt(setting),
            )
          : null;
        settingRow(row, setting, changedFor(setting), reset);
        refreshResets();
      }
    }).observe(root, { childList: true, subtree: true });

    // A row too narrow for its name and its line beside its control puts the control,
    // its reset before it, on a line of its own under the words, so neither is ever cut. Every row is laid
    // out side by side first, then each whose name is cut stacks: one read between the writes.
    const fit = () => {
      const all = [...root.querySelectorAll<HTMLElement>(".k-list-row[data-setting]")];
      for (const row of all) laidOut(row, false);
      // Its name or its plain line cut: either is what the row is for.
      const cut = all.filter((row) => [...row.querySelectorAll<HTMLElement>(".k-list-title, .k-list-title + *")].some((words) => words.scrollWidth > words.clientWidth + 0.5));
      for (const row of cut) laidOut(row, true);
    };
    let fitted = 0;
    new ResizeObserver(() => {
      const width = root.clientWidth;
      if (width === fitted) return;
      fitted = width;
      fit();
    }).observe(root);
    root.addEventListener("st-fit", fit);

    // A choice drawn before a sub-page's wells changes what they set: they are drawn again.
    root.addEventListener("st-redraw", () => draw());

    // A button in a part before the wells opens a sub-page as a row's chevron does.
    root.addEventListener("st-open", (event) => openPage((event as CustomEvent<string>).detail));

    draw();
    return root;
  };
}
