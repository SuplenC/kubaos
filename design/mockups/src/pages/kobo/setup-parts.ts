// A first-boot card's page in the setup window's step rail, as an organism of library parts: its
// header (what it is for), where it stands (the engine's summary beside its state), what is
// happening right now (a prompt, a failure, a result), what it asks, what skipping it costs and
// where it is later, and Under the hood (what detection found, the command, the log).

import { banner } from "../../lib/banner.ts";
import { button } from "../../lib/button.ts";
import { hangs } from "../../lib/button-bar.ts";
import { card } from "../../lib/card.ts";
import { checkbox, radio, radioGroup } from "../../lib/check.ts";
import { chip } from "../../lib/chip.ts";
import { collapsible } from "../../lib/collapsible.ts";
import { descriptionList } from "../../lib/description.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { field, labelled } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { keyHints } from "../../lib/keycap.ts";
import { fill, filling, grid, row, stack } from "../../lib/layout.ts";
import { log } from "../../lib/log.ts";
import { progressBar } from "../../lib/progress.ts";
import { rollingNumber } from "../../lib/rolling-number.ts";
import { select } from "../../lib/select.ts";
import { spinner } from "../../lib/spinner.ts";
import type { RailPage, RailStatus } from "../../lib/step-rail.ts";
import { text, wrapped } from "../../lib/text.ts";
import { body, showToggle } from "./kobo-parts.ts";
import { type Ask, type Card, type CardId, CARDS, open, type Prompt, PROMPT_WHERE, running, type View, type Views } from "./setup-cards.ts";

/** The symbol a prompt shows big, beside its words. */
const PROMPT_GLYPH: Record<Prompt["kind"], string> = { password: "lock", finger: "fingerprint", key: "key", wait: "schedule", plug: "key", phone: "smartphone", pair: "smartphone" };

/** The fingers fprintd knows, the next one to enrol first. */
const FINGERS = ["Right index", "Right middle", "Left index", "Left middle", "Right thumb", "Left thumb"];

/** The vault's folders a person can take SSH keys from; `env/` ones hold project env, not keys. */
const FOLDERS = ["ssh", "work", "old laptop"];

function capital(words: string): string {
  return words.charAt(0).toUpperCase() + words.slice(1);
}

/** A card's state as its chip says it: to do plain, done tonal with a check, a reboot on the accent. */
export function stateChip(view: View, skipped = false): HTMLElement {
  if (running(view)) return chip({ label: "Running", kind: "Neutral", icon: "progress_activity" });
  // Something asks for the person now: it is the next thing to do, so it carries the accent.
  if (view.wants) return chip({ label: view.wants.label, kind: "Filled", icon: view.wants.glyph });
  switch (view.state) {
    case "done":
      return chip({ label: "Done", kind: "Accent", icon: "check" });
    case "reboot":
      return chip({ label: "Reboot", kind: "Filled", icon: "restart_alt" });
    case "waiting":
      return chip({ label: "Waiting", kind: "Neutral", icon: "schedule" });
    case "absent":
      return chip({ label: "Not here", kind: "Neutral" });
    default:
      return chip({ label: skipped ? "Later" : "To do", kind: "Neutral" });
  }
}

/** Where the card stands: its state's chip, then the engine's summary in its own slot. */
function statusLine(view: View): HTMLElement {
  const line = row("s12", "start", stateChip(view), filling(wrapped(view.found ?? view.summary, "body", "on_surface_variant")));
  line.classList.add("setup-status");
  return line;
}

/**
 * What the person has to do right now, big: the symbol (humming while it waits on a hand or a
 * device, a spinner while the machine works), the words, the touches counted, and where to do it.
 * While touches are counted, the words keep two lines' room, so the next touch's words move nothing
 * under them; a prompt whose words never change takes only the lines it has.
 */
export function promptCard(prompt: Prompt): HTMLElement {
  // The hum is the symbol's box's, not the drawing's: a box's opacity moves on the compositor, an
  // SVG's restyles the page every frame.
  const hum = document.createElement("span");
  hum.className = "setup-hum";
  hum.append(icon(PROMPT_GLYPH[prompt.kind], "var(--icon-s48)", "var(--color-primary)"));
  // A request waits on a decision, not on a hand or a device: its symbol stays still.
  const lead = prompt.kind === "wait" ? spinner("s48") : prompt.kind === "pair" ? icon(PROMPT_GLYPH.pair, "var(--icon-s48)", "var(--color-primary)") : hum;
  const heading = wrapped(prompt.text, "heading");
  heading.classList.add("setup-prompt-words");
  heading.toggleAttribute("data-counted", prompt.count !== undefined);
  const words = stack("s8", "start", filling(heading));
  if (prompt.count) {
    const [done, of] = prompt.count;
    const count = rollingNumber({ value: done, widest: "0", style: "small", color: "on_surface_variant" });
    words.append(filling(stack("s4", "start", filling(progressBar({ fraction: done / of })), row(0, "centre", count.element, text(` of ${of}`, "small", "on_surface_variant")))));
  }
  if (prompt.key) {
    const key = text(prompt.key, "mono_large");
    words.append(key);
  }
  words.append(wrapped(PROMPT_WHERE[prompt.kind], "small", "on_surface_variant"));
  const shown = card({ body: [row("s20", "centre", lead, filling(words))] });
  shown.classList.add("setup-prompt");
  return shown;
}

/** A labelled field the person types in, its note always there so a row of them lines up. */
function input(label: string, note: string, options: Parameters<typeof field>[0] = {}, invalid = false, action?: HTMLElement): HTMLElement {
  return labelled(label, field(options), { note, ...(invalid ? { invalid: true } : {}), ...(action ? { action } : {}) });
}

/** The buttons a card offers in its body, beside what they act on: the quiet ones last. */
function buttons(labels: readonly string[]): HTMLElement {
  const quiet = new Set(["Log out", "Turn the phone link off"]);
  return hangs(row("s12", "centre", ...labels.map((label) => button({ label, kind: quiet.has(label) ? "Text" : "Tonal" }))));
}

/** The vault's folders as checkboxes, none ticked for the whole vault, behind their summary. */
function folders(): HTMLElement {
  const boxes = row("s20", "centre", ...FOLDERS.map((label) => checkbox({ label })));
  const content = stack("s12", "start", wrapped("The whole vault. Tick folders to take keys from those only; Pull again to apply.", "small", "on_surface_variant"), boxes);
  return hangs(collapsible({ title: "SSH keys from", summary: "The whole vault", content }));
}

/** Bitwarden's login: the server, the account and the code upfront; another way in behind it. */
function bitwardenForm(ask: Extract<Ask, { is: "bitwarden" }>): HTMLElement {
  const servers = ["bitwarden.eu", "bitwarden.com", "Self-hosted"].map((label, index) => radio({ label, checked: index === ask.server }));
  const parts: Element[] = [labelled("Server", radioGroup("s20", ...servers))];
  if (ask.server === 2) parts.push(input("Server address", "Your server's https:// address.", { value: "https://vault.nowak.dev", icon: "public" }));
  const password = (show: boolean) => input("Master password", "In memory, never on disk.", { value: "correct horse battery staple", secret: true, icon: "key" }, false, show ? showToggle(() => []) : undefined);
  // The whole login on one row: the account, its password and its code read as one question.
  if (ask.method === "apikey") {
    parts.push(grid(3, "s20", [input("client_id", "Web vault: Security, Keys.", { value: "user.6f1c…" }), input("client_secret", "Beside client_id.", { value: "8Hq2kX9v", secret: true }), password(true)]));
  } else {
    const code = ask.missing === "code" ? {} : { value: "482 913" };
    parts.push(grid(3, "s20", [input("Email", "Saved, with the server.", { value: "mika@nowak.dev", icon: "mail" }), password(true), input("Two-step code", "From your authenticator app.", { placeholder: "123456", ...code })]));
  }
  const methods = ["Authenticator app", "YubiKey OTP", "API key"].map((label, index) => radio({ label, checked: index === (ask.method === "apikey" ? 2 : 0) }));
  const way = stack("s12", "start", radioGroup("s20", ...methods), wrapped("bw can't use the YubiKey Bio for two-step login: use the authenticator app's code, or the API key for an account with no code.", "small", "on_surface_variant"));
  parts.push(hangs(collapsible({ title: "Another way to log in", summary: ask.method === "apikey" ? "API key" : "Authenticator app", content: way, open: ask.method === "apikey" })));
  return stack("s20", "start", ...parts.map((each) => filling(each as HTMLElement)));
}

/** The SSH keys the last pull found, read-only: what each became, and what serves them. */
function keysCard(): HTMLElement {
  return card({
    header: { glyph: "key", title: "SSH keys" },
    body: [
      descriptionList([
        { label: "github (ssh)", value: "Host block for github.com; signs your commits" },
        { label: "hetzner (ssh)", value: "Host block for *.nowak.dev" },
        { label: "The agent", value: "Serves 2 keys" },
      ]),
    ],
  });
}

/** One of the things a card asks, drawn. */
function drawAsk(ask: Ask): HTMLElement {
  switch (ask.is) {
    case "para":
      return wrapped(ask.words, "body", "on_surface_variant");
    case "banner":
      return banner({ tone: ask.tone, icon: ask.icon, title: ask.title, body: ask.lines[0] ?? "", paragraphs: ask.lines.slice(1) });
    case "notice":
      return card({ header: { glyph: ask.icon, title: ask.title }, body: ask.lines.map((line) => wrapped(line, "body", "on_surface_variant")) });
    case "mok": {
      const first = input("One-time password", "8 to 16 letters and digits: it assumes a US keyboard.", { value: "kobo2026", secret: true, icon: "key" });
      // Show reveals both, so it ends the pair, over the second field, as the installer's does.
      const again = input("Same again", ask.mismatch ? "The two passwords don't match." : "Typed twice: a typo here costs a reboot.", { value: ask.mismatch ? "kobo2062" : "kobo2026", secret: true, icon: "key", ...(ask.mismatch ? { state: "Invalid" as const } : {}) }, ask.mismatch, showToggle(() => []));
      return grid(2, "s20", [first, again]);
    }
    case "finger":
      return grid(2, "s20", [labelled("Finger", select({ options: FINGERS, picked: 0, icon: "fingerprint" }), { note: "Any finger works; enrol a second one later." })]);
    case "fingers":
      return hangs(row("s8", "centre", ...ask.names.map((name) => chip({ label: name, kind: "Neutral", icon: "fingerprint" })), fill(), button({ label: "Add another finger", kind: "Tonal" })));
    case "bitwarden":
      return bitwardenForm(ask);
    case "unlock":
      return stack("s20", "start", filling(grid(2, "s20", [input("Master password", "Logged in already: this only unlocks the vault.", { value: "correct horse battery staple", secret: true, icon: "key" }, false, showToggle(() => []))])), filling(folders()));
    case "keys":
      return stack("s20", "start", filling(keysCard()), filling(folders()));
    case "git":
      return grid(2, "s20", [
        input("Your name", "From your account; the way commits show it.", { value: "Mika Nowak", icon: "person" }),
        input("Email", ask.email === "mika@nowak" ? "That doesn't look like an email." : "Public on every commit. GitHub's noreply address works too.", { value: ask.email, icon: "mail", ...(ask.email === "mika@nowak" ? { state: "Invalid" as const } : {}) }, ask.email === "mika@nowak"),
      ]);
    case "buttons":
      return buttons(ask.labels);
  }
}

/** What skipping costs, and the Command menu's Item that does it later. */
function laterLine(spec: Card): HTMLElement {
  const line = row("s12", "centre", wrapped(spec.skip, "small", "on_surface_variant"), keyHints([["Super+Space", `then “${spec.later}”`]]));
  line.classList.add("setup-later");
  return line;
}

/** Under the hood: what detection found, the command that does the same, and the log. */
function hood(spec: Card, name: string, view: View, opened: boolean): HTMLElement {
  const facts = spec.facts(name);
  const parts: HTMLElement[] = [descriptionList(facts), stack("s8", "start", text("The same in a terminal", "label", "on_surface_variant"), wrapped(spec.command, "mono"))];
  if (view.log) parts.push(log(view.log).box);
  const first = facts[0];
  const summary = view.log ? `${view.log.length} lines of log` : first ? `${first.label}: ${first.value}` : "";
  return hangs(collapsible({ title: "Under the hood", summary, content: stack("s16", "start", ...parts.map(filling)), open: opened || view.failed !== undefined }));
}

/** A card's page body at `name`: everything but its footer band, which the rail page draws. */
export function cardBody(id: CardId, name: string, opened = false): HTMLElement {
  const spec = CARDS[id];
  const view = spec.views[name]!;
  const parts: HTMLElement[] = [statusLine(view)];
  if (view.prompt) parts.push(promptCard(view.prompt));
  if (view.failed) parts.push(errorBlock({ message: `Fuck. ${capital(view.failed.error)}.`, hint: view.failed.hint }));
  if (view.result) parts.push(banner({ tone: "Info", icon: "check_circle", title: "Done.", body: view.result }));
  // The form hides while its card runs: nothing in it can change what is already running.
  if (!running(view)) parts.push(...spec.asks(name).map(drawAsk));
  if (open(view) || view.state === "reboot") parts.push(laterLine(spec));
  parts.push(hood(spec, name, view, opened));
  return body(spec.accent, spec.title, spec.why, stack("s20", "start", ...parts.map(filling)));
}

/** A card's step in the rail: its state as a mark, its other words' symbol muted. */
export function railStatus(id: CardId, name: string, skipped = false): RailStatus {
  const view = CARDS[id].views[name]!;
  const going = running(view);
  const failed = view.failed !== undefined;
  if (view.wants) return { state: { is: "action", label: view.wants.label, glyph: view.wants.glyph } };
  switch (view.state) {
    case "done":
      return { state: { is: "done" } };
    case "reboot":
      return { state: { is: "action", label: "Reboot", glyph: "restart_alt" } };
    case "absent":
      return { state: { is: "todo" }, words: { label: "Not here", glyph: "remove" }, out: true };
    case "waiting":
      return { state: { is: "todo" }, words: { label: "Waiting", glyph: "schedule" }, ...(going ? { running: true } : {}) };
    default:
      return { state: { is: "todo" }, ...(skipped ? { words: { label: "Later", glyph: "schedule" } } : {}), ...(going ? { running: true } : {}), ...(failed ? { failed: true } : {}) };
  }
}

/** What a card's footer band shows at `name`: its work, Skip while it is open, what blocks it. */
export function cardPage(id: CardId, views: () => Views, draw: () => HTMLElement, act: () => boolean, skipped: () => boolean = () => false): RailPage {
  const spec = CARDS[id];
  const at = () => views()[id];
  const view = () => spec.views[at()]!;
  return {
    step: { title: spec.title, icon: spec.icon },
    body: draw,
    status: () => railStatus(id, at(), skipped()),
    next: () => spec.action(at())?.label ?? "Continue",
    nextIcon: () => spec.action(at())?.icon,
    skip: () => (spec.action(at()) && (open(view()) || view().state === "reboot") ? (view().state === "reboot" ? "Later" : "Skip") : undefined),
    // While its work runs nothing is wrong, so nothing blocks: the band says it is working.
    blocker: () => (running(view()) ? undefined : spec.blocker?.(at())),
    busy: () => (running(view()) ? "Running…" : undefined),
    act,
  };
}
