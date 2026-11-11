# 4. Wallpaper depth: cut-outs shipped for KubaOS's wallpapers, and a Suite for the user's own

Status: accepted (2026-11-04)

## Context

The redesigned lock screen (the Lock page in `design/mockups/`) puts its rest clock behind the city: a foreground cut-out of the wallpaper is drawn over the digits. In the lock and unlock motion the picture and the cut-out push in at different rates, so the city comes forward as it steps in front of the clock. That needs a cut-out per wallpaper, and the right place and tint for the clock on it.

Cutting a foreground out of a photo by hand, or with colour rules (ImageMagick), fails on fine structures and on skylines; the first Tokyo Tower cut-out stopped halfway down the tower. Open machine-learning models do it well, on the CPU, in seconds:
- **BiRefNet** finds a single object, such as Tokyo Tower spire to legs. It finds nothing in a skyline, which has no single object.
- **Depth Anything V2 Small** separates a skyline's sky in about a second, and layers a street with no sky at all, but coarsely (518 px).
- **SAM** traces an edge once it is pointed at a region.

The models are large, so they can't be part of every install. KubaOS's own wallpapers live in a separate repository, taken as a flake input.

## Decision

**KubaOS's wallpapers ship their depth, made once, offline.** The wallpapers repository has a script that runs the models on each picture. Its results are checked by eye and ship beside the picture: the cut-out, the clock's place (a point in shares of the picture) and the clock's tint. With depth, the depth map ships too. No model is installed on a machine for these; the lock reads the files. A picture without them gets the clock in front of it.

**The user's own pictures get it from a Suite, off by default.** `kubaos suite add wallpaper-depth`, or its Command menu Item, installs it; from then on it just works, and `kubaos suite remove wallpaper-depth` takes it all away, the models with it.
- **The tool** is a small Rust binary in this repo, `kubaos-depth`, which runs the ONNX models through ONNX Runtime (the `ort` crate), with no Python on the system. Setting a wallpaper already runs matugen. The same hook queues `kubaos-depth` as a low-priority systemd user job, which waits while on battery. It writes its results into a folder per picture under `~/.cache/kubaos/wallpapers/`, in the same form the shipped files take.
- **The models** are pinned and fetched by the Suite's Nix module, like any other dependency. They are chosen for open licences: Depth Anything V2 Small (Apache-2.0; its larger sizes are non-commercial), BiRefNet lite (MIT) and SAM (Apache-2.0), about 400 MB together.
- **One result per picture, or none.** The tool tries the sky (from depth) and the single object (BiRefNet), traces the winner's edge, and keeps whichever scores best under the clock. When none is confident it writes no cut-out, since a bad cut-out is worse than none. The same tool places the clock, so the cut-out covers part of the digits and never the date line. It also works out the clock's tint, so neither is ever set by hand.
- **Settings**, each with a Command menu Item:
  - depth for my wallpapers, on or off;
  - redo this wallpaper's cut-out;
  - no cut-out on this wallpaper;
  - how far the city pushes in.

**The lock draws three layers**: the picture, the clock and the cut-out. They move only in the lock and unlock motion. The cut-out is always scaled at least as much as the picture about a shared origin, so it only ever covers more of its own outline and nothing behind it is revealed; no filled-in background is needed. At rest nothing moves and nothing redraws.

**A Suite is no longer only a language setup.** It is an optional, KubaOS-managed setup the user installs, configures and removes whole: a programming language or platform, or a feature heavy enough to be optional, such as wallpaper depth. `GLOSSARY.md` says so.

## Considered options

- **Models in every install.** Rejected: about 400 MB on every machine for a lock-screen effect, most of which would only ever read the shipped files.
- **Python (rembg) on the system.** Rejected: a Python environment for one effect, outside Nix's pins. The same ONNX models run from Rust.
- **Colour rules (ImageMagick) instead of models.** Rejected: they can't tell a tower from the sky behind it.
- **A filled-in background (LaMa) for sideways parallax.** Not now: the motion only pushes in, which never reveals what the cut-out hides. Sideways parallax, such as following the pointer, would need it. It can come later as a fourth model in the same Suite.
- **Following the pointer, or a slow drift at rest.** Rejected for now: a drift keeps the lock redrawing every frame for as long as it sits there.
