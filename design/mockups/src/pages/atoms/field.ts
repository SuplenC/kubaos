// The Field page, with Labelled and Note: `docs/design/components/field.md`, in the state model
// (`docs/DESIGN.md`, "States"). A field rests, lights up hovered or with the keyboard in it (then
// with its caret, which strikes on and glows while you type), is disabled, read-only or invalid, by
// night and by day; it sits under its label with its note under it.

import { button } from "../../lib/button.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { type FieldOptions, field, fieldRoom, labelled, note, setSecret } from "../../lib/field.ts";
import { stack } from "../../lib/layout.ts";
import { label } from "../../site/dom.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const VALUE: Words = { short: "tuxedo", long: "tuxedo-workstation-in-the-living-room" };
const PLACEHOLDER: Words = { short: "Name", long: "Your full name, as it appears on your ID" };
const LABEL: Words = { short: "Hostname", long: "The name this computer shows on your network" };
const NOTE: Words = { short: "Optional.", long: "Letters, digits and dashes, up to 63 characters: it names this computer to every device on your network." };
const ERROR: Words = { short: "Required.", long: "That's not an email address: it needs an @ and a domain after it, like kuba@example.com. Typo?" };

/** The room every sample's field fills here; a field fills whatever room it is given. */
const ROOM = "var(--size-menu)";

/** `options`' field in the samples' room. */
function boxed(options: FieldOptions): HTMLElement {
  return fieldRoom(ROOM, field(options));
}

/** The states the matrix shows a field in. */
type Shown = "rest" | "lit" | "disabled" | "error";

/** A row of the matrix: what its field holds, in each state. */
interface Row {
  readonly name: string;
  readonly options: FieldOptions;
  /** A read-only field: it lights and selects, but it is never invalid. */
  readonly locked?: boolean;
}

const ROWS: readonly Row[] = [
  { name: "Empty", options: {} },
  { name: "Placeholder", options: { placeholder: "Your name" } },
  { name: "Typed", options: { value: "tuxedo" } },
  { name: "Typed, under the pointer", options: { value: "tuxedo", pointer: true } },
  { name: "With an icon", options: { value: "Shinj", icon: "search", placeholder: "Search" } },
  { name: "A word selected", options: { value: "Shinji Ikari", caret: { pos: 12, anchor: 7 } } },
  { name: "Password", options: { value: "hunter2", secret: true } },
  { name: "A long value, scrolled", options: { value: "tuxedo-workstation-in-the-living-room" } },
  { name: "Invalid", options: { value: "not-an-email", icon: "mail", state: "Invalid" } },
  { name: "Read-only", options: { value: "kuba", icon: "lock" }, locked: true },
];

/** `row`'s field in `state`; lit is pinned with the keyboard in it unless the row is the pointer's. */
function cell(row: Row, state: Shown | "signs-off"): HTMLElement {
  if (row.locked) {
    switch (state) {
      case "error":
        return label("Never invalid: it takes no input", "small");
      case "disabled":
        return boxed({ ...row.options, state: "Disabled" });
      case "rest":
        return boxed({ ...row.options, state: "Locked" });
      case "lit":
      case "signs-off":
        return boxed({ ...row.options, state: "Locked", pinned: state });
    }
  }
  switch (state) {
    case "rest":
      return boxed(row.options);
    case "lit":
    case "signs-off":
      return boxed({ ...row.options, pinned: state });
    case "disabled":
      return boxed({ ...row.options, state: "Disabled" });
    case "error":
      return boxed({ ...row.options, state: "Invalid" });
  }
}

/** A plain field and an invalid one, in `pinned` or at rest: what each move plays on. */
function pair(pinned?: "lit"): () => HTMLElement {
  return () =>
    stack(
      "s12",
      "start",
      boxed({ value: "tuxedo", ...(pinned ? { pinned } : {}) }),
      boxed({ value: "not-an-email", icon: "mail", state: "Invalid", ...(pinned ? { pinned } : {}) }),
    );
}

/** A password field under its label, its Show action turning the bullets into the value and back. */
function password(): HTMLElement {
  const input = field({ value: "hunter2", secret: true });
  const action = (hidden: boolean): HTMLElement => {
    const made = button({ label: hidden ? "Show" : "Hide", kind: "Text", size: "Sm", icon: hidden ? "visibility" : "visibility_off" });
    made.addEventListener("click", () => {
      setSecret(input, !hidden);
      made.replaceWith(action(!hidden));
    });
    return made;
  };
  return fieldRoom(ROOM, labelled("Password", input, { action: action(true) }));
}

/** The hostname field under its label with its note, its words at `words`' length. */
function hostname(words: string = shown(VALUE)): HTMLElement {
  return fieldRoom(ROOM, labelled(shown(LABEL), field({ value: words, icon: "computer" }), { note: shown(NOTE) }));
}

/** An invalid email field under its label, with the reason in its note. */
function email(reason: string = shown(ERROR)): HTMLElement {
  return fieldRoom(ROOM, labelled("Email for git", field({ value: "not-an-email", icon: "mail", state: "Invalid" }), { note: reason, invalid: true }));
}

const DECIDED = [
  "Decided, lit by the pointer: hovering and the keyboard are one lit state, as the state model has it, so the pointer lights the tube and its glow too; only the caret says the keyboard is in the field, and it never shows on hover. Today hovering only darkens the hairline to `outline` and the tube is the keyboard's.",
  "Decided, lighting up: the field lights up and goes back to rest over `short`, as every control does. Today the hairline's colour changes over `micro`.",
  "Decided, the caret on an invalid field: it takes the field's tone, the error's (`error_core` at night, `error` by day), as the tube does, so the field reads as one thing in one colour; the selection keeps the accent. Today it stays `primary`.",
  "Decided, the caret strikes: as the keyboard comes in, the caret strikes on through `neon.strike`, the one light besides the power changes that flickers. Its glow strikes with it, then fades as after typing. In both libraries the caret still blinks, in the accent, the line's full height, tight against the letter.",
  "Decided, an invalid field lit: as today, its fill stays `surface_container_high` where a valid one rises to the highest container, so the error's tube sits on the field's own fill.",
  "Decided, pressed: a field has no pressed look. A press puts the caret where it lands, so the field is lit with the keyboard in it, never overdriven.",
  "Decided, read-only: its value can be selected and copied. It is a Tab stop and lights under the pointer and with the keyboard in it as any field does, and the pointer and the keys select in it as in an editable one, with Ctrl+C to copy; it never edits. With the keyboard in it, its caret shows, so a selection from the keys has a place to start: it strikes on with focus but never glows, since nothing is typed. Today it never lights, is no Tab stop and can't be selected.",
  "Decided, a long note: it breaks between words onto as many lines as it needs in the field's width, as iced draws it, in both libraries. Today QML's `Note` takes no room, so its line runs on past the field.",
];

const ON = "a field and an invalid one, the keyboard in each";

/**
 * The caret striking on as the keyboard comes in, its glow with it, then the glow holding and
 * fading as after typing.
 */
const caretStrikes: Move = {
  name: "Focus arrives: the caret strikes on",
  what: "The keyboard comes into the field: as it lights up, the caret strikes on through `neon.strike`'s steps over `medium`, by night and by day, the one light besides the power changes that flickers. At night its glow strikes with it, holds for `caret_hold`, then fades over `caret_fade` on `standard`, as after typing. A read-only field's caret strikes the same, with no glow. With the flicker off the strike is a fade over `short`.",
  tracks: [
    { part: "caret", on: ON, target: ".k-field-caret", property: "opacity", from: 0, to: 1, duration: "medium", curve: "neon.strike" },
    { part: "caret's glow", on: ON, target: ".k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "caret_fade", cue: ["medium", "caret_hold"], curve: "standard" },
  ],
  reduced: [
    { part: "caret", on: ON, target: ".k-field-caret", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "caret's glow", on: ON, target: ".k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "instant", cue: "caret_hold", curve: "linear" },
  ],
  unflickered: [
    { part: "caret", on: ON, target: ".k-field-caret", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "caret's glow", on: ON, target: ".k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "caret_fade", cue: ["short", "caret_hold"], curve: "standard" },
  ],
  back: "cuts",
  sample: pair("lit"),
};

/** The caret's glow while you type: in on a key, held while keys follow, faded once they stop. */
const caretGlows: Move = {
  name: "Typing: the caret's glow",
  what: "At night, a key typed brings a glow in the tone round the caret over `caret_glow` on `standard`; it holds while keys follow, and `caret_hold` after the last one it fades over `caret_fade` on `standard` to nothing. The caret itself never changes. By day nothing glows, and nothing changes while you type.",
  tracks: [
    { part: "caret's glow", on: ON, target: ".k-field-caret-glow", property: "opacity", from: 0, to: 1, duration: "caret_glow", curve: "standard" },
    { part: "caret's glow", on: ON, target: ".k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "caret_fade", cue: ["caret_glow", "caret_hold"], curve: "standard" },
  ],
  reduced: [
    { part: "caret's glow", on: ON, target: ".k-field-caret-glow", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "caret's glow", on: ON, target: ".k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "instant", cue: "caret_hold", curve: "linear" },
  ],
  back: "cuts",
  sample: pair("lit"),
};

export default page({
  level: "atoms",
  slug: "field",
  title: "Field, Labelled and Note",
  summary: "One line of text to type into, under its label with a note under it. Its states follow the neon: it rests, lights up hovered or with the keyboard in it, and shows its caret while you type.",
  live: () =>
    stack(
      "s24",
      "start",
      fieldRoom(ROOM, labelled("Full name", field({ placeholder: "Your name" }))),
      hostname(),
      password(),
      email(),
    ),
  description: [
    "A field takes one line of text: a name, a hostname, a password, a search. It is `size.control_md` tall, as tall as a regular button so the two line up side by side, at the control radius (`radius.control`), and it fills the room it is given. Its fill is `surface_container_high` with a hairline of `outline_variant` inside its edge. Its value is `body_large` in `on_surface`, `space.s12` in from each side and centred down the field; an optional icon (`body_large`'s, `type.body_large.icon`) leads it `space.s12` in, centred on a whole device pixel, in `on_surface_variant`, and the text starts `space.s8` past it. While it is empty it shows its placeholder, `on_surface_variant` at `opacity.placeholder_text`: a hint, never a value that looks typed. A password shows one bullet per character.",
    "Its states follow the neon (`docs/DESIGN.md`, \"States\"): rest, lit and disabled, plus invalid and read-only. Hovered or with the keyboard in it, one state, it is lit: its fill rises to `surface_container_highest`, and at night the tube burns over its hairline, `primary` at `focus.tube` inside the edge, with the glow outside it. With the signs off, by day always and at night while a power change has the lights out, the tube is solid glass, `primary_glass`, with no glow. Nothing but the glow reaches past the field.",
    "With the keyboard in it, it shows the caret: one line the thin stroke wide (`stroke.thin`, whole device pixels) with rounded ends, `caret.height` tall and centred on the value's line, so it stops short of the line at each end, centred on the boundary between the letters either side, on a whole device pixel. At night it is the tone's core (`primary_core`, `error_core` on an invalid field); by day it is the tone (`primary`, `error`); with the signs off it is the tone's glass, as the line is, with no glow. It is steady: it never blinks or breathes, and it moves and the text appears at once. As the keyboard comes in it strikes on through `neon.strike`'s steps over `medium`, by night and by day. At night, while you type, a glow in the tone shows round it, `caret.glow` at its edge fading to nothing `caret.near` out: it comes in over `caret_glow`, holds while keys follow, and `caret_hold` after the last one fades over `caret_fade` to nothing. The strike brings the glow with it, which then fades as after typing. By day nothing glows. A selection is the accent at `opacity.selection` behind the selected text, at the small radius, and hides the caret. A value longer than the field scrolls sideways to keep the caret inside its room; without the keyboard in it, the field shows its value's start.",
    "Invalid, the value is wrong, and the field says so in the error's tone: its hairline is `error` at rest, and lit, its tube, glow and caret are the error's (`error`, with `error_glass` with the signs off), with the reason in the note under it. Read-only (`Locked`), it shows a value it won't change but lets you select and copy it: its fill and hairline at `opacity.locked`, lit or not, its value in `on_surface_variant`. It is a Tab stop and lights as any field does, the pointer and the keys select in it, and Ctrl+C copies; no key edits it. With the keyboard in it the caret shows and strikes on, but never glows. Disabled, it is grey: no tone, no line and no glow, the field `on_surface` at `opacity.disabled_container`, its value, placeholder and icon at `opacity.disabled_content`, and no Tab stop. It shows no caret and no key edits it, but the pointer still selects its value, in the same selection, and Ctrl+C copies it.",
    "`Labelled` puts a control under its label, `label` type in `on_surface_variant`, in a row `size.control_sm` tall whether or not it has a small action at its right (a Text button at sm: Show), so fields side by side line up; the field is `space.s4` under it. `Note` is the `small` line `space.s4` under the field: `on_surface_variant`, or `error` when the value is invalid, breaking between words in the field's width.",
    "The app owns the value and its editing in both libraries: it keeps the caret, turns keys into edits, and gives the field both; the field draws them and turns presses and drags into caret moves. A press puts the caret where it lands, a double press selects the word, a triple press all of it, and a drag selects.",
    ...DECIDED,
  ],
  variants: [
    { name: "Field", for: "One line of text alone: a search over a list, a field in a form row.", sample: () => boxed({ placeholder: "Search" }) },
    { name: "With an icon", for: "A field whose kind the icon says before anything is typed: search, a hostname, an email.", sample: () => boxed({ value: "Shinj", icon: "search" }) },
    { name: "Password", for: "A secret: one bullet per character, and a Show action beside its label that turns them into the value and back.", sample: password },
    { name: "Labelled", for: "Every field of a form: the label says what goes in it.", sample: () => fieldRoom(ROOM, labelled("Full name", field({ placeholder: "Your name" }))) },
    { name: "With a note", for: "A field whose rule or use needs a line: what it accepts, what it's for.", sample: () => hostname() },
    { name: "With an error note", for: "An invalid field: its note says why, in the error colour, and what to do.", sample: () => email() },
    {
      name: "Read-only",
      for: "A value shown where it can't be changed, with a note saying where it can.",
      sample: () => fieldRoom(ROOM, labelled("User name", field({ value: "kuba", icon: "lock", state: "Locked" }), { note: "Set when you installed; read-only here." })),
    },
    {
      name: "Disabled",
      for: "A field not usable now, with a note saying what turns it on.",
      sample: () => fieldRoom(ROOM, labelled("Proxy", field({ placeholder: "proxy.example.com:3128", icon: "lan", state: "Disabled" }), { note: "Turn the proxy on to set it." })),
    },
  ],
  sizes: doesntApply("a field has one size, `size.control_md` tall, as tall as a regular button; its width is its room's."),
  states: {
    shown: ["rest", "lit", "disabled", "error"],
    absent: {
      pressed: "a press puts the caret where it lands, so the field is lit with the keyboard in it, never overdriven.",
      selected: "a field holds a value, not a choice; selected text is its row, A word selected.",
      loading: "a field doesn't wait; what checks its value says so in its note.",
      empty: "an empty field is a value, its rows Empty and Placeholder, shown in every state.",
    },
    rows: nonEmpty(ROWS.map((row) => ({ name: row.name, cell: (state: Shown) => cell(row, state) }))),
    described: {
      rest: "Unlit: `surface_container_high` with an `outline_variant` hairline inside its edge, the value `on_surface`, the placeholder `on_surface_variant` at `opacity.placeholder_text`, the icon `on_surface_variant`. Invalid, the hairline is `error`. Read-only, the fill and hairline are at `opacity.locked` and the value `on_surface_variant`. By day the same roles in daylight colours.",
      lit: "Hovered or with the keyboard in it, here with the keyboard in it and the signs on (Typed, under the pointer, shows it hovered: no caret). The fill rises to `surface_container_highest` (an invalid field keeps `surface_container_high`), the tube burns over the hairline, `primary` at `focus.tube`, or `error` on an invalid field, with the glow outside it in the same tone, and the caret shows at its place, or the selection in its stead. By day lit is its signs-off look in daylight colours: the tube as glass, no glow. A read-only field lights the same, its fill at `opacity.locked`, its caret never glowing.",
      disabled: "Grey, with no tone, no line and no glow: the field `on_surface` at `opacity.disabled_container`, its value, placeholder and icon `on_surface` at `opacity.disabled_content`. No hover, no caret, no Tab stop.",
      error: "Invalid, at rest: the hairline is `error`, the value stays `on_surface`, and its `Note` gives the reason in `error`. Lit, the Invalid row shows its tube, glow and caret in the error's tone.",
    },
  },
  focus: {
    tubes: [
      { name: "Normal", tone: "accent", filled: false, content: false },
      { name: "Invalid", tone: "error", filled: false, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn: the tube over the field's hairline inside its edge, `primary` at `focus.tube` (`error` on an invalid field), with the glow outside it, all that reaches past the field. The value doesn't light; the caret is the tone's core, steady, glowing round it only while you type.",
      signsOff: "The same field still hovered or with the keyboard in it, the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. The tube is solid unlit glass, `primary_glass` (`error_glass` on an invalid field), with no glow; the fill stays at the highest container, and the caret is the tone's glass, as the line is, with no glow.",
    },
    signsOff: (index: number) => cell(ROWS[index]!, "signs-off"),
  },
  content: {
    text: [
      { name: "A value", words: VALUE, sample: (words: string) => boxed({ value: words, icon: "computer" }) },
      { name: "A value, the keyboard in it", words: VALUE, sample: (words: string) => boxed({ value: words, icon: "computer", pinned: "lit" }) },
      { name: "A placeholder", words: PLACEHOLDER, sample: (words: string) => boxed({ placeholder: words }) },
      { name: "A label", words: LABEL, sample: (words: string) => fieldRoom(ROOM, labelled(words, field({ value: "tuxedo" }))) },
      { name: "A note", words: NOTE, sample: (words: string) => fieldRoom(ROOM, note(words)) },
      { name: "An error note", words: ERROR, sample: (words: string) => fieldRoom(ROOM, note(words, true)) },
    ],
    missingPicture: [
      { name: "No icon", for: "The icon is optional: without one the text starts `space.s12` in. Every symbol is one of KubaOS Symbols, so a named symbol is never missing.", sample: () => boxed({ value: "tuxedo" }) },
    ],
    counts: [
      { name: "No note", for: "A field that needs no line under it: the label, then the field.", sample: () => fieldRoom(ROOM, labelled("Full name", field({ placeholder: "Your name" }))) },
      { name: "One note", for: "A note is one message, however many lines it wraps onto; a field has one note at most, its rule or its error.", sample: () => hostname(shown(fixed("tuxedo"))) },
      {
        name: "Fields side by side",
        for: "Labelled fields in a row line up: every label row is `size.control_sm` tall, with an action or without.",
        sample: () => stack("s16", "start", password(), fieldRoom(ROOM, labelled("Full name", field({ value: "Shinji Ikari" })))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("a field comes with the surface or the form it is on."),
    leaves: doesntApply("a field goes with the surface or the form it is on."),
    between: [
      {
        name: "Focus arrives",
        what: "Hovered or with the keyboard in it, the field lights up: a fade over `short` on `standard`, never the flicker. The fill rises, the tube and its glow fade in, in the field's tone, as `--lit` goes from 0 to 1. The caret strikes on with the keyboard, its own move below.",
        tracks: lights(".k-field", "a field and an invalid one", 0, 1),
        reduced: "same",
        back: "cuts",
        sample: pair(),
      },
      {
        name: "Focus leaves",
        what: "The pointer leaves or the keyboard moves on: the field fades back to rest over `short` on `standard`, and the caret goes at once.",
        tracks: lights(".k-field", "a field and an invalid one", 1, 0),
        reduced: "same",
        back: "cuts",
        sample: pair("lit"),
      },
      {
        name: "Turns invalid",
        what: "The app finds the value wrong: the field's tone fades from the accent to the error's over `short` on `standard`, its hairline at rest, its tube, glow and caret lit, as `--invalid` goes from 0 to 1; its note turns to the reason with it. Fixed, it plays back.",
        tracks: [{ part: "invalid", on: "a field at rest and lit", target: ".k-field", property: "--invalid", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => stack("s12", "start", boxed({ value: "not-an-email", icon: "mail" }), boxed({ value: "not-an-email", icon: "mail", pinned: "lit" })),
      },
      caretStrikes,
      caretGlows,
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "The keyboard into the field, or on to the next or previous control. A disabled field is skipped; a read-only one is a stop." },
      { keys: ["← →"], does: "The caret a character left or right; with a selection, to its start or end." },
      { keys: ["Ctrl+← →", "Alt+← →"], does: "The caret a word left or right." },
      { keys: ["Shift+← →", "Shift+Home", "Shift+End"], does: "Grows or shrinks the selection." },
      { keys: ["Home", "End"], does: "The caret to the start or the end of the value." },
      { keys: ["Ctrl+A"], does: "Selects the whole value." },
      { keys: ["Ctrl+C"], does: "Copies the selection, a read-only field's too; a password's never." },
      { keys: ["Backspace", "Delete"], does: "Deletes the character before or after the caret, or the selection; with Ctrl, the word. Nothing in a read-only field." },
      { keys: ["Space"], does: "A space, never a press; nothing in a read-only field." },
      { keys: ["Enter"], does: "What was typed is done: the field's surface goes on, as its Filled action does. Once per press." },
      { keys: ["↑ ↓", "Ctrl+N", "Ctrl+P", "Ctrl+J", "Ctrl+K"], does: "In a field with a list under it (a search, the Command menu), they move in the list; the field keeps the keyboard." },
      { keys: ["Esc"], does: "Nothing of the field's: the surface goes back one step." },
    ],
    order: "One stop in the surface's reading order, its label and note no stop of their own; a Labelled field's action is the next stop after it. Focus moving in or out never moves anything: the tube and its glow take no room.",
    lands: "A screen or a form starts in its first field, the caret at the end of its value; a dialog that asks for something starts in its field.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Field", for: "The field and its text.", sample: () => boxed({ value: "tuxedo" }) },
      { name: "With an icon", for: "The icon and where the text starts.", sample: () => boxed({ value: "Shinj", icon: "search", pinned: "lit" }) },
      { name: "Labelled, with a note", for: "The label row, the field and the note.", sample: () => hostname(shown(fixed("tuxedo"))) },
    ]),
    measures: [
      { what: "The field's height.", token: "size-control_md" },
      { what: "Its corners.", token: "radius-control" },
      { what: "The hairline inside its edge: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The value's and the placeholder's line, centred down the field.", token: "type-body_large-line" },
      { what: "The text's inset at each side, and the icon's.", token: "space-s12" },
      { what: "The icon.", token: "type-body_large-icon" },
      { what: "The gap between the icon and the text.", token: "space-s8" },
      { what: "The caret's width, rounded half up, and its rounded ends.", token: "stroke-thin" },
      { what: "The caret's height, centred on the value's line.", token: "caret-height" },
      { what: "The caret's glow's reach, while you type at night.", token: "caret-near" },
      { what: "The selection's corners.", token: "radius-small" },
      { what: "The label's row.", token: "size-control_sm" },
      { what: "The label's line.", token: "type-label-line" },
      { what: "The gap under the label's row and under the field.", token: "space-s4" },
      { what: "The note's line.", token: "type-small-line" },
      { what: "The lit tube, rounded half up.", token: "focus-tube" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
    ],
    reflow: {
      described: "A field fills its room: as wide as its column, from the narrowest a form's column gets up to `size.column`, the widest a column of text and fields grows. Its height never changes; a value wider than the field scrolls, and its note wraps.",
      sample: () => fieldRoom("min(100%, var(--size-column))", labelled(shown(LABEL), field({ value: shown(VALUE), icon: "computer" }), { note: shown(NOTE) })),
    },
  },
});
