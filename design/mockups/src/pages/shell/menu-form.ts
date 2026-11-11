// A form View on the Command menu (`FormView.qml`): the fields an Extension asks for, each under its
// label, in a column centred in the body, and its submit button at the end. Tab or ↓ moves between
// fields, Enter submits from anywhere; a long form scrolls under the head.

import type { Words } from "../../lib/content.ts";
import type { KeyHint } from "../../lib/keycap.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { FIELDS, type FieldSpec, footerWords, formView, MANY_FIELDS, panel, titleHead } from "./menu-parts.ts";

type Shown = "rest" | "lit" | "selected" | "error";

const keysFor = (submit: string): readonly KeyHint[] => [
  ["Tab", "Next field"],
  ["↵", submit],
  ["Esc", "Back"],
];

type Screen = "full" | "laptop" | "narrow";

const form = (extra: { screen?: Screen; fields?: readonly FieldSpec[]; submit?: string; lit?: number; down?: number; title?: string; rows?: number; failure?: string } = {}) => {
  const submit = extra.submit ?? "Create";
  return panel({
    screen: extra.screen ?? "full",
    head: titleHead(extra.title ?? "New project"),
    body: formView(extra.fields ?? FIELDS, submit, extra.lit ?? 0, extra.down),
    hints: keysFor(submit),
    ...(extra.failure ? { footer: footerWords(extra.failure, true) } : {}),
    ...(extra.rows ? { rows: extra.rows } : {}),
  });
};

const few = { screen: "narrow" as const, rows: 5 };

const LABEL: Words = { short: "Name", long: "The name the project's folder and its repository take" };

export default page<Shown>({
  level: "shell",
  slug: "menu-form",
  title: "Command menu form",
  lede: "A form View: the fields an Extension asks for, under their labels, and its submit button.",
  code: { qml: "FormView" },
  from: nonEmpty(["list-detail", "field", "checkbox", "radio", "button", "scroll"]),
  stage: { live: () => form(), hint: "click a field; Tab and Shift+Tab move", daylight: true },
  variants: [
    { name: "Text, choice, checkbox", for: "every kind of field a View sends", sample: () => form() },
    { name: "Longer than the body", for: "it scrolls under the head", sample: () => form({ fields: MANY_FIELDS, submit: "Save", title: "" }) },
    { name: "Scrolled to the end", for: "the last field and the button in view", sample: () => form({ fields: MANY_FIELDS, submit: "Save", title: "", lit: 11, down: 1 }) },
  ],
  sizes: [
    { name: "Full HD", for: "the column `size.column` wide, centred", sample: () => form() },
    { name: "1280 × 800", for: "the same column; less room down", sample: () => form({ screen: "narrow" }) },
  ],
  states: {
    shown: ["rest", "lit", "selected", "error"],
    absent: {
      pressed: "its button presses on its own page",
      disabled: "a View's fields are never off",
      loading: "submitting closes the View or says why",
      empty: "a form with no fields isn't pushed",
    },
    rows: [
      {
        name: "The form",
        cell: (state) => {
          if (state === "rest") return form({ ...few, lit: -1 });
          if (state === "selected") return form({ ...few, lit: 2 });
          if (state === "error") return form({ ...few, failure: "A project named kubaos is already there" });
          return form(few);
        },
      },
    ],
    looks: {
      rest: "fields in their wells, the title in `subheading`",
      lit: "the field with the keyboard: its tube, its caret",
      selected: "a choice's radio lit on the option picked",
      error: "why it failed across the footer, in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "A text field", tone: "accent", filled: false, content: false },
      { name: "A checkbox, a radio", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the field's tube, or the box's", signsOff: "each tube as `primary_glass`, no glow" },
    signsOff: () => form(few),
  },
  content: {
    text: [
      { name: "A field's label", words: LABEL, sample: (words: string) => form({ fields: [{ ...FIELDS[0]!, label: words }, ...FIELDS.slice(1)] }) },
      { name: "The View's title", words: { short: "New project", long: "Start a new project from the template you picked" }, sample: (words: string) => form({ title: words }) },
    ],
    missingPicture: doesntApply("a form has no pictures."),
    counts: [
      { name: "One field", for: "the button under it", sample: () => form({ fields: FIELDS.slice(0, 1) }) },
      { name: "A few", for: "fits the body", sample: () => form() },
      { name: "Many", for: "scrolls; the thumb shows", sample: () => form({ fields: MANY_FIELDS, submit: "Save" }) },
    ],
  },
  motion: {
    arrives: doesntApply("pushed as every View is: Command menu Views."),
    leaves: doesntApply("popped as every View is: Command menu Views."),
    between: [
      { name: "The next field lights", what: "Tab: the field reached lights its tube, the one left fades out.", tracks: lights(".k-field:not([data-pinned])", "a field", 0, 1), reduced: "same", back: "plays", sample: () => form(few) },
      ...powerMoves(() => form(few)),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "↓"], does: "the next field, round from the last" },
      { keys: ["Shift+Tab", "↑"], does: "the field before" },
      { keys: ["Space"], does: "ticks a checkbox" },
      { keys: ["←", "→"], does: "steps a choice's options" },
      { keys: ["Enter"], does: "submits, from any field" },
      { keys: ["Esc"], does: "back to the View under it" },
    ],
    order: "The fields in order, round; the button takes only the pointer.",
    lands: "In the first field; a field reached scrolls into view with its label.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The form", for: "the column, its gaps, the button", sample: () => form() },
      { name: "Scrolled", for: "the fade and the scroll's room", sample: () => form({ fields: MANY_FIELDS, submit: "Save", down: 0.5 }) },
    ]),
    measures: [
      { part: "Column's most", token: "size-column" },
      { part: "Between fields", token: "space-s16" },
      { part: "Fields to the button", token: "space-s24" },
      { part: "Label's row", token: "size-control_sm" },
      { part: "A field", token: "size-control_md" },
      { part: "A choice's radios apart", token: "space-s12" },
      { part: "Scroll reaches past the body", token: "space-s12" },
    ],
    reflow: { note: "The column holds `size.column` and centres; a short screen scrolls sooner.", sample: () => form({ screen: "laptop" }) },
  },
});
