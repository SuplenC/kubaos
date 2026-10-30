// The Type foundation: every type style with real text, its families and weights, tabular figures,
// the scale in device pixels, and how a long word and a translation wrap (`docs/DESIGN.md`, "Type").

import { button, METRICS, SIZES } from "../../lib/button.ts";
import { translated } from "../../lib/content.ts";
import { icon } from "../../lib/icon.ts";
import { row, stack } from "../../lib/layout.ts";
import { type TypeStyle, text, typeset } from "../../lib/text.ts";
import { length, names, raw, words } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, table, token } from "../../site/parts/swatches.ts";
import { SCALES, settings } from "../../site/settings.ts";

/** Every type style, largest first, then by its line. */
function styles(): TypeStyle[] {
  const measure = (style: TypeStyle) => [length(`type-${style}-size`), length(`type-${style}-line`)] as const;
  return names("type-")
    .filter((name) => name.endsWith("-size"))
    .map((name) => name.slice("type-".length, -"-size".length) as TypeStyle)
    .sort((a, b) => measure(b)[0] - measure(a)[0] || measure(b)[1] - measure(a)[1]);
}

/** Real words for each style, as the surfaces that use it show them. */
const WORDS: Record<TypeStyle, string> = {
  clock: "21:47",
  hero: "Welcome",
  hero_number: "64%",
  display: "That's it",
  title: "Pick a disk",
  headline: "Erase the disk?",
  heading: "Network",
  subheading: "Couldn't connect",
  body_large: "Everything on this disk will be erased.",
  body: "KubaOS takes the whole disk and encrypts it.",
  body_strong: "Samsung 990 Pro",
  button_large: "Install",
  button: "Continue",
  button_small: "Show log",
  label: "Passphrase",
  label_number: "72%",
  small: "At least 12 characters",
  caption: "Version 0.4",
  accent: "ようこそ",
  mono: "/dev/nvme0n1p2",
  keycap: "Ctrl",
  mono_large: "kubaos update",
};

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

/** Every style: its measures by token, its family and weight, its icon, real words, and its use. */
function everyStyle(): HTMLElement {
  return table(
    ["Style", "Size / line", "Weight, family", "Icon", "Its words, its line box", "For"],
    styles().map((style) => [
      token(style),
      label(`${px(`type-${style}-size`)} / ${px(`type-${style}-line`)}`, "mono", "on_surface"),
      label(`${WEIGHTS[raw(`type-${style}-weight`)] ?? raw(`type-${style}-weight`)}, ${familyOf(style)}${raw(`type-${style}-features`) === "normal" ? "" : ", tabular"}`, "body", "on_surface"),
      label(px(`type-${style}-icon`), "mono", "on_surface"),
      line(style),
      prose(words(`type-${style}-use`)),
    ]),
  );
}

/** Each family, with every weight a style draws it in, each in the words of a style that does. */
function families(): HTMLElement {
  return table(
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
  );
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
  return table(
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
  );
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
  return table(
    ["Style", ...SCALES.map((scale) => `${scale}×`)],
    styles().map((style) => [
      token(style),
      ...SCALES.map((scale) => {
        const size = length(`type-${style}-size`) * scale;
        const line = length(`type-${style}-line`) * scale;
        return label(`${size} / ${line}`, "mono", Number.isInteger(line) ? "on_surface" : "error");
      }),
    ]),
  );
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

export default foundation({
  level: "foundations",
  slug: "type",
  title: "Type",
  summary: "A closed set of type styles, picked by what the text is: three families, eleven sizes, every line on the 4 grid, and an icon size for each.",
  live: () =>
    stack(
      "s8",
      "start",
      text(WORDS.accent, "accent", "primary"),
      text("Welcome to KubaOS", "title", "on_surface"),
      wrapping("Everything on this disk will be erased. KubaOS takes the whole disk and encrypts it.", "body_large", "on_surface_variant"),
      row("s8", "start", text(WORDS.label, "label", "on_surface_variant"), text(WORDS.mono, "mono", "on_surface_variant")),
    ),
  description: [
    "A piece of text takes a style by what it is (a page's title, a row's title, a field's note), never by how big it should look. A style is a size, a line height, a weight, a family and the icon that sits beside it; nothing else draws text.",
    "Geist is the interface, Geist Mono anything typed or read by a machine, and M PLUS 1 the Japanese accent above a title. The sizes are a closed list, and every line height is a multiple of 4, so a text box lands on whole device pixels at every scale.",
    "A number that changes in place takes tabular figures, every figure one width, so it never shifts as it ticks: `label_number`. iced has no font features, so its components hold such a number in a box as wide as its widest value instead.",
    "Text grows: a translation runs up to 40% longer, so text fits by wrapping or by the layout giving it room, never by a width chosen for the English.",
  ],
  tokens: [
    {
      name: "Every style",
      described: "Largest first, each in real words with its icon before them, on its line box: its size and line in logical pixels, its weight and family, the icon it takes, and what it is for, as the tokens say it.",
      sample: everyStyle,
    },
    {
      name: "Families and weights",
      described: "Each family in every weight a style draws it in, in that style's words, with the styles that do.",
      sample: families,
    },
  ],
  rules: [
    {
      name: "Every line is on the grid",
      described: "Each line box is a multiple of `grid` tall, so stacked lines land on the 4 grid drawn behind them, and a text's top is a whole device pixel at every scale. The icon is centred down the line on a whole device pixel and never makes it taller.",
      sample: onTheGrid,
    },
    {
      name: "A control's label is its content box",
      described: "A single-line control's label takes the control's content box as its line, so it centres with no arithmetic: the rest of the height is split above and below.",
      sample: controlLabels,
    },
    {
      name: "Tabular figures keep a number's width",
      described: "In `label` the figures are as wide as their shapes, so 11% is narrower than 48%; in `label_number` every figure is one width, and a column of numbers ends in one place. iced draws them proportional and holds the number in a box as wide as its widest value.",
      sample: figures,
    },
  ],
  modes: [
    {
      name: "The text roles in each mode",
      described: "Text is drawn in `on_surface`, `on_surface_variant` for what is secondary, `primary` for an action, `error` for a failure, each holding 4.5:1 on its surface (the colour page lists the pairs). The style is the same in both modes.",
      sample: () => el("div", "f-row", ...MODES.map(textRoles)),
    },
  ],
  themes: doesntApply("a type style takes no colour of its own: its roles follow the theme, which the colour page shows in every theme."),
  scales: [
    {
      name: "Sizes and lines in device pixels",
      described: "Every line is a whole number of device pixels at every scale, since it is a multiple of 4; a size may be fractional (13 at 1.25 is 16.25), which the font's own rendering takes. A line that isn't whole would be red.",
      sample: deviceLengths,
    },
    {
      name: "The same words at every scale",
      described: "Each box below is drawn at its own scale, whatever the toolbar's: every length in it is that scale's, in device pixels.",
      sample: () =>
        el("div", "f-row", ...SCALES.map((scale) => atScale(scale, stack("s4", "start", line("subheading"), line("body"), line("caption"))))),
    },
  ],
  content: [
    {
      name: "A translation 40% longer",
      described: "A title and its paragraph in a column as wide as a panel, in English and in a translation that runs 40% longer: the text wraps onto more lines and the column grows down, never sideways.",
      sample: translations,
    },
    {
      name: "A long word",
      described: "A word longer than its column breaks where it meets the edge, rather than running out of its box: in a title, and in running text.",
      sample: longWord,
    },
  ],
  motion: doesntApply("text doesn't move on its own: typed text and the caret appear at once (`instant`), and text moves only with what it is in."),
  checks: [
    { check: "kubaos-tokens", holds: "Refuses a size that isn't in `rules.font_sizes`, a line height off the grid, and an icon taller than its style's line." },
    { check: "kubaos-design lint", holds: "Fails on a font size, weight or family written out in QML or Rust UI code: text takes a style." },
    { check: "The geometry check", holds: "Holds every text box to whole device pixels and to its style's line height, at every scale, in the specimens and the mockups." },
  ],
});
