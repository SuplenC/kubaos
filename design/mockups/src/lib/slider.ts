// A value from 0 to 1, or one of a few stops, as a pill: `kubaos-ui`'s `slider::slider`,
// `KubaOS.Components`' `Slider` (`docs/design/components/slider.md`). The fill is the value: its
// round end marks it, the value being that end circle's centre, so at 0 the fill is the circle
// behind the icon and at 1 it reaches the far end. The icon sits in that circle, on the fill; the
// value, or a name for it, sits `space.s12` in from the far end, in the track's colours over the
// track and the fill's over the fill. Regular (`size.slider`) or compact (`size.slider_compact`);
// a compact slider shows a number only lit or held, and a name always.
//
// A continuous slider may have a toggle, which makes its icon a button: a second state, Toggled
// (volume muted, brightness on Auto), the icon turns on and off. Toggled, the fill fades to the
// softer accent (`secondary`, its content `on_secondary`) over `short`, the icon is the toggled
// one and the value reads as a name; the fill stays where the value is (a muted volume's level,
// the sensor's on Auto). Moving the value (a drag, a key, the wheel) leaves Toggled from where it
// is. On a slider with stops the icon steps through them instead, its name the stop it goes to.
//
// Steps show as dots on both sides of the fill, each in its side's colour; the mark the value rests
// on, a stop or a sticky point it holds at, turns solid and pops in. No dot is drawn under the icon
// or the label, `space.s4` clear of each.
//
// It is a control in the state model (`.k-control` in `lib.css`): lit under the pointer or with the
// keyboard on it, its fill brightening by `opacity.press` toward the palette's bright end, the tube
// inside its edge with the glow outside; held (pressed or dragged), the fill brightens by
// `opacity.held`. Its icon's button lights as an icon button does on the fill, and while it is lit
// the slider isn't; Space presses it, as a click does. It owns its value here, as the app does in both libraries: the pointer, the
// keys and the horizontal wheel move it, and it draws itself again.

import { settings } from "../site/settings.ts";
import { pin, type Pinned, sideways } from "./control.ts";
import { focusRing } from "./focus.ts";
import { withHint } from "./hint.ts";
import { icon } from "./icon.ts";
import { probe, variant } from "./probe.ts";
import { advance, shaped, text, type TypeStyle, typeset } from "./text.ts";
import { length, raw } from "./tokens.ts";

export const SLIDER_SIZES = ["Regular", "Compact"] as const;
export type SliderSize = (typeof SLIDER_SIZES)[number];

/** A step of a continuous slider from an arrow key or a wheel's notch: a twentieth. */
const NUDGE = 0.05;

// The locale's percentage: a value is never put together from a number and a sign by hand.
const PERCENT = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 0 });

export interface SliderOptions {
  readonly icon: string;
  readonly size?: SliderSize;
  /** Where it stands, 0 to 1, on a continuous slider. */
  readonly value?: number;
  /** How many stops it snaps to, two or more; none for a continuous slider. */
  readonly stops?: number;
  /** The stop it is on, from 0. */
  readonly index?: number;
  /** Each stop's name, the one nearest the value shown. */
  readonly names?: readonly string[];
  /** The number shown in place of the percentage, as the owner reads the value. */
  readonly number?: (value: number) => string;
  /** The widest the number reads, so the number ends in one place: "100%" unless given. */
  readonly widest?: string;
  /** A sticky point, a fraction along the track, the value holds at within `size.slider_hold`. */
  readonly sticky?: number;
  /** The toggle its icon turns on and off, on a continuous slider; with none, its icon is no button. */
  readonly toggle?: SliderToggle;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
  /** A state the icon's button is pinned in, where the icon is one. */
  readonly pinnedIcon?: Pinned;
  /** Held, as a press or a drag holds it: the specimens' view of the state. */
  readonly held?: boolean;
  /** Drawn still, as a player's sample is: no pop, nothing eased. */
  readonly still?: boolean;
}

/**
 * A continuous slider's second state, Toggled, which its icon turns on and off: a volume muted, a
 * brightness on Auto. Its labels name what a press does, in the icon's hint and as its name.
 */
export interface SliderToggle {
  /** The icon while toggled: `volume_off`, the Auto symbol. */
  readonly icon: string;
  /** The value's name while toggled, always shown: "Muted", "Auto · 55%". */
  readonly words: string;
  /** What a press does while it isn't toggled: "Mute". */
  readonly label: string;
  /** What a press does while it is: "Unmute". */
  readonly toggledLabel: string;
  /** Whether it starts toggled. */
  readonly on?: boolean;
  /** Told each time Toggled turns on or off, by the icon or by the value moving. */
  readonly changed?: (on: boolean) => void;
}

/** The duration token `name` in milliseconds under the speed setting, none with reduce motion. */
function motion(name: string): number {
  return settings().reduced ? 0 : Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The scale `part` is drawn at: the page's, or that of a box drawn at a scale of its own. */
function scaleOf(part: HTMLElement): number {
  return Number.parseFloat(getComputedStyle(part).getPropertyValue("--scale")) || settings().scale;
}

export function slider(options: SliderOptions): HTMLElement {
  const compact = options.size === "Compact";
  const stops = options.stops !== undefined && options.stops > 1 ? options.stops : 0;
  const last = stops - 1;
  const stepping = stops > 0;
  const sticky = options.sticky;
  let index = Math.max(0, Math.min(last, options.index ?? 0));
  let value = Math.max(0, Math.min(1, options.value ?? 0));
  const toggle = stepping ? undefined : options.toggle;
  let toggled = toggle?.on ?? false;
  let direction: -1 | 1 = -1;
  let dragging = false;
  let shown = stepping ? index / last : value;

  const root = document.createElement("div");
  root.className = "k-slider k-control";
  root.tabIndex = options.disabled ? -1 : 0;
  root.setAttribute("role", "slider");
  root.setAttribute("aria-valuemin", "0");
  root.setAttribute("aria-valuemax", stepping ? String(last) : "100");
  root.toggleAttribute("data-compact", compact);
  root.toggleAttribute("data-held", options.held ?? false);
  root.toggleAttribute("data-still", options.still ?? false);
  if (options.disabled) root.setAttribute("aria-disabled", "true");
  pin(root, options.pinned);

  const track = document.createElement("span");
  track.className = "k-slider-track";
  const fill = document.createElement("span");
  fill.className = "k-slider-fill";
  const onFill = document.createElement("span");
  onFill.className = "k-slider-on-fill";
  fill.append(onFill);
  const style = (): TypeStyle => (numeric() ? "label_number" : "label");
  const underTrack = text("", "label", "on_surface");
  underTrack.classList.add("k-slider-label");
  underTrack.style.color = "var(--on-track)";
  const overFill = text("", "label", "on_primary");
  overFill.classList.add("k-slider-label");
  overFill.style.color = "var(--on-fill)";
  const glyph = document.createElement("span");
  glyph.className = "k-slider-icon";
  onFill.append(overFill);
  const dots = document.createElement("span");
  dots.className = "k-slider-dots";
  const points = stepping ? Array.from({ length: stops }, (_, i) => i / last) : sticky !== undefined ? [sticky] : [];
  const dotOf = points.map((point) => {
    const dot = document.createElement("span");
    dot.className = "k-slider-dot";
    dot.style.setProperty("--at", String(point));
    dots.append(dot);
    return dot;
  });
  const anchor = document.createElement("span");
  anchor.className = "k-slider-anchor";
  root.append(track, underTrack, fill, dots, anchor);

  const button = !options.disabled && (stepping || toggle) ? iconButton() : undefined;
  // The icon is drawn in its button where it has one, so the button's circle lies under it.
  (button ?? onFill).prepend(glyph);
  focusRing(root, "full", "accent", false).dataset.pressable = "";

  /**
   * The icon's button: the circle at the fill's start, no stop for Tab of its own. It lights as an
   * icon button does on the fill, with its hint, from the slider's start, naming what a press does.
   */
  function iconButton(): HTMLButtonElement {
    const made = document.createElement("button");
    made.type = "button";
    made.className = "k-slider-icon-button k-control";
    made.tabIndex = -1;
    pin(made, options.pinnedIcon);
    focusRing(made, "full", "accent", false).dataset.pressable = "";
    made.addEventListener("pointerdown", (event) => event.stopPropagation());
    made.addEventListener("click", () => pressIcon());
    root.append(made);
    return made;
  }

  /** Draws what Toggled changes: the fill's tone, the icon, and the button's name and hint. */
  function drawToggle(): void {
    root.style.setProperty("--toggled", toggled ? "1" : "0");
    glyph.replaceChildren(icon(toggled && toggle ? toggle.icon : options.icon, "var(--type-label-icon)", "var(--on-fill)"));
    if (toggle && button) button.setAttribute("aria-pressed", String(toggled));
    nameIcon();
  }

  /**
   * Names the icon's button, as its accessible name and in its hint, for what a press does: the
   * toggle's label, or on a slider with stops the stop the press goes to.
   */
  function nameIcon(): void {
    if (!button) return;
    const said = !toggle ? stopName(nextStop()) : toggled ? toggle.toggledLabel : toggle.label;
    if (button.getAttribute("aria-label") === said) return;
    button.setAttribute("aria-label", said);
    button.querySelector(":scope > .k-hint")?.remove();
    withHint(button, said, { held: "start" });
  }

  function setToggled(on: boolean): void {
    if (!toggle || on === toggled) return;
    toggled = on;
    drawToggle();
    draw();
    toggle.changed?.(on);
  }

  /** The stop the icon's next press goes to: one on in the direction, or back at either end. */
  function nextStop(): number {
    const ahead = index + direction;
    return ahead < 0 || ahead > last ? index - direction : ahead;
  }

  /** Stop `at`'s name, or with none, its value as the label reads one. */
  function stopName(at: number): string {
    return options.names?.[at] ?? reading(at / last);
  }

  /** A value, 0 to 1, as the label reads a number: the owner's reading, or the locale's percentage. */
  function reading(at: number): string {
    return options.number ? options.number(at) : PERCENT.format(at);
  }

  function numeric(): boolean {
    return !toggled && !(stepping && options.names !== undefined && options.names.length > 0);
  }

  function label(): string {
    if (toggled && toggle) return toggle.words;
    if (stepping && options.names && options.names.length > 0) return options.names[Math.max(0, Math.min(last, Math.round(shown * last)))] ?? "";
    return reading(stepping ? index / last : value);
  }

  /**
   * The mark the value rests on, a fraction, or none: a stop at rest, or the sticky point held.
   * Never ahead of the fill: a value set there counts once the fill has arrived.
   */
  function resting(): number | undefined {
    if (root.hasAttribute("data-settling")) return undefined;
    if (stepping) return dragging ? undefined : index / last;
    return sticky !== undefined && Math.abs(shown - sticky) < 1e-4 ? sticky : undefined;
  }

  /**
   * Draws the label, the dots and the anchor for where the value stands now: worked out in script,
   * since whether a dot shows depends on the label's width and the fill's end.
   */
  function draw(): void {
    const scale = scaleOf(root);
    const px = (name: string) => length(name) * scale;
    const width = root.offsetWidth;
    const side = root.offsetHeight;
    const said = label();
    const kind = style();
    for (const each of [underTrack, overFill]) {
      each.textContent = said;
      typeset(each, kind);
      each.toggleAttribute("data-shaped", shaped(said));
      each.style.setProperty("--natural", String(advance(kind, said)));
    }
    const widest = numeric() ? Math.max(advance(kind, said), advance(kind, options.widest ?? "100%")) : advance(kind, said);
    const labelLeft = width - px("space-s12") - widest * scale;
    root.style.setProperty("--shown", String(shown));
    root.setAttribute("aria-valuenow", stepping ? String(index) : String(Math.round(value * 100)));
    root.setAttribute("aria-valuetext", said);
    root.toggleAttribute("data-numeric", numeric());
    const span = Math.max(1, width - side);
    const centre = (f: number) => side / 2 + span * f;
    const valueX = centre(shown);
    const fillEnd = valueX + side / 2;
    const clear = (x: number, size: number) => x - size / 2 >= side + px("space-s4") && x + size / 2 <= labelLeft - px("space-s4");
    const rest = resting();
    const stop = px("size-slider_stop");
    const anchorSide = px(compact ? "size-slider_anchor_compact" : "size-slider_anchor");
    points.forEach((point, i) => {
      const dot = dotOf[i]!;
      const x = centre(point);
      const anchored = rest !== undefined && Math.abs(rest - point) < 1e-4;
      const shows = clear(x, stop) && !anchored && Math.abs(x - valueX) >= anchorSide / 2 && Math.abs(x - fillEnd) >= stop / 2;
      dot.toggleAttribute("data-hidden", !shows);
      dot.toggleAttribute("data-passed", x < fillEnd);
    });
    const showsAnchor = rest !== undefined && clear(centre(rest), anchorSide);
    if (showsAnchor) anchor.style.setProperty("--at", String(rest));
    const was = anchor.hasAttribute("data-shown");
    anchor.toggleAttribute("data-shown", showsAnchor);
    if (showsAnchor && !was && root.isConnected && !root.hasAttribute("data-still")) pop();
  }

  function pop(): void {
    const duration = motion("short");
    if (duration === 0) return;
    anchor.animate([{ scale: 0 }, { scale: 1 }], { duration, easing: raw("easing-pop") });
  }

  /** Eases the fill to where the value rests: onto a stop after a drag, or to a value set. */
  function settle(): void {
    const from = shown;
    const to = stepping ? index / last : value;
    if (Math.abs(from - to) < 1e-6) {
      draw();
      return;
    }
    const duration = root.hasAttribute("data-still") ? 0 : motion("short");
    const curve = raw(stepping ? "easing-standard_decel" : "easing-standard");
    if (duration === 0) {
      shown = to;
      draw();
      return;
    }
    root.dataset.settling = "";
    const settling = root.animate([{ "--shown": String(from) }, { "--shown": String(to) }], { duration, easing: curve });
    shown = to;
    settling.finished.then(
      () => {
        delete root.dataset.settling;
        draw();
      },
      () => undefined,
    );
    draw();
  }

  /** The fraction under `x` (the pointer's, in the slider's box), held on the sticky point near it. */
  // The page's box is zoomed to the display's ratio, so the pointer's viewport pixels are not the
  // slider's own: its box's width over its layout width converts one to the other.
  /** How far along the slider `event` is, in its own CSS pixels from its left edge. */
  function along(event: PointerEvent): number {
    const box = root.getBoundingClientRect();
    return ((event.clientX - box.left) * root.offsetWidth) / Math.max(1, box.width);
  }

  function fractionAt(x: number): number {
    const side = root.offsetHeight;
    const span = Math.max(1, root.offsetWidth - side);
    const fraction = Math.max(0, Math.min(1, (x - side / 2) / span));
    const reach = (length("size-slider_hold") * scaleOf(root)) / span;
    return sticky !== undefined && Math.abs(fraction - sticky) <= reach ? sticky : fraction;
  }

  /** Moves a continuous slider's value to `fraction`, out of Toggled, from where it is. */
  function moveTo(fraction: number): void {
    value = Math.max(0, Math.min(1, fraction));
    setToggled(false);
    settle();
  }

  function stepTo(to: number): void {
    index = Math.max(0, Math.min(last, to));
    nameIcon();
    settle();
  }

  // The icon's press: on a slider with stops it steps from high to low and back, turning at either
  // end; on a continuous one it turns Toggled on or off.
  function pressIcon(): void {
    if (!stepping) {
      setToggled(!toggled);
      return;
    }
    if (index + direction < 0 || index + direction > last) direction = direction === 1 ? -1 : 1;
    stepTo(index + direction);
  }

  if (!options.disabled) {
    root.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      root.setPointerCapture(event.pointerId);
      for (const running of root.getAnimations()) running.cancel();
      delete root.dataset.settling;
      dragging = true;
      root.dataset.dragging = "";
      shown = fractionAt(along(event));
      if (!stepping) value = shown;
      setToggled(false);
      draw();
    });
    root.addEventListener("pointermove", (event) => {
      if (!dragging) return;
      shown = fractionAt(along(event));
      if (!stepping) value = shown;
      draw();
    });
    const release = () => {
      if (!dragging) return;
      dragging = false;
      delete root.dataset.dragging;
      if (stepping) index = Math.round(shown * last);
      nameIcon();
      settle();
    };
    root.addEventListener("pointerup", release);
    root.addEventListener("pointercancel", release);
    // Space presses the icon's button as it would a focused button: pressed while the key is down,
    // acting as it comes up, never on key repeat; the focus leaving lets it go without a press.
    // With no button, Space is left to the page.
    root.addEventListener("keydown", (event) => {
      if (event.key !== " " || !button) return;
      event.preventDefault();
      if (!event.repeat) button.dataset.keyed = "";
    });
    root.addEventListener("keyup", (event) => {
      if (event.key !== " " || !button?.hasAttribute("data-keyed")) return;
      delete button.dataset.keyed;
      pressIcon();
    });
    root.addEventListener("blur", () => {
      if (button) delete button.dataset.keyed;
    });
    root.addEventListener("keydown", (event) => {
      const back = event.key === "ArrowLeft" || event.key === "h";
      const forward = event.key === "ArrowRight" || event.key === "l";
      if (event.key === "Home" || event.key === "End") {
        const end = event.key === "End";
        if (stepping) stepTo(end ? last : 0);
        else moveTo(end ? 1 : 0);
      } else if (back || forward) {
        if (stepping) stepTo(index + (forward ? 1 : -1));
        else moveTo(value + (forward ? NUDGE : -NUDGE));
      } else {
        return;
      }
      event.preventDefault();
    });
    // Only a sideways turn of the wheel steps it, wheel right raising it: the vertical wheel scrolls
    // the page past it, as everywhere else.
    sideways(root, (notches) => {
      if (stepping) stepTo(index + notches);
      else moveTo(value + notches * NUDGE);
    });
  }

  // Drawn once it is laid out, and again whenever its width or the scale changes it.
  new ResizeObserver(() => draw()).observe(root);
  drawToggle();
  draw();
  return probe(root, { role: "component", name: "Slider", variant: variant(compact ? "compact" : "regular", stepping && "stepped"), fixed: { width: false, height: true } });
}
