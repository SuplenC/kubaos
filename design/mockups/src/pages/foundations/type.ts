// The Type foundation: every type style with real text, its families and weights, tabular figures,
// the scale in device pixels, and how a long word and a translation wrap (`docs/DESIGN.md`, "Type").

import { button, METRICS, SIZES } from "../../lib/button.ts";
import { shown, translated, type Words } from "../../lib/content.ts";
import { icon } from "../../lib/icon.ts";
import { row, stack } from "../../lib/layout.ts";
import { STYLE_WORDS as WORDS, styles, type TypeStyle, text, typeset } from "../../lib/text.ts";
import { length, names, raw, words } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, table, token } from "../../site/parts/swatches.ts";
import { SCALES, settings } from "../../site/settings.ts";
import { across } from "./across.ts";

/** A weight's name, as the tokens write it. */
const WEIGHTS: Record<string, string> = { "300": "light", "400": "regular", "500": "medium", "600": "semibold" };

/** A style's family, as the tokens' roles name it: sans, mono or jp. */
function familyOf(style: TypeStyle): string {
  const family = raw(`type-${style}-family`);
  return names("font-").find((name) => raw(name) === family)?.slice("font-".length) ?? family;
}

/** A length token in logical pixels, as written. */
function px(name: string): string {
  return String(length(name));
}

/** The style's symbol in a box as wide as itself and its line tall, centred down it: as a component sets it beside its words. */
function iconBeside(style: TypeStyle): HTMLElement {
  const slot = el("span", "k-icon-line");
  slot.style.setProperty("--line", `var(--type-${style}-line)`);
  slot.style.setProperty("--icon-size", `var(--type-${style}-icon)`);
  slot.append(icon("info", `var(--type-${style}-icon)`, "var(--color-on_surface_variant)"));
  return slot;
}

/** `style`'s words with its icon before them, its line box tinted so its height shows. */
function line(style: TypeStyle, words = WORDS[style]): HTMLElement {
  return el("div", "f-line", row("s8", "start", iconBeside(style), text(words, style, "on_surface")));
}

/**
 * Every style: its measures by token, its family and weight, its icon, real words, and its use. The
 * rest clock's digits are wider than the words' column can be beside the others, so they take a row
 * of their own across the table, under the clock's.
 */
function everyStyle(): HTMLElement {
  const drawn = across(table(
    ["Style", "Size / line", "Weight, family", "Icon", "Its words, its line box", "For"],
    styles().map((style) => [
      token(style),
      label(`${px(`type-${style}-size`)} / ${px(`type-${style}-line`)}`, "mono", "on_surface"),
      label(`${WEIGHTS[raw(`type-${style}-weight`)] ?? raw(`type-${style}-weight`)}, ${familyOf(style)}${raw(`type-${style}-features`) === "normal" ? "" : ", tabular"}`, "body", "on_surface"),
      label(px(`type-${style}-icon`), "mono", "on_surface"),
      style === "clock_rest" ? "" : line(style),
      prose(words(`type-${style}-use`)),
    ]),
  ));
  const digits = el("td", "", line("clock_rest"));
  digits.colSpan = 6;
  drawn.querySelector("tbody > tr")?.after(el("tr", "", digits));
  return drawn;
}

/** Each family, with every weight a style draws it in, each in the words of a style that does. */
function families(): HTMLElement {
  return across(table(
    ["Family", "Weight", "Drawn", "By"],
    names("font-").flatMap((name) => {
      const family = name.slice("font-".length);
      const using = styles().filter((style) => familyOf(style) === family);
      const weights = [...new Set(using.map((style) => raw(`type-${style}-weight`)))].sort();
      return weights.map((weight) => {
        const by = using.filter((style) => raw(`type-${style}-weight`) === weight);
        return [
          token(`${family} = ${raw(name)}`),
          label(`${WEIGHTS[weight] ?? weight} ${weight}`, "body", "on_surface"),
          text(WORDS[by[0]!], by[0]!, "on_surface"),
          el("div", "f-swatch-cell", ...by.map((style) => token(style))),
        ];
      });
    }),
  ));
}

/** Text that wraps in the room it has, in `style`, as the page's own words do but in the role picked. */
function wrapping(words: string, style: TypeStyle, color = "on_surface"): HTMLElement {
  const element = el("p", "f-wrap", words);
  typeset(element, style);
  element.style.color = `var(--color-${color})`;
  return element;
}

/** Several styles' line boxes stacked on the 4 grid, the grid drawn behind them. */
function onTheGrid(): HTMLElement {
  const shown: TypeStyle[] = ["headline", "heading", "body_strong", "body", "small", "caption"];
  return el("div", "f-grid-paper", stack(0, "start", ...shown.map((style) => line(style))));
}

/** Each button size: its height, the label's line inside it, and what is above and below. */
function controlLabels(): HTMLElement {
  return across(table(
    ["Size", "Button", "Height = above + line + below"],
    SIZES.map((size) => {
      const metrics = METRICS[size];
      const height = length(`size-${metrics.height}`);
      const line = length(`type-${metrics.label}-line`);
      return [
        label(size.toLowerCase(), "body_strong", "on_surface"),
        button({ label: WORDS.button, kind: "Filled", size }),
        token(`size.${metrics.height} ${height} = ${(height - line) / 2} + type.${metrics.label}.line ${line} + ${(height - line) / 2}`),
      ];
    }),
  ));
}

/** The same numbers in a style without tabular figures and in one with them. */
function figures(): HTMLElement {
  const numbers = ["11%", "48%", "100%", "7%"];
  const column = (style: TypeStyle) =>
    el(
      "div",
      "f-figures",
      label(style, "label"),
      ...numbers.map((number) => {
        const shown = wrapping(number, style);
        shown.dataset.figures = "";
        return shown;
      }),
    );
  return el("div", "f-row", column("label"), column("label_number"));
}

/** Each style's size and line in device pixels at every scale. */
function deviceLengths(): HTMLElement {
  return across(table(
    ["Style", ...SCALES.map((scale) => `${scale}×`)],
    styles().map((style) => [
      token(style),
      ...SCALES.map((scale) => {
        const size = length(`type-${style}-size`) * scale;
        const line = length(`type-${style}-line`) * scale;
        return label(`${size} / ${line}`, "mono", Number.isInteger(line) ? "on_surface" : "error");
      }),
    ]),
  ));
}

/** A title and a paragraph in a panel's width, English beside its translation. */
function translations(): HTMLElement {
  const heading = "Choose where to install";
  const body = "KubaOS takes the whole disk and encrypts it. Everything on it now will be erased.";
  const column = (name: string, transform: (english: string) => string) =>
    el(
      "div",
      "f-swatch-cell",
      label(name, "label"),
      el("div", "f-measure", wrapping(transform(heading), "title"), wrapping(transform(body), "body", "on_surface_variant")),
    );
  return el("div", "f-row", column("English", (words) => words), column("Translated, 40% longer", translated));
}

/** A long word in a narrow column: where it breaks. */
function longWord(): HTMLElement {
  const words = "Datenträgerverschlüsselungseinstellungen";
  return el(
    "div",
    "f-row",
    el("div", "f-swatch-cell", label("A title", "label"), el("div", "f-narrow", wrapping(words, "title"))),
    el("div", "f-swatch-cell", label("Running text", "label"), el("div", "f-narrow", wrapping(`Open ${words} to pick another key.`, "body"))),
  );
}

/** The text roles on their surface, in `mode`. */
function textRoles(mode: "dark" | "light"): HTMLElement {
  return el(
    "div",
    "f-swatch-cell",
    label(modeName(mode), "label"),
    painted(
      settings().theme,
      mode,
      stack(
        "s8",
        "start",
        text("Couldn't connect", "subheading", "on_surface"),
        text("The network asked for a password.", "body", "on_surface_variant"),
        text("Wrong password", "body_strong", "error"),
        text("Show the log", "button", "primary"),
        text("/var/log/kubaos/install.log", "mono", "on_surface_variant"),
      ),
    ),
  );
}

const WELCOME: Words = { short: "Welcome to KubaOS", long: "Welcome to KubaOS, the whole disk encrypted and yours alone" };
const ERASED: Words = { short: "Everything on this disk will be erased.", long: "Everything on this disk will be erased: KubaOS takes the whole of it and encrypts it with your passphrase and your YubiKey." };

/** The styles at work: a page's opening beside a column of the styles under it, each on its line box. */
function specimen(): HTMLElement {
  const under: TypeStyle[] = ["heading", "subheading", "body_strong", "body", "small", "label", "mono"];
  return el(
    "div",
    "f-row",
    el("div", "f-measure", text(WORDS.accent, "accent", "primary"), wrapping(shown(WELCOME), "title"), wrapping(shown(ERASED), "body_large", "on_surface_variant")),
    el("div", "f-measure", ...under.map((style) => line(style))),
  );
}

export default foundation({
  level: "foundations",
  slug: "type",
  title: "Type",
  lede: "A closed set of type styles, picked by what the text is: three families, every line on the 4 grid.",
  stage: { live: specimen, hint: "pick Text long: it wraps and grows down", daylight: true },
  tokens: [
    { name: "Every style", note: "Largest first, in real words on their line box; size and line in logical pixels.", sample: everyStyle },
    { name: "Families and weights", note: "Each family in every weight a style draws it in.", sample: families },
  ],
  rules: [
    { name: "Lines land on the grid", note: "Each line box is a multiple of `grid`; its icon never makes it taller.", sample: onTheGrid },
    { name: "A label is its content box", note: "The label's line fills the control's content box; the rest splits above and below.", sample: controlLabels },
    { name: "Tabular figures", note: "`label_number` gives every figure one width; iced, with no font features, boxes the widest value instead.", sample: figures },
  ],
  modes: [{ name: "The text roles", note: "Each role holds 4.5:1 on its surface; the style doesn't change.", sample: () => el("div", "f-row", ...MODES.map(textRoles)) }],
  themes: doesntApply("a style's roles follow the theme; the Colour page shows them."),
  scales: [
    { name: "In device pixels", note: "Every line is whole; a size may be fractional; a line in red isn't whole.", sample: deviceLengths },
    {
      name: "The same words",
      note: "Each box at its own scale, whatever the top bar picks.",
      sample: () => el("div", "f-row", ...SCALES.map((scale) => atScale(scale, stack("s4", "start", line("subheading"), line("body"), line("caption"))))),
    },
  ],
  content: [
    { name: "A translation 40% longer", note: "It wraps and grows down, never sideways.", sample: translations },
    { name: "A long word", note: "It breaks at the edge rather than running out of its box.", sample: longWord },
  ],
  motion: doesntApply("text moves only with what it is in; typed text and the caret are `instant`."),
  checks: [
    { check: "kubaos-tokens", holds: "refuses off-list sizes, off-grid lines, icons taller than their line" },
    { check: "kubaos-design lint", holds: "fails a font size, weight or family in UI code" },
    { check: "The geometry check", holds: "holds every text box to whole device pixels and its line" },
  ],
});
