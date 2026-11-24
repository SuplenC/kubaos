// A camera's live picture, as a page that sets the camera shows it beside its controls (Settings ›
// Camera): a draft part with no twin yet. The picture is a `Media` at `aspect.camera` and the
// content radius, filling its room; in the mockups it is a drawn stand-in of a person before a
// bright window, never the real camera. While another app has the camera the box keeps its size
// and says who, over the well's sunken fill, since the picture can't be read then.

import "./camera-preview.css";
import backlit from "../assets/camera/backlit.svg?url";
import lifted from "../assets/camera/lifted.svg?url";
import { icon } from "./icon.ts";
import { stack } from "./layout.ts";
import { media } from "./media.ts";
import { declare, probe } from "./probe.ts";
import { role } from "./radius.ts";
import { clamped } from "./text.ts";

/** The stand-in picture: the face dark against the window, or lifted out of it. */
export type PreviewLook = "backlit" | "lifted";

export interface CameraPreviewOptions {
  readonly look: PreviewLook;
  /**
   * Another app has the camera: its name, or null when the system can't tell which. Absent while
   * the picture is this page's.
   */
  readonly inUseBy?: string | null;
}

const PICTURES: Record<PreviewLook, string> = { backlit, lifted };

/** What the box says while another app has the camera. */
function inUse(by: string | null): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-preview-busy k-sunken";
  // An app's name is what it says, and a long one ends in an ellipsis.
  const said = declare(clamped(by === null ? "In use by another app" : `In use by ${by}`, "subheading", "on_surface_variant"), { variable: true });
  box.append(stack("s12", "centre", icon("video", "var(--icon-s32)", "var(--color-on_surface_variant)"), said));
  const edge = document.createElement("span");
  edge.className = "k-preview-edge";
  box.append(edge);
  return box;
}

export function cameraPreview(options: CameraPreviewOptions): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-preview";
  room.style.setProperty("--aspect", "var(--aspect-camera)");
  const busy = options.inUseBy !== undefined;
  room.append(busy ? inUse(options.inUseBy ?? null) : media({ source: PICTURES[options.look], corner: role("content"), aspect: "camera" }));
  return probe(room, { role: "component", name: "CameraPreview", variant: busy ? "in-use" : options.look, fixed: { width: false, height: false } });
}

/** Shows `look` on `preview`, a `cameraPreview`'s room, as the settings change it. */
export function showLook(preview: HTMLElement, look: PreviewLook): void {
  const picture = preview.querySelector<HTMLImageElement>(".k-media-picture");
  if (picture && !picture.src.endsWith(PICTURES[look])) picture.src = PICTURES[look];
}
