// The Field page, with Labelled and Note: `docs/design/components/field.md`, in the state model
// (`docs/DESIGN.md`, "States"). A field rests, lights up hovered or with the keyboard in it (then
// with its caret, which strikes on and glows while you type), is disabled, read-only or invalid, by
// night and by day; it sits under its label with its note under it.

import { button } from "../../lib/button.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { type FieldOptions, field, fieldRoom, labelled, note, setSecret } from "../../lib/field.ts";
import { row, stack } from "../../lib/layout.ts";
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
  { name: "With a trailing mark", options: { value: "hunter2", icon: "lock", secret: true, trailing: { icon: "fingerprint", color: "primary" } } },
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

const ON = "a field and an invalid one, the keyboard in each";

/**
 * The caret striking on as the keyboard comes in, its glow with it, then the glow holding and
 * fading as after typing.
 */
const caretStrikes: Move = {
  name: "Focus arrives: the caret strikes on",
  what: "With the keyboard, it strikes through `neon.strike` over `medium`, the one other flicker; its glow fades after `caret_hold`.",
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
  what: "At night a key brings a glow over `caret_glow`; `caret_hold` after the last it fades over `caret_fade`.",
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
  lede: "One line of text to type into, under its label with a note under it.",
  code: { rust: "field::field(value, placeholder), field::labelled, field::note", qml: "Field, Labelled, Note" },
  stage: {
    live: () => row("s32", "start", stack("s24", "start", fieldRoom(ROOM, labelled("Full name", field({ placeholder: "Your name" }))), hostname()), stack("s24", "start", password(), email())),
    hint: "click to type; Show reveals the password",
    daylight: true,
  },
  variants: [
    { name: "Field", for: "alone: a search over a list", sample: () => boxed({ placeholder: "Search" }) },
    { name: "With an icon", for: "its kind said before anything is typed", sample: () => boxed({ value: "Shinj", icon: "search" }) },
    { name: "With a trailing mark", for: "another way in: the lock's fingerprint reader", sample: () => boxed({ icon: "lock", secret: true, placeholder: "Password", trailing: { icon: "fingerprint", color: "primary" } }) },
    { name: "Password", for: "one bullet per character, Show beside its label", sample: password },
    { name: "Labelled", for: "every field of a form", sample: () => fieldRoom(ROOM, labelled("Full name", field({ placeholder: "Your name" }))) },
    { name: "With a note", for: "what it accepts, what it's for", sample: () => hostname() },
    { name: "With an error note", for: "why it's invalid and what to do", sample: () => email() },
    {
      name: "Read-only",
      for: "can't change here; selects and copies; the note says where",
      sample: () => fieldRoom(ROOM, labelled("User name", field({ value: "kuba", icon: "lock", state: "Locked" }), { note: "Set when you installed; read-only here." })),
    },
    {
      name: "Disabled",
      for: "not usable now; the note says what turns it on",
      sample: () => fieldRoom(ROOM, labelled("Proxy", field({ value: "proxy.example.com:3128", icon: "lan", state: "Disabled" }), { note: "Turn the proxy on to set it." })),
    },
  ],
  sizes: doesntApply("one height, `size.control_md`, a regular button's; its width is its room's."),
  states: {
    shown: ["rest", "lit", "disabled", "error"],
    absent: {
      pressed: "a press puts the caret where it lands",
      selected: "selected text is a row, A word selected",
      loading: "what checks its value says so in its note",
      empty: "a value: the rows Empty and Placeholder",
    },
    rows: nonEmpty(ROWS.map((each) => ({ name: each.name, cell: (state: Shown) => cell(each, state) }))),
    looks: {
      rest: "`surface_container_high`, an `outline_variant` hairline",
      lit: "fill to `…_highest`, the tube and glow, the caret if typing",
      disabled: "grey: no line, no glow, no caret, no Tab stop",
      error: "`error` hairline; lit, tube, glow and caret in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "Normal", tone: "accent", filled: false, content: false },
      { name: "Invalid", tone: "error", filled: false, content: false },
    ],
    looks: {
      lit: "the tube over the hairline, the glow outside; the caret the core",
      signsOff: "`primary_glass` or `error_glass`, no glow; the caret is glass too",
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
    missingPicture: [{ name: "No icon", for: "the text starts `space.s12` in", sample: () => boxed({ value: "tuxedo" }) }],
    counts: [
      { name: "No note", for: "the label, then the field", sample: () => fieldRoom(ROOM, labelled("Full name", field({ placeholder: "Your name" }))) },
      { name: "One note", for: "at most one: its rule or its error", sample: () => hostname(shown(fixed("tuxedo"))) },
      {
        name: "Fields side by side",
        for: "every label row is `size.control_sm`, so they line up",
        sample: () => stack("s16", "start", password(), fieldRoom(ROOM, labelled("Full name", field({ value: "Shinji Ikari" })))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface or form."),
    leaves: doesntApply("it goes with its surface or form."),
    between: [
      {
        name: "Focus arrives",
        what: "A fade over `short` on `standard`, never the flicker: the fill rises, the tube and glow come in.",
        tracks: lights(".k-field", "a field and an invalid one", 0, 1),
        reduced: "same",
        back: "cuts",
        sample: pair(),
      },
      {
        name: "Focus leaves",
        what: "Back to rest over `short` on `standard`; the caret goes at once.",
        tracks: lights(".k-field", "a field and an invalid one", 1, 0),
        reduced: "same",
        back: "cuts",
        sample: pair("lit"),
      },
      {
        name: "Turns invalid",
        what: "The tone fades from the accent to the error's over `short` on `standard`; fixed, it plays back.",
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
      { keys: ["Tab", "Shift+Tab"], does: "in, or on; disabled is skipped, read-only is a stop" },
      { keys: ["← →"], does: "the caret a character; with a selection, to its end" },
      { keys: ["Ctrl+← →", "Alt+← →"], does: "the caret a word" },
      { keys: ["Shift+← →", "Shift+Home", "Shift+End"], does: "grows or shrinks the selection" },
      { keys: ["Home", "End"], does: "the caret to the value's start or end" },
      { keys: ["Ctrl+A"], does: "selects the whole value" },
      { keys: ["Ctrl+C"], does: "copies the selection; a password's never" },
      { keys: ["Backspace", "Delete"], does: "a character or the selection; with Ctrl, a word" },
      { keys: ["Space"], does: "a space, never a press" },
      { keys: ["Enter"], does: "done: the surface goes on, once per press" },
      { keys: ["↑ ↓", "Ctrl+N", "Ctrl+P", "Ctrl+J", "Ctrl+K"], does: "over a list, moves in it; the field keeps the keyboard" },
      { keys: ["Esc"], does: "nothing of the field's" },
    ],
    order: "One stop in reading order, its label and note none; a Labelled field's action is the next.",
    lands: "A form starts in its first field, the caret at its value's end.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Field", for: "the field and its text", sample: () => boxed({ value: "tuxedo" }) },
      { name: "With an icon", for: "the icon and the caret", sample: () => boxed({ value: "Shinj", icon: "search", pinned: "lit" }) },
      { name: "Labelled, with a note", for: "label row, field, note", sample: () => hostname(shown(fixed("tuxedo"))) },
    ]),
    measures: [
      { part: "Height", token: "size-control_md" },
      { part: "Corner", token: "radius-control" },
      { part: "Hairline", token: "stroke-hairline_device_px" },
      { part: "Value and placeholder line", token: "type-body_large-line" },
      { part: "Text and icon inset", token: "space-s12" },
      { part: "Icon", token: "type-body_large-icon" },
      { part: "Icon → text", token: "space-s8" },
      { part: "Caret width", token: "stroke-thin" },
      { part: "Caret height", token: "caret-height" },
      { part: "Caret's glow", token: "caret-near" },
      { part: "Selection corner", token: "radius-small" },
      { part: "Label row", token: "size-control_sm" },
      { part: "Label line", token: "type-label-line" },
      { part: "Under label, under field", token: "space-s4" },
      { part: "Note line", token: "type-small-line" },
      { part: "Tube", token: "focus-tube" },
      { part: "Near glow", token: "focus-near" },
      { part: "Far glow", token: "focus-far" },
    ],
    reflow: {
      note: "It fills its room up to `size.column`; a long value scrolls and its note wraps.",
      sample: () => fieldRoom("min(100%, var(--size-column))", labelled(shown(LABEL), field({ value: shown(VALUE), icon: "computer" }), { note: shown(NOTE) })),
    },
  },
});
