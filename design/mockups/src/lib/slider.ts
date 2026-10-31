// A value from 0 to 1, or one of a few stops, as a pill: `kubaos-ui`'s `slider::slider`,
// `KubaOS.Components`' `Slider` (`docs/design/components/slider.md`). The fill is the value: its
// round end marks it, the value being that end circle's centre, so at 0 the fill is the circle
// behind the icon and at 1 it reaches the far end. The icon sits in that circle, on the fill; the
// value, or a name for it, sits `space.s12` in from the far end, in the track's colours over the
// track and the fill's over the fill. Regular (`size.slider`) or compact (`size.slider_compact`);
// a compact slider shows a number only lit or held, and a name always.
//
// Steps show as dots on both sides of the fill, each in its side's colour; the mark the value rests
// on, a stop or a sticky point it holds at, turns solid and pops in. No dot is drawn under the icon
// or the label, `space.s4` clear of each.
//
// It is a control in the state model (`.k-control` in `lib.css`): lit under the pointer or with the
// keyboard on it, its fill brightening by `opacity.press` toward the palette's bright end, the tube
// inside its edge with the glow outside; held (pressed or dragged), the fill brightens by
// `opacity.held`. It owns its value here, as the app does in both libraries: the pointer, the keys
// and the wheel move it, and it draws itself again.

import { settings } from "../site/settings.ts";
import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { probe, variant } from "./probe.ts";
import { advance, shaped, text, type TypeStyle, typeset } from "./text.ts";
import { length, raw } from "./tokens.ts";

export const SLIDER_SIZES = ["Regular", "Compact"] as const;
export type SliderSize = (typeof SLIDER_SIZES)[number];

/** A step of a continuous slider from an arrow key or a wheel's notch: a twentieth. */
const NUDGE = 0.05;

/** A wheel's notch, as the toolkits count it: Qt's `angleDelta` and a browser's line of 100. */
const NOTCH = 100;

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
  /** A name shown in place of the value, always: "Muted", "Auto · 55%". */
  readonly words?: string;
  /** The number shown in place of the percentage, as the owner reads the value. */
  readonly number?: (value: number) => string;
  /** The widest the number reads, so the number ends in one place: "100%" unless given. */
  readonly widest?: string;
  /** The fill in the softer, desaturated accent: a level something else picked (Auto brightness). */
  readonly soft?: boolean;
  /** A sticky point, a fraction along the track, the value holds at within `size.slider_hold`. */
  readonly sticky?: number;
  /** What a press on the icon does; with none, the icon of a continuous slider isn't a button. */
  readonly onIcon?: (control: SliderControl) => void;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
  /** Held, as a press or a drag holds it: the specimens' view of the state. */
  readonly held?: boolean;
  /** Drawn still, as a player's sample is: no pop, nothing eased. */
  readonly still?: boolean;
}

/** What a slider's owner can do to it once it is drawn: what its icon's press reaches. */
export interface SliderControl {
  readonly element: HTMLElement;
  /** Moves a continuous slider to `value`, easing the fill there. */
  set(value: number): void;
  /** Shows `words` in place of the value, or the value again with none. */
  name(words: string | undefined): void;
  readonly value: number;
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
  let words = options.words;
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
  root.toggleAttribute("data-soft", options.soft ?? false);
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
  glyph.append(icon(options.icon, "var(--type-label-icon)", "var(--on-fill)"));
  onFill.append(glyph, overFill);
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

  const pressesIcon = !options.disabled && (stepping || options.onIcon !== undefined);
  if (pressesIcon) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "k-slider-icon-button";
    button.tabIndex = -1;
    button.setAttribute("aria-label", stepping ? "Step" : "Mute");
    button.addEventListener("pointerdown", (event) => event.stopPropagation());
    button.addEventListener("click", () => pressIcon());
    root.append(button);
  }
  focusRing(root, "full", "accent", false).dataset.pressable = "";

  function numeric(): boolean {
    return words === undefined && !(stepping && options.names !== undefined && options.names.length > 0);
  }

  function label(): string {
    if (words !== undefined) return words;
    if (stepping && options.names && options.names.length > 0) return options.names[Math.max(0, Math.min(last, Math.round(shown * last)))] ?? "";
    const at = stepping ? index / last : value;
    return options.number ? options.number(at) : PERCENT.format(at);
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
  function fractionAt(x: number): number {
    const side = root.offsetHeight;
    const span = Math.max(1, root.offsetWidth - side);
    const fraction = Math.max(0, Math.min(1, (x - side / 2) / span));
    const reach = (length("size-slider_hold") * scaleOf(root)) / span;
    return sticky !== undefined && Math.abs(fraction - sticky) <= reach ? sticky : fraction;
  }

  function moveTo(fraction: number): void {
    value = Math.max(0, Math.min(1, fraction));
    words = options.words === undefined ? undefined : words;
    settle();
  }

  function stepTo(to: number): void {
    index = Math.max(0, Math.min(last, to));
    settle();
  }

  // The icon's press: on a slider with stops it steps from high to low and back, turning at either
  // end; on a continuous one it does what its owner says.
  function pressIcon(): void {
    if (!stepping) {
      options.onIcon?.(control);
      return;
    }
    if (index + direction < 0 || index + direction > last) direction = direction === 1 ? -1 : 1;
    stepTo(index + direction);
  }

  const control: SliderControl = {
    element: root,
    set: moveTo,
    name(next) {
      words = next;
      draw();
    },
    get value() {
      return value;
    },
  };

  if (!options.disabled) {
    root.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      root.setPointerCapture(event.pointerId);
      for (const running of root.getAnimations()) running.cancel();
      delete root.dataset.settling;
      dragging = true;
      root.dataset.dragging = "";
      shown = fractionAt(event.offsetX);
      if (!stepping) value = shown;
      draw();
    });
    root.addEventListener("pointermove", (event) => {
      if (!dragging) return;
      const box = root.getBoundingClientRect();
      shown = fractionAt(event.clientX - box.left);
      if (!stepping) value = shown;
      draw();
    });
    const release = () => {
      if (!dragging) return;
      dragging = false;
      delete root.dataset.dragging;
      if (stepping) index = Math.round(shown * last);
      settle();
    };
    root.addEventListener("pointerup", release);
    root.addEventListener("pointercancel", release);
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
    let turned = 0;
    root.addEventListener(
      "wheel",
      (event) => {
        event.preventDefault();
        turned -= event.deltaY;
        const notches = Math.trunc(turned / NOTCH);
        if (notches === 0) return;
        turned -= notches * NOTCH;
        if (stepping) stepTo(index + notches);
        else moveTo(value + notches * NUDGE);
      },
      { passive: false },
    );
  }

  // Drawn once it is laid out, and again whenever its width or the scale changes it.
  new ResizeObserver(() => draw()).observe(root);
  draw();
  return probe(root, { role: "component", name: "Slider", variant: variant(compact ? "compact" : "regular", stepping && "stepped"), fixed: { width: false, height: true } });
}
