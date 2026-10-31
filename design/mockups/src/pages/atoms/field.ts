// The Field page, with Labelled and Note: `docs/design/components/field.md`, in the state model
// (`docs/DESIGN.md`, "States"). A field rests, lights up hovered or with the keyboard in it (then
// with its caret), is disabled, read-only or invalid, by night and by day; it sits under its label
// with its note under it.

import { button } from "../../lib/button.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { type FieldOptions, field, fieldRoom, labelled, note, setSecret } from "../../lib/field.ts";
import { stack } from "../../lib/layout.ts";
import { label } from "../../site/dom.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

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
  /** A read-only field: it never lights and is never invalid. */
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
    if (state === "error") return label("Never invalid: it takes no input", "small");
    return boxed({ ...row.options, state: state === "disabled" ? "Disabled" : "Locked" });
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

const DIFFERENCES = [
  "Question for Kuba, lit by the pointer: today hovering only darkens the hairline to `outline` and the tube is for the keyboard. Here, as the state model has it, hovering and the keyboard are one lit state, so the pointer lights the tube and its glow too; only the caret says the keyboard is in it. Recommended.",
  "Question for Kuba, lighting up: today the hairline's colour changes over `micro`. Here the field lights up and goes back to rest over `short`, as every control does.",
  "Question for Kuba, the caret on an invalid field: today it stays `primary`. Here it takes the field's tone, `error`, as the tube does, so the field reads as one thing in one colour; the selection keeps the accent. Recommended.",
  "Question for Kuba, the caret and the signs: the caret isn't a light. It is `primary` (or `error`) with the signs on and off, by night and by day: it never burns to the core, never turns to glass and never glows. Recommended.",
  "Question for Kuba, an invalid field lit: as today, its fill stays `surface_container_high` where a valid one rises to the highest container, so the error's tube sits on the field's own fill.",
  "Question for Kuba, pressed: a field has no pressed look. A press puts the caret where it lands, so the field is lit with the keyboard in it, never overdriven. Recommended.",
  "Question for Kuba, read-only: as today, it never lights, even under the pointer, and is no Tab stop, so its value can't be selected and copied from the keyboard. Recommended for now; a read-only field you can copy from would be a new state.",
  "Question for Kuba, a long note: here it breaks between words onto as many lines as it needs in the field's width, as iced draws it; QML's `Note` takes no room today, so its line runs on past the field. Recommended: wrap in both.",
];

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
    "With the keyboard in it, it shows the caret: the thin stroke wide (`stroke.thin`, whole device pixels), centred on its place on a whole device pixel, in the field's tone. It blinks, shown then hidden for 530 ms each, and stays solid for 530 ms after it moves, so it never hides while you type; the caret and the text appear at once, never animated. A selection is the accent at `opacity.selection` behind the selected text, at the small radius, and hides the caret. A value longer than the field scrolls sideways to keep the caret in view, and is clipped a caret's width past its room, so a caret at either end shows whole; without the keyboard in it, the field shows its value's start.",
    "Invalid, the value is wrong, and the field says so in the error's tone: its hairline is `error` at rest, and lit, its tube, glow and caret are the error's (`error`, with `error_glass` with the signs off), with the reason in the note under it. Read-only (`Locked`), it shows a value it won't change: its fill and hairline at `opacity.locked`, its value in `on_surface_variant`, no hover, no caret, no Tab stop. Disabled, it is grey: no tone, no line and no glow, the field `on_surface` at `opacity.disabled_container`, its value, placeholder and icon at `opacity.disabled_content`, and no Tab stop.",
    "`Labelled` puts a control under its label, `label` type in `on_surface_variant`, in a row `size.control_sm` tall whether or not it has a small action at its right (a Text button at sm: Show), so fields side by side line up; the field is `space.s4` under it. `Note` is the `small` line `space.s4` under the field: `on_surface_variant`, or `error` when the value is invalid, breaking between words in the field's width.",
    "The app owns the value and its editing in both libraries: it keeps the caret, turns keys into edits, and gives the field both; the field draws them and turns presses and drags into caret moves. A press puts the caret where it lands, a double press selects the word, a triple press all of it, and a drag selects.",
    ...DIFFERENCES,
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
      lit: "Hovered or with the keyboard in it, here with the keyboard in it and the signs on (Typed, under the pointer, shows it hovered: no caret). The fill rises to `surface_container_highest` (an invalid field keeps `surface_container_high`), the tube burns over the hairline, `primary` at `focus.tube`, or `error` on an invalid field, with the glow outside it in the same tone, and the caret shows at its place, or the selection in its stead. By day lit is its signs-off look in daylight colours: the tube as glass, no glow. A read-only field never lights.",
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
      lit: "At night at rest, the signs burn: the tube over the field's hairline inside its edge, `primary` at `focus.tube` (`error` on an invalid field), with the glow outside it, all that reaches past the field. The caret and the value don't light: the caret is the tone, never its core.",
      signsOff: "The same field still hovered or with the keyboard in it, the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. The tube is solid unlit glass, `primary_glass` (`error_glass` on an invalid field), with no glow; the fill stays at the highest container, and the caret keeps its colour and keeps blinking.",
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
        what: "Hovered or with the keyboard in it, the field lights up: a fade over `short` on `standard`, never the flicker. The fill rises, the tube and its glow fade in, in the field's tone, as `--lit` goes from 0 to 1. The caret appears at once with the keyboard: it never fades.",
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
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "The keyboard into the field, or on to the next or previous control. A read-only or disabled field is skipped." },
      { keys: ["← →"], does: "The caret a character left or right; with a selection, to its start or end." },
      { keys: ["Ctrl+← →", "Alt+← →"], does: "The caret a word left or right." },
      { keys: ["Shift+← →", "Shift+Home", "Shift+End"], does: "Grows or shrinks the selection." },
      { keys: ["Home", "End"], does: "The caret to the start or the end of the value." },
      { keys: ["Ctrl+A"], does: "Selects the whole value." },
      { keys: ["Backspace", "Delete"], does: "Deletes the character before or after the caret, or the selection; with Ctrl, the word." },
      { keys: ["Space"], does: "A space, never a press." },
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
      { what: "The caret's width, and how far past its room the text is clipped, rounded half up.", token: "stroke-thin" },
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
