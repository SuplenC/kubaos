// Camera: what fixes a camera's picture, from the controls the camera itself offers (UVC's, read
// and set through V4L2), beside its live picture. Each camera keeps its own values, and a reset puts
// back the camera's own defaults, not KubaOS's. A camera shows only the controls it offers; an
// infrared camera, which only face login reads, is never offered.

import { cameraPreview, type PreviewLook, showLook } from "../../../../lib/camera-preview.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import type { Setting, SettingsSection, SettingsSub } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { choice, pick, stops, toggle } from "./kit.ts";

/** Every control the page can set, by the camera's own name for it in each row's brackets. */
export type CameraControl = "light" | "sharpness" | "autoColour" | "temperature" | "autoExposure" | "exposure" | "dim" | "boost" | "flicker";

/** A camera's values, each a stop of its row's control or a switch. */
export interface CameraValues {
  light: number;
  sharpness: number;
  autoColour: boolean;
  temperature: number;
  autoExposure: boolean;
  exposure: number;
  dim: boolean;
  boost: number;
  flicker: number;
}

/** A camera as the system lists it. */
interface Camera {
  readonly id: string;
  readonly name: string;
  /** An infrared camera, for face login only: never offered here. */
  readonly infrared?: true;
  /** The controls it offers. */
  readonly offers: readonly CameraControl[];
  /** How many levels its backlight compensation offers: two is a switch, more a slider of stops. */
  readonly lightLevels: number;
  /** Its own defaults, which a reset puts back. */
  readonly defaults: CameraValues;
}

const LIGHT_LEVELS = ["Off", "Low", "High"];
const SHARPNESS = Array.from({ length: 8 }, (_, i) => String(i + 1));
const KELVIN = new Intl.NumberFormat("en-GB", { useGrouping: false });
const TEMPERATURES = Array.from({ length: 38 }, (_, i) => `${KELVIN.format(2800 + i * 100)} K`);
const EXPOSURES = ["1/1000 s", "1/500 s", "1/250 s", "1/125 s", "1/60 s", "1/30 s", "1/15 s"];
const BOOSTS = Array.from({ length: 21 }, (_, i) => `${i * 5} %`);
const FLICKER = ["Off", "50 Hz", "60 Hz"];

const EVERY: readonly CameraControl[] = ["light", "sharpness", "autoColour", "temperature", "autoExposure", "exposure", "dim", "boost", "flicker"];

/** What a UVC camera ships set to. */
const SHIPPED: CameraValues = {
  light: 0,
  sharpness: 3,
  autoColour: true,
  temperature: TEMPERATURES.indexOf("4600 K"),
  autoExposure: true,
  exposure: EXPOSURES.indexOf("1/60 s"),
  dim: false,
  boost: 0,
  flicker: 1,
};

/** The cameras the samples plug in, by id. */
export const CAMERAS: Readonly<Record<string, Camera>> = {
  integrated: { id: "integrated", name: "Integrated Camera", offers: EVERY, lightLevels: 3, defaults: SHIPPED },
  infrared: { id: "infrared", name: "Integrated IR Camera", infrared: true, offers: [], lightLevels: 0, defaults: SHIPPED },
  c920: { id: "c920", name: "Logitech C920", offers: EVERY, lightLevels: 2, defaults: { ...SHIPPED, sharpness: 4 } },
  basic: { id: "basic", name: "USB Camera", offers: ["sharpness", "flicker"], lightLevels: 0, defaults: SHIPPED },
};

/** The page's own values: the cameras plugged in, the one picked, each one's values, and who else has it. */
export interface CameraState {
  /** The cameras plugged in, by id, the infrared one among them where there is one. */
  plugged: readonly string[];
  /** The one the page sets, among those offered. */
  picked: number;
  /** Each camera's values, by id, made from its defaults when first shown. */
  values: Record<string, Partial<CameraValues>>;
  /** Another app has the camera: its name, or null when the system can't tell; false while none has. */
  busy: string | null | false;
}

const DEFAULTS: CameraState = { plugged: ["integrated", "infrared"], picked: 0, values: {}, busy: false };

/**
 * `setting`, dimmed in place while `automatic` holds, saying `why` in place of its line: a manual
 * control its automatic switch sets. Read as the region draws it, so the switch's row redraws it.
 */
function setBy(setting: Setting, automatic: () => boolean, why: string): Setting {
  return {
    ...setting,
    get disabled() {
      return automatic() ? why : undefined;
    },
  };
}

/** Camera, whole. */
export function camera(state: Values): SettingsSection {
  const mine = own(state, "camera", (): CameraState => ({ ...DEFAULTS, values: {} }));
  const offered = mine.plugged.map((id) => CAMERAS[id]!).filter((each) => !each.infrared);
  const base = {
    ...(state.opened ? { opened: state.opened } : {}),
    icon: "video",
    title: "Camera",
    accent: "カメラ",
    lede: "Seen in a better light.",
  };
  if (offered.length === 0) {
    return {
      ...base,
      subs: [],
      instead: () => emptyState({ icon: "video", title: "No camera connected.", body: "Its settings show here once one is plugged in." }),
    };
  }

  const cam = () => offered[Math.min(mine.picked, offered.length - 1)]!;
  /** The picked camera's values, made from its own defaults the first time. */
  const now = (): CameraValues => {
    const id = cam().id;
    const kept = { ...cam().defaults, ...mine.values[id] };
    mine.values[id] = kept;
    return kept;
  };
  const shipped = () => cam().defaults;
  const offers = (control: CameraControl) => cam().offers.includes(control);

  let preview: HTMLElement | null = null;
  const look = (): PreviewLook => (offers("light") && now().light > 0 ? "lifted" : "backlit");
  /** A value changed: the picture follows it at once, as the camera's own does. */
  const changed = () => {
    if (preview) showLook(preview, look());
  };
  const set = <K extends keyof CameraValues>(key: K) => (value: CameraValues[K]) => {
    now()[key] = value;
    changed();
  };

  const light: Setting =
    cam().lightLevels > 2
      ? stops("light", "Light behind you", "Lifts your face when a window or lamp is behind you. (Backlight compensation)", "light_mode", LIGHT_LEVELS.slice(0, cam().lightLevels), () => now().light, set("light"), {
          initial: () => shipped().light,
        })
      : toggle("light", "Light behind you", "Lifts your face when a window or lamp is behind you. (Backlight compensation)", () => now().light > 0, (on) => set("light")(on ? 1 : 0), { initial: () => shipped().light > 0 });

  const picture: Setting[] = [
    ...(offered.length > 1
      ? [
          pick(
            "camera",
            "Camera",
            "Each keeps its own settings.",
            offered.map((each) => each.name),
            () => mine.picked,
            (index) => {
              mine.picked = index;
              changed();
            },
            { drives: true },
          ),
        ]
      : []),
    ...(offers("light") ? [light] : []),
    ...(offers("sharpness")
      ? [stops("sharpness", "Sharpness", "Crisper edges; too much draws bright halos.", "visibility", SHARPNESS, () => now().sharpness, set("sharpness"), { initial: () => shipped().sharpness })]
      : []),
    ...(offers("autoColour")
      ? [toggle("auto-colour", "Automatic colour", "Matches the colour of the room's light. (Auto white balance)", () => now().autoColour, set("autoColour"), { drives: true, initial: () => shipped().autoColour })]
      : []),
    ...(offers("temperature")
      ? [
          setBy(
            stops("temperature", "Colour temperature", "Lower looks warmer, higher looks cooler. (White balance temperature)", "thermostat", TEMPERATURES, () => now().temperature, set("temperature"), {
              initial: () => shipped().temperature,
            }),
            () => offers("autoColour") && now().autoColour,
            "Set by Automatic colour; turn it off to set it here. (White balance temperature)",
          ),
        ]
      : []),
  ];

  const fine: Setting[] = [
    ...(offers("autoExposure")
      ? [toggle("auto-exposure", "Automatic exposure", "Follows the light in the room. (Auto exposure)", () => now().autoExposure, set("autoExposure"), { drives: true, initial: () => shipped().autoExposure })]
      : []),
    ...(offers("exposure")
      ? [
          setBy(
            stops("exposure", "Exposure time", "Longer is brighter; too long blurs what moves. (Exposure, absolute)", "timer", EXPOSURES, () => now().exposure, set("exposure"), {
              initial: () => shipped().exposure,
            }),
            () => offers("autoExposure") && now().autoExposure,
            "Set by Automatic exposure; turn it off to set it here. (Exposure, absolute)",
          ),
        ]
      : []),
    ...(offers("dim")
      ? [toggle("dim-light", "Brighter in dim light", "Lets the picture slow to fewer frames a second when the room is dark. (Exposure priority)", () => now().dim, set("dim"), { initial: () => shipped().dim })]
      : []),
    ...(offers("boost")
      ? [stops("boost", "Boost", "Brightens the picture electronically; more adds grain. (Gain)", "brightness_high", BOOSTS, () => now().boost, set("boost"), { initial: () => shipped().boost })]
      : []),
    ...(offers("flicker")
      ? [
          choice("flicker", "Room light flicker", "Stops stripes from lights on the mains: 50 Hz in Europe, 60 Hz in the Americas. (Power-line frequency)", FLICKER, () => now().flicker, set("flicker"), {
            initial: () => shipped().flicker,
          }),
        ]
      : []),
  ];

  const subs: SettingsSub[] = [
    { title: "Picture", body: "What fixes a dark, soft or off-colour picture.", wells: [picture] },
    { title: "Fine-tuning", body: "For a picture that still isn't right.", wells: [fine] },
  ];
  return {
    ...base,
    subs,
    aside: () => {
      preview = cameraPreview({ look: look(), ...(mine.busy !== false ? { inUseBy: mine.busy } : {}) });
      return preview;
    },
  };
}
